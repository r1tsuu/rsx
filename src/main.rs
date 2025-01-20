use std::process::ExitCode;

use interpreter::evaluate_expression;
use parser::Parser;
use tokenizer::{Token, Tokenizer};

mod error;
mod interpreter;
mod parser;
mod tokenizer;

fn main() -> ExitCode {
    let code = String::from("let x =1;");

    let tokenizer = Tokenizer::from_source(code);
    let mut tokens: Vec<Token> = vec![];

    for token in tokenizer.to_iter() {
        match token {
            Ok(token) => {
                tokens.push(token);
            }
            Err(err) => {
                err.print();
                return ExitCode::FAILURE;
            }
        }
    }

    let mut parser = Parser::new(tokens);

    let expr = parser.parse_program();

    println!("{:#?}", expr);
    // println!("{}", evaluate_expression(&expr));

    return ExitCode::SUCCESS;
}
