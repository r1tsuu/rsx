use std::process::ExitCode;

use chumsky::Parser;
use rsx::Rsx;
mod parser;
mod rsx;

fn main() -> Result<(), ()> {
    let source = "
1+1;
";

    let program = parser::program_parser().parse(source).map_err(|err| {
        eprintln!("ERROR: {:#?}", err);
    })?;

    println!("PROGRAM: {:#?}", program);

    let mut rsx = Rsx::new();

    rsx.execute_program(&program).map_err(|err| {
        eprintln!("ERROR: {:#?}", err);
    })?;

    println!("STACK VALUE: {:#?}", rsx.last_stack());

    Ok(())
}
