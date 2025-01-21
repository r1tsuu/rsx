use std::process::ExitCode;

use execution_engine::ExecutionEngine;
mod error;
mod execution_engine;
mod execution_scope;
mod javascript_object;
mod memory;
mod parser;
mod tokenizer;

fn main() -> ExitCode {
    let source = String::from("let x = 1; let b = 2; x + b"); // 3

    match ExecutionEngine::execute_source(source) {
        Ok(value) => {
            println!("Executed with value: {:#?}", value);
            ExitCode::SUCCESS
        }
        Err(err) => {
            err.print();
            ExitCode::FAILURE
        }
    }
}
