use super::Simplifier;
use crate::ast::{
    unary::{Unary, UnaryOp},
    Ast,
};

//

impl Simplifier {
    // calculate unary operations
    // example: replace 4! with 24
    pub fn unary_num_ops(mut ast: Ast) -> Ast {
        match ast {
            Ast::Unary(Unary {
                operator: UnaryOp::Fac,
                operand: box Ast::Num(n),
            }) if n <= 10 => {
                ast = Ast::Num((1..=n).product());
                // log::debug!("unary_num_ops: {ast} == {ast:?}");
            }
            _ => {}
        }
        ast
    }
}
