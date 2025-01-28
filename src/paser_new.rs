use std::rc::Rc;

use chumsky::{
    error::Simple,
    prelude::{choice, just, todo},
    recursive, Parser,
};

use crate::{
    parser::Expression,
    tokenizer::{Token, TokenKind},
};

fn parser() -> impl Parser<TokenKind, Expression, Error = Simple<TokenKind>> {
    todo()
}

fn m() {
    let a = parser().parse([TokenKind::CloseBrace]);
}
