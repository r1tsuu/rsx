use crate::{
    parser::Expression,
    tokenizer::{Token, TokenKind},
};

pub fn evaluate_expression(expression: &Expression) -> f32 {
    match expression {
        Expression::Program { expressions } => {
            let first_expr = expressions.get(0).unwrap();
            let reordered = &reorder_expression(first_expr.clone());
            evaluate_expression(reordered)
        }
        Expression::NumberLiteral { value } => *value,
        Expression::BinaryOp { left, op, right } => match op.kind {
            TokenKind::Plus => evaluate_expression(left) + evaluate_expression(right),
            TokenKind::Minus => evaluate_expression(left) - evaluate_expression(right),
            TokenKind::Multiply => evaluate_expression(left) * evaluate_expression(right),
            TokenKind::Divide => evaluate_expression(left) / evaluate_expression(right),
            _ => {
                todo!()
            }
        },
        _ => {
            todo!()
        }
    }
}

fn reorder_expression(expr: Expression) -> Expression {
    match expr {
        Expression::BinaryOp { left, op, right } => {
            let left = reorder_expression(*left);
            let right = reorder_expression(*right);

            if let Expression::BinaryOp {
                left: right_left,
                op: right_op,
                right: right_right,
            } = right.clone()
            {
                if get_precedence(&op) > get_precedence(&right_op) {
                    let new_left = Expression::BinaryOp {
                        left: Box::from(left),
                        op,
                        right: right_left,
                    };

                    return Expression::BinaryOp {
                        left: Box::from(new_left),
                        op: right_op,
                        right: right_right,
                    };
                }
            }

            return Expression::BinaryOp {
                left: Box::from(left),
                op,
                right: Box::from(right),
            };
        }
        _ => expr,
    }
}

fn get_precedence(token: &Token) -> i32 {
    match token.kind {
        TokenKind::Plus | TokenKind::Minus => 1,
        TokenKind::Multiply | TokenKind::Divide => 2,
        _ => 0, // Default precedence for unsupported operators
    }
}
