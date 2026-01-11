use std::collections::HashMap;

use crate::{
    ast::{ASTParser, Expression, Statement},
    lexer::Token,
};

#[derive(Clone)]
pub enum JSValue {
    String(String),
    Number(f32),
    Undefined,
}

impl JSValue {
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
    stack: Vec<JSValue>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        let mut global_scope = Scope::new();

        global_scope
            .variables
            .insert("undefined".to_string(), JSValue::Undefined);

        Self {
            scopes: vec![global_scope],
            stack: vec![],
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

    fn get_undefined(&self) -> JSValue {
        self.get_global_scope()
            .variables
            .get("undefined")
            .unwrap()
            .clone()
    }

    fn get_variable(&self, name: &str) -> JSValue {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return value.clone();
            }
        }

        self.get_undefined()
    }

    fn set_variable(&mut self, name: String, value: JSValue) {
        self.get_current_scope_mut().variables.insert(name, value);
    }

    fn stack_push(&mut self, value: JSValue) {
        self.stack.push(value)
    }

    fn stack_pop(&mut self) -> JSValue {
        self.stack.pop().unwrap()
    }

    pub fn execute_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Identifier(identifier) => {
                self.stack_push(self.get_variable(&identifier.name).clone())
            }
            Expression::Binary(binary) => {
                self.execute_expression(&binary.left);
                let left = self.stack_pop();
                self.execute_expression(&binary.right);
                let right = self.stack_pop();

                match binary.operator {
                    Token::Plus => {
                        self.stack_push(left.add(&right));
                    }
                    Token::Minus => {
                        self.stack_push(left.sub(&right));
                    }
                    Token::Star => {
                        self.stack_push(left.multiply(&right));
                    }
                    Token::Slash => {
                        self.stack_push(left.divide(&right));
                    }
                    _ => unimplemented!(),
                }

                unimplemented!()
            }
            Expression::NumericLiteral(numeric) => {
                self.stack_push(JSValue::Number(numeric.value));
            }
        }
    }

    pub fn execute_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Let(let_statement) => {
                self.execute_expression(&let_statement.value);
                let value = self.stack_pop();
                self.set_variable(let_statement.name.clone(), value);
            }
            Statement::Expression(expression_statement) => {
                self.execute_expression(&expression_statement.expression);
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

pub fn evaluate_source(source: &str) -> Result<JSValue, String> {
    let ast = ASTParser::parse_from_source(source)?;
    let mut ctx = ExecutionContext::new();

    for statement in ast.iter() {
        ctx.execute_statement(statement);
    }

    Ok(ctx.stack_pop())
}
