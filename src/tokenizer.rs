use crate::error::EngineError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number,
    Plus,
    Minus,
    Multiply,
    Divide,
    OpenParen,  // (
    CloseParen, // )
    OpenBrace,  // }
    CloseBrace, // {
    Semicolon,
    Let,
    Function,
    Return,
    Identifier,
    Equals,
    String,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl Token {
    pub fn is_semicolon(&self) -> bool {
        match self.kind {
            TokenKind::Semicolon => true,
            _ => false,
        }
    }

    pub fn is_equals(&self) -> bool {
        match self.kind {
            TokenKind::Equals => true,
            _ => false,
        }
    }

    pub fn is_binary_operator(&self) -> bool {
        match self.kind {
            TokenKind::Divide
            | TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Multiply
            | TokenKind::Equals => true,
            _ => false,
        }
    }
}

impl Token {
    fn build(
        tokenizer: &mut Tokenizer,
        kind: TokenKind,
        text: String,
        start: usize,
    ) -> Option<Result<Token, EngineError>> {
        let token = Token {
            kind,
            text,
            start,
            end: tokenizer.cursor.position,
        };

        tokenizer.increment_position();

        return Some(Ok(token));
    }
}

#[derive(Clone)]
struct Cursor {
    line: usize,
    position: usize,
    column: usize,
}

#[derive(Clone)]
pub struct Tokenizer {
    chars: Vec<char>,
    cursor: Cursor,
}

impl Cursor {
    fn new() -> Self {
        Cursor {
            column: 1,
            position: 1,
            line: 1,
        }
    }
}

impl Tokenizer {
    pub fn from_source(source: String) -> Self {
        Tokenizer {
            chars: Vec::from_iter(source.chars()),
            cursor: Cursor::new(),
        }
    }

    pub fn to_iter(self) -> TokenizerIterator {
        TokenizerIterator { tokenizer: self }
    }

    fn increment_position(&mut self) {
        self.cursor.position += 1;
    }

    fn increment_column(&mut self) {
        self.increment_position();
        self.cursor.column += 1;
    }

    fn increment_line(&mut self) {
        self.cursor.line += 1;
        self.cursor.column = 0;
        self.increment_position();
    }

    fn next_token(&mut self) -> Option<Result<Token, EngineError>> {
        let char = self.chars.get(self.cursor.position - 1);

        match char {
            Some(&char) => {
                self.cursor.column += 1;

                if char.is_whitespace() {
                    if char == '\n' {
                        self.increment_line()
                    } else {
                        self.increment_position();
                    }

                    return self.next_token();
                }

                let start = self.cursor.position;

                if char.is_digit(10) {
                    let mut digit = char.to_string();

                    while let Some(next_char) =
                        self.chars.get(self.cursor.position).map(char::clone)
                    {
                        if next_char.is_digit(10) || next_char == '.' {
                            self.increment_column();
                            digit.push(next_char);
                        } else {
                            break;
                        }
                    }

                    if digit.ends_with(".") {
                        return Some(Err(EngineError::tokenizer_unknown_token(
                            digit,
                            self.cursor.column,
                            self.cursor.line,
                        )));
                    }

                    return Token::build(self, TokenKind::Number, digit, start);
                }

                if char == '"' {
                    let mut string = String::new();
                    while let Some(next_char) =
                        self.chars.get(self.cursor.position).map(char::clone)
                    {
                        self.increment_column();

                        if next_char == '"' {
                            break;
                        } else {
                            string.push(next_char);
                        }
                    }

                    return Token::build(self, TokenKind::String, string, start);
                }

                let char_str = char.to_string();

                if char.is_alphabetic() {
                    let mut identifier = char.to_string();
                    while let Some(next_char) =
                        self.chars.get(self.cursor.position).map(char::clone)
                    {
                        if next_char.is_alphabetic()
                            || next_char.is_alphanumeric()
                            || next_char == '_'
                        {
                            self.increment_column();
                            identifier.push(next_char);
                        } else {
                            break;
                        }
                    }

                    return match identifier.as_str() {
                        "function" => Token::build(self, TokenKind::Function, identifier, start),
                        "let" => Token::build(self, TokenKind::Let, identifier, start),
                        "return" => Token::build(self, TokenKind::Return, identifier, start),
                        _ => Token::build(self, TokenKind::Identifier, identifier, start),
                    };
                }

                return match char {
                    '+' => Token::build(self, TokenKind::Plus, char_str, start),
                    '-' => Token::build(self, TokenKind::Minus, char_str, start),
                    '*' => Token::build(self, TokenKind::Multiply, char_str, start),
                    '/' => Token::build(self, TokenKind::Divide, char_str, start),
                    '(' => Token::build(self, TokenKind::OpenParen, char_str, start),
                    ')' => Token::build(self, TokenKind::CloseParen, char_str, start),
                    '{' => Token::build(self, TokenKind::OpenBrace, char_str, start),
                    '}' => Token::build(self, TokenKind::CloseBrace, char_str, start),
                    ';' => Token::build(self, TokenKind::Semicolon, char_str, start),
                    '=' => Token::build(self, TokenKind::Equals, char_str, start),
                    _ => Some(Err(EngineError::tokenizer_unknown_token(
                        char_str,
                        self.cursor.column,
                        self.cursor.line,
                    ))),
                };
            }

            None => {
                return None;
            }
        };
    }
}

pub struct TokenizerIterator {
    tokenizer: Tokenizer,
}

impl Iterator for TokenizerIterator {
    type Item = Result<Token, EngineError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokenizer.next_token()
    }
}
