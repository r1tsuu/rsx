use std::{
    cell::RefCell,
    collections::HashMap,
    process::ExitCode,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use execution_engine::ExecutionEngine;
mod error;
mod execution_engine;
mod execution_scope;
mod javascript_object;
mod memory;
mod parser;
mod tests;
mod tokenizer;

fn main() -> ExitCode {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();

    let source = String::from("1 == 1"); // 3

    match ExecutionEngine::execute_source(source) {
        Ok(value) => {
            println!(
                "Executed with value: {value:?}, time: {}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros()
                    - now
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            err.print();
            ExitCode::FAILURE
        }
    }
}
