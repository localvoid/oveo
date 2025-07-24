//! Terminology:
//!
//! - "Hoist Scope" - scope that can contain Hoisted Expressions. By default,
//!   there is only a program level scope. Additional scopes can be created with
//!   the intrinsic function `scope()`.
//! - "Hoisted Expression" - expression that should be hoisted to the outermost
//!   Hoist Scope.
//! - "Hoisted Expression Scope" - scopes created inside of a hoisted
//!   expression.
//! - "Inner Scope" - the closest Hoist Scope.
//! - "Outer Scopes" - scopes outside of the closest Hoist Scope.
//!
//! ```js
//!                         // outer scope (hoist scope)
//! {                       // outer scope
//!   scope((a) => {        // inner scope (hoist scope)
//!     return () => {      // inner scope function
//!       if (a) {          // conditional path prevents expr from hoisting
//!         hoist((i) => {  // hoisted expr
//!                         // hoisted expr scope
//!           i();          // symbol from hoisted expr scope is ignored
//!           a();          // symbol from the inner scope
//!         });
//!       }
//!     };
//!   })
//! }
//! ```
//!
//! Hoisting heuristics:
//!
//! - All symbols should be accessible from the Hoist Scope.
//! - Hoisted expression should have a type:
//!   - `ArrowFunctionExpression`
//!   - `FunctionExpression`
//!   - `CallExpression`
//!   - `NewExpression`
//!   - `ObjectExpression`
//!   - `ArrayExpression`
//!   - `TaggedTemplateExpression`
//! - No conditionals on the path to the Hoist Scope.
//!   - `ConditionalExpression`
//!   - `IfStatement`
//!   - `SwitchStatement`
//! - Expressions hoisted to the Inner Scope should be inside of a function
//!   scope.
//!

use oxc_allocator::Address;
use oxc_semantic::{ScopeId, Scoping};

#[derive(Debug)]
pub struct HoistStackEntry {
    pub scope_id: ScopeId,
    pub kind: HoistStackEntryKind,
}

#[derive(Debug)]
pub enum HoistStackEntryKind {
    Scope(HoistScope),
    FunctionBody,
    HoistExpr,
    Conditional,
}

pub struct HoistArgument {
    pub address: Address,
    pub hoist: bool,
    pub scope: bool,
}

#[derive(Debug)]
pub struct HoistScope {
    pub current_statement: Option<Address>,
}

pub struct HoistExpr {
    pub address: Address,
    pub outermost_scope_id: ScopeId,
    pub hoist_scope_id: Option<ScopeId>,
}

#[derive(Debug)]
enum State {
    HoistedExpr,
    Inner,
    Outer,
}

pub fn reduce_hoistable_scope(
    expr: &mut HoistExpr,
    scoping: &Scoping,
    current_scope_id: ScopeId,
    sym_scope_id: ScopeId,
    hoist_scopes: &[HoistStackEntry],
) {
    let outermost_scope_id = expr.outermost_scope_id;
    // Ignore symbols outside of the outermost scope id
    if sym_scope_id < outermost_scope_id {
        return;
    }
    let mut hoist_scopes = hoist_scopes.iter().rev().peekable();

    let mut state = State::HoistedExpr;
    let mut inner_hoist_scope = true;
    let mut inner_inside_func = false;
    let mut conditional = false;

    let mut current_hoist_scope_id = None;
    for ancestor_scope_id in scoping.scope_ancestors(current_scope_id) {
        'hoist_scopes: loop {
            let Some(entry) = hoist_scopes.peek() else {
                if cfg!(debug_assertions) {
                    panic!(
                        "there should be at least one hoist scope left before reaching the root scope"
                    );
                }
                return;
            };
            match state {
                // Inside of the Hoisted Expression
                State::HoistedExpr => {
                    match entry.kind {
                        HoistStackEntryKind::Scope(_) | HoistStackEntryKind::FunctionBody => {
                            if ancestor_scope_id == entry.scope_id {
                                hoist_scopes.next();
                            }
                        }
                        HoistStackEntryKind::HoistExpr => {
                            state = State::Inner;
                            hoist_scopes.next();
                            continue 'hoist_scopes;
                        }
                        HoistStackEntryKind::Conditional => {
                            hoist_scopes.next();
                            continue 'hoist_scopes;
                        }
                    }
                    if ancestor_scope_id == sym_scope_id {
                        return;
                    }
                }
                // Inside of the Inner Scope
                State::Inner => match &entry.kind {
                    HoistStackEntryKind::Scope(_) => {
                        if ancestor_scope_id == entry.scope_id {
                            hoist_scopes.next();
                            current_hoist_scope_id = Some(ancestor_scope_id);
                            state = State::Outer;
                        }
                    }
                    HoistStackEntryKind::FunctionBody => {
                        if ancestor_scope_id == entry.scope_id {
                            hoist_scopes.next();
                            inner_inside_func = true;
                        }
                    }
                    HoistStackEntryKind::HoistExpr => {
                        hoist_scopes.next();
                        continue 'hoist_scopes;
                    }
                    HoistStackEntryKind::Conditional => {
                        hoist_scopes.next();
                        conditional = true;
                        continue 'hoist_scopes;
                    }
                },
                // Outside of the Inner Scope
                State::Outer => match &entry.kind {
                    HoistStackEntryKind::Scope(_) => {
                        if ancestor_scope_id == entry.scope_id {
                            hoist_scopes.next();
                            current_hoist_scope_id = Some(ancestor_scope_id);
                            inner_hoist_scope = false;
                        }
                    }
                    HoistStackEntryKind::FunctionBody => {
                        if ancestor_scope_id == entry.scope_id {
                            hoist_scopes.next();
                        }
                    }
                    HoistStackEntryKind::HoistExpr => {
                        hoist_scopes.next();
                        continue 'hoist_scopes;
                    }
                    HoistStackEntryKind::Conditional => {
                        hoist_scopes.next();
                        conditional = true;
                        continue 'hoist_scopes;
                    }
                },
            }
            if ancestor_scope_id == sym_scope_id {
                expr.outermost_scope_id = ancestor_scope_id;
                if conditional || (inner_hoist_scope && !inner_inside_func) {
                    expr.hoist_scope_id = None;
                } else {
                    expr.hoist_scope_id = current_hoist_scope_id;
                }
                return;
            }
            break 'hoist_scopes;
        }
    }
}
