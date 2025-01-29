use chumsky::{prelude::*, recursive};

#[derive(Debug)]
pub enum Expression {
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
    Function(Option<String>, Vec<String>, Box<Statement>),
    /// Object Expression, Element expression
    ElementAccess(Box<Expression>, Box<Expression>),
    /// Key expression, Value expression
    Object(Vec<(Expression, Expression)>),
}

#[derive(Debug)]
pub enum Statement {
    Let(String, Box<Expression>),
    Assign(Box<Expression>, Box<Expression>),
    Return(Box<Expression>),
    Expression(Box<Expression>),
    Block(Vec<Statement>),
    Function(String, Vec<String>, Box<Statement>),
    Condition(
        (Box<Expression>, Box<Statement>),
        Option<Vec<(Box<Expression>, Box<Statement>)>>,
        Option<Box<Statement>>,
    ),
}

enum FuncCallArgsOrProperty {
    Arguments(Vec<Expression>),
    Property(Expression),
}

fn number_parser() -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
    text::int(10)
        .map(|s: String| Expression::Num(s.parse().unwrap()))
        .padded()
        .or(text::int(10)
            .then_ignore(just('.'))
            .then(text::int(10))
            .map(|(s1, s2)| Expression::Num(format!("{s1}.{s2}").parse().unwrap())))
}

fn string_parser() -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
    let escaped_char = just('\\').ignore_then(choice((
        just('\\'),
        just('\"'),
        just('n').to('\n'),
        just('r').to('\r'),
        just('t').to('\t'),
    )));

    just('"')
        .ignore_then(
            filter(|c| *c != '\"' && *c != '\\')
                .or(escaped_char)
                .repeated(),
        )
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Expression::String)
        .padded()
}

fn identifier_parser() -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
    text::ident().map(Expression::Identifier).padded()
}

fn array_parser(
    expr: impl Parser<char, Expression, Error = Simple<char>> + Clone,
) -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
    expr.padded()
        .separated_by(just(',').padded())
        .delimited_by(just('['), just(']'))
        .map(Expression::Array)
}

fn object_parser(
    expr: impl Parser<char, Expression, Error = Simple<char>> + Clone,
) -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
    text::ident()
        .padded()
        .map(Expression::String)
        .or(text::int(10).padded().map(Expression::String))
        .or(expr.clone().padded().delimited_by(just('['), just(']')))
        .padded()
        .then_ignore(just(':').padded())
        .then(expr.clone())
        .separated_by(just(',').padded())
        .delimited_by(just('{').padded(), just('}').padded())
        .map(Expression::Object)
}

fn named_function_base_parser(
    stmt_parser: impl Parser<char, Statement, Error = Simple<char>> + Clone,
) -> impl Parser<char, ((String, Vec<String>), Statement), Error = Simple<char>> + Clone {
    text::keyword("function")
        .padded()
        .ignore_then(text::ident())
        .padded()
        .then(
            text::ident()
                .separated_by(just(',').padded())
                .delimited_by(just('('), just(')')),
        )
        .then(stmt_parser.padded())
}

fn block_parser(
    stmt_parser: impl Parser<char, Statement, Error = Simple<char>> + Clone,
) -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
    stmt_parser
        .clone()
        .repeated()
        .delimited_by(just('{'), just('}'))
        .padded()
        .map(Statement::Block)
}

fn function_expression_parser(
    stmt_parser: impl Parser<char, Statement, Error = Simple<char>> + Clone,
) -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
    let func_declr_expr = named_function_base_parser(stmt_parser.clone())
        .map(|((name, args), block)| Expression::Function(Some(name), args, Box::new(block)));

    let arrow_func_expr = text::ident()
        .separated_by(just(',').padded())
        .delimited_by(just('('), just(')'))
        .padded()
        .then_ignore(just("=>"))
        .then(block_parser(stmt_parser.clone()).padded())
        .map(|(args, block)| Expression::Function(None, args, Box::new(block)));

    let unnamed_func_expr = text::keyword("function")
        .padded()
        .then(
            text::ident()
                .separated_by(just(',').padded())
                .delimited_by(just('('), just(')')),
        )
        .then(stmt_parser.clone().padded())
        .map(|((_, args), block)| Expression::Function(None, args, Box::new(block)));

    choice((func_declr_expr, unnamed_func_expr, arrow_func_expr))
}

fn atom_parser(
    expr_parser: impl Parser<char, Expression, Error = Simple<char>> + Clone,
    stmt_parser: impl Parser<char, Statement, Error = Simple<char>> + Clone,
) -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
    let atom = choice((
        expr_parser.clone().delimited_by(just('('), just(')')),
        function_expression_parser(stmt_parser),
        identifier_parser(),
        number_parser(),
        string_parser(),
        array_parser(expr_parser.clone()),
        object_parser(expr_parser.clone()),
    ));

    let property = just('.')
        .ignore_then(text::ident().map(Expression::String))
        .or(expr_parser.clone().delimited_by(just('['), just(']')))
        .map(|prop| (FuncCallArgsOrProperty::Property(prop)));

    // Function calls - now returns (bool, Expression) instead of (bool, Vec<Expression>)
    let args = expr_parser
        .clone()
        .padded()
        .separated_by(just(',').padded())
        .delimited_by(just('('), just(')'))
        .map(|args| (FuncCallArgsOrProperty::Arguments(args))); // Wrap the Vec in an Expression variant

    // Combined member expression
    atom.clone()
        .then(property.or(args).repeated())
        .foldl(|expr, right| match right {
            FuncCallArgsOrProperty::Arguments(args) => Expression::Call(Box::new(expr), args),
            FuncCallArgsOrProperty::Property(prop) => {
                Expression::ElementAccess(Box::new(expr), Box::new(prop))
            }
        })
}

pub fn expr_parser<'a>(
    stmt_parser: impl Parser<char, Statement, Error = Simple<char>> + Clone + 'a,
) -> impl Parser<char, Expression, Error = Simple<char>> + Clone + 'a {
    recursive(|expr_parser| {
        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .then(atom_parser(expr_parser, stmt_parser))
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
    })
}

fn stmt_parser() -> impl Parser<char, Statement, Error = Simple<char>> {
    recursive(|stmt_parser| {
        let let_stmt = text::keyword("let")
            .padded()
            .ignore_then(text::ident().padded())
            .then_ignore(just('=').padded())
            .then(expr_parser(stmt_parser.clone()))
            .then_ignore(choice((just(';'), just('\n'))))
            .map(|(name, expr)| Statement::Let(name, Box::new(expr)));

        let else_clause = text::keyword("else")
            .padded()
            .ignore_then(block_parser(stmt_parser.clone()))
            .padded();

        let else_if_clauses = just("else if")
            .padded()
            .ignore_then(
                expr_parser(stmt_parser.clone())
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .padded()
            .then(block_parser(stmt_parser.clone()))
            .padded()
            .repeated()
            .at_least(1)
            .padded();

        let condition_stmt = text::keyword("if")
            .padded()
            .ignore_then(
                expr_parser(stmt_parser.clone())
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .then(block_parser(stmt_parser.clone()))
            .padded()
            .then(else_if_clauses.or_not())
            .padded()
            .then(else_clause.or_not())
            .map(|(((if_expr, if_stmt), else_if_clauses), else_clause)| {
                let else_if_clauses = else_if_clauses.map(|v| {
                    Vec::from_iter(
                        v.into_iter()
                            .map(|(expr, stmt)| (Box::new(expr), Box::new(stmt))),
                    )
                });

                Statement::Condition(
                    (Box::new(if_expr), Box::new(if_stmt)),
                    else_if_clauses,
                    else_clause.map(Box::new),
                )
            });

        let assign_stmt = expr_parser(stmt_parser.clone())
            .padded()
            .then_ignore(just('=').padded())
            .then(expr_parser(stmt_parser.clone()))
            .then_ignore(choice((just(';'), just('\n'))))
            .try_map(|(name, expr), span| match name {
                Expression::Identifier(_) | Expression::ElementAccess(..) => {
                    Ok(Statement::Assign(Box::new(name), Box::new(expr)))
                }
                _ => Err(Simple::custom(
                    span,
                    "Invalid left-hand side in assigment. Must be a reference.",
                )),
            });

        let return_stmt = text::keyword("return")
            .padded()
            .ignore_then(expr_parser(stmt_parser.clone()))
            .then_ignore(just(';').or_not())
            .map(|x| Statement::Return(Box::new(x)));

        let func_stmt = named_function_base_parser(stmt_parser.clone())
            .map(|((name, args), block)| Statement::Function(name, args, Box::new(block)));

        let expr_stmt = expr_parser(stmt_parser.clone())
            .then_ignore(just(';'))
            .map(|x| Statement::Expression(Box::new(x)));

        choice((
            let_stmt,
            return_stmt,
            condition_stmt,
            func_stmt,
            assign_stmt,
            block_parser(stmt_parser),
            expr_stmt,
        ))
        .padded()
    })
}

pub fn parser() -> impl Parser<char, Vec<Statement>, Error = Simple<char>> {
    stmt_parser().repeated().then_ignore(end())
}
