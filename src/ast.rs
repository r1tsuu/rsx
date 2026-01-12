use crate::{
    error::EngineError,
    lexer::{Lexer, Token},
};

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct IdentifierExpression {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct NumericLiteralExpression {
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct FunctionCallExpression {
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct ElementAccessExpression {
    pub expression: Box<Expression>,
    pub element: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum ObjectPropertyName {
    Name(String),
    Computed(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub name: ObjectPropertyName,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ObjectLiteralExpression {
    pub properties: Vec<ObjectProperty>,
}

#[derive(Debug, Clone)]
pub struct ArrayLiteralExpression {
    pub elements: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct PropertyAccessExpression {
    pub expression: Box<Expression>,
    pub property: String,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(BinaryExpression),
    Identifier(IdentifierExpression),
    NumericLiteral(NumericLiteralExpression),
    ObjectLiteral(ObjectLiteralExpression),
    ArrayLiteral(ArrayLiteralExpression),
    ElementAccess(ElementAccessExpression),
    PropertyAccess(PropertyAccessExpression),
    FunctionCall(FunctionCallExpression),
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub name: String,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinitionStatement {
    pub name: String,
    pub arguments: Vec<String>,
    pub block: Box<BlockStatement>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Box<Expression>,
    pub then: Box<Statement>,
    pub else_: Option<Box<Statement>>,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(ExpressionStatement),
    Let(LetStatement),
    Block(BlockStatement),
    FunctionDefinition(FunctionDefinitionStatement),
    If(IfStatement),
    Return(ReturnStatement),
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

    pub fn try_as_element_access(&self) -> Option<&ElementAccessExpression> {
        match self {
            Expression::ElementAccess(expr) => Some(expr),
            _ => None,
        }
    }

    pub fn try_as_property_access(&self) -> Option<&PropertyAccessExpression> {
        match self {
            Expression::PropertyAccess(expr) => Some(expr),
            _ => None,
        }
    }

    pub fn try_as_function_call(&self) -> Option<&FunctionCallExpression> {
        match self {
            Expression::FunctionCall(expr) => Some(expr),
            _ => None,
        }
    }

    pub fn try_as_object_literal(&self) -> Option<&ObjectLiteralExpression> {
        match self {
            Expression::ObjectLiteral(expr) => Some(expr),
            _ => None,
        }
    }

    pub fn try_as_array_literal(&self) -> Option<&ArrayLiteralExpression> {
        match self {
            Expression::ArrayLiteral(expr) => Some(expr),
            _ => None,
        }
    }
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

    pub fn try_as_block(&self) -> Option<&BlockStatement> {
        match self {
            Statement::Block(stmt) => Some(stmt),
            _ => None,
        }
    }

    pub fn try_as_function_definition(&self) -> Option<&FunctionDefinitionStatement> {
        match self {
            Statement::FunctionDefinition(stmt) => Some(stmt),
            _ => None,
        }
    }

    pub fn try_as_if(&self) -> Option<&IfStatement> {
        match self {
            Statement::If(stmt) => Some(stmt),
            _ => None,
        }
    }

    pub fn try_as_return(&self) -> Option<&ReturnStatement> {
        match self {
            Statement::Return(stmt) => Some(stmt),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ASTParser {
    tokens: Vec<Token>,
    pos: usize,
    inside_function: bool,
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

    fn parse_primary(&mut self) -> Result<Expression, EngineError> {
        let token = self.peek_token().unwrap();

        let mut expr = match token {
            Token::NumericLiteral(token) => {
                self.advance_token();
                Expression::NumericLiteral(NumericLiteralExpression { value: token.value })
            }
            Token::Identifier(token) => {
                self.advance_token();

                Expression::Identifier(IdentifierExpression { name: token.name })
            }
            Token::LBracket => {
                self.advance_token();
                let mut elements: Vec<Expression> = vec![];

                loop {
                    let next = self
                        .peek_token()
                        .ok_or_else(|| EngineError::ast("Expected a token in array defintion"))?;

                    if matches!(next, Token::RBracket) {
                        self.advance_token();
                        break;
                    }

                    elements.push(self.parse_expression()?);

                    let next = self
                        .peek_token()
                        .ok_or_else(|| EngineError::ast("Expected a token in array defintion"))?;

                    if matches!(next, Token::Comma) {
                        self.advance_token();
                        continue;
                    }

                    if !matches!(next, Token::RBracket) {
                        return Err(EngineError::ast(format!(
                            "
                      Expected either COMMA or RBracket after array element, got: {:#?}
                      ",
                            next
                        )));
                    }
                }

                Expression::ArrayLiteral(ArrayLiteralExpression { elements })
            }
            Token::LBrace => {
                self.advance_token();
                let mut properties: Vec<ObjectProperty> = vec![];

                loop {
                    let next = self
                        .advance_token()
                        .ok_or_else(|| EngineError::ast("Expected a token in object defintion"))?;

                    if matches!(next, Token::RBrace) {
                        break;
                    }

                    let name: ObjectPropertyName;

                    if let Token::Identifier(identifier) = next {
                        name = ObjectPropertyName::Name(identifier.name);
                    } else if matches!(next, Token::LBracket) {
                        name = ObjectPropertyName::Computed(Box::new(self.parse_expression()?));

                        self.advance_token()
                            .ok_or_else(|| {
                                EngineError::ast("Expected RBracket in computed property name")
                            })
                            .and_then(|token| {
                                if matches!(token, Token::RBracket) {
                                    Ok(())
                                } else {
                                    Err(EngineError::ast(format!(
                                        "Expected RBracket in computed property name, got: {:#?}",
                                        token
                                    )))
                                }
                            })?;
                    } else {
                        return Err(EngineError::ast(format!(
                            "Expected either an identifier or a computed property starting with RBracket in object definition, got: {:#?}",
                            next
                        )));
                    }

                    let next = self
                        .advance_token()
                        .ok_or_else(|| EngineError::ast("Expected a token in object defintion"))?;

                    if !matches!(next, Token::Colon) {
                        return Err(EngineError::ast(format!(
                            "Expected Colon  in object definition after ObjectPropertyName, got: {:#?}",
                            next
                        )));
                    }

                    let property = ObjectProperty {
                        name,
                        value: Box::new(self.parse_expression()?),
                    };

                    let next = self
                        .peek_token()
                        .ok_or_else(|| EngineError::ast("Expected a token in object defintion"))?;

                    properties.push(property);

                    if matches!(next, Token::Comma) {
                        self.advance_token();
                        continue;
                    }

                    if !matches!(next, Token::RBrace) {
                        return Err(EngineError::ast(format!(
                            "
                    Expected Comma or RBrace in object definition after property, got: {:#?}
                    ",
                            next
                        )));
                    }
                }

                Expression::ObjectLiteral(ObjectLiteralExpression { properties })
            }
            Token::LParen => {
                self.advance_token();
                let expression = self.parse_expression()?;

                self.peek_token()
                    .ok_or_else(|| EngineError::ast("Expected a token after LParent"))
                    .and_then(|next| {
                        if matches!(next, Token::RParen) {
                            Ok(())
                        } else {
                            Err(EngineError::ast(format!(
                                "Expected RParen after expression end, got: {next:#?}"
                            )))
                        }
                    })?;

                self.advance_token();

                expression
            }
            _ => {
                return Err(EngineError::ast(format!(
                    "Expression starting with {:#?} is not impl",
                    token
                )));
            }
        };

        let mut clone = self.clone();

        while let Some(token) = clone.peek_token() {
            match token {
                Token::LBracket => {
                    clone.advance_token();
                    let element = clone.parse_expression()?;
                    let next = clone.advance_token();
                    if let Some(token) = &next
                        && matches!(token, Token::RBracket)
                    {
                        expr = Expression::ElementAccess(ElementAccessExpression {
                            expression: Box::new(expr),
                            element: Box::new(element),
                        });
                    } else {
                        return Err(EngineError::ast(format!(
                            "Expected RBracket for ElementAccessExpression, got: {:#?}",
                            next
                        )));
                    }
                }
                Token::Dot => {
                    clone.advance_token();
                    let next = clone.advance_token();

                    if let Some(token) = &next
                        && let Token::Identifier(identifier) = token
                    {
                        expr = Expression::PropertyAccess(PropertyAccessExpression {
                            expression: Box::new(expr),
                            property: identifier.name.clone(),
                        });
                    } else {
                        return Err(EngineError::ast(format!(
                            "Expected Identifier for PropertyAccessExpression, got: {:#?}",
                            next
                        )));
                    }
                }
                Token::LParen => {
                    clone.advance_token();
                    let mut arguments: Vec<Expression> = vec![];

                    if clone
                        .peek_token()
                        .map(|token| !matches!(token, Token::RParen))
                        .unwrap_or(true)
                    {
                        loop {
                            arguments.push(clone.parse_expression()?);

                            let next_token = clone.advance_token().ok_or_else(|| {
                                EngineError::ast("Expected a token in function call arguments")
                            })?;

                            if matches!(next_token, Token::Comma) {
                                continue;
                            }

                            if matches!(next_token, Token::RParen) {
                                break;
                            }

                            return Err(EngineError::ast(format!(
                                "Expected Comma or RParen in function call arguments, got: {:#?}",
                                next_token
                            )));
                        }
                    } else {
                        clone.advance_token();
                    }

                    expr = Expression::FunctionCall(FunctionCallExpression {
                        function: Box::new(expr),
                        arguments,
                    })
                }
                _ => {
                    break;
                }
            }
        }

        self.pos = clone.pos;

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, EngineError> {
        let mut expr = self.parse_primary()?;

        while let Some(token) = self.peek_token()
            && (matches!(token, Token::Slash) || matches!(token, Token::Star))
        {
            self.advance_token();
            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: token,
                right: Box::new(self.parse_primary()?),
            });
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, EngineError> {
        let mut expr = self.parse_factor()?;

        while let Some(token) = self.peek_token()
            && (matches!(token, Token::Plus) || matches!(token, Token::Minus))
        {
            self.advance_token();

            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: token,
                right: Box::new(self.parse_factor()?),
            });
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, EngineError> {
        let mut expr = self.parse_term()?;

        while let Some(token) = self.peek_token()
            && (matches!(token, Token::LessThanEqual)
                || matches!(token, Token::LessThan)
                || matches!(token, Token::GreaterThan)
                || matches!(token, Token::GreaterThanEqual))
        {
            self.advance_token();

            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: token,
                right: Box::new(self.parse_term()?),
            });
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression, EngineError> {
        let mut expr = self.parse_comparison()?;

        while let Some(token) = self.peek_token()
            && (matches!(token, Token::EqualEqual)
                || matches!(token, Token::EqualEqualEqual)
                || matches!(token, Token::BangEqual)
                || matches!(token, Token::BangEqualEqual))
        {
            self.advance_token();

            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: token,
                right: Box::new(self.parse_comparison()?),
            });
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, EngineError> {
        let mut expr = self.parse_equality()?;

        while let Some(token) = self.peek_token()
            && matches!(token, Token::AndAnd)
        {
            self.advance_token();

            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: token,
                right: Box::new(self.parse_equality()?),
            });
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expression, EngineError> {
        let mut expr = self.parse_logical_and()?;

        while let Some(token) = self.peek_token()
            && matches!(token, Token::OrOr)
        {
            self.advance_token();

            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: token,
                right: Box::new(self.parse_logical_and()?),
            });
        }

        Ok(expr)
    }

    fn parse_assignment(&mut self) -> Result<Expression, EngineError> {
        let mut expr = self.parse_logical_or()?;

        if let Some(Token::Equal) = self.peek_token() {
            self.advance_token();

            let value = self.parse_assignment()?;

            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator: Token::Equal,
                right: Box::new(value),
            })
        }

        Ok(expr)
    }

    fn parse_expression(&mut self) -> Result<Expression, EngineError> {
        self.parse_assignment()
    }

    fn parse_statement(&mut self) -> Result<Statement, EngineError> {
        match self.peek_token().unwrap() {
            Token::LetKeyword => {
                self.advance_token();
                if let Some(token) = self.advance_token()
                    && let Token::Identifier(identifier_token) = token
                {
                    self.advance_token();
                    Ok(Statement::Let(LetStatement {
                        name: identifier_token.name,
                        value: Box::new(self.parse_expression()?),
                    }))
                } else {
                    Err(EngineError::ast(
                        "Expected identifier and a statement after let",
                    ))
                }
            }
            Token::ReturnKeyword => {
                if !self.inside_function {
                    return Err(EngineError::ast(
                        "ReturnKeyword is allowed only within a function body",
                    ));
                }

                self.advance_token();
                Ok(Statement::Return(ReturnStatement {
                    expression: Box::new(self.parse_expression()?),
                }))
            }
            Token::IfKeyword => {
                self.advance_token();
                let condition = self.parse_expression()?;
                let then = self.parse_statement()?;
                let mut else_: Option<Statement> = None;

                if let Some(next) = self.peek_token()
                    && matches!(next, Token::ElseKeyword)
                {
                    self.advance_token();
                    else_ = Some(self.parse_statement()?);
                }

                Ok(Statement::If(IfStatement {
                    condition: Box::new(condition),
                    then: Box::new(then),
                    else_: else_.map(Box::new),
                }))
            }
            Token::FunctionKeyword => {
                self.advance_token();

                let token = self.advance_token().ok_or_else(|| {
                    EngineError::ast("Expected an identifier after function keyword, got: None")
                })?;

                let Token::Identifier(identifier) = token else {
                    return Err(EngineError::ast(format!(
                        "Expected an identifier after function keyword, got: {:#?}",
                        token
                    )));
                };

                let token = self.advance_token().ok_or_else(|| {
                    EngineError::ast(
                        "Expected LParen after function keyword and identifier, got: None",
                    )
                })?;

                if !matches!(token, Token::LParen) {
                    return Err(EngineError::ast(format!(
                        "Expected LParen after function keyword and identifier, got: {:#?}",
                        token
                    )));
                }

                let mut arguments: Vec<String> = vec![];

                loop {
                    let token = self.advance_token().ok_or_else(|| {
                        EngineError::ast("Expected a token in function arguments")
                    })?;

                    if matches!(token, Token::Comma) {
                        continue;
                    }

                    if matches!(token, Token::RParen) {
                        break;
                    }

                    if let Token::Identifier(identifier) = token {
                        let next = self.peek_token().ok_or_else(|| {
                        EngineError::ast("Expected a COMMA/RParen token in function arguments after identifier")
                    })?;

                        if !matches!(next, Token::Comma) && !matches!(next, Token::RParen) {
                            return Err(EngineError::ast(format!(
                                "Expected a COMMA/RParen token in function arguments after identifier, got: {:#?}",
                                next
                            )));
                        }

                        arguments.push(identifier.name.clone());
                    }
                }

                let prev_inside_function = self.inside_function;
                self.inside_function = true;
                let body = self.parse_statement()?;
                self.inside_function = prev_inside_function;

                let Statement::Block(block) = body else {
                    return Err(EngineError::ast(format!(
                        "Expected a block after function arguments, got: {:#?}",
                        body
                    )));
                };

                Ok(Statement::FunctionDefinition(FunctionDefinitionStatement {
                    name: identifier.name,
                    arguments,
                    block: Box::new(block),
                }))
            }
            Token::LBrace => {
                let mut statements: Vec<Statement> = vec![];
                self.advance_token();

                while let Some(token) = self.peek_token() {
                    if matches!(token, Token::End) {
                        break;
                    }

                    if matches!(token, Token::RBrace) {
                        self.advance_token();

                        if self.tokens.len() == self.pos + 1 {
                            self.advance_token();
                        }

                        break;
                    }

                    let statement = self.parse_statement()?;
                    statements.push(statement);

                    if let Some(next) = self.peek_token() {
                        if matches!(next, Token::RBrace) {
                            continue;
                        }

                        if !matches!(next, Token::Semicolon) {
                            return Err(EngineError::ast(format!(
                                "BLOCK: Expected a semicolon, got: {:?}",
                                next
                            )));
                        }
                    }

                    self.advance_token();
                }

                Ok(Statement::Block(BlockStatement { body: statements }))
            }
            _ => Ok(Statement::Expression(ExpressionStatement {
                expression: Box::new(self.parse_expression()?),
            })),
        }
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement>, EngineError> {
        let mut result: Vec<Statement> = vec![];

        while let Some(token) = self.peek_token() {
            if matches!(token, Token::End) {
                break;
            }

            let statement = self.parse_statement()?;

            result.push(statement);

            if let Some(token) = self.peek_token()
                && !matches!(token, Token::Semicolon)
            {
                return Err(EngineError::ast(format!(
                    "Expected a semicolon, got: {:?}",
                    token
                )));
            }

            self.advance_token();
        }

        Ok(result)
    }

    pub fn parse_from_tokens(tokens: Vec<Token>) -> Result<Vec<Statement>, EngineError> {
        let mut ast = Self {
            pos: 0,
            tokens,
            inside_function: false,
        };
        ast.parse_statements()
    }

    pub fn parse_from_source(source: &str) -> Result<Vec<Statement>, EngineError> {
        let tokens = Lexer::tokenize(source)?;
        Self::parse_from_tokens(tokens)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ASTParser, Expression, ObjectPropertyName},
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
    fn test_parse_equal_equal() {
        let result = ASTParser::parse_from_source("a == b;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::EqualEqual));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::Identifier(_)));
    }

    #[test]
    fn test_parse_equal_equal_equal() {
        let result = ASTParser::parse_from_source("x === 5;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::EqualEqualEqual));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_bang_equal() {
        let result = ASTParser::parse_from_source("x != y;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::BangEqual));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::Identifier(_)));
    }

    #[test]
    fn test_parse_bang_equal_equal() {
        let result = ASTParser::parse_from_source("a !== 10;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::BangEqualEqual));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_less_than() {
        let result = ASTParser::parse_from_source("x < 5;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::LessThan));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_less_than_equal() {
        let result = ASTParser::parse_from_source("a <= b;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::LessThanEqual));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::Identifier(_)));
    }

    #[test]
    fn test_parse_greater_than() {
        let result = ASTParser::parse_from_source("x > 10;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::GreaterThan));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_greater_than_equal() {
        let result = ASTParser::parse_from_source("y >= 20;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::GreaterThanEqual));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_and_and() {
        let result = ASTParser::parse_from_source("a && b;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::AndAnd));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::Identifier(_)));
    }

    #[test]
    fn test_parse_or_or() {
        let result = ASTParser::parse_from_source("x || y;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::OrOr));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
        assert!(matches!(*expr.right, Expression::Identifier(_)));
    }

    #[test]
    fn test_parse_assignment() {
        let result = ASTParser::parse_from_source("x = 5;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();
        assert!(matches!(expr.operator, Token::Equal));
        assert!(matches!(*expr.left, Expression::Identifier(_)));
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

    #[test]
    fn test_parse_element_access_numeric() {
        let result = ASTParser::parse_from_source("arr[0];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_element_access().unwrap();

        let id = expr.expression.try_as_identifier().unwrap();
        assert_eq!(id.name, "arr");

        let num = expr.element.try_as_numeric_literal().unwrap();
        assert_eq!(num.value, 0.0);
    }

    #[test]
    fn test_parse_element_access_identifier() {
        let result = ASTParser::parse_from_source("obj[key];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_element_access().unwrap();

        let obj_id = expr.expression.try_as_identifier().unwrap();
        assert_eq!(obj_id.name, "obj");

        let key_id = expr.element.try_as_identifier().unwrap();
        assert_eq!(key_id.name, "key");
    }

    #[test]
    fn test_parse_element_access_expression() {
        let result = ASTParser::parse_from_source("arr[i + 1];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_element_access().unwrap();

        assert!(matches!(*expr.expression, Expression::Identifier(_)));
        assert!(matches!(*expr.element, Expression::Binary(_)));
    }

    #[test]
    fn test_error_parse_element_access_expression() {
        let result = ASTParser::parse_from_source("arr[i + 1;").unwrap_err();
        assert!(
            result
                .message()
                .contains("Expected RBracket for ElementAccessExpression")
        );
    }

    #[test]
    fn test_parse_chained_element_access() {
        let result = ASTParser::parse_from_source("matrix[0][1];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_element_access().unwrap();

        // Outer access should have element 1
        let num = expr.element.try_as_numeric_literal().unwrap();
        assert_eq!(num.value, 1.0);

        // Inner expression should be another element access
        let inner = expr.expression.try_as_element_access().unwrap();
        inner.expression.try_as_identifier().unwrap();
        inner.element.try_as_numeric_literal().unwrap();
    }

    #[test]
    fn test_error_parse_property_access() {
        let result = ASTParser::parse_from_source("obj.123;").unwrap_err();
        assert!(
            result
                .message()
                .contains("Expected Identifier for PropertyAccessExpression")
        );
    }

    #[test]
    fn test_parse_element_access_in_expression() {
        let result = ASTParser::parse_from_source("arr[0] + arr[1];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();

        assert!(matches!(expr.operator, Token::Plus));
        assert!(matches!(*expr.left, Expression::ElementAccess(_)));
        assert!(matches!(*expr.right, Expression::ElementAccess(_)));
    }

    #[test]
    fn test_parse_property_access() {
        let result = ASTParser::parse_from_source("obj.prop;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_property_access().unwrap();

        let obj_id = expr.expression.try_as_identifier().unwrap();
        assert_eq!(obj_id.name, "obj");
        assert_eq!(expr.property, "prop");
    }

    #[test]
    fn test_parse_chained_property_access() {
        let result = ASTParser::parse_from_source("obj.a.b;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_property_access().unwrap();

        assert_eq!(expr.property, "b");

        let inner = expr.expression.try_as_property_access().unwrap();
        assert_eq!(inner.property, "a");
        let obj = inner.expression.try_as_identifier().unwrap();
        assert_eq!(obj.name, "obj");
    }

    #[test]
    fn test_parse_property_access_in_expression() {
        let result = ASTParser::parse_from_source("obj.x + obj.y;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_binary().unwrap();

        assert!(matches!(expr.operator, Token::Plus));
        assert!(matches!(*expr.left, Expression::PropertyAccess(_)));
        assert!(matches!(*expr.right, Expression::PropertyAccess(_)));
    }

    #[test]
    fn test_parse_property_access_after_element_access() {
        let result = ASTParser::parse_from_source("arr[0].prop;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_property_access().unwrap();

        assert_eq!(expr.property, "prop");
        let elem = expr.expression.try_as_element_access().unwrap();
        elem.expression.try_as_identifier().unwrap();
        elem.element.try_as_numeric_literal().unwrap();
    }

    #[test]
    fn test_parse_element_access_after_property_access() {
        let result = ASTParser::parse_from_source("obj.arr[0];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_element_access().unwrap();

        let num = expr.element.try_as_numeric_literal().unwrap();
        assert_eq!(num.value, 0.0);

        let prop = expr.expression.try_as_property_access().unwrap();
        assert_eq!(prop.property, "arr");
        let obj = prop.expression.try_as_identifier().unwrap();
        assert_eq!(obj.name, "obj");
    }

    #[test]
    fn test_parse_complex_member_access() {
        let result = ASTParser::parse_from_source("obj.items[0].name;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_property_access().unwrap();

        assert_eq!(expr.property, "name");

        let elem = expr.expression.try_as_element_access().unwrap();
        elem.element.try_as_numeric_literal().unwrap();

        let prop = elem.expression.try_as_property_access().unwrap();
        assert_eq!(prop.property, "items");
        let obj = prop.expression.try_as_identifier().unwrap();
        assert_eq!(obj.name, "obj");
    }

    #[test]
    fn test_parse_empty_block() {
        let result = ASTParser::parse_from_source("{}").unwrap();
        assert_eq!(result.len(), 1);

        let block = result[0].try_as_block().unwrap();
        assert_eq!(block.body.len(), 0);
    }

    #[test]
    fn test_parse_block_with_single_statement() {
        let result = ASTParser::parse_from_source("{ 42; }").unwrap();
        assert_eq!(result.len(), 1);

        let block = result[0].try_as_block().unwrap();
        assert_eq!(block.body.len(), 1);

        let stmt = block.body[0].try_as_expression().unwrap();
        let expr = stmt.expression.try_as_numeric_literal().unwrap();
        assert_eq!(expr.value, 42.0);
    }

    #[test]
    fn test_parse_block_with_multiple_statements() {
        let result = ASTParser::parse_from_source("{ let x = 1; let y = 2; }").unwrap();
        assert_eq!(result.len(), 1);

        let block = result[0].try_as_block().unwrap();
        assert_eq!(block.body.len(), 2);

        let stmt1 = block.body[0].try_as_let().unwrap();
        assert_eq!(stmt1.name, "x");

        let stmt2 = block.body[1].try_as_let().unwrap();
        assert_eq!(stmt2.name, "y");
    }

    #[test]
    fn test_parse_nested_blocks() {
        let result = ASTParser::parse_from_source("{ { 1; } }").unwrap();
        assert_eq!(result.len(), 1);

        let outer = result[0].try_as_block().unwrap();
        assert_eq!(outer.body.len(), 1);

        let inner = outer.body[0].try_as_block().unwrap();
        assert_eq!(inner.body.len(), 1);

        let stmt = inner.body[0].try_as_expression().unwrap();
        stmt.expression.try_as_numeric_literal().unwrap();
    }

    #[test]
    fn test_parse_block_with_expressions() {
        let result = ASTParser::parse_from_source("{ 1 + 2; 3 * 4; }").unwrap();
        assert_eq!(result.len(), 1);

        let block = result[0].try_as_block().unwrap();
        assert_eq!(block.body.len(), 2);

        let stmt1 = block.body[0].try_as_expression().unwrap();
        assert!(matches!(*stmt1.expression, Expression::Binary(_)));

        let stmt2 = block.body[1].try_as_expression().unwrap();
        assert!(matches!(*stmt2.expression, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_block_mixed_statements() {
        let result = ASTParser::parse_from_source("{ let x = 5; x + 10; }").unwrap();
        assert_eq!(result.len(), 1);

        let block = result[0].try_as_block().unwrap();
        assert_eq!(block.body.len(), 2);

        let let_stmt = block.body[0].try_as_let().unwrap();
        assert_eq!(let_stmt.name, "x");

        let expr_stmt = block.body[1].try_as_expression().unwrap();
        assert!(matches!(*expr_stmt.expression, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_function_no_params() {
        let result = ASTParser::parse_from_source("function foo() { }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        assert_eq!(func.name, "foo");
        assert_eq!(func.arguments.len(), 0);
        assert_eq!(func.block.body.len(), 0);
    }

    #[test]
    fn test_parse_function_single_param() {
        let result = ASTParser::parse_from_source("function add(x) { }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        assert_eq!(func.name, "add");
        assert_eq!(func.arguments.len(), 1);
        assert_eq!(func.arguments[0], "x");
    }

    #[test]
    fn test_parse_function_multiple_params() {
        let result = ASTParser::parse_from_source("function add(x, y) { }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        assert_eq!(func.name, "add");
        assert_eq!(func.arguments.len(), 2);
        assert_eq!(func.arguments[0], "x");
        assert_eq!(func.arguments[1], "y");
    }

    #[test]
    fn test_parse_function_with_body() {
        let result = ASTParser::parse_from_source("function test() { let x = 1; }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        assert_eq!(func.name, "test");
        assert_eq!(func.block.body.len(), 1);

        let stmt = func.block.body[0].try_as_let().unwrap();
        assert_eq!(stmt.name, "x");
    }

    #[test]
    fn test_parse_function_complex_body() {
        let result =
            ASTParser::parse_from_source("function calc(a, b) { let x = a + b; x * 2; }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        assert_eq!(func.name, "calc");
        assert_eq!(func.arguments.len(), 2);
        assert_eq!(func.block.body.len(), 2);

        let let_stmt = func.block.body[0].try_as_let().unwrap();
        assert_eq!(let_stmt.name, "x");

        let expr_stmt = func.block.body[1].try_as_expression().unwrap();
        assert!(matches!(*expr_stmt.expression, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_nested_function() {
        let result =
            ASTParser::parse_from_source("function outer() { function inner() { } }").unwrap();
        assert_eq!(result.len(), 1);

        let outer_func = result[0].try_as_function_definition().unwrap();
        assert_eq!(outer_func.name, "outer");
        assert_eq!(outer_func.block.body.len(), 1);

        let inner_func = outer_func.block.body[0]
            .try_as_function_definition()
            .unwrap();
        assert_eq!(inner_func.name, "inner");
    }

    #[test]
    fn test_parse_function_three_params() {
        let result = ASTParser::parse_from_source("function test(a, b, c) { }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        assert_eq!(func.arguments.len(), 3);
        assert_eq!(func.arguments[0], "a");
        assert_eq!(func.arguments[1], "b");
        assert_eq!(func.arguments[2], "c");
    }

    #[test]
    fn test_parse_function_call_no_args() {
        let result = ASTParser::parse_from_source("foo();").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let call = stmt.expression.try_as_function_call().unwrap();

        let func_id = call.function.try_as_identifier().unwrap();
        assert_eq!(func_id.name, "foo");
        assert_eq!(call.arguments.len(), 0);
    }

    #[test]
    fn test_parse_function_call_single_arg() {
        let result = ASTParser::parse_from_source("add(5);").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let call = stmt.expression.try_as_function_call().unwrap();

        let func_id = call.function.try_as_identifier().unwrap();
        assert_eq!(func_id.name, "add");
        assert_eq!(call.arguments.len(), 1);

        let arg = call.arguments[0].try_as_numeric_literal().unwrap();
        assert_eq!(arg.value, 5.0);
    }

    #[test]
    fn test_parse_function_call_multiple_args() {
        let result = ASTParser::parse_from_source("add(1, 2, 3);").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let call = stmt.expression.try_as_function_call().unwrap();

        assert_eq!(call.arguments.len(), 3);
        assert!(matches!(call.arguments[0], Expression::NumericLiteral(_)));
        assert!(matches!(call.arguments[1], Expression::NumericLiteral(_)));
        assert!(matches!(call.arguments[2], Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_function_call_with_expression_args() {
        let result = ASTParser::parse_from_source("calc(x + 1, y * 2);").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let call = stmt.expression.try_as_function_call().unwrap();

        assert_eq!(call.arguments.len(), 2);
        assert!(matches!(call.arguments[0], Expression::Binary(_)));
        assert!(matches!(call.arguments[1], Expression::Binary(_)));
    }

    #[test]
    fn test_parse_nested_function_call() {
        let result = ASTParser::parse_from_source("outer(inner());").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let outer_call = stmt.expression.try_as_function_call().unwrap();

        let outer_id = outer_call.function.try_as_identifier().unwrap();
        assert_eq!(outer_id.name, "outer");
        assert_eq!(outer_call.arguments.len(), 1);

        let inner_call = outer_call.arguments[0].try_as_function_call().unwrap();
        let inner_id = inner_call.function.try_as_identifier().unwrap();
        assert_eq!(inner_id.name, "inner");
    }

    #[test]
    fn test_parse_function_call_on_property() {
        let result = ASTParser::parse_from_source("obj.method();").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let call = stmt.expression.try_as_function_call().unwrap();

        let prop = call.function.try_as_property_access().unwrap();
        assert_eq!(prop.property, "method");

        let obj = prop.expression.try_as_identifier().unwrap();
        assert_eq!(obj.name, "obj");
    }

    #[test]
    fn test_parse_chained_function_calls() {
        let result = ASTParser::parse_from_source("foo()();").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let outer_call = stmt.expression.try_as_function_call().unwrap();

        assert_eq!(outer_call.arguments.len(), 0);

        let inner_call = outer_call.function.try_as_function_call().unwrap();
        let func_id = inner_call.function.try_as_identifier().unwrap();
        assert_eq!(func_id.name, "foo");
    }

    #[test]
    fn test_parse_function_call_in_expression() {
        let result = ASTParser::parse_from_source("foo() + bar();").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let binary = stmt.expression.try_as_binary().unwrap();

        assert!(matches!(binary.operator, Token::Plus));
        assert!(matches!(*binary.left, Expression::FunctionCall(_)));
        assert!(matches!(*binary.right, Expression::FunctionCall(_)));
    }

    #[test]
    fn test_parse_empty_object_literal() {
        let result = ASTParser::parse_from_source("({});").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let obj = stmt.expression.try_as_object_literal().unwrap();
        assert_eq!(obj.properties.len(), 0);
    }

    #[test]
    fn test_parse_object_literal_single_property() {
        let result = ASTParser::parse_from_source("({x: 1});").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let obj = stmt.expression.try_as_object_literal().unwrap();
        assert_eq!(obj.properties.len(), 1);

        let prop = &obj.properties[0];
        assert!(matches!(prop.name, ObjectPropertyName::Name(_)));
        if let ObjectPropertyName::Name(name) = &prop.name {
            assert_eq!(name, "x");
        }
        assert!(matches!(*prop.value, Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_object_literal_multiple_properties() {
        let result = ASTParser::parse_from_source("({x: 1, y: 2, z: 3});").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let obj = stmt.expression.try_as_object_literal().unwrap();
        assert_eq!(obj.properties.len(), 3);
    }

    #[test]
    fn test_parse_object_literal_identifier_values() {
        let result = ASTParser::parse_from_source("({x: a, y: b});").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let obj = stmt.expression.try_as_object_literal().unwrap();
        assert_eq!(obj.properties.len(), 2);

        assert!(matches!(
            *obj.properties[0].value,
            Expression::Identifier(_)
        ));
        assert!(matches!(
            *obj.properties[1].value,
            Expression::Identifier(_)
        ));
    }

    #[test]
    fn test_parse_object_literal_expression_values() {
        let result = ASTParser::parse_from_source("({x: a + 1, y: b * 2});").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let obj = stmt.expression.try_as_object_literal().unwrap();
        assert_eq!(obj.properties.len(), 2);

        assert!(matches!(*obj.properties[0].value, Expression::Binary(_)));
        assert!(matches!(*obj.properties[1].value, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_object_literal_computed_property() {
        let result = ASTParser::parse_from_source("({[key]: value});").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let obj = stmt.expression.try_as_object_literal().unwrap();
        assert_eq!(obj.properties.len(), 1);

        let prop = &obj.properties[0];
        assert!(matches!(prop.name, ObjectPropertyName::Computed(_)));
        if let ObjectPropertyName::Computed(expr) = &prop.name {
            assert!(matches!(**expr, Expression::Identifier(_)));
        }
    }

    #[test]
    fn test_parse_nested_object_literal() {
        let result = ASTParser::parse_from_source("({a: {b: 1}});").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let obj = stmt.expression.try_as_object_literal().unwrap();
        assert_eq!(obj.properties.len(), 1);

        let inner_obj = obj.properties[0].value.try_as_object_literal().unwrap();
        assert_eq!(inner_obj.properties.len(), 1);
    }

    #[test]
    fn test_parse_object_literal_in_expression() {
        let result = ASTParser::parse_from_source("({x: 1}).x;").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let prop_access = stmt.expression.try_as_property_access().unwrap();

        assert_eq!(prop_access.property, "x");
        assert!(matches!(
            *prop_access.expression,
            Expression::ObjectLiteral(_)
        ));
    }

    #[test]
    fn test_parse_object_literal_with_function_call_value() {
        let result = ASTParser::parse_from_source("({x: foo()});").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let obj = stmt.expression.try_as_object_literal().unwrap();
        assert_eq!(obj.properties.len(), 1);

        assert!(matches!(
            *obj.properties[0].value,
            Expression::FunctionCall(_)
        ));
    }

    #[test]
    fn test_parse_empty_array_literal() {
        let result = ASTParser::parse_from_source("[];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let arr = stmt.expression.try_as_array_literal().unwrap();
        assert_eq!(arr.elements.len(), 0);
    }

    #[test]
    fn test_parse_array_literal_single_element() {
        let result = ASTParser::parse_from_source("[1];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let arr = stmt.expression.try_as_array_literal().unwrap();
        assert_eq!(arr.elements.len(), 1);

        let elem = arr.elements[0].try_as_numeric_literal().unwrap();
        assert_eq!(elem.value, 1.0);
    }

    #[test]
    fn test_parse_array_literal_multiple_elements() {
        let result = ASTParser::parse_from_source("[1, 2, 3];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let arr = stmt.expression.try_as_array_literal().unwrap();
        assert_eq!(arr.elements.len(), 3);

        assert!(matches!(arr.elements[0], Expression::NumericLiteral(_)));
        assert!(matches!(arr.elements[1], Expression::NumericLiteral(_)));
        assert!(matches!(arr.elements[2], Expression::NumericLiteral(_)));
    }

    #[test]
    fn test_parse_array_literal_identifier_elements() {
        let result = ASTParser::parse_from_source("[a, b, c];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let arr = stmt.expression.try_as_array_literal().unwrap();
        assert_eq!(arr.elements.len(), 3);

        assert!(matches!(arr.elements[0], Expression::Identifier(_)));
        assert!(matches!(arr.elements[1], Expression::Identifier(_)));
        assert!(matches!(arr.elements[2], Expression::Identifier(_)));
    }

    #[test]
    fn test_parse_array_literal_expression_elements() {
        let result = ASTParser::parse_from_source("[a + 1, b * 2];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let arr = stmt.expression.try_as_array_literal().unwrap();
        assert_eq!(arr.elements.len(), 2);

        assert!(matches!(arr.elements[0], Expression::Binary(_)));
        assert!(matches!(arr.elements[1], Expression::Binary(_)));
    }

    #[test]
    fn test_parse_nested_array_literal() {
        let result = ASTParser::parse_from_source("[[1, 2], [3, 4]];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let arr = stmt.expression.try_as_array_literal().unwrap();
        assert_eq!(arr.elements.len(), 2);

        let inner1 = arr.elements[0].try_as_array_literal().unwrap();
        assert_eq!(inner1.elements.len(), 2);

        let inner2 = arr.elements[1].try_as_array_literal().unwrap();
        assert_eq!(inner2.elements.len(), 2);
    }

    #[test]
    fn test_parse_array_literal_in_expression() {
        let result = ASTParser::parse_from_source("[1, 2][0];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let elem_access = stmt.expression.try_as_element_access().unwrap();

        let num = elem_access.element.try_as_numeric_literal().unwrap();
        assert_eq!(num.value, 0.0);

        assert!(matches!(
            *elem_access.expression,
            Expression::ArrayLiteral(_)
        ));
    }

    #[test]
    fn test_parse_array_literal_with_function_call() {
        let result = ASTParser::parse_from_source("[foo(), bar()];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let arr = stmt.expression.try_as_array_literal().unwrap();
        assert_eq!(arr.elements.len(), 2);

        assert!(matches!(arr.elements[0], Expression::FunctionCall(_)));
        assert!(matches!(arr.elements[1], Expression::FunctionCall(_)));
    }

    #[test]
    fn test_parse_array_literal_mixed_types() {
        let result = ASTParser::parse_from_source("[1, a, foo()];").unwrap();
        assert_eq!(result.len(), 1);

        let stmt = result[0].try_as_expression().unwrap();
        let arr = stmt.expression.try_as_array_literal().unwrap();
        assert_eq!(arr.elements.len(), 3);

        assert!(matches!(arr.elements[0], Expression::NumericLiteral(_)));
        assert!(matches!(arr.elements[1], Expression::Identifier(_)));
        assert!(matches!(arr.elements[2], Expression::FunctionCall(_)));
    }

    #[test]
    fn test_parse_if_statement() {
        let result = ASTParser::parse_from_source("if (x) { 1; }").unwrap();
        assert_eq!(result.len(), 1);

        let if_stmt = result[0].try_as_if().unwrap();

        let cond = if_stmt.condition.try_as_identifier().unwrap();
        assert_eq!(cond.name, "x");

        let then_block = if_stmt.then.try_as_block().unwrap();
        assert_eq!(then_block.body.len(), 1);

        assert!(if_stmt.else_.is_none());
    }

    #[test]
    fn test_parse_if_with_else() {
        let result = ASTParser::parse_from_source("if (x) { 1; } else { 2; }").unwrap();
        assert_eq!(result.len(), 1);

        let if_stmt = result[0].try_as_if().unwrap();

        let cond = if_stmt.condition.try_as_identifier().unwrap();
        assert_eq!(cond.name, "x");

        let then_block = if_stmt.then.try_as_block().unwrap();
        assert_eq!(then_block.body.len(), 1);

        assert!(if_stmt.else_.is_some());
        let else_block = if_stmt.else_.as_ref().unwrap().try_as_block().unwrap();
        assert_eq!(else_block.body.len(), 1);
    }

    #[test]
    fn test_parse_if_with_else_if() {
        let result = ASTParser::parse_from_source("if (x) { 1; } else if (y) { 2; }").unwrap();
        assert_eq!(result.len(), 1);

        let if_stmt = result[0].try_as_if().unwrap();

        let cond = if_stmt.condition.try_as_identifier().unwrap();
        assert_eq!(cond.name, "x");

        assert!(if_stmt.else_.is_some());
        let else_if = if_stmt.else_.as_ref().unwrap().try_as_if().unwrap();

        let else_if_cond = else_if.condition.try_as_identifier().unwrap();
        assert_eq!(else_if_cond.name, "y");
    }

    #[test]
    fn test_parse_if_with_complex_condition() {
        let result = ASTParser::parse_from_source("if (x > 5) { 1; }").unwrap();
        assert_eq!(result.len(), 1);

        let if_stmt = result[0].try_as_if().unwrap();

        let cond = if_stmt.condition.try_as_binary().unwrap();
        assert!(matches!(cond.operator, Token::GreaterThan));
    }

    #[test]
    fn test_parse_if_with_logical_condition() {
        let result = ASTParser::parse_from_source("if (x && y) { 1; }").unwrap();
        assert_eq!(result.len(), 1);

        let if_stmt = result[0].try_as_if().unwrap();

        let cond = if_stmt.condition.try_as_binary().unwrap();
        assert!(matches!(cond.operator, Token::AndAnd));
    }

    #[test]
    fn test_parse_nested_if() {
        let result = ASTParser::parse_from_source("if (x) { if (y) { 1; } }").unwrap();
        assert_eq!(result.len(), 1);

        let outer_if = result[0].try_as_if().unwrap();
        let outer_block = outer_if.then.try_as_block().unwrap();
        assert_eq!(outer_block.body.len(), 1);

        let inner_if = outer_block.body[0].try_as_if().unwrap();
        let inner_cond = inner_if.condition.try_as_identifier().unwrap();
        assert_eq!(inner_cond.name, "y");
    }

    #[test]
    fn test_parse_if_with_expression_in_then() {
        let result = ASTParser::parse_from_source("if (x) { x + 1; }").unwrap();
        assert_eq!(result.len(), 1);

        let if_stmt = result[0].try_as_if().unwrap();
        let then_block = if_stmt.then.try_as_block().unwrap();
        assert_eq!(then_block.body.len(), 1);

        let expr_stmt = then_block.body[0].try_as_expression().unwrap();
        assert!(matches!(*expr_stmt.expression, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_if_else_chain() {
        let result =
            ASTParser::parse_from_source("if (x) { 1; } else if (y) { 2; } else { 3; }").unwrap();
        assert_eq!(result.len(), 1);

        let if_stmt = result[0].try_as_if().unwrap();

        assert!(if_stmt.else_.is_some());
        let else_if = if_stmt.else_.as_ref().unwrap().try_as_if().unwrap();

        assert!(else_if.else_.is_some());
        let final_else = else_if.else_.as_ref().unwrap().try_as_block().unwrap();
        assert_eq!(final_else.body.len(), 1);
    }

    #[test]
    fn test_parse_if_with_multiple_statements() {
        let result = ASTParser::parse_from_source("if (x) { let a = 1; let b = 2; }").unwrap();
        assert_eq!(result.len(), 1);

        let if_stmt = result[0].try_as_if().unwrap();
        let then_block = if_stmt.then.try_as_block().unwrap();
        assert_eq!(then_block.body.len(), 2);

        assert!(then_block.body[0].try_as_let().is_some());
        assert!(then_block.body[1].try_as_let().is_some());
    }

    #[test]
    fn test_parse_return_statement() {
        let result = ASTParser::parse_from_source("function foo() { return x; }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        assert_eq!(func.block.body.len(), 1);

        let ret_stmt = func.block.body[0].try_as_return().unwrap();
        let expr = ret_stmt.expression.try_as_identifier().unwrap();
        assert_eq!(expr.name, "x");
    }

    #[test]
    fn test_parse_return_with_expression() {
        let result = ASTParser::parse_from_source("function foo() { return x + 1; }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        let ret_stmt = func.block.body[0].try_as_return().unwrap();
        assert!(matches!(*ret_stmt.expression, Expression::Binary(_)));
    }

    #[test]
    fn test_parse_return_numeric() {
        let result = ASTParser::parse_from_source("function foo() { return 42; }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        let ret_stmt = func.block.body[0].try_as_return().unwrap();
        let num = ret_stmt.expression.try_as_numeric_literal().unwrap();
        assert_eq!(num.value, 42.0);
    }

    #[test]
    fn test_parse_return_outside_function_error() {
        let result = ASTParser::parse_from_source("return 42;").unwrap_err();
        assert!(
            result
                .message()
                .contains("ReturnKeyword is allowed only within a function body")
        );
    }

    #[test]
    fn test_parse_return_in_function() {
        let result = ASTParser::parse_from_source("function foo() { return a * b + c; }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        let ret_stmt = func.block.body[0].try_as_return().unwrap();
        let binary = ret_stmt.expression.try_as_binary().unwrap();
        assert!(matches!(binary.operator, Token::Plus));
    }

    #[test]
    fn test_parse_return_function_call() {
        let result = ASTParser::parse_from_source("function bar() { return foo(); }").unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        let ret_stmt = func.block.body[0].try_as_return().unwrap();
        let call = ret_stmt.expression.try_as_function_call().unwrap();
        let func_id = call.function.try_as_identifier().unwrap();
        assert_eq!(func_id.name, "foo");
    }

    #[test]
    fn test_parse_multiple_returns_in_if() {
        let result = ASTParser::parse_from_source(
            "function foo() { if (x) { return 1; } else { return 2; } }",
        )
        .unwrap();
        assert_eq!(result.len(), 1);

        let func = result[0].try_as_function_definition().unwrap();
        let if_stmt = func.block.body[0].try_as_if().unwrap();

        let then_block = if_stmt.then.try_as_block().unwrap();
        assert!(then_block.body[0].try_as_return().is_some());

        let else_block = if_stmt.else_.as_ref().unwrap().try_as_block().unwrap();
        assert!(else_block.body[0].try_as_return().is_some());
    }
}
