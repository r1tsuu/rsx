use std::collections::HashMap;

use crate::{
    error::EngineError,
    parser::{Expression, Parser},
    tokenizer::{Token, TokenKind, Tokenizer},
};

#[derive(Clone, Debug)]
pub struct Executor {
    expression: Box<Expression>,
    variables: HashMap<String, f32>,
}

impl Executor {
    pub fn from_source(source: String) -> Result<Self, EngineError> {
        let tokenizer = Tokenizer::from_source(source);
        let mut tokens: Vec<Token> = vec![];

        for token in tokenizer.to_iter() {
            match token {
                Ok(token) => {
                    tokens.push(token);
                }
                Err(err) => return Err(err),
            }
        }

        let mut parser = Parser::new(tokens);

        let expr = parser.parse_program();

        match expr {
            Ok(program) => Ok(Executor {
                expression: Box::from(program),
                variables: HashMap::new(),
            }),
            Err(err) => Err(err),
        }
    }

    pub fn execute(&mut self) -> Result<f32, EngineError> {
        match self.expression.as_ref() {
            Expression::Program { expressions } => {
                let mut value: Result<f32, EngineError> =
                    Err(EngineError::executor_error("No value"));

                for expression in expressions.iter() {
                    let mut executor = Executor {
                        expression: Box::from(expression.clone()),
                        variables: self.variables.clone(),
                    };

                    value = executor.execute();
                    self.variables = executor.variables;
                }

                value
            }
            Expression::NumberLiteral { value } => Ok(*value),
            Expression::Parenthesized { expression } => {
                let mut executor = Executor {
                    expression: Box::from(expression.clone()),
                    variables: self.variables.clone(),
                };

                executor.execute()
            }
            Expression::Identifier { name } => {
                println!("{:#?}", self.variables);
                let value = self.variables.get(name);

                match value {
                    Some(val) => Ok(*val),
                    None => Err(EngineError::executor_error(format!(
                        "Access to non existing identifier {}",
                        name
                    ))),
                }
            }
            Expression::LetVariableDeclaration { name, initializer } => {
                let mut initializer_executor = Executor {
                    expression: Box::from(initializer.clone()),
                    variables: self.variables.clone(),
                };

                match initializer_executor.execute() {
                    Ok(val) => {
                        self.variables.insert(name.clone(), val);
                        println!("{:#?}", self.variables);
                        Ok(val)
                    }
                    Err(err) => Err(err),
                }
            }
            Expression::BinaryOp { left, op, right } => {
                let reordered = &reorder_expression(Expression::BinaryOp {
                    left: left.clone(),
                    op: op.clone(),
                    right: right.clone(),
                });

                if let Expression::BinaryOp { left, op, right } = reordered {
                    let evaluated_left = match (Executor {
                        expression: left.clone(),
                        variables: self.variables.clone(),
                    }
                    .execute())
                    {
                        Ok(val) => val,
                        Err(err) => return Err(err),
                    };

                    let evaluated_right = match (Executor {
                        expression: right.clone(),
                        variables: self.variables.clone(),
                    }
                    .execute())
                    {
                        Ok(val) => val,
                        Err(err) => return Err(err),
                    };

                    let evaluated = match op.kind {
                        TokenKind::Plus => evaluated_left + evaluated_right,
                        TokenKind::Minus => evaluated_left - evaluated_right,
                        TokenKind::Multiply => evaluated_left * evaluated_right,
                        TokenKind::Divide => evaluated_left / evaluated_right,
                        _ => return Err(EngineError::executor_error("invalid")),
                    };

                    return Ok(evaluated);
                }

                unreachable!()
            }
            _ => {
                todo!()
            }
        }
    }
}

fn reorder_expression(expr: Expression) -> Expression {
    match expr {
        Expression::BinaryOp { left, op, right } => {
            let left = reorder_expression(*left);
            let right = reorder_expression(*right);

            if let Expression::BinaryOp {
                left: right_left,
                op: right_op,
                right: right_right,
            } = right.clone()
            {
                if get_precedence(&op) > get_precedence(&right_op) {
                    let new_left = Expression::BinaryOp {
                        left: Box::from(left),
                        op,
                        right: right_left,
                    };

                    return Expression::BinaryOp {
                        left: Box::from(new_left),
                        op: right_op,
                        right: right_right,
                    };
                }
            }

            return Expression::BinaryOp {
                left: Box::from(left),
                op,
                right: Box::from(right),
            };
        }
        _ => expr,
    }
}

fn get_precedence(token: &Token) -> i32 {
    match token.kind {
        TokenKind::Plus | TokenKind::Minus => 1,
        TokenKind::Multiply | TokenKind::Divide => 2,
        _ => 0,
    }
}
