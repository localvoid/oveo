use oxc_allocator::{Address, Allocator, GetAddress, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_semantic::{ReferenceFlags, Scoping, SymbolFlags, SymbolId};
use oxc_span::{GetSpan, SPAN};
use oxc_traverse::{BoundIdentifier, Traverse, traverse_mut};
use rustc_hash::FxHashMap;

mod dedupe;

use crate::{
    OptimizerOptions,
    annotation::Annotation,
    chunk::dedupe::{DedupeKind, DedupeState, dedupe_hash},
    comments::{CommentAnnotation, build_comment_annotations, is_annotation_content},
    context::{TraverseCtx, TraverseCtxState},
    globals::{GlobalValue, get_global_value},
    property_names::LocalPropertyMap,
    statements::Statements,
};

pub fn optimize_chunk<'a, 'ctx>(
    program: &mut Program<'a>,
    source_text: &str,
    options: &OptimizerOptions,
    property_map: LocalPropertyMap<'a, 'ctx>,
    allocator: &'a Allocator,
    scoping: Scoping,
) {
    let comment_annotations = build_comment_annotations(source_text, &program.comments);
    let mut optimizer = ChunkOptimizer::new(options, property_map, comment_annotations);
    let scoping =
        traverse_mut(&mut optimizer, allocator, program, scoping, TraverseCtxState::default());
    program.comments.retain(|comment| {
        if !comment.is_leading() {
            return true;
        }
        !is_annotation_content(comment.content_span().source_text(source_text))
    });
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
    comment_annotations: FxHashMap<u32, CommentAnnotation>,
    globals_symbols: FxHashMap<SymbolId, &'ctx GlobalValue>,
    globals_ids: FxHashMap<*const GlobalValue, BoundIdentifier<'a>>,
    singletons: FxHashMap<*const GlobalValue, BoundIdentifier<'a>>,
    dedupe: DedupeState,
}

impl<'a, 'ctx> ChunkOptimizer<'a, 'ctx> {
    fn new(
        options: &'ctx OptimizerOptions,
        property_map: LocalPropertyMap<'a, 'ctx>,
        comment_annotations: FxHashMap<u32, CommentAnnotation>,
    ) -> Self {
        Self {
            options,
            property_map,
            statements: Statements::new(),
            annotations: Vec::new(),
            comment_annotations,
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
        // Replaces `new URL("./url", import.meta.url).href` (also `.pathname`,
        // `["href"]` / `["pathname"]`, and zero-argument `.toString()`) with an
        // absolute URL.
        if let Some(base_url) = &self.options.url {
            let rel: Option<&str> = match node {
                Expression::StaticMemberExpression(expr)
                    if !expr.optional
                        && (expr.property.name == "href" || expr.property.name == "pathname") =>
                {
                    if let Expression::NewExpression(new_expr) = &expr.object {
                        get_new_url_rel(new_expr, ctx.scoping())
                    } else {
                        None
                    }
                }
                Expression::ComputedMemberExpression(expr) if !expr.optional => {
                    let prop: Option<&str> = match &expr.expression {
                        Expression::StringLiteral(s) => Some(s.value.as_str()),
                        Expression::TemplateLiteral(t)
                            if t.expressions.is_empty() && t.quasis.len() == 1 =>
                        {
                            t.quasis.first().and_then(|q| q.value.cooked.as_ref()).map(|s| {
                                s.as_str()
                            })
                        }
                        _ => None,
                    };
                    match prop {
                        Some("href") | Some("pathname") => {
                            if let Expression::NewExpression(new_expr) = &expr.object {
                                get_new_url_rel(new_expr, ctx.scoping())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
                Expression::CallExpression(call)
                    if !call.optional
                        && call.type_arguments.is_none()
                        && call.arguments.is_empty() =>
                {
                    let url_object: Option<&Expression<'a>> = match &call.callee {
                        Expression::StaticMemberExpression(m)
                            if !m.optional && m.property.name == "toString" =>
                        {
                            Some(&m.object)
                        }
                        Expression::ComputedMemberExpression(m) if !m.optional => match &m.expression
                        {
                            Expression::StringLiteral(s) if s.value.as_str() == "toString" => {
                                Some(&m.object)
                            }
                            _ => None,
                        },
                        _ => None,
                    };
                    match url_object {
                        Some(Expression::NewExpression(new_expr)) => {
                            get_new_url_rel(new_expr, ctx.scoping())
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
            if let Some(rel_url) = rel {
                *node = Expression::StringLiteral(StringLiteral::boxed(
                    SPAN,
                    Str::from_strs_array_in(
                        [base_url, rel_url.strip_prefix("./").unwrap_or(rel_url)],
                        ctx,
                    ),
                    None,
                    ctx,
                ));
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
                                from_comment: false,
                            });
                        }
                    }
                }
            }
            // Comment-based const annotation (`/*@__CONST__*/expr`).
            if self.options.dedupe
                && matches!(
                    self.comment_annotations.get(&node.span().start),
                    Some(CommentAnnotation::Const)
                )
            {
                self.annotations.push(AnnotatedExpr {
                    address,
                    annotation: Annotation::dedupe(),
                    from_comment: true,
                });
            }
        }
    }

    fn exit_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.globals.hoist || self.options.globals.singletons {
            'hoist_globals: {
                match node {
                    // Replaces global identifier with a reference to a const symbol.
                    Expression::Identifier(expr) => {
                        if !self.options.globals.hoist {
                            break 'hoist_globals;
                        }
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
                                                Expression::Identifier(IdentifierReference::boxed(
                                                    SPAN, expr.name, ctx,
                                                )),
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
                        if !self.options.globals.hoist {
                            break 'hoist_globals;
                        }
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
                                                        expr.property.name.into(),
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
                    // Only zero-argument constructions are hoisted: `TextDecoder` accepts
                    // `label`/`options` and `decode()` accepts `{ stream: true }`, so
                    // sharing an instance constructed with arguments (or reusing one
                    // across streaming decodes) would change semantics.
                    Expression::NewExpression(expr) => {
                        if !self.options.globals.singletons {
                            break 'hoist_globals;
                        }
                        if !expr.arguments.is_empty() || expr.type_arguments.is_some() {
                            break 'hoist_globals;
                        }
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
                                                        ArenaVec::new_in(ctx),
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
                if a.from_comment {
                    let _ = dedupe_hash(&mut self.dedupe, node, ctx.scoping());
                    return;
                }
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
            *node = Expression::new_void_0(SPAN, ctx);
        }
    }

    fn exit_identifier_name(&mut self, node: &mut IdentifierName<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.rename_properties {
            if let Some(v) = self.property_map.get(node.name.into(), &ctx.ast) {
                node.name = v.into();
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
    from_comment: bool,
}

// `const uid = expr;`
fn stmt_const_decl<'a>(
    uid: &BoundIdentifier<'a>,
    expr: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Statement<'a> {
    Statement::VariableDeclaration(VariableDeclaration::boxed(
        SPAN,
        VariableDeclarationKind::Const,
        ArenaVec::from_value_in(
            VariableDeclarator::new(
                SPAN,
                BindingPattern::BindingIdentifier(BindingIdentifier::boxed(SPAN, uid.name, ctx)),
                None,
                Some(expr),
                false,
                ctx,
            ),
            ctx,
        ),
        false,
        ctx,
    ))
}

// `const uid = object_id.property_name;`
fn create_static_member_decl<'a>(
    uid: &BoundIdentifier<'a>,
    object_id: &BoundIdentifier<'a>,
    property_name: Str<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Statement<'a> {
    stmt_const_decl(
        uid,
        Expression::StaticMemberExpression(StaticMemberExpression::boxed(
            SPAN,
            object_id.create_expression(ReferenceFlags::read(), ctx),
            IdentifierName::new(SPAN, property_name, ctx),
            false,
            ctx,
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
        Expression::NewExpression(NewExpression::boxed(
            SPAN,
            callee_id.create_read_expression(ctx),
            None,
            arguments,
            ctx,
        )),
        ctx,
    )
}

fn get_new_url_rel<'a>(new_expr: &NewExpression<'a>, scoping: &Scoping) -> Option<&'a str> {
    if new_expr.type_arguments.is_some() {
        return None;
    }
    // Only rewrite the global `URL` constructor. A shadowed local `URL`
    // (import, parameter, declaration) must keep its runtime semantics.
    if let Expression::Identifier(callee) = &new_expr.callee {
        if callee.name != "URL" {
            return None;
        }
        if scoping.get_reference(callee.reference_id()).symbol_id().is_some() {
            return None;
        }
    } else {
        return None;
    }
    let args = &new_expr.arguments;
    if args.len() != 2 {
        return None;
    }
    let rel = match &args[0] {
        Argument::StringLiteral(s) => s.value.as_str(),
        Argument::TemplateLiteral(t) if t.expressions.is_empty() && t.quasis.len() == 1 => {
            t.quasis.first()?.value.cooked.as_ref()?.as_str()
        }
        _ => return None,
    };
    if !is_import_meta_url(&args[1]) {
        return None;
    }
    if is_non_rewritable_rel(rel) {
        return None;
    }
    Some(rel)
}

/// Returns `true` when `rel` must be left alone: empty, root-absolute,
/// query/hash-only, or an absolute URL with a scheme (`https:`, `data:`, …).
fn is_non_rewritable_rel(rel: &str) -> bool {
    let bytes = rel.as_bytes();
    if bytes.is_empty() {
        return true;
    }
    let first = bytes[0];
    if first == b'/' || first == b'#' || first == b'?' || first == b'\\' {
        return true;
    }
    // Scheme detection: `^[a-zA-Z][a-zA-Z0-9+.-]*:` before any `/`, `?`, `#`.
    if !first.is_ascii_alphabetic() {
        return false;
    }
    for &c in &bytes[1..] {
        if c == b':' {
            return true;
        }
        if c == b'/' || c == b'?' || c == b'#' {
            return false;
        }
        if !(c.is_ascii_alphanumeric() || c == b'+' || c == b'-' || c == b'.') {
            return false;
        }
    }
    false
}

fn is_import_meta_url<'a>(expr: &Argument<'a>) -> bool {
    match expr {
        Argument::StaticMemberExpression(url) if url.property.name == "url" && !url.optional => {
            matches!(&url.object, Expression::ImportMeta(_))
        }
        Argument::ComputedMemberExpression(url) if !url.optional => {
            let is_url_key = match &url.expression {
                Expression::StringLiteral(s) => s.value.as_str() == "url",
                Expression::TemplateLiteral(t)
                    if t.expressions.is_empty() && t.quasis.len() == 1 =>
                {
                    t.quasis
                        .first()
                        .and_then(|q| q.value.cooked.as_ref())
                        .is_some_and(|s| s.as_str() == "url")
                }
                _ => false,
            };
            is_url_key && matches!(&url.object, Expression::ImportMeta(_))
        }
        _ => false,
    }
}
