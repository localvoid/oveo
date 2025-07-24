use oxc_ast::{AstBuilder, ast::*};
use oxc_span::SPAN;
use serde_json::Value;

pub fn json_into_expr<'a>(value: &Value, ast: &mut AstBuilder<'a>) -> Expression<'a> {
    match value {
        Value::Null => ast.expression_null_literal(SPAN),
        Value::Bool(v) => ast.expression_boolean_literal(SPAN, *v),
        Value::Number(v) => {
            ast.expression_numeric_literal(SPAN, v.as_f64().unwrap(), None, NumberBase::Decimal)
        }
        Value::String(s) => ast.expression_string_literal(SPAN, ast.atom(s), None),
        Value::Array(values) => ast.expression_array(
            SPAN,
            ast.vec_from_iter(values.iter().map(|v| json_into_expr(v, ast).into())),
        ),
        Value::Object(map) => ast.expression_object(
            SPAN,
            ast.vec_from_iter(map.iter().map(|(k, v)| {
                ast.object_property_kind_object_property(
                    SPAN,
                    PropertyKind::Init,
                    ast.expression_string_literal(SPAN, ast.atom(k), None).into(),
                    json_into_expr(v, ast),
                    false,
                    false,
                    false,
                )
            })),
        ),
    }
}
