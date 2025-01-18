#[derive(Debug)]
pub enum TokenKind {
    Number,
    Plus,
    Minus,
    Multiply,
    Divide,
    OParen,
    CParen,
}

#[derive(Debug)]

pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl Token {
    fn build(
        tokenizer: &mut Tokenizer,
        kind: TokenKind,
        text: String,
        start: usize,
    ) -> Option<Result<Token, String>> {
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
        return TokenizerIterator {
            tokenizer: Box::from(self),
        };
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

    fn next_token(&mut self) -> Option<Result<Token, String>> {
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

                    return Token::build(self, TokenKind::Number, digit, start);
                }

                let char_str = char.to_string();

                return match char {
                    '+' => Token::build(self, TokenKind::Plus, char_str, start),
                    '-' => Token::build(self, TokenKind::Minus, char_str, start),
                    '*' => Token::build(self, TokenKind::Multiply, char_str, start),
                    '/' => Token::build(self, TokenKind::Divide, char_str, start),
                    '(' => Token::build(self, TokenKind::OParen, char_str, start),
                    ')' => Token::build(self, TokenKind::CParen, char_str, start),
                    _ => Some(Err(format!(
                        "Unknown token: '{}' on line: '{}', column: '{}', position: '{}'",
                        char_str, self.cursor.line, self.cursor.column, self.cursor.position
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
    tokenizer: Box<Tokenizer>,
}

impl Iterator for TokenizerIterator {
    type Item = Result<Token, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokenizer.next_token()
    }
}
