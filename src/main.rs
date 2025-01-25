use std::{
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

mod error;
mod execution_engine;
mod execution_scope;
mod js_value;
mod parser;
mod tests;
mod tokenizer;

use execution_engine::ExpressionEvaluator;
use parser::Parser;

fn main() -> ExitCode {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();

    // println!("{:#?}", Parser::parse_source("let x = {a: 2, c: 100}"));
    // return ExitCode::SUCCESS;

    let source = String::from(
        "
    function one() {
            return 1;
    }

    function apply(f) {
            return f();
    }

    apply(one) + apply(one);
            ",
    ); // 3

    match ExpressionEvaluator::evaluate_source(source) {
        Ok(value) => {
            println!(
                "Executed with value: {}, time: {}",
                value.get_debug_string(),
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
