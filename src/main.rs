use std::process::ExitCode;

use tokenizer::{Token, Tokenizer};

mod parser;
mod tokenizer;

fn main() -> ExitCode {
    let code = String::from("1 + 43 + 1");

    let tokenizer = Tokenizer::from_source(code);
    let mut tokens: Vec<Token> = vec![];

    for token in tokenizer.to_iter() {
        match token {
            Ok(token) => {
                tokens.push(token);
            }
            Err(msg) => {
                eprintln!("{}", msg);
                return ExitCode::FAILURE;
            }
        }
    }

    println!("---TOKENS---");
    println!("{:#?}", tokens);
    println!("---TOKENS---");

    return ExitCode::SUCCESS;
}
