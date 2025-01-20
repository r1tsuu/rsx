use std::process::ExitCode;

use executor::Executor;

mod error;
mod executor;
mod parser;
mod tokenizer;

fn main() -> ExitCode {
    let code = String::from("let x = 1; let b = 2; x+b");
    let mut executor = Executor::from_source(code);

    match executor.as_mut() {
        Ok(executor) => match executor.execute() {
            Ok(val) => {
                println!("Evaluated to: {}", val);
                ExitCode::SUCCESS
            }
            Err(err) => {
                err.print();
                ExitCode::FAILURE
            }
        },
        Err(err) => {
            err.print();
            ExitCode::FAILURE
        }
    }
    // // let tokenizer = Tokenizer::from_source(code);
    // // let mut tokens: Vec<Token> = vec![];

    // // for token in tokenizer.to_iter() {
    // //     match token {
    // //         Ok(token) => {
    // //             tokens.push(token);
    // //         }
    // //         Err(err) => {
    // //             err.print();
    // //             return ExitCode::FAILURE;
    // //         }
    // //     }
    // // }

    // // let mut parser = Parser::new(tokens);

    // // let expr = parser.parse_program();
    // //
    // // println!("{:#?}", expr);
    // // println!("{}", evaluate_expression(&expr));

    // return ExitCode::SUCCESS;
}
