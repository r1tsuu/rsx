use crate::lexer::{Lexer, Token};

#[derive(Debug)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug)]
pub struct IdentifierExpression {
    pub name: String,
}

#[derive(Debug)]
pub struct NumericLiteralExpression {
    pub value: f32,
}

#[derive(Debug)]
pub enum Expression {
    Binary(BinaryExpression),
    Identifier(IdentifierExpression),
    NumericLiteral(NumericLiteralExpression),
}

impl Expression {
    pub fn try_as_binary(&self) -> Option<&BinaryExpression> {
        match self {
            Expression::Binary(expr) => Some(expr),
            _ => None,
        }
    }

    pub fn try_as_identifier(&self) -> Option<&IdentifierExpression> {
        match self {
            Expression::Identifier(expr) => Some(expr),
            _ => None,
        }
    }

    pub fn try_as_numeric_literal(&self) -> Option<&NumericLiteralExpression> {
        match self {
            Expression::NumericLiteral(expr) => Some(expr),
            _ => None,
        }
    }
}

pub struct ExpressionStatement {
    pub expression: Box<Expression>,
}

pub struct LetStatement {
    pub name: String,
    pub value: Box<Expression>,
}

pub enum Statement {
    Expression(ExpressionStatement),
    Let(LetStatement),
}

impl Statement {
    pub fn try_as_expression(&self) -> Option<&ExpressionStatement> {
        match self {
            Statement::Expression(stmt) => Some(stmt),
            _ => None,
        }
    }

    pub fn try_as_let(&self) -> Option<&LetStatement> {
        match self {
            Statement::Let(stmt) => Some(stmt),
            _ => None,
        }
    }
}

pub struct ASTParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ASTParser {
    fn peek_token(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn advance_token(&mut self) -> Option<Token> {
        let token = self.peek_token();
        self.pos += 1;
        token
    }

    fn parse_primary(&mut self) -> Expression {
        let token = self.peek_token().unwrap();

        match token {
            Token::NumericLiteral(token) => {
                self.advance_token();
                Expression::NumericLiteral(NumericLiteralExpression { value: token.value })
            }
            Token::Identifier(token) => {
                self.advance_token();
                Expression::Identifier(IdentifierExpression { name: token.name })
            }
            Token::LParen => {
                self.advance_token();
                let expression = self.parse_expression();

                let next = self.peek_token().unwrap();
                if !matches!(next, Token::RParen) {
                    panic!("Must match")
                }
                self.advance_token();

                expression
            }
            _ => todo!("asd"),
        }
    }

    fn parse_term(&mut self) -> Expression {
        let mut expr = self.parse_primary();

        while let Some(token) = self.peek_token()
            && (matches!(token, Token::Slash) || matches!(token, Token::Star))
        {
            self.advance_token();
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: token,
                right: Box::new(self.parse_primary()),
            });
        }

        expr
    }

    fn parse_expression(&mut self) -> Expression {
        let mut expr = self.parse_term();

        while let Some(token) = self.peek_token()
            && (matches!(token, Token::Plus) || matches!(token, Token::Minus))
        {
            self.advance_token();

            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: token,
                right: Box::new(self.parse_term()),
            });
        }

        expr
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek_token().unwrap() {
            Token::LetKeyword => {
                self.advance_token();
                if let Some(token) = self.advance_token()
                    && let Token::Identifier(identifier_token) = token
                {
                    self.advance_token();
                    Ok(Statement::Let(LetStatement {
                        name: identifier_token.name,
                        value: Box::new(self.parse_expression()),
                    }))
                } else {
                    Err("Expected identifier and a statement after let".to_string())
                }
            }
            _ => Ok(Statement::Expression(ExpressionStatement {
                expression: Box::new(self.parse_expression()),
            })),
        }
    }

    pub fn parse_from_tokens(tokens: Vec<Token>) -> Result<Vec<Statement>, String> {
        let mut ast = Self { pos: 0, tokens };
        let mut result: Vec<Statement> = vec![];

        while let Some(token) = ast.peek_token()
            && !matches!(token, Token::End)
        {
            let statement = ast.parse_statement()?;

            result.push(statement);

            if let Some(token) = ast.peek_token()
                && !matches!(token, Token::Semicolon)
            {
                return Err("Expected a semicolon".to_string());
            }

            ast.advance_token();
        }

        Ok(result)
    }

    pub fn parse_from_source(source: &str) -> Result<Vec<Statement>, String> {
        let tokens = Lexer::tokenize(source)?;
        Self::parse_from_tokens(tokens)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ASTParser, Expression},
        lexer::Token,
    };

    #[test]
    fn test_parse_empty_tokens() {
        let result = ASTParser::parse_from_source("").unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_numeric_literal() {
        let result = ASTParser::parse_from_source("42;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_numeric_literal().unwrap();
        assert_eq!(expr.value, 42.0);
    }

    #[test]
    fn test_parse_identifier() {
        let result = ASTParser::parse_from_source("x;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_identifier().unwrap();
        assert_eq!(expr.name, "x");
    }

    #[test]
    fn test_parse_addition() {
        let result = ASTParser::parse_from_source("5 + 3;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Plus));
        assert!(matches!(*expr.left, Expression::NumericLiteral(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_subtraction() {
        let result = ASTParser::parse_from_source("10 - 4;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Minus));
        assert!(matches!(*expr.left, Expression::NumericLiteral(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_multiplication() {
        let result = ASTParser::parse_from_source("6 * 7;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Star));
        assert!(matches!(*expr.left, Expression::NumericLiteral(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_division() {
        let result = ASTParser::parse_from_source("20 / 4;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Slash));
        assert!(matches!(*expr.left, Expression::NumericLiteral(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_complex_expression() {
        let result = ASTParser::parse_from_source("2 + 3 * 4;").unwrap();
        assert_eq!(result.len(), 1);

        // Should be: 2 + (3 * 4) - multiplication has higher precedence
        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Plus));
        assert!(matches!(*expr.left, Expression::NumericLiteral(_)));
        // Right side should be a Binary expression (3 * 4)
        assert!(matches!(*expr.right, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_parenthesized_expression() {
        let result = ASTParser::parse_from_source("(5 + 3) * 2;").unwrap();
        assert_eq!(result.len(), 1);

        // Should be: (5 + 3) * 2 - parentheses override precedence
        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Star));
        // Left side should be a Binary expression (5 + 3)
        assert!(matches!(*expr.left, Expression::Binary(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_identifier_in_expression() {
        let result = ASTParser::parse_from_source("x + 10;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Plus));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_multiple_statements() {
        let result = ASTParser::parse_from_source("1; 2;").unwrap();
        assert_eq!(result.len(), 2);

        let stmt0 = result[0].try_as_expression().unwrap();
        assert!(matches!(*stmt0.expression, Expression::NumericLiteral(_)));

        let stmt1 = result[1].try_as_expression().unwrap();
        assert!(matches!(*stmt1.expression, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_chained_operations() {
        let result = ASTParser::parse_from_source("1 + 2 + 3;").unwrap();
        assert_eq!(result.len(), 1);

        // Should be left-associative: (1 + 2) + 3
        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Plus));
        // Left side should be a Binary expression (1 + 2)
        assert!(matches!(*expr.left, Expression::Binary(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_nested_parentheses() {
        let result = ASTParser::parse_from_source("((1 + 2) * 3);").unwrap();
        assert_eq!(result.len(), 1);

        // Should be: ((1 + 2) * 3)
        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Star));
        // Left side should be nested Binary expression
        assert!(matches!(*expr.left, Expression::Binary(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_mixed_identifiers_and_numbers() {
        let result = ASTParser::parse_from_source("a * b + 5;").unwrap();
        assert_eq!(result.len(), 1);

        // Should be: (a * b) + 5 - multiplication has higher precedence
        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Plus));
        // Left side should be a Binary expression (a * b)
        assert!(matches!(*expr.left, Expression::Binary(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_let_statement() {
        let result = ASTParser::parse_from_source("let x = 42;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_let().unwrap();
        assert_eq!(stmt.name, "x");

        let expr = stmt.value.try_as_numeric_literal().unwrap();
        assert_eq!(expr.value, 42.0);
    }

    #[test]
    fn test_parse_let_with_expression() {
        let result = ASTParser::parse_from_source("let y = 10 + 5;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_let().unwrap();
        assert_eq!(stmt.name, "y");

        let expr = stmt.value.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Plus));
    }
}
