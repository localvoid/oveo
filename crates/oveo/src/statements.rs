use std::{cell::RefCell, collections::hash_map::Entry};

use oxc_traverse::Traverse;
use rustc_hash::FxHashMap;

use oxc_allocator::{Address, GetAddress, Vec as ArenaVec};
use oxc_ast::ast::*;

use crate::context::{TraverseCtx, TraverseCtxState};

pub struct Statements<'a> {
    top_level: RefCell<Vec<Statement<'a>>>,
    modifications: RefCell<FxHashMap<Address, StatementModification<'a>>>,
}

impl<'a> Statements<'a> {
    pub fn new() -> Self {
        Self { top_level: RefCell::default(), modifications: RefCell::default() }
    }

    pub fn insert_top_level_statement(&self, stmt: Statement<'a>) {
        self.top_level.borrow_mut().push(stmt);
    }

    #[inline]
    pub fn insert_before<A: GetAddress>(&self, target: &A, stmt: Statement<'a>) {
        self.insert_before_address(target.address(), stmt);
    }

    fn insert_before_address(&self, target: Address, stmt: Statement<'a>) {
        let mut insertions = self.modifications.borrow_mut();
        let modifications = insertions.entry(target).or_default();
        modifications.insertions.push(stmt);
    }

    #[inline]
    pub fn remove<A: GetAddress>(&self, target: A) {
        self.remove_address(target.address());
    }

    pub fn remove_address(&self, target: Address) {
        let mut modifications = self.modifications.borrow_mut();
        match modifications.entry(target) {
            Entry::Occupied(entry) => {
                entry.into_mut().remove = true;
            }
            Entry::Vacant(entry) => {
                entry.insert(StatementModification { insertions: Vec::new(), remove: true });
            }
        }
    }
}

impl<'a> Traverse<'a, TraverseCtxState<'a>> for Statements<'a> {
    fn exit_statements(
        &mut self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let modifications = &mut self.modifications.borrow_mut();
        if modifications.is_empty() {
            return;
        }

        let mut new_statement_count = statements.len();
        let mut dirty = false;
        for s in statements.iter() {
            if let Some(m) = modifications.get(&s.address()) {
                new_statement_count += m.insertions.len();
                if m.remove {
                    new_statement_count -= 1;
                }
                dirty = true;
            }
        }
        if !dirty {
            return;
        }

        let mut new_statements = ctx.ast.vec_with_capacity(new_statement_count);

        for stmt in statements.drain(..) {
            match modifications.remove(&stmt.address()) {
                Some(modifications) => {
                    new_statements.extend(modifications.insertions.into_iter());
                    if !modifications.remove {
                        new_statements.push(stmt);
                    }
                }
                _ => {
                    new_statements.push(stmt);
                }
            }
        }

        *statements = new_statements;
    }

    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        debug_assert!(self.modifications.borrow().is_empty());

        let stmts = &mut self.top_level.borrow_mut();
        if stmts.is_empty() {
            return;
        }

        // Insert statements before the first non-import statement
        let index = program
            .body
            .iter()
            .position(|stmt| !matches!(stmt, Statement::ImportDeclaration(_)))
            .unwrap_or(program.body.len());

        program.body.splice(index..index, stmts.drain(..));
    }
}

#[derive(Default, Debug)]
struct StatementModification<'a> {
    insertions: Vec<Statement<'a>>,
    remove: bool,
}
