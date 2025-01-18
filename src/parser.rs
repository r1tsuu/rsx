use crate::tokenizer::{Token, TokenKind};

enum Expression {
    NumberLiteral {
        value: f32,
    },
    BinaryOp {
        left: Box<Expression>,
        op: Token,
        right: Box<Expression>,
        parent: Box<Expression>,
    },
    Program {
        statemenets: Vec<Expression>,
    },
}

struct Parser {
    tokens: Vec<Token>,
    current_token: usize,
    current_expression: Box<Expression>,
}

impl Parser {
    fn from_tokens(tokens: Vec<Token>) -> Self {
        let expression = Expression::Program {
            statemenets: vec![],
        };

        Parser {
            tokens,
            current_token: 0,
            current_expression: Box::from(expression),
        }
    }
}
