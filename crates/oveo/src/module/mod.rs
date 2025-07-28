use oxc_allocator::{Address, Allocator, GetAddress, TakeIn, Vec as ArenaVec};
use oxc_ast::{AstBuilder, NONE, ast::*};
use oxc_semantic::{Scoping, SymbolFlags};
use oxc_span::SPAN;
use oxc_traverse::{Traverse, traverse_mut};
use rustc_hash::FxHashSet;

use crate::{
    OptimizerOptions,
    annotation::Annotation,
    context::{TraverseCtx, TraverseCtxState},
    externs::{ExternMap, ExternValue, INTRINSICS_MODULE_NAME, IntrinsicFunction},
    module::{
        externs::Externs,
        hoist::{
            HoistArgument, HoistExpr, HoistScope, HoistStackEntry, HoistStackEntryKind,
            reduce_hoistable_scope,
        },
        json::json_into_expr,
    },
    statements::Statements,
};

mod externs;
mod hoist;
mod json;

pub fn optimize_module<'a>(
    program: &mut Program<'a>,
    options: &OptimizerOptions,
    externs: &ExternMap,
    allocator: &'a Allocator,
    scoping: Scoping,
) {
    let mut optimizer = ModuleOptimizer::new(options, externs);
    traverse_mut(&mut optimizer, allocator, program, scoping, TraverseCtxState::default());
}

struct ModuleOptimizer<'a, 'ctx> {
    options: &'ctx OptimizerOptions,
    statements: Statements<'a>,
    externs: Externs<'ctx>,

    hoist_arguments: Vec<HoistArgument>,
    hoist_scope_expressions: FxHashSet<Address>,

    hoist_stack: Vec<HoistStackEntry>,
    hoistable_expr_stack: Vec<HoistExpr>,
}

impl<'ctx> ModuleOptimizer<'_, 'ctx> {
    pub fn new(options: &'ctx OptimizerOptions, extern_map: &'ctx ExternMap) -> Self {
        Self {
            options,
            statements: Statements::new(),
            externs: Externs::new(extern_map),
            hoist_arguments: Vec::new(),
            hoist_scope_expressions: FxHashSet::default(),
            hoist_stack: Vec::new(),
            hoistable_expr_stack: Vec::new(),
        }
    }
}

impl<'a> Traverse<'a, TraverseCtxState<'a>> for ModuleOptimizer<'a, '_> {
    fn enter_program(&mut self, node: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        // push program hoist scope
        if self.options.hoist {
            self.hoist_stack.push(HoistStackEntry {
                scope_id: node.scope_id(),
                kind: HoistStackEntryKind::Scope(HoistScope { current_statement: None }),
            });
        }
    }

    fn exit_program(&mut self, _node: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        // pop program hoist scope
        if self.options.hoist {
            self.hoist_stack.pop();
        }
    }

    fn enter_statements(
        &mut self,
        node: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.hoist {
            // Normalize variable declarations (1 declarator per declaration) to avoid dealing with edge cases like:
            // `const a = 1, b = hoist({ a });`
            let mut new_stmts_len = 0;
            for s in node.iter() {
                if let Statement::VariableDeclaration(decl) = s {
                    new_stmts_len += decl.declarations.len();
                } else {
                    new_stmts_len += 1;
                }
            }

            if node.len() == new_stmts_len {
                return;
            }

            let mut new_stmts = ctx.ast.vec_with_capacity(new_stmts_len);

            for mut s in node.drain(..) {
                if let Statement::VariableDeclaration(decl) = &mut s
                    && decl.declarations.len() > 1
                {
                    let span = decl.span;
                    let kind = decl.kind;
                    let declare = decl.declare;
                    new_stmts.extend(decl.declarations.drain(..).map(|d| {
                        ctx.ast.declaration_variable(span, kind, ctx.ast.vec1(d), declare).into()
                    }));
                } else {
                    new_stmts.push(s);
                }
            }

            *node = new_stmts;
        }
    }

    fn exit_statements(
        &mut self,
        node: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.statements.exit_statements(node, ctx); // update statements
    }

    fn enter_statement(&mut self, node: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.hoist {
            // update current statement
            if let Some(entry) = self.hoist_stack.last_mut() {
                if let HoistStackEntryKind::Scope(scope) = &mut entry.kind {
                    if scope.current_statement.is_none() {
                        scope.current_statement = Some(node.address());
                    }
                }
            }

            match node {
                Statement::IfStatement(_) | Statement::SwitchStatement(_) => {
                    self.hoist_stack.push(HoistStackEntry {
                        scope_id: ctx.current_scope_id(),
                        kind: HoistStackEntryKind::Conditional,
                    });
                }
                _ => {}
            }
        }
    }

    fn exit_statement(&mut self, node: &mut Statement<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.options.hoist {
            if let Some(entry) = self.hoist_stack.last_mut() {
                if let HoistStackEntryKind::Scope(scope) = &mut entry.kind {
                    scope.current_statement = None;
                }
            }
            match node {
                Statement::IfStatement(_) | Statement::SwitchStatement(_) => {
                    self.hoist_stack.pop();
                }
                _ => {}
            }
        }

        // removes `import {} from "oveo"` statements
        if let Statement::ImportDeclaration(import_decl) = node {
            if import_decl.source.value == INTRINSICS_MODULE_NAME {
                self.statements.remove(node.address());
            }
        }
    }

    fn enter_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match node {
            Expression::Identifier(_) | Expression::StaticMemberExpression(_) => {
                if self.options.externs.inline_const_values {
                    // Inline extern consts
                    if let Some(ExternValue::Const(v)) = self.externs.resolve(node, ctx) {
                        *node = json_into_expr(&v.value, &mut ctx.ast);
                    }
                }
            }
            Expression::CallExpression(call_expr) => {
                if self.options.hoist {
                    // Hoist expressions
                    if call_expr.arguments.is_empty() {
                        return;
                    }
                    if let Some(ExternValue::Function(f)) =
                        self.externs.resolve(&call_expr.callee, ctx)
                    {
                        for (i, meta) in f.arguments.iter().enumerate() {
                            if meta.hoist || meta.scope {
                                if let Some(arg) = call_expr.arguments.get(i) {
                                    self.hoist_arguments.push(HoistArgument {
                                        address: arg.address(),
                                        hoist: meta.hoist,
                                        scope: meta.scope,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Expression::ConditionalExpression(_) => {
                if self.options.hoist {
                    self.hoist_stack.push(HoistStackEntry {
                        scope_id: ctx.current_scope_id(),
                        kind: HoistStackEntryKind::Conditional,
                    });
                }
            }
            _ => {}
        }
    }

    fn exit_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match node {
            // Intrinsic functions
            Expression::CallExpression(expr) => {
                if let Some(ExternValue::Function(f)) = self.externs.resolve(&expr.callee, ctx) {
                    if let Some(intrinsic) = &f.intrinsic {
                        match intrinsic {
                            IntrinsicFunction::Hoist | IntrinsicFunction::Scope => {
                                *node = unwrap_call_expr(expr, &mut ctx.ast);
                            }
                            IntrinsicFunction::Dedupe => {
                                if self.options.dedupe
                                    && let Some(arg) = expr.arguments.pop()
                                {
                                    *node = annotate(
                                        arg.into_expression(),
                                        Annotation::dedupe(),
                                        &mut ctx.ast,
                                    );
                                } else {
                                    *node = unwrap_call_expr(expr, &mut ctx.ast);
                                }
                            }
                            IntrinsicFunction::Key => {
                                if self.options.rename_properties
                                    && let Some(arg) = expr.arguments.pop()
                                {
                                    *node = annotate(
                                        arg.into_expression(),
                                        Annotation::key(),
                                        &mut ctx.ast,
                                    );
                                } else {
                                    *node = unwrap_call_expr(expr, &mut ctx.ast);
                                }
                            }
                        }
                    }
                }
            }
            Expression::ConditionalExpression(_) => {
                if self.options.hoist {
                    self.hoist_stack.pop();
                }
            }
            _ => {}
        }
    }

    fn enter_function_body(&mut self, _node: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.hoist {
            // push hoist scope
            let parent = ctx.parent();
            if parent.is_arrow_function_expression() {
                let address = parent.address();
                if self.hoist_scope_expressions.remove(&address) {
                    self.hoist_stack.push(HoistStackEntry {
                        scope_id: ctx.current_scope_id(),
                        kind: HoistStackEntryKind::Scope(HoistScope { current_statement: None }),
                    });
                    return;
                }
            }
            self.hoist_stack.push(HoistStackEntry {
                scope_id: ctx.current_scope_id(),
                kind: HoistStackEntryKind::FunctionBody,
            });
        }
    }

    fn exit_function_body(&mut self, _node: &mut FunctionBody<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.options.hoist {
            // pop hoist scope
            self.hoist_stack.pop();
        }
    }

    fn enter_argument(&mut self, node: &mut Argument<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.hoist {
            let address = node.address();
            if let Some((i, arg)) =
                self.hoist_arguments.iter().enumerate().find(|(_, arg)| arg.address == address)
            {
                if arg.scope {
                    if let Some(Expression::ArrowFunctionExpression(expr)) = node.as_expression() {
                        let addr = expr.address();
                        self.hoist_scope_expressions.insert(addr);
                    }
                }
                if arg.hoist {
                    let root_scope_id = ctx.scoping().root_scope_id();
                    let scope_id = ctx.current_hoist_scope_id();
                    if root_scope_id != scope_id {
                        if let Expression::ArrowFunctionExpression(_)
                        | Expression::FunctionExpression(_)
                        | Expression::NewExpression(_)
                        | Expression::ObjectExpression(_)
                        | Expression::ArrayExpression(_)
                        | Expression::TaggedTemplateExpression(_)
                        | Expression::CallExpression(_) = node.to_expression()
                        {
                            self.hoistable_expr_stack.push(HoistExpr {
                                address,
                                outermost_scope_id: root_scope_id,
                                hoist_scope_id: Some(root_scope_id),
                            });
                            self.hoist_stack.push(HoistStackEntry {
                                scope_id: ctx.current_scope_id(),
                                kind: HoistStackEntryKind::HoistExpr,
                            });
                        }
                    }
                }
                self.hoist_arguments.remove(i);
            }
        }
    }

    fn exit_argument(&mut self, node: &mut Argument<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.hoist {
            if let Some(s) = self.hoistable_expr_stack.last() {
                let address = node.address();
                if s.address != address {
                    return;
                }
                self.hoist_stack.pop();
                let s = self.hoistable_expr_stack.pop().unwrap();

                let Some(expr) = node.as_expression_mut() else {
                    return;
                };

                // Outer hoistable expr scope should be reduced to the outermost
                // scope of the inner hoistable expr.
                if let Some(last) = self.hoistable_expr_stack.last_mut() {
                    reduce_hoistable_scope(
                        last,
                        ctx.scoping(),
                        ctx.current_scope_id(),
                        s.outermost_scope_id,
                        &self.hoist_stack,
                    );
                }
                if self.options.dedupe {
                    *expr = annotate(
                        expr.take_in(ctx.ast.allocator),
                        Annotation::dedupe(),
                        &mut ctx.ast,
                    );
                }
                let Some(hoist_scope_id) = s.hoist_scope_id else {
                    return;
                };

                let uid = ctx.generate_uid("_HOISTED_", hoist_scope_id, SymbolFlags::ConstVariable);

                // const _HOISTED_ = expr;
                let hoisted_var_decl = ctx.ast.declaration_variable(
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
                        Some(expr.take_in(ctx.ast.allocator)),
                        false,
                    )),
                    false,
                );
                *expr = uid.create_read_expression(ctx);

                if let Some(scope) = self.hoist_stack.iter().find(|x| x.scope_id == hoist_scope_id)
                {
                    if let HoistStackEntryKind::Scope(scope) = &scope.kind {
                        if let Some(address) = scope.current_statement {
                            self.statements.insert_before(&address, hoisted_var_decl.into());
                        }
                    }
                }
            }
        }
    }

    fn enter_identifier_reference(
        &mut self,
        node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.hoist {
            // Checks identifiers and reduces hoistable scope
            if let Some(expr) = self.hoistable_expr_stack.last_mut() {
                let r = ctx.scoping().get_reference(node.reference_id());
                if let Some(symbol_id) = r.symbol_id() {
                    let sym_scope_id = ctx.scoping().symbol_scope_id(symbol_id);
                    reduce_hoistable_scope(
                        expr,
                        ctx.scoping(),
                        ctx.current_scope_id(),
                        sym_scope_id,
                        &self.hoist_stack,
                    );
                }
            }
        }
    }

    fn exit_import_declaration(
        &mut self,
        node: &mut ImportDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        // Resolve extern modules
        if let Some(specifiers) = &node.specifiers {
            let source = &node.source;
            let Some(module) = self.externs.modules().get(source.value.as_str()).cloned() else {
                return;
            };

            for spec in specifiers {
                match spec {
                    // import { imported } from "source"
                    // import { imported as local } from "source"
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        if let Some(v) = module.exports.get(spec.imported.name().as_str()) {
                            self.externs.insert(spec.local.symbol_id(), v.clone());
                        }
                    }
                    // import local from "source"
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        if let Some(v) = module.exports.get("default") {
                            self.externs.insert(spec.local.symbol_id(), v.clone());
                        }
                    }
                    // import * as local from "source"
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        self.externs
                            .insert(spec.local.symbol_id(), ExternValue::Namespace(module.clone()));
                    }
                }
            }
        }
    }
}

// __oveo__(expr, annotation_flags)
fn annotate<'a>(
    expr: Expression<'a>,
    annotation: Annotation,
    ast: &mut AstBuilder<'a>,
) -> Expression<'a> {
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, Annotation::ID_NAME),
        NONE,
        ast.vec_from_array([
            expr.into(),
            ast.expression_numeric_literal(
                SPAN,
                annotation.flags as f64,
                None,
                NumberBase::Decimal,
            )
            .into(),
        ]),
        false,
    )
}

fn unwrap_call_expr<'a>(expr: &mut CallExpression<'a>, ast: &mut AstBuilder<'a>) -> Expression<'a> {
    if let Some(arg) = expr.arguments.pop() { arg.into_expression() } else { ast.void_0(SPAN) }
}
