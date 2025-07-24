//! Calculates SHA1 hashes for simple expressions.
//!
use oxc_allocator::{Address, GetAddress};
use oxc_ast::ast::*;
use oxc_index::Idx;
use oxc_semantic::Scoping;
use sha1::{Digest, Sha1, Sha1Core, digest::core_api::CoreWrapper};

use crate::chunk::dedupe::DedupeState;

pub fn dedupe_hash<'a>(
    state: &mut DedupeState,
    node: &Expression<'a>,
    scoping: &Scoping,
) -> Option<()> {
    walk_expr(state, None, node, scoping, node.address())?;
    Some(())
}

fn walk_expr<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &Expression<'a>,
    scoping: &Scoping,
    address: Address,
) -> Option<()> {
    match node {
        Expression::BooleanLiteral(node) => walk_boolean_literal(w, node),
        Expression::NullLiteral(_) => walk_null_literal(w),
        Expression::NumericLiteral(node) => walk_numeric_literal(w, node),
        Expression::BigIntLiteral(node) => walk_big_int_literal(state, w, node, address),
        Expression::RegExpLiteral(node) => walk_reg_exp_literal(state, w, node, address),
        Expression::StringLiteral(node) => walk_string_literal(state, w, node, address),
        Expression::TemplateLiteral(node) => {
            walk_template_literal(state, w, node, scoping, address)
        }
        Expression::Identifier(node) => walk_identifier_reference(w, node, scoping),
        Expression::CallExpression(node) => walk_call_expression(state, w, node, scoping, address),
        Expression::ArrayExpression(node) => {
            walk_array_expression(state, w, node, scoping, address)
        }
        Expression::ObjectExpression(node) => {
            walk_object_expression(state, w, node, scoping, address)
        }
        Expression::ParenthesizedExpression(node) => {
            walk_parenthesized_expression(state, w, node, scoping, address)
        }
        Expression::TaggedTemplateExpression(node) => {
            walk_tagged_template_expression(state, w, node, scoping, address)
        }
        Expression::MetaProperty(_)
        | Expression::Super(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::ChainExpression(_)
        | Expression::ClassExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ImportExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::NewExpression(_)
        | Expression::SequenceExpression(_)
        | Expression::ThisExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::YieldExpression(_)
        | Expression::PrivateInExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_)
        | Expression::TSAsExpression(_)
        | Expression::TSSatisfiesExpression(_)
        | Expression::TSTypeAssertion(_)
        | Expression::TSNonNullExpression(_)
        | Expression::TSInstantiationExpression(_)
        | Expression::V8IntrinsicExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => None,
    }
}

fn walk_call_expression(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &CallExpression,
    scoping: &Scoping,
    address: Address,
) -> Option<()> {
    let mut h = Sha1::default();
    h.update(CALL.to_ne_bytes());
    walk_expr(state, Some(&mut h), &node.callee, scoping, node.callee.address())?;
    h.update(node.arguments.len().to_ne_bytes());
    for arg in &node.arguments {
        if let Some(expr) = arg.as_expression() {
            walk_expr(state, Some(&mut h), expr, scoping, expr.address())?;
        } else {
            return None;
        }
    }
    let hash = h.finalize();
    state.add(address, hash.into());

    if let Some(w) = w {
        w.update(HASH.to_ne_bytes());
        w.update(hash);
    }
    Some(())
}

fn walk_array_expression<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &ArrayExpression<'a>,
    scoping: &Scoping,
    address: Address,
) -> Option<()> {
    let mut h = Sha1::default();
    h.update(ARRAY_EXPRESSION.to_ne_bytes());
    h.update(node.elements.len().to_ne_bytes());
    for item in &node.elements {
        walk_array_expression_element(state, &mut h, item, scoping)?;
    }
    let hash = h.finalize();
    state.add(address, hash.into());

    if let Some(w) = w {
        w.update(HASH.to_ne_bytes());
        w.update(hash);
    }
    Some(())
}

fn walk_array_expression_element<'a>(
    state: &mut DedupeState,
    w: &mut CoreWrapper<Sha1Core>,
    node: &ArrayExpressionElement<'a>,
    scoping: &Scoping,
) -> Option<()> {
    match node {
        ArrayExpressionElement::SpreadElement(node) => walk_spread_element(state, w, node, scoping),
        ArrayExpressionElement::Elision(_) => walk_elision(w),
        ArrayExpressionElement::BooleanLiteral(_)
        | ArrayExpressionElement::NullLiteral(_)
        | ArrayExpressionElement::NumericLiteral(_)
        | ArrayExpressionElement::BigIntLiteral(_)
        | ArrayExpressionElement::RegExpLiteral(_)
        | ArrayExpressionElement::StringLiteral(_)
        | ArrayExpressionElement::TemplateLiteral(_)
        | ArrayExpressionElement::Identifier(_)
        | ArrayExpressionElement::MetaProperty(_)
        | ArrayExpressionElement::Super(_)
        | ArrayExpressionElement::ArrayExpression(_)
        | ArrayExpressionElement::ArrowFunctionExpression(_)
        | ArrayExpressionElement::AssignmentExpression(_)
        | ArrayExpressionElement::AwaitExpression(_)
        | ArrayExpressionElement::BinaryExpression(_)
        | ArrayExpressionElement::CallExpression(_)
        | ArrayExpressionElement::ChainExpression(_)
        | ArrayExpressionElement::ClassExpression(_)
        | ArrayExpressionElement::ConditionalExpression(_)
        | ArrayExpressionElement::FunctionExpression(_)
        | ArrayExpressionElement::ImportExpression(_)
        | ArrayExpressionElement::LogicalExpression(_)
        | ArrayExpressionElement::NewExpression(_)
        | ArrayExpressionElement::ObjectExpression(_)
        | ArrayExpressionElement::ParenthesizedExpression(_)
        | ArrayExpressionElement::SequenceExpression(_)
        | ArrayExpressionElement::TaggedTemplateExpression(_)
        | ArrayExpressionElement::ThisExpression(_)
        | ArrayExpressionElement::UnaryExpression(_)
        | ArrayExpressionElement::UpdateExpression(_)
        | ArrayExpressionElement::YieldExpression(_)
        | ArrayExpressionElement::PrivateInExpression(_)
        | ArrayExpressionElement::JSXElement(_)
        | ArrayExpressionElement::JSXFragment(_)
        | ArrayExpressionElement::TSAsExpression(_)
        | ArrayExpressionElement::TSSatisfiesExpression(_)
        | ArrayExpressionElement::TSTypeAssertion(_)
        | ArrayExpressionElement::TSNonNullExpression(_)
        | ArrayExpressionElement::TSInstantiationExpression(_)
        | ArrayExpressionElement::V8IntrinsicExpression(_)
        | ArrayExpressionElement::ComputedMemberExpression(_)
        | ArrayExpressionElement::StaticMemberExpression(_)
        | ArrayExpressionElement::PrivateFieldExpression(_) => {
            let expr = node.as_expression()?;
            walk_expr(state, Some(w), expr, scoping, expr.address())
        }
    }
}

fn walk_object_expression<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &ObjectExpression<'a>,
    scoping: &Scoping,
    address: Address,
) -> Option<()> {
    let mut h = Sha1::default();
    h.update(OBJECT_EXPRESSION.to_ne_bytes());
    h.update(node.properties.len().to_ne_bytes());
    for item in &node.properties {
        walk_object_property_kind(state, &mut h, item, scoping)?;
    }
    let hash = h.finalize();
    state.add(address, hash.into());

    if let Some(w) = w {
        w.update(HASH.to_ne_bytes());
        w.update(hash);
    }
    Some(())
}

fn walk_object_property_kind<'a>(
    state: &mut DedupeState,
    w: &mut CoreWrapper<Sha1Core>,
    node: &ObjectPropertyKind<'a>,
    scoping: &Scoping,
) -> Option<()> {
    match node {
        ObjectPropertyKind::ObjectProperty(node) => walk_object_property(state, w, node, scoping),
        ObjectPropertyKind::SpreadProperty(node) => walk_spread_element(state, w, node, scoping),
    }
}

fn walk_object_property<'a>(
    state: &mut DedupeState,
    w: &mut CoreWrapper<Sha1Core>,
    node: &ObjectProperty<'a>,
    scoping: &Scoping,
) -> Option<()> {
    w.update(OBJECT_PROPERTY_KEY.to_ne_bytes());
    walk_property_key(state, Some(w), &node.key, scoping, node.key.address())?;
    w.update(OBJECT_PROPERTY_VALUE.to_ne_bytes());
    walk_expr(state, Some(w), &node.value, scoping, node.value.address())?;
    Some(())
}

fn walk_property_key<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &PropertyKey<'a>,
    scoping: &Scoping,
    address: Address,
) -> Option<()> {
    match node {
        PropertyKey::StaticIdentifier(node) => walk_identifier_name(w, node),
        PropertyKey::PrivateIdentifier(node) => walk_private_identifier(w, node),
        PropertyKey::BooleanLiteral(node) => walk_boolean_literal(w, node),
        PropertyKey::NullLiteral(_) => walk_null_literal(w),
        PropertyKey::NumericLiteral(node) => walk_numeric_literal(w, node),
        PropertyKey::BigIntLiteral(node) => walk_big_int_literal(state, w, node, address),
        PropertyKey::RegExpLiteral(node) => walk_reg_exp_literal(state, w, node, address),
        PropertyKey::StringLiteral(node) => walk_string_literal(state, w, node, address),
        PropertyKey::TemplateLiteral(node) => {
            walk_template_literal(state, w, node, scoping, address)
        }
        PropertyKey::Identifier(node) => walk_identifier_reference(w, node, scoping),
        PropertyKey::MetaProperty(_)
        | PropertyKey::Super(_)
        | PropertyKey::ArrayExpression(_)
        | PropertyKey::ArrowFunctionExpression(_)
        | PropertyKey::AssignmentExpression(_)
        | PropertyKey::AwaitExpression(_)
        | PropertyKey::BinaryExpression(_)
        | PropertyKey::CallExpression(_)
        | PropertyKey::ChainExpression(_)
        | PropertyKey::ClassExpression(_)
        | PropertyKey::ConditionalExpression(_)
        | PropertyKey::FunctionExpression(_)
        | PropertyKey::ImportExpression(_)
        | PropertyKey::LogicalExpression(_)
        | PropertyKey::NewExpression(_)
        | PropertyKey::ObjectExpression(_)
        | PropertyKey::ParenthesizedExpression(_)
        | PropertyKey::SequenceExpression(_)
        | PropertyKey::TaggedTemplateExpression(_)
        | PropertyKey::ThisExpression(_)
        | PropertyKey::UnaryExpression(_)
        | PropertyKey::UpdateExpression(_)
        | PropertyKey::YieldExpression(_)
        | PropertyKey::PrivateInExpression(_)
        | PropertyKey::JSXElement(_)
        | PropertyKey::JSXFragment(_)
        | PropertyKey::TSAsExpression(_)
        | PropertyKey::TSSatisfiesExpression(_)
        | PropertyKey::TSTypeAssertion(_)
        | PropertyKey::TSNonNullExpression(_)
        | PropertyKey::TSInstantiationExpression(_)
        | PropertyKey::V8IntrinsicExpression(_)
        | PropertyKey::ComputedMemberExpression(_)
        | PropertyKey::StaticMemberExpression(_)
        | PropertyKey::PrivateFieldExpression(_) => None,
    }
}

fn walk_template_literal<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &TemplateLiteral<'a>,
    scoping: &Scoping,
    address: Address,
) -> Option<()> {
    let mut h = Sha1::default();
    h.update(TEMPLATE_LITERAL.to_ne_bytes());
    h.update(node.quasis.len().to_ne_bytes());
    for item in &node.quasis {
        walk_template_element(&mut h, item)?;
    }
    h.update(node.expressions.len().to_ne_bytes());
    for item in &node.expressions {
        walk_expr(state, Some(&mut h), item, scoping, item.address())?;
    }
    let hash = h.finalize();
    state.add(address, hash.into());

    if let Some(w) = w {
        w.update(HASH.to_ne_bytes());
        w.update(hash);
    }
    Some(())
}

fn walk_template_element<'a>(
    w: &mut CoreWrapper<Sha1Core>,
    node: &TemplateElement<'a>,
) -> Option<()> {
    w.update(TEMPLATE_ELEMENT.to_ne_bytes());
    let s = &node.value.raw;
    w.update(s.len().to_ne_bytes());
    w.update(s.as_bytes());
    Some(())
}

fn walk_tagged_template_expression<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &TaggedTemplateExpression<'a>,
    scoping: &Scoping,
    address: Address,
) -> Option<()> {
    let mut h = Sha1::default();
    h.update(TAGGED_TEMPLATE_EXPRESSION.to_ne_bytes());
    walk_expr(state, Some(&mut h), &node.tag, scoping, address)?;
    h.update(node.quasi.quasis.len().to_ne_bytes());
    for item in &node.quasi.quasis {
        walk_template_element(&mut h, item)?;
    }
    h.update(node.quasi.expressions.len().to_ne_bytes());
    for item in &node.quasi.expressions {
        walk_expr(state, Some(&mut h), item, scoping, item.address())?;
    }

    let hash = h.finalize();
    state.add(address, hash.into());

    if let Some(w) = w {
        w.update(HASH.to_ne_bytes());
        w.update(hash);
    }
    Some(())
}

fn walk_parenthesized_expression<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &ParenthesizedExpression<'a>,
    scoping: &Scoping,
    address: Address,
) -> Option<()> {
    walk_expr(state, w, &node.expression, scoping, address)
}

fn walk_boolean_literal(
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &BooleanLiteral,
) -> Option<()> {
    if let Some(h) = w {
        h.update((node.value as u8).to_ne_bytes());
    }
    Some(())
}

fn walk_null_literal(w: Option<&mut CoreWrapper<Sha1Core>>) -> Option<()> {
    if let Some(w) = w {
        w.update(NULL_LITERAL.to_ne_bytes());
    }
    Some(())
}

fn walk_numeric_literal<'a>(
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &NumericLiteral<'a>,
) -> Option<()> {
    if let Some(h) = w {
        h.update(NUMERIC_LITERAL.to_ne_bytes());
        h.update(node.value.to_ne_bytes());
    }
    Some(())
}

fn walk_string_literal<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &StringLiteral<'a>,
    address: Address,
) -> Option<()> {
    let s = &node.value;
    if s.len() > 16 {
        let mut h = Sha1::default();
        h.update(STRING_LITERAL.to_ne_bytes());
        h.update(s.len().to_ne_bytes());
        h.update(s.as_bytes());

        let hash = h.finalize();
        state.add(address, hash.into());

        if let Some(w) = w {
            w.update(HASH.to_ne_bytes());
            w.update(hash);
        }
    } else if let Some(h) = w {
        h.update(STRING_LITERAL.to_ne_bytes());
        h.update(s.len().to_ne_bytes());
        h.update(s.as_bytes());
    };
    Some(())
}

fn walk_big_int_literal<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &BigIntLiteral<'a>,
    address: Address,
) -> Option<()> {
    let mut h = Sha1::default();
    h.update(BIG_INT_LITERAL.to_ne_bytes());
    let s = &node.value;
    h.update(s.len().to_ne_bytes());
    h.update(s.as_bytes());

    let hash = h.finalize();
    state.add(address, hash.into());

    if let Some(w) = w {
        w.update(HASH.to_ne_bytes());
        w.update(hash);
    }

    Some(())
}

fn walk_reg_exp_literal<'a>(
    state: &mut DedupeState,
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &RegExpLiteral<'a>,
    address: Address,
) -> Option<()> {
    let mut h = Sha1::default();
    h.update(REG_EXP_LITERAL.to_ne_bytes());
    let Some(s) = &node.raw else {
        return None;
    };
    h.update(s.len().to_ne_bytes());
    h.update(s.as_bytes());

    let hash = h.finalize();
    state.add(address, hash.into());

    if let Some(w) = w {
        w.update(HASH.to_ne_bytes());
        w.update(hash);
    }

    Some(())
}

fn walk_spread_element<'a>(
    state: &mut DedupeState,
    w: &mut CoreWrapper<Sha1Core>,
    node: &SpreadElement<'a>,
    scoping: &Scoping,
) -> Option<()> {
    w.update(SPREAD_ELEMENT.to_ne_bytes());
    walk_expr(state, Some(w), &node.argument, scoping, node.argument.address())?;
    Some(())
}

fn walk_elision(w: &mut CoreWrapper<Sha1Core>) -> Option<()> {
    w.update(ELISION.to_ne_bytes());
    Some(())
}

fn walk_identifier_reference<'a>(
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &IdentifierReference<'a>,
    scoping: &Scoping,
) -> Option<()> {
    if let Some(h) = w {
        let r = scoping.get_reference(node.reference_id());
        if let Some(s) = r.symbol_id() {
            h.update(IDENTIFIER_REFERENCE_SYMBOL.to_ne_bytes());
            h.update(s.index().to_ne_bytes());
        } else {
            h.update(IDENTIFIER_REFERENCE_GLOBAL.to_ne_bytes());
            let s = &node.name;
            h.update(s.len().to_ne_bytes());
            h.update(s.as_bytes());
        }
    }
    Some(())
}

fn walk_identifier_name<'a>(
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &IdentifierName<'a>,
) -> Option<()> {
    if let Some(h) = w {
        h.update(IDENTIFIER_NAME.to_ne_bytes());
        h.update(node.name.as_bytes());
    }
    Some(())
}

fn walk_private_identifier<'a>(
    w: Option<&mut CoreWrapper<Sha1Core>>,
    node: &PrivateIdentifier<'a>,
) -> Option<()> {
    if let Some(h) = w {
        h.update(PRIVATE_IDENTIFIER.to_ne_bytes());
        h.update(node.name.as_bytes());
    }
    Some(())
}

// const BOOLEAN_LITERAL_FALSE: u8 = 0;
// const BOOLEAN_LITERAL_TRUE: u8 = 1;
const NUMERIC_LITERAL: u8 = 2;
const STRING_LITERAL: u8 = 3;
const NULL_LITERAL: u8 = 4;
const BIG_INT_LITERAL: u8 = 5;
const REG_EXP_LITERAL: u8 = 6;
const TEMPLATE_LITERAL: u8 = 7;
const TEMPLATE_ELEMENT: u8 = 8;
const TAGGED_TEMPLATE_EXPRESSION: u8 = 9;
const IDENTIFIER_REFERENCE_SYMBOL: u8 = 10;
const IDENTIFIER_REFERENCE_GLOBAL: u8 = 11;
const ARRAY_EXPRESSION: u8 = 12;
const OBJECT_EXPRESSION: u8 = 13;
const OBJECT_PROPERTY_KEY: u8 = 14;
const OBJECT_PROPERTY_VALUE: u8 = 15;
const IDENTIFIER_NAME: u8 = 16;
const PRIVATE_IDENTIFIER: u8 = 17;
const SPREAD_ELEMENT: u8 = 18;
const ELISION: u8 = 19;
const CALL: u8 = 20;
const HASH: u8 = 21;
