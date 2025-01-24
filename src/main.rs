use std::{
    backtrace::Backtrace,
    cell::RefCell,
    collections::HashMap,
    process::ExitCode,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

mod error;
mod execution_engine;
mod execution_scope;
mod js_value;
mod parser;
mod tests;
mod tokenizer;

use std::cell::{Cell, OnceCell};

trait MyTrait {
    fn print(&self);
}

struct MyStruct {
    value: i32,
}

impl MyTrait for MyStruct {
    fn print(&self) {
        println!("Value: {}", self.value);
    }
}

thread_local!(static TRUE: OnceCell<Rc<f32>> = OnceCell::new());

fn get_thread_local_true() -> Rc<f32> {
    TRUE.with(|value| {
        value
            .get_or_init(|| {
                println!("INIT");
                return Rc::new(0.0);
            })
            .clone()
    })
}

fn main() -> ExitCode {
    let x = get_thread_local_true();
    let b = get_thread_local_true();

    println!("{:#?}", x == b);

    ExitCode::SUCCESS
    // let my_struct = MyStruct { value: 42 };
    // let my_trait = &my_struct as &dyn MyTrait;

    // let source = String::from(
    //     "

    //         ",
    // ); // 3
    // let mut tokens = vec![];

    // for token in Tokenizer::from_source(source.to_string()).to_iter() {
    //     match token {
    //         Ok(token) => tokens.push(token),
    //         Err(err) => {
    //             err.print();
    //             return ExitCode::FAILURE;
    //         }
    //     };
    // }

    // let program = Parser::new(tokens).parse_program();

    // println!("{program:#?}");

    // ExitCode::SUCCESS

    //     let now = SystemTime::now()
    //         .duration_since(UNIX_EPOCH)
    //         .unwrap()
    //         .as_micros();

    //     let source = String::from(
    //         "
    // function one() {
    //         return 1;
    // }

    // function apply(f) {
    //         return f();
    // }

    // apply(one);
    //         ",
    //     ); // 3

    //     match ExpressionEvaluator::evaluate_source(source) {
    //         Ok(value) => {
    //             println!(
    //                 "Executed with value: {value:?}, time: {}",
    //                 SystemTime::now()
    //                     .duration_since(UNIX_EPOCH)
    //                     .unwrap()
    //                     .as_micros()
    //                     - now
    //             );
    //             ExitCode::SUCCESS
    //         }
    //         Err(err) => {
    //             err.print();
    //             ExitCode::FAILURE
    //         }
    //     }
}
