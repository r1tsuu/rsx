use std::{cell::RefCell, rc::Rc};

use crate::{
    error::EngineError,
    execution_scope::ExecutionScope,
    javascript_object::{JavascriptObjectKind, JavascriptObjectRef},
    memory::{Memory, MemoryRef},
    parser::{Expression, Parser},
    tokenizer::{TokenKind, Tokenizer},
};

pub struct ExecutionEngine {
    scopes: Vec<ExecutionScope>,
    memory: MemoryRef,
    execution_tick: u64,
}

const UNDEFINED_NAME: &str = "undefined";

impl ExecutionEngine {
    fn new() -> Self {
        let mut engine = ExecutionEngine {
            scopes: vec![],
            memory: Rc::new(RefCell::new(Memory::new())),
            execution_tick: 0,
        };

        engine.initialize_global_scope();

        engine
    }

    pub fn execute_source<T: ToString>(source: T) -> Result<JavascriptObjectRef, EngineError> {
        let mut tokens = vec![];

        for token in Tokenizer::from_source(source.to_string()).to_iter() {
            match token {
                Ok(token) => tokens.push(token),
                Err(err) => return Err(err),
            };
        }

        match Parser::new(tokens).parse_program() {
            Ok(program) => Self::new().execute_expression(program),
            Err(err) => return Err(err),
        }
    }

    fn initialize_global_scope(&mut self) {
        let mut global_scope = ExecutionScope::new(None, self.memory.clone());

        global_scope
            .define(
                UNDEFINED_NAME.to_string(),
                self.memory.borrow_mut().allocate_undefined(),
            )
            .unwrap();

        self.scopes.push(global_scope);
    }

    fn get_global_scope(&self) -> &ExecutionScope {
        self.scopes.get(0).unwrap()
    }

    fn get_undefined(&self) -> JavascriptObjectRef {
        self.get_global_scope()
            .get(UNDEFINED_NAME.to_string())
            .unwrap()
    }

    fn get_current_scope(&mut self) -> &mut ExecutionScope {
        self.scopes.last_mut().unwrap()
    }

    fn execute_expression(
        &mut self,
        expression: Expression,
    ) -> Result<JavascriptObjectRef, EngineError> {
        let result = match expression {
            Expression::Program { expressions } => {
                for (index, expr) in expressions.iter().enumerate() {
                    match self.execute_expression(expr.clone()) {
                        Err(err) => return Err(err),
                        Ok(value) => {
                            if index == expressions.len() - 1 {
                                return Ok(value);
                            }
                        }
                    }
                }

                return Ok(self.get_undefined());
            }
            Expression::LetVariableDeclaration { name, initializer } => {
                match self.execute_expression(*initializer.clone()) {
                    Ok(object) => self.get_current_scope().define(name, object),
                    Err(err) => Err(err),
                }
            }
            Expression::NumberLiteral { value } => {
                Ok(self.memory.borrow_mut().allocate_number(value))
            }
            Expression::Parenthesized { expression } => {
                self.execute_expression(*expression.clone())
            }
            Expression::Identifier { name } => match self.get_current_scope().get(name.clone()) {
                Some(value) => Ok(value),
                None => Err(EngineError::execution_engine_error(format!(
                    "No variable {} found in the scope",
                    name
                ))),
            },
            Expression::BinaryOp { left, op, right } => {
                if op.is_equals() {
                    match *left.clone() {
                        Expression::Identifier { name } => {
                            match self.get_current_scope().get(name.clone()) {
                                Some(var) => var,
                                None => {
                                    return Err(EngineError::execution_engine_error(format!(
                                        "No variable {} found in the scope",
                                        name
                                    )))
                                }
                            }
                        }
                        _ => {
                            return Err(EngineError::execution_engine_error(format!(
                                "Expected identifier in assigment, got: {:#?}",
                                left
                            )))
                        }
                    };

                    let value = match self.execute_expression(*right) {
                        Ok(res) => res,
                        Err(err) => return Err(err),
                    };

                    self.get_current_scope()
                        .assign(left.unwrap_name(), value.clone());

                    return Ok(value);
                }

                let left_value =
                    match self.evaluate_expression_to_number(Parser::reorder_expression(*left)) {
                        Ok(val) => val,
                        Err(err) => return Err(err),
                    };

                let right_value =
                    match self.evaluate_expression_to_number(Parser::reorder_expression(*right)) {
                        Ok(val) => val,
                        Err(err) => return Err(err),
                    };

                match op.kind {
                    TokenKind::Plus => Ok(self
                        .memory
                        .borrow_mut()
                        .allocate_number(left_value + right_value)),
                    TokenKind::Minus => Ok(self
                        .memory
                        .borrow_mut()
                        .allocate_number(left_value - right_value)),
                    TokenKind::Multiply => Ok(self
                        .memory
                        .borrow_mut()
                        .allocate_number(left_value * right_value)),
                    TokenKind::Divide => Ok(self
                        .memory
                        .borrow_mut()
                        .allocate_number(left_value / right_value)),
                    _ => Err(EngineError::execution_engine_error(format!(
                        "Failed to execute binary expression with operator: {:#?}",
                        op
                    ))),
                }
            }
            Expression::StringLiteral { value } => {
                Ok(self.memory.borrow_mut().allocate_string(value))
            }
        };

        self.execution_tick += 1;

        println!(
            "BEF: {:#?}",
            self.get_current_scope().get_variable_ids().len()
        );
        self.collect_garbage();
        println!(
            "AFTER: {:#?}",
            self.get_current_scope().get_variable_ids().len()
        );

        result
    }

    fn collect_garbage(&mut self) {
        for scope in self.scopes.iter() {
            self.memory
                .borrow_mut()
                .deallocate_except_ids(&scope.get_variable_ids());
        }
    }

    fn evaluate_expression_to_number(
        &mut self,
        expression: Expression,
    ) -> Result<f32, EngineError> {
        match self.execute_expression(expression) {
            Ok(value) => match value.clone().borrow().kind {
                JavascriptObjectKind::Number { value } => Ok(value),
                _ => {
                    return Err(EngineError::execution_engine_error(format!(
                        "Binary expression accepts only numbers! {:#?} is not",
                        value
                    )))
                }
            },
            Err(err) => return Err(err),
        }
    }
}
