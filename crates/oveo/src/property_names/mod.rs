use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::Scoping;
use oxc_traverse::{Traverse, traverse_mut};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    context::{TraverseCtx, TraverseCtxState},
    property_names::base54::base54,
};

mod base54;

pub fn collect_property_names<'a>(
    program: &mut Program<'a>,
    allocator: &'a Allocator,
    scoping: Scoping,
) -> FxHashSet<Atom<'a>> {
    let mut collect = CollectPropertyNames::new();
    traverse_mut(&mut collect, allocator, program, scoping, TraverseCtxState::default());
    collect.names
}

pub struct CollectPropertyNames<'a> {
    names: FxHashSet<Atom<'a>>,
}

impl<'a> CollectPropertyNames<'a> {
    pub fn new() -> Self {
        Self { names: FxHashSet::default() }
    }
}

impl<'a> Traverse<'a, TraverseCtxState<'a>> for CollectPropertyNames<'a> {
    fn exit_static_member_expression(
        &mut self,
        node: &mut StaticMemberExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.names.insert(node.property.name);
    }
}

pub fn generate_unique_names(index: &mut FxHashMap<String, String>, names: &[String]) {
    let mut used = FxHashSet::default();
    add_keywords(&mut used);

    for v in index.values() {
        used.insert(v.clone());
    }

    let mut i = 0;
    for name in names {
        index.entry(name.clone()).or_insert_with(|| {
            loop {
                let uid = base54(i);
                i += 1;
                if used.insert(uid.to_string()) {
                    return uid.to_string();
                }
            }
        });
    }
}

fn add_keywords(index: &mut FxHashSet<String>) {
    index.insert("as".to_string());
    index.insert("do".to_string());
    index.insert("if".to_string());
    index.insert("in".to_string());
    index.insert("is".to_string());
    index.insert("of".to_string());
    index.insert("any".to_string());
    index.insert("for".to_string());
    index.insert("get".to_string());
    index.insert("let".to_string());
    index.insert("new".to_string());
    index.insert("out".to_string());
    index.insert("set".to_string());
    index.insert("try".to_string());
    index.insert("var".to_string());
    index.insert("case".to_string());
    index.insert("else".to_string());
    index.insert("enum".to_string());
    index.insert("from".to_string());
    index.insert("meta".to_string());
    index.insert("null".to_string());
    index.insert("this".to_string());
    index.insert("true".to_string());
    index.insert("type".to_string());
    index.insert("void".to_string());
    index.insert("with".to_string());
}
