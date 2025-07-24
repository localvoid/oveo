use std::sync::Arc;

use oxc_ast::ast::Expression;
use oxc_semantic::SymbolId;
use rustc_hash::FxHashMap;

use crate::{
    context::TraverseCtx,
    externs::{ExternMap, ExternModule, ExternValue},
};

pub struct Externs<'ctx> {
    map: &'ctx ExternMap,
    symbols: FxHashMap<SymbolId, ExternValue>,
}

impl<'ctx> Externs<'ctx> {
    pub fn new(map: &'ctx ExternMap) -> Self {
        Self { map, symbols: FxHashMap::default() }
    }

    pub fn resolve<'a>(&self, node: &Expression<'a>, ctx: &TraverseCtx<'a>) -> Option<ExternValue> {
        match node {
            Expression::Identifier(id) => {
                if let Some(symbold_id) = ctx.scoping().get_reference(id.reference_id()).symbol_id()
                {
                    return self.symbols.get(&symbold_id).cloned();
                }
            }
            Expression::StaticMemberExpression(expr) => {
                if let Some(ExternValue::Namespace(m)) = self.resolve(&expr.object, ctx) {
                    return m.exports.get(expr.property.name.as_str()).cloned();
                }
            }
            _ => {}
        }
        None
    }

    pub fn insert(&mut self, symbol_id: SymbolId, value: ExternValue) {
        self.symbols.insert(symbol_id, value);
    }

    pub fn modules(&self) -> &FxHashMap<String, Arc<ExternModule>> {
        &self.map.modules
    }
}
