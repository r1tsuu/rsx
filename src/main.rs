use std::{
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

use as_any::Downcast;
use chumsky::{prelude::*, text::ident};

mod addon_math;
mod error;
mod execution_engine;
mod execution_scope;
mod js_value;
mod js_value_2;
mod parser;
mod paser_new;
mod tests;
mod tokenizer;

// use parser::Parser;

// enum Expr {

// }

#[derive(Debug)]
enum Token {
    PLUS,
    MINUS,
    MULTIPLY,
    DIVIDE,
}

#[derive(Debug)]
enum Expression {
    Num(f64),
    String(String),
    Identifier(String),
    /// Left, Right
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Negative(Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Array(Vec<Expression>),
    Function(Vec<String>, Box<Statement>),
    /// Object Expression, Element expression
    ElementAccess(Box<Expression>, Box<Expression>),
    /// Key expression, Value expression
    Object(Vec<(Expression, Expression)>),
}

#[derive(Debug)]
enum Statement {
    Let(String, Expression),
    Assign(Expression, Expression),
    Return(Box<Expression>),
    Expression(Box<Expression>),
    Block(Vec<Statement>),
    Function(String, Vec<String>, Box<Statement>),
}

fn parser() -> impl Parser<char, Vec<Statement>, Error = Simple<char>> {
    // let block_stmt_parser: fn(
    //     Recursive<'_, char, Statement, Simple<char>>,
    // ) -> BoxedParser<'_, char, Statement, Simple<char>> = |stmt_parser| {
    //     stmt_parser
    //         .repeated()
    //         .delimited_by(just('{'), just('}'))
    //         .padded()
    //         .map(Statement::Block)
    //         .boxed()
    // };

    recursive(|stmt_parser| {
        let block_stmt = stmt_parser
            .repeated()
            .delimited_by(just('{'), just('}'))
            .padded()
            .map(Statement::Block)
            .boxed();

        let expr_parser = recursive(|expr| {
            let int = text::int(10)
                .map(|s: String| Expression::Num(s.parse().unwrap()))
                .padded();

            let float = text::int(10)
                .then_ignore(just('.'))
                .then(text::int(10))
                .map(|(s1, s2)| Expression::Num(format!("{s1}.{s2}").parse().unwrap()));

            let num = int.or(float);

            let escaped_char = just('\\').ignore_then(choice((
                just('\\'),
                just('\"'),
                just('n').to('\n'),
                just('r').to('\r'),
                just('t').to('\t'),
            )));

            let string = just('"')
                .ignore_then(
                    filter(|c| *c != '\"' && *c != '\\')
                        .or(escaped_char)
                        .repeated(),
                )
                .then_ignore(just('"'))
                .collect::<String>()
                .map(Expression::String)
                .padded();

            let identifier = text::ident().map(Expression::Identifier).padded();

            let call = identifier
                .clone()
                .then(
                    expr.clone()
                        .padded()
                        .separated_by(just(',').padded())
                        .delimited_by(just('('), just(')')),
                )
                .map(|(func, args)| Expression::Call(Box::new(func), args));

            let array = expr
                .clone()
                .padded()
                .separated_by(just(',').padded())
                .delimited_by(just('['), just(']'))
                .map(Expression::Array);

            let object = text::ident()
                .padded()
                .map(Expression::String)
                .or(text::int(10).padded().map(Expression::String))
                .or(expr.clone().padded().delimited_by(just('['), just(']')))
                .padded()
                .then_ignore(just(':').padded())
                .then(expr.clone())
                .separated_by(just(',').padded())
                .delimited_by(just('{').padded(), just('}').padded())
                .map(Expression::Object);

            let func_declr_expr = text::keyword("function")
                .padded()
                .then(
                    text::ident()
                        .separated_by(just(',').padded())
                        .delimited_by(just('('), just(')')),
                )
                .then(block_stmt.clone().padded())
                .map(|((_, args), block)| Expression::Function(args, Box::new(block)));

            let arrow_func_expr = text::ident()
                .separated_by(just(',').padded())
                .delimited_by(just('('), just(')'))
                .padded()
                .then_ignore(just("=>"))
                .then(block_stmt.clone().padded())
                .map(|(args, block)| Expression::Function(args, Box::new(block)));

            let base_atom = choice((
                func_declr_expr,
                arrow_func_expr,
                call,
                identifier,
                float,
                num,
                string,
                array,
                object,
                expr.clone().delimited_by(just('('), just(')')),
            ));

            let property = just('.')
                .ignore_then(text::ident().map(Expression::String))
                .or(expr.clone().delimited_by(just('['), just(']')))
                .map(|prop| |expr| Expression::ElementAccess(Box::new(expr), Box::new(prop)));

            let atom = base_atom
                .clone()
                .then(property.repeated())
                .foldl(|expr, prop_fn| prop_fn(expr));

            let op = |c| just(c).padded();

            let unary = op('-')
                .repeated()
                .then(atom)
                .foldr(|_op, rhs| Expression::Negative(Box::new(rhs)));

            let product = unary
                .clone()
                .then(
                    op('*')
                        .to(Expression::Mul as fn(_, _) -> _)
                        .or(op('/').to(Expression::Div as fn(_, _) -> _))
                        .then(unary)
                        .repeated(),
                )
                .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

            let sum = product
                .clone()
                .then(
                    op('+')
                        .to(Expression::Add as fn(_, _) -> _)
                        .or(op('-').to(Expression::Sub as fn(_, _) -> _))
                        .then(product)
                        .repeated(),
                )
                .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

            sum
        });

        let let_stmt = text::keyword("let")
            .padded()
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then(expr_parser.clone())
            .then_ignore(just(';'))
            .map(|(name, expr)| Statement::Let(name, expr));

        let assign_stmt = expr_parser
            .clone()
            .padded()
            .then_ignore(just('=').padded())
            .then(expr_parser.clone())
            .then_ignore(just(';'))
            .map(|(name, expr)| Statement::Assign(name, expr));

        let return_stmt = text::keyword("return")
            .padded()
            .ignore_then(expr_parser.clone())
            .then_ignore(just(';'))
            .map(|x| Statement::Return(Box::new(x)));

        let func_stmt = text::keyword("function")
            .padded()
            .ignore_then(text::ident())
            .padded()
            .then(
                text::ident()
                    .separated_by(just(',').padded())
                    .delimited_by(just('('), just(')')),
            )
            .then(block_stmt.clone().padded())
            .map(|((name, args), block)| Statement::Function(name, args, Box::new(block)));

        let expr_stmt = expr_parser
            .clone()
            .then_ignore(just(';'))
            .map(|x| Statement::Expression(Box::new(x)));

        choice((
            let_stmt,
            return_stmt,
            func_stmt,
            assign_stmt,
            expr_stmt,
            block_stmt,
        ))
        .padded()
    })
    .repeated()
}

fn main() -> ExitCode {
    let source = "let x = (c,b)=>{return 1;};";

    println!("{:#?}", parser().parse(source));
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
