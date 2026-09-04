use std::sync::Arc;

use oxc_allocator::{Address, Allocator, GetAddress, TakeIn, Vec as ArenaVec};
use oxc_ast::{ast::*, builder::AstBuilder};
use oxc_semantic::{Scoping, SymbolFlags};
use oxc_span::{GetSpan, SPAN};
use oxc_traverse::{Traverse, traverse_mut};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    OptimizerOptions,
    annotation::Annotation,
    comments::{CommentAnnotation, build_comment_annotations},
    context::{TraverseCtx, TraverseCtxState},
    externs::{ExternMap, ExternValue, INTRINSICS_MODULE_NAME, IntrinsicFunction},
    module::{
        externs::Externs,
        hoist::{
            HoistArgument, HoistExpr, HoistScope, HoistStackEntry, HoistStackEntryKind,
            reduce_hoistable_scope,
        },
    },
    statements::Statements,
};

mod externs;
mod hoist;

pub fn optimize_module<'a>(
    program: &mut Program<'a>,
    source_text: &str,
    options: &OptimizerOptions,
    externs: &ExternMap,
    allocator: &'a Allocator,
    scoping: Scoping,
) {
    let comment_annotations = build_comment_annotations(source_text, &program.comments);
    let mut optimizer = ModuleOptimizer::new(options, externs, comment_annotations);
    traverse_mut(&mut optimizer, allocator, program, scoping, TraverseCtxState::default());
    strip_annotation_comments(program, source_text);
}

struct ModuleOptimizer<'a, 'ctx> {
    options: &'ctx OptimizerOptions,
    statements: Statements<'a>,
    externs: Externs<'ctx>,

    hoist_arguments: Vec<HoistArgument>,
    hoist_scope_expressions: FxHashSet<Address>,

    hoist_stack: Vec<HoistStackEntry>,
    hoistable_expr_stack: Vec<HoistExpr>,

    comment_annotations: FxHashMap<u32, CommentAnnotation>,
    comment_const_addresses: FxHashSet<Address>,
}

impl<'a, 'ctx> ModuleOptimizer<'a, 'ctx> {
    pub fn new(
        options: &'ctx OptimizerOptions,
        extern_map: &'ctx ExternMap,
        comment_annotations: FxHashMap<u32, CommentAnnotation>,
    ) -> Self {
        Self {
            options,
            statements: Statements::new(),
            externs: Externs::new(extern_map),
            hoist_arguments: Vec::new(),
            hoist_scope_expressions: FxHashSet::default(),
            hoist_stack: Vec::new(),
            hoistable_expr_stack: Vec::new(),
            comment_annotations,
            comment_const_addresses: FxHashSet::default(),
        }
    }

    /// Shared finalization for a hoisted expression: reduces the outer
    /// hoistable scope, optionally marks the expression for dedupe, and moves
    /// it into a generated `const _HOISTED_` before the target statement.
    fn finish_hoisted_expr(
        &mut self,
        s: HoistExpr,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
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
            *expr = annotate(expr.take_in(ctx), Annotation::dedupe(), &mut ctx.ast);
        }
        let Some(hoist_scope_id) = s.hoist_scope_id else {
            return;
        };

        let uid = ctx.generate_uid("_HOISTED_", hoist_scope_id, SymbolFlags::ConstVariable);

        // const _HOISTED_ = expr;
        let hoisted_var_decl = Declaration::VariableDeclaration(VariableDeclaration::boxed(
            SPAN,
            VariableDeclarationKind::Const,
            ArenaVec::from_value_in(
                VariableDeclarator::new(
                    SPAN,
                    BindingPattern::BindingIdentifier(BindingIdentifier::boxed(
                        SPAN, uid.name, ctx,
                    )),
                    None,
                    Some(expr.take_in(ctx)),
                    false,
                    ctx,
                ),
                ctx,
            ),
            false,
            ctx,
        ));
        *expr = uid.create_read_expression(ctx);

        if let Some(scope) = self.hoist_stack.iter().find(|x| x.scope_id == hoist_scope_id) {
            if let HoistStackEntryKind::Scope(scope) = &scope.kind {
                if let Some(address) = scope.current_statement {
                    self.statements.insert_before(&address, hoisted_var_decl.into());
                }
            }
        }
    }
}

fn is_hoistable_expression(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::NewExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::CallExpression(_)
    )
}

/// Removes consumed annotation comments (comments containing `@__HOIST__`,
/// `@__SCOPE__`, or `@__CONST__`) so they don't leak into the emitted code.
fn strip_annotation_comments(program: &mut Program, source_text: &str) {
    use crate::comments::is_annotation_content;
    program.comments.retain(|comment| {
        if !comment.is_leading() {
            return true;
        }
        !is_annotation_content(comment.content_span().source_text(source_text))
    });
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

            let mut new_stmts = ArenaVec::with_capacity_in(new_stmts_len, ctx);

            for mut s in node.drain(..) {
                if let Statement::VariableDeclaration(decl) = &mut s
                    && decl.declarations.len() > 1
                {
                    let span = decl.span;
                    let kind = decl.kind;
                    let declare = decl.declare;
                    new_stmts.extend(decl.declarations.drain(..).map(|d| {
                        Declaration::VariableDeclaration(VariableDeclaration::boxed(
                            span,
                            kind,
                            ArenaVec::from_value_in(d, ctx),
                            declare,
                            ctx,
                        ))
                        .into()
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
            Expression::CallExpression(call_expr) => {
                if self.options.hoist && !call_expr.arguments.is_empty() {
                    // Hoist expressions
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

        // Comment-based metadata (`/*@__HOIST__*/`, `/*@__SCOPE__*/`,
        // `/*@__CONST__*/`).
        if (self.options.hoist || self.options.dedupe)
            && let Some(annotation) = self.comment_annotations.get(&node.span().start).copied()
        {
            match annotation {
                CommentAnnotation::Scope => {
                    if self.options.hoist
                        && matches!(
                            node,
                            Expression::ArrowFunctionExpression(_)
                                | Expression::FunctionExpression(_)
                        )
                    {
                        self.hoist_scope_expressions.insert(node.address());
                    }
                }
                CommentAnnotation::Hoist => {
                    if self.options.hoist && is_hoistable_expression(node) {
                        let root_scope_id = ctx.scoping().root_scope_id();
                        let scope_id = ctx.current_hoist_scope_id();
                        if root_scope_id != scope_id {
                            self.hoistable_expr_stack.push(HoistExpr {
                                address: node.address(),
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
                CommentAnnotation::Const => {
                    if self.options.dedupe {
                        self.comment_const_addresses.insert(node.address());
                    }
                }
            }
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

        // Comment-driven hoist finalization. Runs after the intrinsic match
        // above, so a node replaced here (e.g. with the internal
        // `__oveo__` marker) is not passed through call-expression handling.
        if self.options.hoist {
            let address = node.address();
            if let Some(s) = self.hoistable_expr_stack.pop_if(|s| s.address == address) {
                self.hoist_stack.pop();
                self.finish_hoisted_expr(s, node, ctx);
                return;
            }
        }

        // Comment-driven const annotation (`/*@__CONST__*/expr` behaves like
        // `dedupe(expr)` and is lowered to the internal `__oveo__` marker for
        // the chunk phase).
        {
            let address = node.address();
            if self.comment_const_addresses.remove(&address) && self.options.dedupe {
                let expr = node.take_in(ctx);
                *node = annotate(expr, Annotation::dedupe(), &mut ctx.ast);
            }
        }
    }

    fn enter_arrow_function_body(
        &mut self,
        node: &mut ArrowFunctionBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if matches!(node, ArrowFunctionBody::FunctionBody(_)) {
            return;
        }
        if self.options.hoist {
            // push hoist scope
            let parent = ctx.parent();
            {
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

    fn exit_arrow_function_body(
        &mut self,
        node: &mut ArrowFunctionBody<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        if matches!(node, ArrowFunctionBody::FunctionBody(_)) {
            return;
        }
        if self.options.hoist {
            // pop hoist scope
            self.hoist_stack.pop();
        }
    }

    fn enter_function_body(&mut self, _node: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.hoist {
            // push hoist scope
            let parent = ctx.parent();
            {
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
                        if let Some(expr) = node.as_expression() {
                            if is_hoistable_expression(expr) {
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

                self.finish_hoisted_expr(s, expr, ctx);
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
                        self.externs.insert(
                            spec.local.symbol_id(),
                            ExternValue::Namespace(Arc::clone(&module)),
                        );
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
    Expression::CallExpression(CallExpression::boxed(
        SPAN,
        Expression::Identifier(IdentifierReference::boxed(SPAN, Annotation::ID_NAME, ast)),
        None,
        ArenaVec::from_array_in(
            [
                expr.into(),
                Expression::NumericLiteral(NumericLiteral::boxed(
                    SPAN,
                    annotation.flags as f64,
                    None,
                    NumberBase::Decimal,
                    ast,
                ))
                .into(),
            ],
            ast,
        ),
        false,
        ast,
    ))
}

fn unwrap_call_expr<'a>(expr: &mut CallExpression<'a>, ast: &mut AstBuilder<'a>) -> Expression<'a> {
    if let Some(arg) = expr.arguments.pop() {
        arg.into_expression()
    } else {
        Expression::new_void_0(SPAN, ast)
    }
}
