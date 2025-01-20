use std::backtrace::Backtrace;

#[derive(Debug, Clone)]
pub enum EngineErrorKind {
    TokenizerUnknownToken {
        char: String,
        column: usize,
        line: usize,
    },
    ParserError {
        message: String,
    },
}

#[derive(Debug)]
pub struct EngineError {
    backtrace: Backtrace,
    kind: EngineErrorKind,
}

impl EngineError {
    pub fn message(&self) -> String {
        match self.kind.clone() {
            EngineErrorKind::TokenizerUnknownToken {
                char, column, line, ..
            } => format!(
                "Unknown token '{}' line: '{}', column: '{}'",
                char, line, column,
            ),
            EngineErrorKind::ParserError { message } => {
                format!("Parser error: {}", message,)
            }
        }
    }

    pub fn tokenizer_unknown_token(char: String, column: usize, line: usize) -> Self {
        Self {
            kind: EngineErrorKind::TokenizerUnknownToken { char, column, line },
            backtrace: Backtrace::capture(),
        }
    }

    pub fn parser_error<T: ToString>(message: T) -> Self {
        Self {
            kind: EngineErrorKind::ParserError {
                message: String::from(message.to_string()),
            },
            backtrace: Backtrace::capture(),
        }
    }

    pub fn print(&self) {
        eprintln!("{}", self.message());
    }
}
