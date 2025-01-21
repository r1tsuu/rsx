#[cfg(test)]
use crate::execution_engine::ExecutionEngine;

#[test]
fn test_basic_binary() {
    let result = ExecutionEngine::execute_source("100+200*3+5+(3+5*3+6)").unwrap();

    assert_eq!(result.borrow().cast_to_number(), 729.0);
}

#[test]
fn test_basic_variables() {
    let result = ExecutionEngine::execute_source("let x = 1; let b = 6; x + b;").unwrap();

    assert_eq!(result.borrow().cast_to_number(), 7.0);
}

#[test]
fn test_basic_single_string() {
    let result = ExecutionEngine::execute_source("\"Hello World\"").unwrap();

    assert_eq!(result.borrow().cast_to_string(), "Hello World");
}
