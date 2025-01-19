use crate::tokenizer::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum Expression {
    NumberLiteral {
        value: f32,
    },
    BinaryOp {
        left: Box<Expression>,
        op: Token,
        right: Box<Expression>,
    },
    Program {
        expressions: Vec<Expression>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    current_token: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current_token: 0,
        }
    }

    pub fn parse_program(&mut self) -> Expression {
        let mut expressions = vec![];
        let expression = self.parse_expression();
        expressions.push(expression);

        Expression::Program { expressions }
    }

    fn parse_expression(&mut self) -> Expression {
        let token = self.tokens.get(self.current_token).unwrap();

        match token.kind {
            TokenKind::Number => {
                let next_token = self.tokens.get(self.current_token + 1);

                match next_token {
                    Some(_) => self.parse_binary_op_expression(),
                    None => Expression::NumberLiteral {
                        value: token.text.parse::<f32>().unwrap(),
                    },
                }
            }
            _ => {
                unimplemented!()
            }
        }
    }

    fn parse_binary_op_expression(&mut self) -> Expression {
        let numeric = self.tokens.get(self.current_token).unwrap();

        let op = self.tokens.get(self.current_token + 1).unwrap().clone();

        self.current_token += 2;

        let left = Box::from(Expression::NumberLiteral {
            value: numeric.text.parse::<f32>().unwrap(),
        });

        let right = Box::from(self.parse_expression());

        return Expression::BinaryOp { left, op, right };
    }
}
