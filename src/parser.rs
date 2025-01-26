use std::rc::Rc;

use crate::{
    error::EngineError,
    tokenizer::{Token, TokenKind, Tokenizer},
};

#[derive(Debug, Clone)]
pub enum Expression {
    NumberLiteral {
        value: f64,
    },
    StringLiteral {
        value: String,
    },
    BinaryOp {
        left: Rc<Expression>,
        op: Token,
        right: Rc<Expression>,
    },
    Program {
        expressions: Vec<Expression>,
    },
    Parenthesized {
        expression: Rc<Expression>,
    },
    LetVariableDeclaration {
        name: String,
        initializer: Rc<Expression>,
    },
    Identifier {
        name: String,
    },
    Block {
        expressions: Vec<Expression>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<Expression>,
        body: Rc<Expression>,
    },
    FunctionReturn {
        expression: Rc<Expression>,
    },
    FunctionCall {
        name: Rc<Expression>,
        arguments: Vec<Expression>,
    },
    FunctionParameter {
        name: String,
    },
    PropertyAccessExpression {
        name: Rc<Expression>,
        expression: Rc<Expression>,
    },
    ObjectLiteralExpression {
        properties: Vec<Expression>,
    },
    ArrayLiteralExpression {
        elements: Vec<Expression>,
    },
    PropertyAssignment {
        name: Rc<Expression>,
        initializer: Rc<Expression>,
    },
}

#[derive(Clone)]
enum BraceModeContext {
    ObjectExpression,
    Block,
}

impl Expression {
    /** Get .name in an unsafe way. Use only if you know that it has name */
    pub fn unwrap_name(&self) -> String {
        match self {
            Self::Identifier { name } => name.clone(),
            Self::LetVariableDeclaration { name, .. } => name.clone(),
            Self::FunctionParameter { name, .. } => name.clone(),
            Self::FunctionDeclaration { name, .. } => name.clone(),
            _ => panic!(),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current_token: usize,
    brace_mode: BraceModeContext,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current_token: 0,
            brace_mode: BraceModeContext::Block,
        }
    }

    pub fn parse_program(&mut self) -> Result<Expression, EngineError> {
        let expressions = self.parse_expressions()?;

        Ok(Expression::Program { expressions })
    }

    pub fn parse_source(source: &str) -> Result<Expression, EngineError> {
        let mut tokens = vec![];

        for token in Tokenizer::from_source(source.to_string()).to_iter() {
            match token {
                Ok(token) => tokens.push(token),
                Err(err) => return Err(err),
            };
        }

        Parser::new(tokens).parse_program()
    }

    fn parse_expressions(&mut self) -> Result<Vec<Expression>, EngineError> {
        let mut expressions = vec![];

        while self.current_token < self.tokens.len() {
            let token = match self.tokens.get(self.current_token) {
                Some(val) => val,
                None => {
                    return Err(EngineError::parser_error(
                        "parse_expressions Unexpected token",
                    ))
                }
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

        Ok(expressions)
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
            TokenKind::OpenParen => self.parse_oparen_as_expression(),
            TokenKind::Let => self.parse_let(),
            TokenKind::Identifier => self.parse_identifier(),
            TokenKind::String => self.parse_string(),
            TokenKind::OpenBrace => self.parse_obrace(),
            TokenKind::OpenBracket => self.parse_obracket(),
            TokenKind::Function => self.parse_function_declaration(),
            TokenKind::Return => self.parse_function_return(),
            _ => Err(EngineError::parser_error(format!(
                "Unexpected token {:#?}",
                token
            ))),
        }
    }

    fn parse_function_declaration(&mut self) -> Result<Expression, EngineError> {
        self.current_token += 1;

        let expect_function_name_identifier_token = self
            .tokens
            .get(self.current_token)
            .ok_or(EngineError::parser_error(
                "Expected identifier token after function",
            ))
            .cloned()?;

        if !matches!(
            expect_function_name_identifier_token.kind,
            TokenKind::Identifier
        ) {
            return Err(EngineError::parser_error(format!(
                "Expected identifier token after function, got: {:#?}",
                expect_function_name_identifier_token
            )));
        }

        self.current_token += 1;

        let expect_oparen =
            self.tokens
                .get(self.current_token)
                .ok_or(EngineError::parser_error(
                    "Expected oparen start token after function name",
                ))?;

        if !matches!(expect_oparen.kind, TokenKind::OpenParen) {
            return Err(EngineError::parser_error(format!(
                "Expected oparen start token after function name, got: {:#?}",
                expect_oparen
            )));
        }

        let args = self.parse_oparen_as_function_declaration_parameters()?;

        let expect_obrace =
            self.tokens
                .get(self.current_token)
                .ok_or(EngineError::parser_error(
                    "Expected block start token after function args",
                ))?;

        if !matches!(expect_obrace.kind, TokenKind::OpenBrace) {
            return Err(EngineError::parser_error(format!(
                "Expected block start token after function args, got: {:#?}",
                expect_obrace
            )));
        }

        self.brace_mode = BraceModeContext::Block;

        let body = self.parse_obrace()?;

        Ok(Expression::FunctionDeclaration {
            name: expect_function_name_identifier_token.text,
            parameters: args,
            body: Rc::new(body),
        })
    }

    fn parse_function_return(&mut self) -> Result<Expression, EngineError> {
        self.current_token += 1;
        let expression = self.parse_expression()?;
        Ok(Expression::FunctionReturn {
            expression: Rc::new(expression),
        })
    }

    fn parse_let(&mut self) -> Result<Expression, EngineError> {
        match self.tokens.get(self.current_token) {
            None => return Err(EngineError::parser_error("Unexpected let")),
            _ => {}
        };

        let expect_identifier_token = self
            .tokens
            .get(self.current_token + 1)
            .ok_or_else(|| EngineError::parser_error("Expected identifier token after let"))?;

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

        // Do not treat { } as blocks from here.
        self.brace_mode = BraceModeContext::ObjectExpression;
        let res = match self.parse_expression() {
            Ok(expr) => Expression::LetVariableDeclaration {
                name: expect_identifier_token.text,
                initializer: Rc::from(expr),
            },
            Err(err) => return Err(err),
        };
        // back
        self.brace_mode = BraceModeContext::Block;
        Ok(res)
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
                        value: token.text.parse::<f64>().unwrap(),
                    });
                }

                self.parse_binary_op_expression(None)
            }
            None => Ok(Expression::NumberLiteral {
                value: token.text.parse::<f64>().unwrap(),
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

    fn parse_object_access(
        &mut self,
        name_expression: Expression,
        next_token: Token,
    ) -> Result<Expression, EngineError> {
        let mut assigments = vec![name_expression];
        let mut next_token = Some(next_token);

        loop {
            if next_token.unwrap().is_obracket() {
                let mut extra_brackets: usize = 0;
                let mut cbracket_i = self.current_token + 2;

                loop {
                    if let Some(current) = self.tokens.get(cbracket_i) {
                        if current.is_obracket() {
                            extra_brackets += 1;
                        } else if current.is_cbracket() {
                            if extra_brackets == 0 {
                                break;
                            } else {
                                extra_brackets -= 1;
                            }
                        }

                        cbracket_i += 1;
                    } else {
                        return Err(EngineError::parser_error("Could not find CBRACKET"));
                    }
                }

                let mut bracket_parser =
                    Parser::new(self.tokens[self.current_token + 2..cbracket_i].to_vec());

                assigments.push(bracket_parser.parse_expression()?);
                self.current_token += bracket_parser.current_token + 3;
                next_token = self.tokens.get(self.current_token + 1).cloned();
            } else {
                self.current_token += 2;
                let token = self
                    .tokens
                    .get(self.current_token)
                    .ok_or(EngineError::parser_error("Expected next token after dot"))?;

                if !matches!(token.kind, TokenKind::Identifier) {
                    return Err(EngineError::parser_error(format!(
                        "Expected next identifier token after dot, got: {token:#?}"
                    )));
                }

                assigments.push(Expression::StringLiteral {
                    value: token.text.clone(),
                });

                next_token = self.tokens.get(self.current_token + 1).cloned();
            }

            if let Some(next_token) = next_token.clone() {
                if !next_token.is_dot() && !next_token.is_obracket() {
                    break;
                }
            } else {
                break;
            }
        }

        let mut expr: Option<Expression> = None;

        for ass in assigments {
            if let Some(some_expr) = expr {
                expr = Some(Expression::PropertyAccessExpression {
                    name: Rc::new(ass),
                    expression: Rc::new(some_expr),
                });
            } else {
                expr = Some(ass);
            }
        }

        if let Some(expr) = expr {
            Ok(expr)
        } else {
            return Err(EngineError::parser_error(
                "Failed to parse PropertyAccessExpression",
            ));
        }
    }

    fn parse_identifier(&mut self) -> Result<Expression, EngineError> {
        let token = match self.tokens.get(self.current_token) {
            Some(val) => val.clone(),
            None => return Err(EngineError::parser_error("Unexpected identifier")),
        };

        let next_token = match self.tokens.get(self.current_token + 1) {
            None => return Ok(Expression::Identifier { name: token.text }),
            Some(value) => value,
        };

        let expr = if next_token.is_dot() || next_token.is_obracket() {
            self.parse_object_access(
                Expression::Identifier { name: token.text },
                next_token.clone(),
            )?
        } else {
            Expression::Identifier { name: token.text }
        };

        let next_token = match self.tokens.get(self.current_token + 1) {
            None => return Ok(expr),
            Some(value) => value,
        };

        let expr = if next_token.is_oparen() {
            self.current_token += 1;
            let arguments = self.parse_oparen_as_function_arguments()?;

            let expr = Expression::FunctionCall {
                name: Rc::new(expr),
                arguments,
            };

            if let Some(next_token) = self.tokens.get(self.current_token + 1) {
                if next_token.is_dot() {
                    self.current_token += 2;
                    let mut identifier = self.parse_identifier()?;

                    if let Expression::Identifier { name } = identifier {
                        identifier = Expression::StringLiteral { value: name }
                    }

                    return Ok(Expression::PropertyAccessExpression {
                        name: Rc::new(identifier),
                        expression: Rc::new(expr),
                    });
                }

                if next_token.is_obracket() {
                    self.current_token += 1;
                    let obracket_expr = self.parse_obracket()?;
                    if let Expression::ArrayLiteralExpression { elements } = obracket_expr {
                        if let Some(el) = elements.get(0) {
                            return Ok(Expression::PropertyAccessExpression {
                                name: Rc::new(el.clone()),
                                expression: Rc::new(expr),
                            });
                        }
                    }

                    return Err(EngineError::parser_error("Unexpected obracket"));
                }

                // if next_token.is_dot() || next_token.is_obracket() {
                //     return self.parse_object_access(expr, next_token.clone());
                // }
            }

            expr
        } else {
            expr
        };

        let next_token = match self.tokens.get(self.current_token + 1) {
            None => return Ok(expr),
            Some(value) => value,
        };

        let expr = if next_token.is_binary_operator() {
            self.parse_binary_op_expression(Some(&expr))?
        } else {
            expr
        };

        return Ok(expr);
    }

    fn parse_obracket(&mut self) -> Result<Expression, EngineError> {
        let mut extra_bracket: usize = 0;
        let mut found_close = false;
        self.current_token += 1;
        let start = self.current_token;

        while self.current_token < self.tokens.len() {
            let token = match self.tokens.get(self.current_token) {
                Some(val) => val,
                None => {
                    return Err(EngineError::parser_error(format!(
                        "parse_obracket token not found on pos {}",
                        self.current_token
                    )));
                }
            };

            if token.kind == TokenKind::OpenBracket {
                extra_bracket += 1;
            } else if token.kind == TokenKind::CloseBracket {
                if extra_bracket > 0 {
                    extra_bracket -= 1;
                } else {
                    found_close = true;
                    break;
                }
            }

            self.current_token += 1;
        }

        if !found_close {
            return Err(EngineError::parser_error("Expected closed Obracket"));
        }

        let end = self.current_token;

        let spliced = &self.tokens[start..end];
        let mut parser = Self::new(spliced.to_vec());
        parser.brace_mode = BraceModeContext::ObjectExpression;
        let mut elements = vec![];

        while parser.current_token < parser.tokens.len() {
            let element = parser.parse_expression()?;
            elements.push(element);
            parser.current_token += 1;
            let current_token = parser.tokens.get(parser.current_token);

            if let Some(current_token) = current_token {
                if !matches!(current_token.kind, TokenKind::Comma) {
                    return Err(EngineError::parser_error("Expected comma in array"));
                }
                parser.current_token += 1;
            }
        }

        Ok(Expression::ArrayLiteralExpression { elements })
    }

    fn parse_obrace(&mut self) -> Result<Expression, EngineError> {
        let mut extra_paren: usize = 0;
        let mut found_close = false;
        self.current_token += 1;
        let start = self.current_token;

        while self.current_token < self.tokens.len() {
            let token = match self.tokens.get(self.current_token) {
                Some(val) => val,
                None => {
                    return Err(EngineError::parser_error(format!(
                        "parse_obrace token not found on pos {}",
                        self.current_token
                    )));
                }
            };

            if token.kind == TokenKind::OpenBrace {
                extra_paren += 1;
            } else if token.kind == TokenKind::CloseBrace {
                if extra_paren > 0 {
                    extra_paren -= 1;
                } else {
                    found_close = true;
                    break;
                }
            }

            self.current_token += 1;
        }

        if !found_close {
            return Err(EngineError::parser_error("Expected closed OBrace"));
        }

        let end = self.current_token;

        let spliced = &self.tokens[start..end];

        if matches!(self.brace_mode, BraceModeContext::ObjectExpression) {
            let mut parser = Self::new(spliced.to_vec());
            let mut properties = vec![];

            while parser.current_token < parser.tokens.len() {
                let name_identifier = parser.parse_string()?;
                parser.current_token += 1;

                let expect_colon = parser
                    .tokens
                    .get(parser.current_token)
                    .ok_or(EngineError::parser_error(
                        "Expected COLON in object expression",
                    ))?
                    .clone();

                if !matches!(expect_colon.kind, TokenKind::Colon) {
                    return Err(EngineError::parser_error(format!(
                        "Expected COLON in object expression, got: {expect_colon:#?}"
                    )));
                }

                parser.current_token += 1;
                parser.brace_mode = BraceModeContext::ObjectExpression;

                let initializer = parser.parse_expression()?;

                parser.current_token += 2;

                properties.push(Expression::PropertyAssignment {
                    name: Rc::new(name_identifier),
                    initializer: Rc::new(initializer),
                });
            }

            return Ok(Expression::ObjectLiteralExpression { properties });
        }

        let mut parser = Self::new(spliced.to_vec());

        let expressions = parser.parse_expressions()?;

        self.current_token = end + 1;

        match self.tokens.get(end + 1) {
            Some(v) => {
                if v.kind != TokenKind::Semicolon {
                    self.current_token -= 1;
                }
            }
            _ => {}
        }

        let expr = Expression::Block { expressions };

        Ok(expr)
    }

    fn parse_oparen_as_expression(&mut self) -> Result<Expression, EngineError> {
        let mut extra_paren: usize = 0;
        let mut found_close = false;
        self.current_token += 1;
        let start = self.current_token;

        while self.current_token < self.tokens.len() {
            let token = match self.tokens.get(self.current_token) {
                Some(val) => val,
                None => {
                    return Err(EngineError::parser_error(format!(
                        "parse_oparen_as_expression token not found on pos {}",
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
                    break;
                }
            }

            self.current_token += 1;
        }

        if !found_close {
            return Err(EngineError::parser_error("Expected closed OParen"));
        }

        let end = self.current_token;

        let spliced = &self.tokens[start..end + 1];

        let mut parser = Self::new(spliced.to_vec());

        let expression = match parser.parse_expression() {
            Ok(val) => val,
            Err(err) => return Err(err),
        };

        self.current_token = end + 1;

        let expr = Expression::Parenthesized {
            expression: Rc::from(expression),
        };

        match self.tokens.get(self.current_token) {
            Some(next_token) => {
                if next_token.is_semicolon() {
                    return Ok(expr);
                }

                if next_token.is_binary_operator() {
                    let op = next_token.clone();
                    self.current_token += 1;

                    match self.parse_expression() {
                        Ok(right) => Ok(Expression::BinaryOp {
                            left: Rc::from(expr),
                            op,
                            right: Rc::from(right),
                        }),
                        Err(err) => Err(err),
                    }
                } else {
                    self.current_token -= 1;
                    Ok(expr)
                }
            }
            None => Ok(expr),
        }
    }

    fn parse_oparen_as_function_declaration_parameters(
        &mut self,
    ) -> Result<Vec<Expression>, EngineError> {
        let mut extra_paren: usize = 0;
        let mut found_close = false;
        self.current_token += 1;
        let start = self.current_token;

        while self.current_token < self.tokens.len() {
            let token = match self.tokens.get(self.current_token) {
                Some(val) => val,
                None => {
                    return Err(EngineError::parser_error(format!(
                        "parse_oparen_as_function_parameters token not found on pos {}",
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
                    break;
                }
            }

            self.current_token += 1;
        }

        if !found_close {
            return Err(EngineError::parser_error("Expected closed OParen"));
        }

        let end = self.current_token;

        let spliced = &self.tokens[start..end];

        let mut chunks: Vec<Vec<Token>> = vec![];
        let mut current_chunk = 0;

        for token in spliced {
            if matches!(token.kind, TokenKind::Comma) {
                current_chunk += 1;
                continue;
            } else {
                if current_chunk + 1 > chunks.len() {
                    chunks.push(vec![]);
                }
                chunks.get_mut(current_chunk).unwrap().push(token.clone());
            }
        }

        let mut expressions = vec![];

        if chunks.len() > 0 {
            for chunk in chunks {
                let mut parser = Self::new(chunk);

                let expression = parser.parse_expression()?;
                match expression {
                    Expression::Identifier { name } => {
                        expressions.push(Expression::FunctionParameter { name })
                    }
                    _ => {
                        return Err(EngineError::parser_error(format!(
                            "Invalid function parameter: {expression:#?}"
                        )))
                    }
                }
            }
        }

        self.current_token = end + 1;

        Ok(expressions)
    }

    fn parse_oparen_as_function_arguments(&mut self) -> Result<Vec<Expression>, EngineError> {
        let mut extra_paren: usize = 0;
        let mut found_close = false;
        self.current_token += 1;
        let start = self.current_token;

        while self.current_token < self.tokens.len() {
            let token = match self.tokens.get(self.current_token) {
                Some(val) => val,
                None => {
                    return Err(EngineError::parser_error(format!(
                        "parse_oparen_as_function_arguments token not found on pos {}",
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
                    break;
                }
            }

            self.current_token += 1;
        }

        if !found_close {
            return Err(EngineError::parser_error("Expected closed OParen"));
        }

        let end = self.current_token;

        let spliced = &self.tokens[start..end];

        let mut chunks: Vec<Vec<Token>> = vec![];
        let mut current_chunk = 0;

        for token in spliced {
            if matches!(token.kind, TokenKind::Comma) {
                current_chunk += 1;
                continue;
            } else {
                if current_chunk + 1 > chunks.len() {
                    chunks.push(vec![]);
                }
                chunks.get_mut(current_chunk).unwrap().push(token.clone());
            }
        }

        let mut expressions = vec![];

        if chunks.len() > 0 {
            for chunk in chunks {
                let mut parser = Self::new(chunk);

                let expression = parser.parse_expression()?;

                expressions.push(expression)
            }
        }

        self.current_token = end;

        Ok(expressions)
    }

    fn parse_binary_op_expression(
        &mut self,
        left_expression: Option<&Expression>,
    ) -> Result<Expression, EngineError> {
        let left = if let Some(left_expression) = left_expression {
            left_expression.clone()
        } else {
            match self.tokens.get(self.current_token) {
                Some(left_token) => match left_token.kind {
                    TokenKind::Identifier => Expression::Identifier {
                        name: left_token.clone().text,
                    },
                    TokenKind::Number => Expression::NumberLiteral {
                        value: left_token.text.parse::<f64>().unwrap(),
                    },
                    _ => {
                        return Err(EngineError::parser_error(format!(
                            "parse_binary_op_expression unexpected left token {:#?}",
                            left_token
                        )));
                    }
                },
                None => {
                    return Err(EngineError::parser_error(
                        "parse_binary_op_expression expected token",
                    ));
                }
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

        match self.parse_expression() {
            Ok(right) => Ok(Self::reorder_expression(&Expression::BinaryOp {
                left: Rc::new(left),
                op,
                right: Rc::new(right),
            })),
            Err(err) => Err(err),
        }
    }

    pub fn reorder_expression(expr: &Expression) -> Expression {
        match expr {
            Expression::BinaryOp { left, op, right } => {
                let left = Self::reorder_expression(left);
                let right = Self::reorder_expression(right);

                if let Expression::BinaryOp {
                    left: right_left,
                    op: right_op,
                    right: right_right,
                } = right.clone()
                {
                    if Self::get_precedence(&op) > Self::get_precedence(&right_op) {
                        let new_left = Expression::BinaryOp {
                            left: Rc::from(left),
                            op: op.clone(),
                            right: right_left,
                        };

                        return Expression::BinaryOp {
                            left: Rc::from(new_left),
                            op: right_op,
                            right: right_right,
                        };
                    }
                }

                return Expression::BinaryOp {
                    left: Rc::from(left),
                    op: op.clone(),
                    right: Rc::from(right),
                };
            }
            _ => expr.clone(),
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
