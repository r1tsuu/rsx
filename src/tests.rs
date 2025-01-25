#[cfg(test)]
use crate::execution_engine::ExpressionEvaluator;
#[cfg(test)]
use crate::js_value::JSObject;

#[test]
fn tests() {
    let result = ExpressionEvaluator::evaluate_source("100+200*3+5+(3+5*3+6)").unwrap();
    assert_eq!(result.cast_to_number().value, 729.0);

    let result = ExpressionEvaluator::evaluate_source("100 == 100").unwrap();
    assert_eq!(result.cast_to_boolean().value, true);

    let result = ExpressionEvaluator::evaluate_source("100 == \"100\"").unwrap();
    assert_eq!(result.cast_to_boolean().value, true);

    let result = ExpressionEvaluator::evaluate_source("let x = 1; let b = 6; x + b;").unwrap();
    assert_eq!(result.cast_to_number().value, 7.0);

    let result =
        ExpressionEvaluator::evaluate_source("let x = 1; let b = 6; x = 10 + b; x").unwrap();
    assert_eq!(result.cast_to_number().value, 16.0);

    let result = ExpressionEvaluator::evaluate_source("\"Hello World\"").unwrap();
    assert_eq!(result.cast_to_string().value, "Hello World");

    let result =
        ExpressionEvaluator::evaluate_source("function x(a, b) { return a + b; } x(2, 3);")
            .unwrap();
    assert_eq!(result.cast_to_number().value, 5.0);

    let result =
        ExpressionEvaluator::evaluate_source("function x(a, b) { return a + b; } x == x;").unwrap();
    assert_eq!(result.cast_to_boolean().value, true);

    let result = ExpressionEvaluator::evaluate_source(
        "function x(a, b) { return a + b; } function y(a, b) { return a + b; } x==y;",
    )
    .unwrap();
    assert_eq!(result.cast_to_boolean().value, false);

    let result =
        ExpressionEvaluator::evaluate_source("let x = {a: 10, b: \"Hello World\"}").unwrap();
    let obj = JSObject::cast(result.as_ref()).unwrap();
    assert_eq!(obj.get_key("a").cast_to_number().value, 10.0);
    assert_eq!(obj.get_key("b").cast_to_string().value, "Hello World");

    let result =
        ExpressionEvaluator::evaluate_source("let x = {a: 10, b: \"Hello World\"}; x.b").unwrap();
    assert_eq!(result.cast_to_string().value, "Hello World");
}
