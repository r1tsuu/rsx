use std::collections::HashMap;

use crate::{
    ast::{ASTParser, Expression, Statement},
    error::EngineError,
    lexer::Token,
};

#[derive(Clone)]
pub enum JSValue {
    String(String),
    Number(f32),
    Undefined,
}

impl JSValue {
    pub fn try_as_number(&self) -> Option<f32> {
        match self {
            JSValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn try_as_string(&self) -> Option<&str> {
        match self {
            JSValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn add(&self, other: &JSValue) -> JSValue {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number + *other_number);
        }

        unimplemented!()
    }

    pub fn sub(&self, other: &JSValue) -> JSValue {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number - *other_number);
        }

        unimplemented!()
    }

    pub fn multiply(&self, other: &JSValue) -> JSValue {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number * *other_number);
        }

        unimplemented!()
    }

    pub fn divide(&self, other: &JSValue) -> JSValue {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number / *other_number);
        }

        unimplemented!()
    }
}

struct Scope {
    variables: HashMap<String, JSValue>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

struct ExecutionContext {
    scopes: Vec<Scope>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        let mut global_scope = Scope::new();

        global_scope
            .variables
            .insert("undefined".to_string(), JSValue::Undefined);

        Self {
            scopes: vec![global_scope],
        }
    }

    fn get_global_scope(&self) -> &Scope {
        &self.scopes[0]
    }

    fn get_current_scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    fn get_current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    fn get_variable(&self, name: &str) -> JSValue {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return value.clone();
            }
        }

        JSValue::Undefined
    }

    fn set_variable(&mut self, name: String, value: JSValue) {
        self.get_current_scope_mut().variables.insert(name, value);
    }

    pub fn execute_expression(&mut self, expression: &Expression) -> JSValue {
        match expression {
            Expression::Identifier(identifier) => self.get_variable(&identifier.name),
            Expression::Binary(binary) => {
                let left = self.execute_expression(&binary.left);
                let right = self.execute_expression(&binary.right);

                match binary.operator {
                    Token::Plus => left.add(&right),
                    Token::Minus => left.sub(&right),
                    Token::Star => left.multiply(&right),
                    Token::Slash => left.divide(&right),
                    _ => unimplemented!(),
                }
            }
            Expression::NumericLiteral(numeric) => JSValue::Number(numeric.value),
            _ => unimplemented!(),
        }
    }

    pub fn execute_statement(&mut self, statement: &Statement) -> JSValue {
        match statement {
            Statement::Let(let_statement) => {
                let value = self.execute_expression(&let_statement.value);
                self.set_variable(let_statement.name.clone(), value);
                JSValue::Undefined
            }
            Statement::Expression(expression_statement) => {
                self.execute_expression(&expression_statement.expression)
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

pub fn evaluate_source(source: &str) -> Result<JSValue, EngineError> {
    let ast = ASTParser::parse_from_source(source)?;
    let mut ctx = ExecutionContext::new();

    Ok(ast
        .iter()
        .map(|statement| ctx.execute_statement(statement))
        .last()
        .unwrap_or(JSValue::Undefined))
}

#[cfg(test)]
mod tests {
    use crate::core::evaluate_source;

    #[test]
    fn test_evaluate_numeric_literal() {
        let result = evaluate_source("42;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_evaluate_addition() {
        let result = evaluate_source("5 + 3;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 8.0);
    }

    #[test]
    fn test_evaluate_subtraction() {
        let result = evaluate_source("10 - 4;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_evaluate_multiplication() {
        let result = evaluate_source("6 * 7;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_evaluate_division() {
        let result = evaluate_source("20 / 4;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 5.0);
    }

    #[test]
    fn test_evaluate_complex_expression() {
        let result = evaluate_source("2 + 3 * 4;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 14.0); // 2 + (3 * 4) = 14
    }

    #[test]
    fn test_evaluate_parenthesized_expression() {
        let result = evaluate_source("(5 + 3) * 2;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 16.0); // (5 + 3) * 2 = 16
    }

    #[test]
    fn test_evaluate_let_statement() {
        let result = evaluate_source("let x = 42; x;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_evaluate_let_with_expression() {
        let result = evaluate_source("let y = 10 + 5; y;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_variable_in_expression() {
        let result = evaluate_source("let x = 10; x + 5;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_multiple_variables() {
        let result = evaluate_source("let a = 5; let b = 3; a * b;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_chained_operations() {
        let result = evaluate_source("1 + 2 + 3;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_evaluate_variable_reassignment() {
        let result = evaluate_source("let x = 10; let x = 20; x;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 20.0);
    }

    #[test]
    fn test_evaluate_complex_with_variables() {
        let result = evaluate_source("let a = 2; let b = 3; let c = 4; a + b * c;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 14.0); // 2 + (3 * 4) = 14
    }
}
