use crate::{
    error::EngineError,
    tokenizer::{Token, TokenKind},
};

#[derive(Debug, Clone)]
pub enum Expression {
    NumberLiteral {
        value: f32,
    },
    StringLiteral {
        value: String,
    },
    BinaryOp {
        left: Box<Expression>,
        op: Token,
        right: Box<Expression>,
    },
    Program {
        expressions: Vec<Expression>,
    },
    Parenthesized {
        expression: Box<Expression>,
    },
    LetVariableDeclaration {
        name: String,
        initializer: Box<Expression>,
    },
    Identifier {
        name: String,
    },
}

impl Expression {
    /** Get .name in an unsafe way. Use only if you know that it has name */
    pub fn unwrap_name(&self) -> String {
        match self {
            Self::Identifier { name } => name.clone(),
            Self::LetVariableDeclaration { name, .. } => name.clone(),
            _ => panic!(),
        }
    }
}

#[derive(Clone)]
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

    pub fn parse_program(&mut self) -> Result<Expression, EngineError> {
        let mut expressions = vec![];

        while self.current_token < self.tokens.len() {
            let token = match self.tokens.get(self.current_token) {
                Some(val) => val,
                None => return Err(EngineError::parser_error("parse_program Unexpected token")),
            };

            if !token.is_semicolon() {
                match self.parse_expression() {
                    Ok(expr) => {
                        expressions.push(expr);
                    }
                    Err(err) => return Err(err),
                }
            }

            self.current_token += 1;
        }

        Ok(Expression::Program { expressions })
    }

    fn parse_expression(&mut self) -> Result<Expression, EngineError> {
        let token = match self.tokens.get(self.current_token) {
            Some(val) => val,
            None => {
                return Err(EngineError::parser_error(
                    "parse_expression Unexpected token",
                ))
            }
        };

        match token.kind {
            TokenKind::Number => self.parse_number(),
            TokenKind::OpenParen => self.parse_oparen(),
            TokenKind::Let => self.parse_let(),
            TokenKind::Identifier => self.parse_identifier(),
            TokenKind::String => self.parse_string(),
            _ => Err(EngineError::parser_error(format!(
                "Unexpected token {:#?}",
                token
            ))),
        }
    }

    fn parse_let(&mut self) -> Result<Expression, EngineError> {
        match self.tokens.get(self.current_token) {
            None => return Err(EngineError::parser_error("Unexpected let")),
            _ => {}
        };

        let expect_identifier_token = match self.tokens.get(self.current_token + 1) {
            Some(val) => val,
            None => {
                return Err(EngineError::parser_error(
                    "Expected identifier token after let",
                ))
            }
        };

        if expect_identifier_token.kind != TokenKind::Identifier {
            return Err(EngineError::parser_error(
                "Expected identifier token after let",
            ));
        }

        let expect_equals_token = match self.tokens.get(self.current_token + 2) {
            Some(val) => val,
            None => {
                return Err(EngineError::parser_error(
                    "Expected equals token after let and identifier",
                ))
            }
        };

        if expect_equals_token.kind != TokenKind::Equals {
            return Err(EngineError::parser_error(
                "Expected equals token after let and identifier",
            ));
        }

        self.current_token += 3;

        let expect_identifier_token = expect_identifier_token.clone();

        match self.parse_expression() {
            Ok(expr) => Ok(Expression::LetVariableDeclaration {
                name: expect_identifier_token.text,
                initializer: Box::from(expr),
            }),
            Err(err) => Err(err),
        }
    }

    fn parse_number(&mut self) -> Result<Expression, EngineError> {
        let token = match self.tokens.get(self.current_token) {
            Some(val) => val.clone(),
            None => return Err(EngineError::parser_error("Unexpected number")),
        };

        match self.tokens.get(self.current_token + 1) {
            Some(next_token) => {
                if !next_token.is_binary_operator() {
                    return Ok(Expression::NumberLiteral {
                        value: token.text.parse::<f32>().unwrap(),
                    });
                }

                self.parse_binary_op_expression()
            }
            None => Ok(Expression::NumberLiteral {
                value: token.text.parse::<f32>().unwrap(),
            }),
        }
    }

    fn parse_string(&mut self) -> Result<Expression, EngineError> {
        let token = match self.tokens.get(self.current_token) {
            Some(val) => val.clone(),
            None => return Err(EngineError::parser_error("Unexpected string")),
        };

        Ok(Expression::StringLiteral { value: token.text })
    }

    fn parse_identifier(&mut self) -> Result<Expression, EngineError> {
        let token = match self.tokens.get(self.current_token) {
            Some(val) => val.clone(),
            None => return Err(EngineError::parser_error("Unexpected identifier")),
        };

        match self.tokens.get(self.current_token + 1) {
            Some(next_token) => {
                if !next_token.is_binary_operator() {
                    return Ok(Expression::Identifier { name: token.text });
                }

                self.parse_binary_op_expression()
            }
            None => Ok(Expression::Identifier { name: token.text }),
        }
    }

    fn parse_oparen(&mut self) -> Result<Expression, EngineError> {
        let mut extra_paren: usize = 0;
        let mut found_close = false;
        self.current_token += 1;
        let start = self.current_token;

        while self.current_token < self.tokens.len() {
            let token = match self.tokens.get(self.current_token) {
                Some(val) => val,
                None => {
                    return Err(EngineError::parser_error(format!(
                        "parse_oparen token not found on pos {}",
                        self.current_token
                    )));
                }
            };

            if token.kind == TokenKind::OpenParen {
                extra_paren += 1;
            } else if token.kind == TokenKind::CloseParen {
                if extra_paren > 0 {
                    extra_paren -= 1;
                } else {
                    found_close = true;
                    self.current_token -= 1;
                    break;
                }
            }

            self.current_token += 1;
        }

        if !found_close {
            return Err(EngineError::parser_error("Expected closed OParen"));
        }

        let spliced = &self.tokens[start..self.current_token + 1];

        let mut parser = Self::new(spliced.to_vec());

        let expression = match parser.parse_expression() {
            Ok(val) => val,
            Err(err) => return Err(err),
        };

        self.current_token += parser.current_token;

        let expr = Expression::Parenthesized {
            expression: Box::from(expression),
        };

        match self.tokens.get(self.current_token) {
            Some(next_token) => {
                if next_token.is_semicolon() {
                    return Ok(expr);
                }

                if !next_token.is_binary_operator() {
                    return Err(EngineError::parser_error(
                        "Expected arithmetic operator as next token after Paren",
                    ));
                }

                let op = next_token.clone();
                self.current_token += 1;

                match self.parse_expression() {
                    Ok(right) => Ok(Expression::BinaryOp {
                        left: Box::from(expr),
                        op,
                        right: Box::from(right),
                    }),
                    Err(err) => Err(err),
                }
            }
            None => Ok(expr),
        }
    }

    fn parse_binary_op_expression(&mut self) -> Result<Expression, EngineError> {
        let left_token = match self.tokens.get(self.current_token) {
            Some(val) => val,
            None => {
                return Err(EngineError::parser_error(
                    "parse_binary_op_expression expected token",
                ));
            }
        };

        let op = match self.tokens.get(self.current_token + 1) {
            Some(val) => val.clone(),
            None => {
                return Err(EngineError::parser_error(
                    "parse_binary_op_expression expected operator",
                ))
            }
        };

        self.current_token += 2;

        let left = match left_token.kind {
            TokenKind::Identifier => Box::from(Expression::Identifier {
                name: left_token.clone().text,
            }),
            TokenKind::Number => Box::from(Expression::NumberLiteral {
                value: left_token.text.parse::<f32>().unwrap(),
            }),
            _ => {
                return Err(EngineError::parser_error(format!(
                    "parse_binary_op_expression unexpected left token {:#?}",
                    left_token
                )));
            }
        };

        match self.parse_expression() {
            Ok(right) => Ok(Expression::BinaryOp {
                left,
                op,
                right: Box::from(right),
            }),
            Err(err) => Err(err),
        }
    }

    pub fn reorder_expression(expr: Expression) -> Expression {
        match expr {
            Expression::BinaryOp { left, op, right } => {
                let left = Self::reorder_expression(*left);
                let right = Self::reorder_expression(*right);

                if let Expression::BinaryOp {
                    left: right_left,
                    op: right_op,
                    right: right_right,
                } = right.clone()
                {
                    if Self::get_precedence(&op) > Self::get_precedence(&right_op) {
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
}
