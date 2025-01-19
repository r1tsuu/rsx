use crate::{parser::Expression, tokenizer::TokenKind};

pub fn evaluate_expression(expression: &mut Expression) -> f32 {
    match expression {
        Expression::Program { expressions } => {
            let mut first_expr = expressions.get(0).unwrap().clone();
            evaluate_expression(&mut first_expr)
        }
        Expression::NumberLiteral { value } => *value,
        Expression::BinaryOp { left, op, right } => {
            reorder_expression(left);
            reorder_expression(right);

            match op.kind {
                TokenKind::Plus => evaluate_expression(left) + evaluate_expression(right),
                TokenKind::Minus => evaluate_expression(left) - evaluate_expression(right),
                TokenKind::Multiply => evaluate_expression(left) * evaluate_expression(right),
                TokenKind::Divide => evaluate_expression(left) / evaluate_expression(right),
                _ => {
                    todo!()
                }
            }
        }
        _ => {
            todo!()
        }
    }
}

fn get_precedence(kind: &TokenKind) -> i32 {
    match kind {
        TokenKind::Plus | TokenKind::Minus => 1,
        TokenKind::Multiply | TokenKind::Divide => 2,
        _ => 0, // Default precedence for unsupported operators
    }
}

pub fn reorder_expression(expression: &mut Box<Expression>) {
    match **expression {
        Expression::BinaryOp { left, op, right } => {
            // Recursively reorder the right expression

            // Temporarily take the mutable reference to avoid borrowing conflicts
            if let Expression::BinaryOp {
                left: right_left,
                op: right_op,
                right: right_right,
            } = right.as_mut()
            {
                if get_precedence(&right_op.kind) > get_precedence(&op.kind) {}
            }
        }
        _ => {}
    }
}

fn x(a: impl Into<i32>) {
    let d = 32;
    x(d);

    let mut a = 1;
    let mut b = 2;

    let mut z = &Box::new(a);
    let mut f = &Box::new(b);
    let a = **z;

    std::mem::swap(&mut z, &mut f);
}
