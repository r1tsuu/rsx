use std::process::ExitCode;

use chumsky::Parser;
mod parser;

fn main() -> ExitCode {
    let source = "
{ }
";

    println!("{:#?}", parser::parser().parse(source));
    return ExitCode::SUCCESS;
    // println!(
    //     "{:#?}",
    //     Parser::parse_source("new Promise().then(function(x){return 2});")
    // );
    // return ExitCode::SUCCESS;

    // let now = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_micros();

    // let source = String::from(
    //     "
    // function one() {
    //         return 1;
    // }

    // function apply(f) {
    //         return f();
    // }

    // apply(one) + apply(one);
    //         ",
    // ); // 3

    // match ExpressionEvaluator::evaluate_source(source) {
    //     Ok(value) => {
    //         println!(
    //             "Executed with value: {}, time: {}",
    //             value.get_debug_string(),
    //             SystemTime::now()
    //                 .duration_since(UNIX_EPOCH)
    //                 .unwrap()
    //                 .as_micros()
    //                 - now
    //         );
    //         ExitCode::SUCCESS
    //     }
    //     Err(err) => {
    //         err.print();
    //         ExitCode::FAILURE
    //     }
    // }
}
