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
    ExecutionEngineError {
        message: String,
    },
    ExecutionScopeError {
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
            EngineErrorKind::ExecutionEngineError { message } => {
                format!("Executor error: {}", message,)
            }
            EngineErrorKind::ExecutionScopeError { message } => {
                format!("Execution Scope error: {}", message,)
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

    pub fn execution_engine_error<T: ToString>(message: T) -> Self {
        Self {
            kind: EngineErrorKind::ExecutionEngineError {
                message: String::from(message.to_string()),
            },
            backtrace: Backtrace::capture(),
        }
    }

    pub fn execution_scope_error<T: ToString>(message: T) -> Self {
        Self {
            kind: EngineErrorKind::ExecutionScopeError {
                message: String::from(message.to_string()),
            },
            backtrace: Backtrace::capture(),
        }
    }

    pub fn print(&self) {
        eprintln!("{}", self.backtrace);
        eprintln!("{}", self.message());
    }
}
