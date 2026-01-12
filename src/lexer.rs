use crate::error::EngineError;

#[derive(Debug, Clone)]
pub struct IdentifierToken {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct NumericLiteralToken {
    pub value: f32,
}

#[derive(Debug, Clone)]
pub enum Token {
    Identifier(IdentifierToken),
    NumericLiteral(NumericLiteralToken),
    Equal,
    LetKeyword,
    IfKeyword,
    ElseKeyword,
    FunctionKeyword,
    ReturnKeyword,
    Semicolon,
    Slash,
    Plus,
    Minus,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    AndAnd,
    OrOr,
    EqualEqual,
    EqualEqualEqual,
    BangEqual,
    BangEqualEqual,
    Star,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    LParen,
    RParen,
    End,
    Dot,
}

impl Token {
    pub fn try_as_identifier(&self) -> Option<&IdentifierToken> {
        if let Token::Identifier(t) = self {
            Some(t)
        } else {
            None
        }
    }

    pub fn try_as_numeric_literal(&self) -> Option<&NumericLiteralToken> {
        if let Token::NumericLiteral(t) = self {
            Some(t)
        } else {
            None
        }
    }
}

pub struct Lexer {
    pos: usize,
    source: Vec<char>,
}

impl Lexer {
    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        loop {
            if let Some(c) = self.peek()
                && c.is_whitespace()
            {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_identifier(&mut self) -> Token {
        let mut name = String::new();

        while let Some(character) = self.peek()
            && character.is_alphanumeric()
        {
            name.push(character);
            self.advance();
        }

        match name.as_str() {
            "let" => Token::LetKeyword,
            "function" => Token::FunctionKeyword,
            "return" => Token::ReturnKeyword,
            "if" => Token::IfKeyword,
            "else" => Token::ElseKeyword,
            _ => Token::Identifier(IdentifierToken { name }),
        }
    }

    fn parse_numeric_literal(&mut self) -> Result<Token, EngineError> {
        let mut str_number = String::new();

        while let Some(character) = self.peek()
            && (character.is_digit(10) || (character == '.'))
        {
            str_number.push(character);
            self.advance();
        }

        let parsed = str_number
            .parse::<f32>()
            .map_err(|_| EngineError::lexer(format!("Failed to parse {} into f32", str_number)))?;

        Ok(Token::NumericLiteral(NumericLiteralToken { value: parsed }))
    }

    fn match_char(&mut self, expected: char) -> bool {
        self.peek()
            .map(|char| char == expected)
            .inspect(|_| {
                self.advance();
            })
            .unwrap_or(false)
    }

    fn next_token(&mut self) -> Result<Token, EngineError> {
        self.peek()
            .map(|character| match character {
                character if character.is_alphabetic() => Ok(self.parse_identifier()),
                character if character.is_digit(10) => self.parse_numeric_literal(),
                ';' => {
                    self.advance();
                    Ok(Token::Semicolon)
                }
                '/' => {
                    self.advance();
                    Ok(Token::Slash)
                }
                '+' => {
                    self.advance();
                    Ok(Token::Plus)
                }
                '.' => {
                    self.advance();
                    Ok(Token::Dot)
                }
                '-' => {
                    self.advance();
                    Ok(Token::Minus)
                }
                '*' => {
                    self.advance();
                    Ok(Token::Star)
                }
                ',' => {
                    self.advance();
                    Ok(Token::Comma)
                }
                ':' => {
                    self.advance();
                    Ok(Token::Colon)
                }
                '(' => {
                    self.advance();
                    Ok(Token::LParen)
                }
                ')' => {
                    self.advance();
                    Ok(Token::RParen)
                }
                '{' => {
                    self.advance();
                    Ok(Token::LBrace)
                }
                '}' => {
                    self.advance();
                    Ok(Token::RBrace)
                }
                '[' => {
                    self.advance();
                    Ok(Token::LBracket)
                }
                ']' => {
                    self.advance();
                    Ok(Token::RBracket)
                }
                '=' => {
                    self.advance();
                    if self.match_char('=') {
                        if self.match_char('=') {
                            return Ok(Token::EqualEqualEqual);
                        }

                        return Ok(Token::EqualEqual);
                    }

                    Ok(Token::Equal)
                }
                '!' => {
                    self.advance();
                    if self.match_char('=') {
                        if self.match_char('=') {
                            return Ok(Token::BangEqualEqual);
                        }
                        return Ok(Token::BangEqual);
                    }

                    Err(EngineError::lexer(format!("Invalid Bang usage")))
                }
                '>' => {
                    self.advance();
                    if self.match_char('=') {
                        return Ok(Token::GreaterThanEqual);
                    }

                    Ok(Token::GreaterThan)
                }
                '<' => {
                    self.advance();
                    if self.match_char('=') {
                        return Ok(Token::LessThanEqual);
                    }

                    Ok(Token::LessThan)
                }
                '&' => {
                    self.advance();
                    if self.match_char('&') {
                        return Ok(Token::AndAnd);
                    }

                    Err(EngineError::lexer(format!("Invalid And (&) usage")))
                }
                '|' => {
                    self.advance();
                    if self.match_char('|') {
                        return Ok(Token::OrOr);
                    }

                    Err(EngineError::lexer(format!("Invalid Or (|) usage")))
                }
                _ => Err(EngineError::lexer(format!(
                    "Invalid character: {}",
                    character
                ))),
            })
            .unwrap_or(Ok(Token::End))
    }

    pub fn tokenize(source: &str) -> Result<Vec<Token>, EngineError> {
        let mut tokens: Vec<Token> = vec![];
        let mut lexer = Self {
            pos: 0,
            source: source.chars().collect(),
        };

        loop {
            lexer.skip_whitespace();
            let token = lexer.next_token()?;

            if let Token::End = token {
                tokens.push(token);
                break;
            }

            tokens.push(token);
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tokens() {
        let source = "; / + - * ( )";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 8); // 7 tokens + End
        assert!(matches!(tokens[0], Token::Semicolon));
        assert!(matches!(tokens[1], Token::Slash));
        assert!(matches!(tokens[2], Token::Plus));
        assert!(matches!(tokens[3], Token::Minus));
        assert!(matches!(tokens[4], Token::Star));
        assert!(matches!(tokens[5], Token::LParen));
        assert!(matches!(tokens[6], Token::RParen));
        assert!(matches!(tokens[7], Token::End));
    }

    #[test]
    fn test_numeric_literals() {
        let source = "123 456 789 32.5";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 5); // 4 numbers + End
        assert_eq!(tokens[0].try_as_numeric_literal().unwrap().value, 123.0);
        assert_eq!(tokens[1].try_as_numeric_literal().unwrap().value, 456.0);
        assert_eq!(tokens[2].try_as_numeric_literal().unwrap().value, 789.0);
        assert_eq!(tokens[3].try_as_numeric_literal().unwrap().value, 32.5);

        assert!(matches!(tokens[4], Token::End));
    }

    #[test]
    fn test_identifiers() {
        let source = "foo bar baz123";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 4); // 3 identifiers + End
        assert_eq!(tokens[0].try_as_identifier().unwrap().name, "foo");
        assert_eq!(tokens[1].try_as_identifier().unwrap().name, "bar");
        assert_eq!(tokens[2].try_as_identifier().unwrap().name, "baz123");

        assert!(matches!(tokens[3], Token::End));
    }

    #[test]
    fn test_expression() {
        let source = "x + 5 * (y - 2)";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 10); // 9 tokens + End
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::NumericLiteral(_)));
        assert!(matches!(tokens[3], Token::Star));
        assert!(matches!(tokens[4], Token::LParen));
        assert!(matches!(tokens[5], Token::Identifier(_)));
        assert!(matches!(tokens[6], Token::Minus));
        assert!(matches!(tokens[7], Token::NumericLiteral(_)));
        assert!(matches!(tokens[8], Token::RParen));
        assert!(matches!(tokens[9], Token::End));
    }

    #[test]
    fn test_whitespace_handling() {
        let source = "   42   +   10   ";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 4); // 42, +, 10, End
        assert!(matches!(tokens[0], Token::NumericLiteral(_)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::NumericLiteral(_)));
        assert!(matches!(tokens[3], Token::End));
    }

    #[test]
    fn test_empty_source() {
        let source = "";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::End));
    }

    #[test]
    fn test_only_whitespace() {
        let source = "   \n\t  ";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::End));
    }

    #[test]
    fn test_invalid_character() {
        let source = "x @ y";
        let result = Lexer::tokenize(source);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message(), "Invalid character: @");
    }

    #[test]
    fn test_invalid_point() {
        let source = "12.34.56";
        let result = Lexer::tokenize(source);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message(),
            "Failed to parse 12.34.56 into f32"
        );
    }

    #[test]
    fn test_semicolon_terminated() {
        let source = "x 5;";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 4); // x, 5, ;, End
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::NumericLiteral(_)));
        assert!(matches!(tokens[2], Token::Semicolon));
        assert!(matches!(tokens[3], Token::End));
    }

    #[test]
    fn test_division() {
        let source = "10 / 2";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 4); // 10, /, 2, End
        assert!(matches!(tokens[0], Token::NumericLiteral(_)));
        assert!(matches!(tokens[1], Token::Slash));
        assert!(matches!(tokens[2], Token::NumericLiteral(_)));
        assert!(matches!(tokens[3], Token::End));
    }

    #[test]
    fn test_consecutive_numbers() {
        let source = "123456";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].try_as_numeric_literal().unwrap().value, 123456.0);
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_no_space_between_tokens() {
        let source = "x+y*z";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 6); // x, +, y, *, z, End
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Identifier(_)));
        assert!(matches!(tokens[3], Token::Star));
        assert!(matches!(tokens[4], Token::Identifier(_)));
        assert!(matches!(tokens[5], Token::End));
    }

    #[test]
    fn test_braces() {
        let source = "{ }";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 3); // {, }, End
        assert!(matches!(tokens[0], Token::LBrace));
        assert!(matches!(tokens[1], Token::RBrace));
        assert!(matches!(tokens[2], Token::End));
    }

    #[test]
    fn test_brackets() {
        let source = "[ ]";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 3); // [, ], End
        assert!(matches!(tokens[0], Token::LBracket));
        assert!(matches!(tokens[1], Token::RBracket));
        assert!(matches!(tokens[2], Token::End));
    }

    #[test]
    fn test_comma() {
        let source = "a, b, c";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 6); // a, ,, b, ,, c, End
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::Comma));
        assert!(matches!(tokens[2], Token::Identifier(_)));
        assert!(matches!(tokens[3], Token::Comma));
        assert!(matches!(tokens[4], Token::Identifier(_)));
        assert!(matches!(tokens[5], Token::End));
    }

    #[test]
    fn test_dot() {
        let source = "obj.prop";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 4); // obj, ., prop, End
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::Dot));
        assert!(matches!(tokens[2], Token::Identifier(_)));
        assert!(matches!(tokens[3], Token::End));
    }

    #[test]
    fn test_function_keyword() {
        let source = "function";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // function, End
        assert!(matches!(tokens[0], Token::FunctionKeyword));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_return_keyword() {
        let source = "return";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // return, End
        assert!(matches!(tokens[0], Token::ReturnKeyword));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_equal() {
        let source = "=";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // =, End
        assert!(matches!(tokens[0], Token::Equal));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_equal_equal() {
        let source = "==";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // ==, End
        assert!(matches!(tokens[0], Token::EqualEqual));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_equal_equal_equal() {
        let source = "===";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // ===, End
        assert!(matches!(tokens[0], Token::EqualEqualEqual));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_bang_equal() {
        let source = "!=";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // !=, End
        assert!(matches!(tokens[0], Token::BangEqual));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_bang_equal_equal() {
        let source = "!==";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // !==, End
        assert!(matches!(tokens[0], Token::BangEqualEqual));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_less_than() {
        let source = "<";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // <, End
        assert!(matches!(tokens[0], Token::LessThan));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_less_than_equal() {
        let source = "<=";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // <=, End
        assert!(matches!(tokens[0], Token::LessThanEqual));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_more_than() {
        let source = ">";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // >, End
        assert!(matches!(tokens[0], Token::GreaterThan));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_more_than_equal() {
        let source = ">=";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // >=, End
        assert!(matches!(tokens[0], Token::GreaterThanEqual));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_and_and() {
        let source = "&&";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // &&, End
        assert!(matches!(tokens[0], Token::AndAnd));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_or_or() {
        let source = "||";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // ||, End
        assert!(matches!(tokens[0], Token::OrOr));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_mixed_brackets_and_braces() {
        let source = "{[]}";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 5); // {, [, ], }, End
        assert!(matches!(tokens[0], Token::LBrace));
        assert!(matches!(tokens[1], Token::LBracket));
        assert!(matches!(tokens[2], Token::RBracket));
        assert!(matches!(tokens[3], Token::RBrace));
        assert!(matches!(tokens[4], Token::End));
    }

    #[test]
    fn test_array_like_syntax() {
        let source = "[1, 2, 3]";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 8); // [, 1, ,, 2, ,, 3, ], End
        assert!(matches!(tokens[0], Token::LBracket));
        assert!(matches!(tokens[1], Token::NumericLiteral(_)));
        assert!(matches!(tokens[2], Token::Comma));
        assert!(matches!(tokens[3], Token::NumericLiteral(_)));
        assert!(matches!(tokens[4], Token::Comma));
        assert!(matches!(tokens[5], Token::NumericLiteral(_)));
        assert!(matches!(tokens[6], Token::RBracket));
        assert!(matches!(tokens[7], Token::End));
    }

    #[test]
    fn test_object_like_syntax() {
        let source = "{x: 10, y: 20}";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 10); // {, x, :, 10, ,, y, :, 20, }, End
        assert!(matches!(tokens[0], Token::LBrace));
        assert!(matches!(tokens[1], Token::Identifier(_)));
        assert!(matches!(tokens[2], Token::Colon));
        assert!(matches!(tokens[3], Token::NumericLiteral(_)));
        assert!(matches!(tokens[4], Token::Comma));
        assert!(matches!(tokens[5], Token::Identifier(_)));
        assert!(matches!(tokens[6], Token::Colon));
        assert!(matches!(tokens[7], Token::NumericLiteral(_)));
        assert!(matches!(tokens[8], Token::RBrace));
        assert!(matches!(tokens[9], Token::End));
    }

    #[test]
    fn test_nested_object_syntax() {
        let source = "{a: {b: 1}}";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 10); // {, a, :, {, b, :, 1, }, }, End
        assert!(matches!(tokens[0], Token::LBrace));
        assert!(matches!(tokens[1], Token::Identifier(_)));
        assert!(matches!(tokens[2], Token::Colon));
        assert!(matches!(tokens[3], Token::LBrace));
        assert!(matches!(tokens[4], Token::Identifier(_)));
        assert!(matches!(tokens[5], Token::Colon));
        assert!(matches!(tokens[6], Token::NumericLiteral(_)));
        assert!(matches!(tokens[7], Token::RBrace));
        assert!(matches!(tokens[8], Token::RBrace));
        assert!(matches!(tokens[9], Token::End));
    }

    #[test]
    fn test_object_with_array() {
        let source = "{items: [1, 2]}";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 10); // {, items, :, [, 1, ,, 2, ], }, End
        assert!(matches!(tokens[0], Token::LBrace));
        assert!(matches!(tokens[1], Token::Identifier(_)));
        assert!(matches!(tokens[2], Token::Colon));
        assert!(matches!(tokens[3], Token::LBracket));
        assert!(matches!(tokens[4], Token::NumericLiteral(_)));
        assert!(matches!(tokens[5], Token::Comma));
        assert!(matches!(tokens[6], Token::NumericLiteral(_)));
        assert!(matches!(tokens[7], Token::RBracket));
        assert!(matches!(tokens[8], Token::RBrace));
        assert!(matches!(tokens[9], Token::End));
    }

    #[test]
    fn test_empty_object() {
        let source = "{}";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 3); // {, }, End
        assert!(matches!(tokens[0], Token::LBrace));
        assert!(matches!(tokens[1], Token::RBrace));
        assert!(matches!(tokens[2], Token::End));
    }

    #[test]
    fn test_empty_array() {
        let source = "[]";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 3); // [, ], End
        assert!(matches!(tokens[0], Token::LBracket));
        assert!(matches!(tokens[1], Token::RBracket));
        assert!(matches!(tokens[2], Token::End));
    }

    #[test]
    fn test_assignment() {
        let source = "x = 1";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 4); // x, =, 1, End
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::Equal));
        assert!(matches!(tokens[2], Token::NumericLiteral(_)));
        assert!(matches!(tokens[3], Token::End));
    }

    #[test]
    fn test_if_keyword() {
        let source = "if";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // if, End
        assert!(matches!(tokens[0], Token::IfKeyword));
        assert!(matches!(tokens[1], Token::End));
    }

    #[test]
    fn test_else_keyword() {
        let source = "else";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens.len(), 2); // else, End
        assert!(matches!(tokens[0], Token::ElseKeyword));
        assert!(matches!(tokens[1], Token::End));
    }
}
