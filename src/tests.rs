#[cfg(test)]
use crate::execution_engine::ExpressionEvaluator;
use crate::js_value::{JSNumber, JSValue};

#[test]
fn test_basic_binary() {
    let result = ExpressionEvaluator::evaluate_source("100+200*3+5+(3+5*3+6)").unwrap();

    assert_eq!(result.cast_to_number().value, 729.0);
}

#[test]
fn test_binary_eq_eq_1() {
    let result = ExpressionEvaluator::evaluate_source("100 == 100").unwrap();

    assert_eq!(result.cast_to_boolean().value, true);
}

#[test]
fn test_binary_eq_eq_2() {
    let result = ExpressionEvaluator::evaluate_source("100 == \"100\"").unwrap();

    assert_eq!(result.cast_to_boolean().value, true);
}

#[test]
fn test_basic_variables() {
    let result = ExpressionEvaluator::evaluate_source("let x = 1; let b = 6; x + b;").unwrap();

    assert_eq!(result.cast_to_number().value, 7.0);
}

#[test]
fn test_basic_variables_changing() {
    let result =
        ExpressionEvaluator::evaluate_source("let x = 1; let b = 6; x = 10 + b; x").unwrap();

    let x = JSNumber::cast_rc(&result);

    match x {
        Some(x) => println!("{}", x.get_debug_string()),
        None => println!("Nope"),
    };

    assert_eq!(result.cast_to_number().value, 16.0);
}

#[test]
fn test_basic_single_string() {
    let result = ExpressionEvaluator::evaluate_source("\"Hello World\"").unwrap();

    assert_eq!(result.cast_to_string().value, "Hello World");
}

#[test]
fn test_basic_functions() {
    let result =
        ExpressionEvaluator::evaluate_source("function x(a, b) { return a + b; } x(2, 3);")
            .unwrap();

    assert_eq!(result.cast_to_number().value, 5.0);
}

#[test]
fn test_functions_equality() {
    let result =
        ExpressionEvaluator::evaluate_source("function x(a, b) { return a + b; } x == x;").unwrap();

    assert_eq!(result.cast_to_boolean().value, true);

    let result = ExpressionEvaluator::evaluate_source(
        "function x(a, b) { return a + b; } function y(a, b) { return a + b; } x==y;",
    )
    .unwrap();

    assert_eq!(result.cast_to_boolean().value, false);
}
