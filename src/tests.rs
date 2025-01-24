#[cfg(test)]
use crate::execution_engine::ExpressionEvaluator;

#[test]
fn test_basic_binary() {
    let result = ExpressionEvaluator::evaluate_source("100+200*3+5+(3+5*3+6)").unwrap();

    assert_eq!(result.cast_to_number(), 729.0);
}

#[test]
fn test_binary_eq_eq_1() {
    let result = ExpressionEvaluator::evaluate_source("100 == 100").unwrap();

    assert_eq!(result.cast_to_bool(), true);
}

#[test]
fn test_binary_eq_eq_2() {
    let result = ExpressionEvaluator::evaluate_source("100 == \"100\"").unwrap();

    assert_eq!(result.cast_to_bool(), true);
}

#[test]
fn test_basic_variables() {
    let result = ExpressionEvaluator::evaluate_source("let x = 1; let b = 6; x + b;").unwrap();

    assert_eq!(result.cast_to_number(), 7.0);
}

#[test]
fn test_basic_variables_changing() {
    let result =
        ExpressionEvaluator::evaluate_source("let x = 1; let b = 6; x = 10 + b; x").unwrap();

    assert_eq!(result.cast_to_number(), 16.0);
}

#[test]
fn test_basic_single_string() {
    let result = ExpressionEvaluator::evaluate_source("\"Hello World\"").unwrap();

    assert_eq!(result.cast_to_string(), "Hello World");
}

#[test]
fn test_basic_functions() {
    let result =
        ExpressionEvaluator::evaluate_source("function x(a, b) { return a + b; } x(2, 3);")
            .unwrap();

    assert_eq!(result.cast_to_number(), 5.0);
}

#[test]
fn test_functions_equality() {
    let result =
        ExpressionEvaluator::evaluate_source("function x(a, b) { return a + b; } x == x;").unwrap();

    assert_eq!(result.cast_to_bool(), true);

    let result = ExpressionEvaluator::evaluate_source(
        "function x(a, b) { return a + b; } function y(a, b) { return a + b; } x==y;",
    )
    .unwrap();

    assert_eq!(result.cast_to_bool(), false);
}
