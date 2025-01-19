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

    pub fn parse_program(&mut self) -> Expression {
        let mut expressions = vec![];

        while self.current_token < self.tokens.len() {
            let expression = self.parse_expression();
            if let Some(expression) = expression {
                expressions.push(expression);
            }
            self.current_token += 1;
        }

        Expression::Program { expressions }
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        let token = self.tokens.get(self.current_token).unwrap();

        match token.kind {
            TokenKind::Number => self.parse_number(),
            TokenKind::OpenParen => self.parse_oparen(),
            TokenKind::Let => self.parse_let(),
            TokenKind::Semicolon => None,
            TokenKind::Identifier => self.parse_identifier(),
            _ => unreachable!(),
        }
    }

    fn parse_let(&mut self) -> Option<Expression> {
        let token = self.tokens.get(self.current_token).unwrap();
        let expect_identifier_token = self.tokens.get(self.current_token + 1);

        if let Some(expect_identifier_token) = expect_identifier_token {
            if expect_identifier_token.kind != TokenKind::Identifier {
                panic!()
            }

            let expect_equals_token = self.tokens.get(self.current_token + 2);

            if let Some(expect_equals_token) = expect_equals_token {
                if expect_equals_token.kind != TokenKind::Equals {
                    panic!()
                }

                self.current_token += 3;

                let expect_identifier_token = expect_identifier_token.clone();
                let initializer_expression = self.parse_expression().unwrap();

                return Some(Expression::LetVariableDeclaration {
                    name: expect_identifier_token.text,
                    initializer: Box::from(initializer_expression),
                });
            }
        }

        panic!()
    }

    fn parse_number(&mut self) -> Option<Expression> {
        let token = self.tokens.get(self.current_token).unwrap();
        let next_token = self.tokens.get(self.current_token + 1);

        match next_token {
            Some(next_token) => match next_token.kind {
                TokenKind::Divide | TokenKind::Minus | TokenKind::Plus | TokenKind::Multiply => {
                    let binary = self.parse_binary_op_expression();

                    if let Some(binary) = binary {
                        return Some(binary);
                    }

                    return None;
                }
                _ => {
                    return Some(Expression::NumberLiteral {
                        value: token.text.parse::<f32>().unwrap(),
                    });
                }
            },
            None => Some(Expression::NumberLiteral {
                value: token.text.parse::<f32>().unwrap(),
            }),
        }
    }

    fn parse_identifier(&mut self) -> Option<Expression> {
        let token = self.tokens.get(self.current_token).unwrap().clone();
        let next_token = self.tokens.get(self.current_token + 1);

        match next_token {
            Some(next_token) => match next_token.kind {
                TokenKind::Divide | TokenKind::Minus | TokenKind::Plus | TokenKind::Multiply => {
                    let binary = self.parse_binary_op_expression();

                    if let Some(binary) = binary {
                        return Some(binary);
                    }

                    return None;
                }
                _ => {
                    return Some(Expression::Identifier { name: token.text });
                }
            },
            None => Some(Expression::Identifier { name: token.text }),
        }
    }

    fn parse_oparen(&mut self) -> Option<Expression> {
        let mut extra_paren: usize = 0;
        let mut found_close = false;
        self.current_token += 1;
        let start = self.current_token;

        while self.current_token < self.tokens.len() {
            let token = &self.tokens[self.current_token];

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
            eprintln!("ERR NOT FOUND CLOSE PAREN");
            panic!("");
        }

        let spliced = &self.tokens[start..self.current_token + 1];

        let mut parser = Self::new(spliced.to_vec());
        let expression = parser.parse_expression();

        self.current_token += parser.current_token;

        let next_token = self.tokens.get(self.current_token);

        if let Some(expression) = expression {
            let expr = Expression::Parenthesized {
                expression: Box::from(expression),
            };

            if let Some(next_token) = next_token {
                match next_token.kind {
                    TokenKind::Divide
                    | TokenKind::Minus
                    | TokenKind::Plus
                    | TokenKind::Multiply => {
                        let op = next_token.clone();
                        self.current_token += 1;
                        let right = self.parse_expression();

                        if let Some(right) = right {
                            return Some(Expression::BinaryOp {
                                left: Box::from(expr),
                                op,
                                right: Box::from(right),
                            });
                        }
                    }
                    _ => {}
                }
            }

            return Some(expr);
        }

        return None;
    }

    fn parse_binary_op_expression(&mut self) -> Option<Expression> {
        let left_token = self.tokens.get(self.current_token).unwrap();

        let op = self.tokens.get(self.current_token + 1).unwrap().clone();

        self.current_token += 2;

        let left = match left_token.kind {
            TokenKind::Identifier => Box::from(Expression::Identifier {
                name: left_token.clone().text,
            }),
            TokenKind::Number => Box::from(Expression::NumberLiteral {
                value: left_token.text.parse::<f32>().unwrap(),
            }),
            _ => panic!(),
        };

        let right = self.parse_expression();

        if let Some(right) = right {
            return Some(Expression::BinaryOp {
                left,
                op,
                right: Box::from(right),
            });
        }

        return None;
    }
}
