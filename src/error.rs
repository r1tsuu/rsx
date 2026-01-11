use std::backtrace::Backtrace;

#[derive(Debug)]
pub struct ASTError {
    message: String,
    // backtrace: Backtrace,
}

#[derive(Debug)]
pub struct JSError {
    message: String,
    // backtrace: Backtrace,
}

#[derive(Debug)]
pub struct LexerError {
    message: String,
    // backtrace: Backtrace,
}

#[derive(Debug)]
pub enum EngineError {
    Ast(ASTError),
    JS(JSError),
    Lexer(LexerError),
}

impl EngineError {
    pub fn ast<T: ToString>(message: T) -> Self {
        EngineError::Ast(ASTError {
            message: message.to_string(),
            // backtrace: Backtrace::capture(),
        })
    }

    pub fn js<T: ToString>(message: T) -> Self {
        EngineError::JS(JSError {
            message: message.to_string(),
            // backtrace: Backtrace::capture(),
        })
    }

    pub fn lexer<T: ToString>(message: T) -> Self {
        EngineError::Lexer(LexerError {
            message: message.to_string(),
            // backtrace: Backtrace::capture(),
        })
    }

    pub fn message(&self) -> &str {
        match self {
            EngineError::Ast(err) => &err.message,
            EngineError::JS(err) => &err.message,
            EngineError::Lexer(err) => &err.message,
        }
    }
}
