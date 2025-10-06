use oxc_allocator::{Address, Allocator, GetAddress, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_semantic::{ReferenceFlags, Scoping, SymbolFlags, SymbolId};
use oxc_span::SPAN;
use oxc_traverse::{BoundIdentifier, Traverse, traverse_mut};
use rustc_hash::FxHashMap;

mod dedupe;

use crate::{
    OptimizerOptions,
    annotation::Annotation,
    chunk::dedupe::{DedupeKind, DedupeState, dedupe_hash},
    context::{TraverseCtx, TraverseCtxState},
    globals::{GlobalValue, get_global_value},
    property_names::LocalPropertyMap,
    statements::Statements,
};

pub fn optimize_chunk<'a, 'ctx>(
    program: &mut Program<'a>,
    options: &OptimizerOptions,
    property_map: LocalPropertyMap<'a, 'ctx>,
    allocator: &'a Allocator,
    scoping: Scoping,
) {
    let mut optimizer = ChunkOptimizer::new(options, property_map);
    let scoping =
        traverse_mut(&mut optimizer, allocator, program, scoping, TraverseCtxState::default());
    if options.dedupe && optimizer.dedupe.duplicates > 0 {
        let mut dedupe = Dedupe::new(optimizer.dedupe);
        traverse_mut(&mut dedupe, allocator, program, scoping, TraverseCtxState::default());
    }
}

struct ChunkOptimizer<'a, 'ctx> {
    options: &'ctx OptimizerOptions,
    property_map: LocalPropertyMap<'a, 'ctx>,
    statements: Statements<'a>,
    annotations: Vec<AnnotatedExpr>,
    globals_symbols: FxHashMap<SymbolId, &'ctx GlobalValue>,
    globals_ids: FxHashMap<*const GlobalValue, BoundIdentifier<'a>>,
    singletons: FxHashMap<*const GlobalValue, BoundIdentifier<'a>>,
    dedupe: DedupeState,
}

impl<'a, 'ctx> ChunkOptimizer<'a, 'ctx> {
    fn new(options: &'ctx OptimizerOptions, property_map: LocalPropertyMap<'a, 'ctx>) -> Self {
        Self {
            options,
            property_map,
            statements: Statements::new(),
            annotations: Vec::new(),
            globals_symbols: FxHashMap::default(),
            globals_ids: FxHashMap::default(),
            singletons: FxHashMap::default(),
            dedupe: DedupeState::default(),
        }
    }
}

impl<'a, 'ctx> Traverse<'a, TraverseCtxState<'a>> for ChunkOptimizer<'a, 'ctx> {
    fn exit_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.statements.exit_program(node, ctx);
    }

    fn enter_statements(
        &mut self,
        _node: &mut ArenaVec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.dedupe {
            self.dedupe.scopes.push(FxHashMap::default());
        }
    }

    fn exit_statements(
        &mut self,
        _node: &mut ArenaVec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.dedupe {
            self.dedupe.scopes.pop();
        }
    }

    fn enter_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // Replaces `new URL("./url", import.meta.url).href` with an absolute URL.
        if let Some(base_url) = &self.options.url {
            if let Expression::StaticMemberExpression(expr) = node {
                if expr.property.name == "href" {
                    if let Expression::NewExpression(new_expr) = &mut expr.object {
                        let args = &mut new_expr.arguments;
                        if args.len() == 2 {
                            let arg0 = &args[0];
                            let arg1 = &args[1];
                            if let Argument::StringLiteral(rel_url) = arg0
                                && is_import_meta_url(arg1)
                            {
                                let rel_url = rel_url.value.as_str();
                                *node = ctx.ast.expression_string_literal(
                                    SPAN,
                                    ctx.ast.atom_from_strs_array([
                                        base_url,
                                        rel_url.strip_prefix("./").unwrap_or(rel_url),
                                    ]),
                                    None,
                                );
                            }
                        }
                    }
                }
            }
        }

        if self.options.dedupe || self.options.rename_properties {
            // Unwraps `__oveo__()` expressions and adds annotation to the stack.
            let address = node.address();
            if let Expression::CallExpression(expr) = node {
                if let Expression::Identifier(id) = &expr.callee {
                    let r = ctx.scoping().get_reference(id.reference_id());
                    if r.symbol_id().is_none() && id.name == Annotation::ID_NAME {
                        let mut args = expr.arguments.drain(1..);
                        let flags = args.next().unwrap();
                        if let Expression::NumericLiteral(flags) = flags.to_expression() {
                            self.annotations.push(AnnotatedExpr {
                                address,
                                annotation: Annotation::new(flags.value as u32),
                            });
                        }
                    }
                }
            }
        }
    }

    fn exit_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.globals.hoist {
            'hoist_globals: {
                match node {
                    // Replaces global identifier with a reference to a const symbol.
                    Expression::Identifier(expr) => {
                        let reference = ctx.scoping().get_reference(expr.reference_id());
                        if reference.symbol_id().is_none() {
                            if let Some(v) =
                                get_global_value(self.options.globals.include, expr.name.as_str())
                            {
                                if !v.is_hoistable() {
                                    break 'hoist_globals;
                                }
                                let uid = self
                                    .globals_ids
                                    .entry(v as *const _)
                                    .or_insert_with(|| {
                                        let uid = ctx.generate_uid_in_root_scope(
                                            "_GLOBAL_",
                                            SymbolFlags::ConstVariable,
                                        );
                                        self.globals_symbols.insert(uid.symbol_id, v);
                                        self.statements.insert_top_level_statement(
                                            stmt_const_decl(
                                                &uid,
                                                ctx.ast.expression_identifier(SPAN, expr.name),
                                                ctx,
                                            ),
                                        );
                                        uid
                                    })
                                    .clone();
                                *node = uid.create_read_expression(ctx);
                            }
                        }
                    }
                    Expression::StaticMemberExpression(expr) => {
                        if let Expression::Identifier(object_id_expr) = &expr.object {
                            // Replaces global.property with a reference to a const symbol.
                            if let Some(object_symbol_id) = ctx
                                .scoping()
                                .get_reference(object_id_expr.reference_id())
                                .symbol_id()
                            {
                                if let Some(global) =
                                    self.globals_symbols.get(&object_symbol_id).copied()
                                {
                                    if let Some(v) = global.statics.get(expr.property.name.as_str())
                                    {
                                        if !v.is_hoistable() {
                                            break 'hoist_globals;
                                        }
                                        let object_id = self
                                            .globals_ids
                                            .get(&(global as *const _))
                                            .cloned()
                                            .unwrap();
                                        let uid = self
                                            .globals_ids
                                            .entry(v as *const _)
                                            .or_insert_with(|| {
                                                let uid = ctx.generate_uid_in_root_scope(
                                                    "_GLOBAL_",
                                                    SymbolFlags::ConstVariable,
                                                );
                                                self.globals_symbols.insert(uid.symbol_id, v);
                                                self.statements.insert_top_level_statement(
                                                    create_static_member_decl(
                                                        &uid,
                                                        &object_id,
                                                        expr.property.name,
                                                        ctx,
                                                    ),
                                                );
                                                uid
                                            })
                                            .clone();
                                        *node = uid.create_read_expression(ctx);
                                    }
                                }
                            }
                        }
                    }
                    // Replaces singletons `new TextEncoder()` with a reference to a const symbol.
                    Expression::NewExpression(expr) => {
                        if let Expression::Identifier(object_id_expr) = &expr.callee {
                            if let Some(object_symbol_id) = ctx
                                .scoping()
                                .get_reference(object_id_expr.reference_id())
                                .symbol_id()
                            {
                                if let Some(&global) = self.globals_symbols.get(&object_symbol_id) {
                                    if global.is_singleton_func() {
                                        let uid = self
                                            .singletons
                                            .entry(global as *const _)
                                            .or_insert_with(|| {
                                                let callee_id =
                                                    &self.globals_ids[&(global as *const _)];
                                                let uid = ctx.generate_uid_in_root_scope(
                                                    "_SINGLETON_",
                                                    SymbolFlags::ConstVariable,
                                                );
                                                self.statements.insert_top_level_statement(
                                                    create_new_expr(
                                                        &uid,
                                                        callee_id,
                                                        ctx.ast.vec(),
                                                        ctx,
                                                    ),
                                                );
                                                uid
                                            });
                                        *node = uid.create_read_expression(ctx);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let address = node.address();
        if let Some(a) = self.annotations.pop_if(|a| a.address == address) {
            if self.options.dedupe && a.annotation.is_dedupe() {
                if let Expression::CallExpression(expr) = node {
                    if let Some(arg0) = expr.arguments.pop() {
                        let arg0 = arg0.into_expression();
                        let _ = dedupe_hash(&mut self.dedupe, &arg0, ctx.scoping());
                        *node = arg0;
                        return;
                    }
                }
            } else if self.options.rename_properties && a.annotation.is_key() {
                if let Expression::CallExpression(expr) = node {
                    if let Some(arg0) = expr.arguments.pop() {
                        let mut arg0 = arg0.into_expression();
                        if let Expression::StringLiteral(expr) = &mut arg0 {
                            if let Some(v) = self.property_map.get(expr.value, &ctx.ast) {
                                expr.value = v;
                            }
                        }
                        *node = arg0;
                        return;
                    }
                }
            }
            *node = ctx.ast.void_0(SPAN);
        }
    }

    fn exit_identifier_name(&mut self, node: &mut IdentifierName<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.rename_properties {
            if let Some(v) = self.property_map.get(node.name, &ctx.ast) {
                node.name = v;
            }
        }
    }
}

struct Dedupe<'a> {
    statements: Statements<'a>,
    state: DedupeState,
    statement_stack: Vec<Address>,
    originals: FxHashMap<Address, BoundIdentifier<'a>>,
}

impl<'a> Dedupe<'a> {
    fn new(state: DedupeState) -> Self {
        Self {
            statements: Statements::new(),
            state,
            statement_stack: Vec::new(),
            originals: FxHashMap::default(),
        }
    }
}

impl<'a> Traverse<'a, TraverseCtxState<'a>> for Dedupe<'a> {
    fn exit_statements(
        &mut self,
        node: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.statements.exit_statements(node, ctx);
    }

    fn enter_statement(&mut self, node: &mut Statement<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.statement_stack.push(node.address());
    }

    fn exit_statement(&mut self, _node: &mut Statement<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.statement_stack.pop();
    }

    fn exit_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let address = node.address();
        if let Some(dedupe_kind) = self.state.expressions.get(&address) {
            match dedupe_kind {
                DedupeKind::Original(duplicates) => {
                    if *duplicates > 0
                        && let Some(statement_address) = self.statement_stack.last()
                    {
                        let uid =
                            ctx.generate_uid_in_root_scope("_DEDUPE_", SymbolFlags::ConstVariable);
                        let mut expr2 = uid.create_read_expression(ctx);
                        std::mem::swap(node, &mut expr2);
                        let decl = stmt_const_decl(&uid, expr2, ctx);
                        self.statements.insert_before(statement_address, decl);
                        self.originals.insert(address, uid);
                    }
                }
                DedupeKind::Duplicate(original_address) => {
                    if let Some(id) = self.originals.get(original_address) {
                        *node = id.create_read_expression(ctx);
                    }
                }
            }
        }
    }
}

struct AnnotatedExpr {
    address: Address,
    annotation: Annotation,
}

// `const uid = expr;`
fn stmt_const_decl<'a>(
    uid: &BoundIdentifier<'a>,
    expr: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Statement<'a> {
    Statement::VariableDeclaration(ctx.ast.alloc_variable_declaration(
        SPAN,
        VariableDeclarationKind::Const,
        ctx.ast.vec1(ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            ctx.ast.binding_pattern(
                BindingPatternKind::BindingIdentifier(
                    ctx.alloc(uid.create_binding_identifier(ctx)),
                ),
                NONE,
                false,
            ),
            Some(expr),
            false,
        )),
        false,
    ))
}

// `const uid = object_id.property_name;`
fn create_static_member_decl<'a>(
    uid: &BoundIdentifier<'a>,
    object_id: &BoundIdentifier<'a>,
    property_name: Atom<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Statement<'a> {
    stmt_const_decl(
        uid,
        Expression::StaticMemberExpression(ctx.ast.alloc_static_member_expression(
            SPAN,
            object_id.create_expression(ReferenceFlags::read(), ctx),
            ctx.ast.identifier_name(SPAN, property_name),
            false,
        )),
        ctx,
    )
}

// `const uid = new callee_id(arguments);`
fn create_new_expr<'a>(
    uid: &BoundIdentifier<'a>,
    callee_id: &BoundIdentifier<'a>,
    arguments: ArenaVec<'a, Argument<'a>>,
    ctx: &mut TraverseCtx<'a>,
) -> Statement<'a> {
    stmt_const_decl(
        uid,
        Expression::NewExpression(ctx.ast.alloc_new_expression(
            SPAN,
            callee_id.create_read_expression(ctx),
            NONE,
            arguments,
        )),
        ctx,
    )
}

fn is_import_meta_url<'a>(expr: &Argument<'a>) -> bool {
    if let Argument::StaticMemberExpression(url) = expr
        && url.property.name == "url"
    {
        if let Expression::MetaProperty(meta) = &url.object
            && meta.meta.name == "import"
            && meta.property.name == "meta"
        {
            return true;
        }
    }
    false
}
