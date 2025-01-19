use std::{borrow::Cow, fmt::format};

#[derive(Debug)]
pub enum EngineError {
    TokenizerUnknownToken {
        char: String,
        column: usize,
        line: usize,
    },
    ParserError {
        message: String,
    },
}

impl EngineError {
    pub fn message(&self) -> String {
        match self {
            EngineError::TokenizerUnknownToken { char, column, line } => format!(
                "Unknown token '{}' line: '{}', column: '{}'",
                char, line, column
            ),
            EngineError::ParserError { message } => format!("Parser error: {}", message),
        }
    }

    pub fn tokenizer_unknown_token(char: String, column: usize, line: usize) -> Self {
        EngineError::TokenizerUnknownToken { char, column, line }
    }

    pub fn parser_error<T: ToString>(message: T) -> Self {
        EngineError::ParserError {
            message: String::from(message.to_string()),
        }
    }

    pub fn print(&self) {
        eprintln!("{}", self.message());
    }
}
