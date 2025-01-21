use crate::{
    error::EngineError,
    execution_scope::ExecutionScope,
    javascript_object::{JavascriptObject, JavascriptObjectRef},
    memory::Memory,
    parser::{Expression, Parser},
    tokenizer::{TokenKind, Tokenizer},
};

pub struct ExecutionEngine {
    scopes: Vec<ExecutionScope>,
    memory: Memory,
}

const UNDEFINED_NAME: &str = "undefined";

impl ExecutionEngine {
    fn new() -> Self {
        let mut engine = ExecutionEngine {
            scopes: vec![],
            memory: Memory::new(),
        };

        engine.initialize_global_scope();

        engine
    }

    pub fn execute_source(source: String) -> Result<JavascriptObjectRef, EngineError> {
        let mut tokens = vec![];

        for token in Tokenizer::from_source(source).to_iter() {
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
        let mut global_scope = ExecutionScope::new(None);

        global_scope
            .define(UNDEFINED_NAME.to_string(), self.memory.allocate_undefined())
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
        match expression {
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
            Expression::NumberLiteral { value } => Ok(self.memory.allocate_number(value)),
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
                    TokenKind::Plus => Ok(self.memory.allocate_number(left_value + right_value)),
                    TokenKind::Minus => Ok(self.memory.allocate_number(left_value - right_value)),
                    TokenKind::Multiply => {
                        Ok(self.memory.allocate_number(left_value * right_value))
                    }
                    TokenKind::Divide => Ok(self.memory.allocate_number(left_value / right_value)),
                    _ => Err(EngineError::execution_engine_error(format!(
                        "Failed to execute binary expression with operator: {:#?}",
                        op
                    ))),
                }
            }
            _ => Err(EngineError::execution_engine_error(format!(
                "Unimplemented logic for {:#?} expression",
                expression
            ))),
        }
    }

    fn evaluate_expression_to_number(
        &mut self,
        expression: Expression,
    ) -> Result<f32, EngineError> {
        match self.execute_expression(expression) {
            Ok(value) => match *value.clone().borrow() {
                JavascriptObject::Number { value } => Ok(value),
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
