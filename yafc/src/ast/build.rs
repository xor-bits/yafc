use super::{
    binary::{Binary, BinaryOp},
    Ast,
};
use std::ops::{Add, BitXor, Div, Mul, Sub};

//

impl<T: Into<Ast>> Add<T> for Ast {
    type Output = Self;

    fn add(mut self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        match &mut self {
            Ast::Binary(Binary {
                operator: BinaryOp::Add,
                operands,
            }) => {
                operands.push(rhs);
                self
            }
            _ => Binary::new(BinaryOp::Add).with(self).with(rhs).build(),
        }
    }
}

impl<T: Into<Ast>> Sub<T> for Ast {
    type Output = Self;

    fn sub(mut self, rhs: T) -> Self::Output {
        let rhs = Binary::new(BinaryOp::Mul).with(-1).with(rhs).build();
        match &mut self {
            Ast::Binary(Binary {
                operator: BinaryOp::Add,
                operands,
            }) => {
                operands.push(rhs);
                self
            }
            _ => Binary::new(BinaryOp::Add).with(self).with(rhs).build(),
        }
    }
}

impl<T: Into<Ast>> Mul<T> for Ast {
    type Output = Self;

    fn mul(mut self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        match &mut self {
            Ast::Binary(Binary {
                operator: BinaryOp::Mul,
                operands,
            }) => {
                operands.push(rhs);
                self
            }
            _ => Binary::new(BinaryOp::Mul).with(self).with(rhs).build(),
        }
    }
}

impl<T: Into<Ast>> Div<T> for Ast {
    type Output = Self;

    fn div(mut self, rhs: T) -> Self::Output {
        let rhs = Binary::new(BinaryOp::Pow).with(rhs).with(-1).build();
        match &mut self {
            Ast::Binary(Binary {
                operator: BinaryOp::Mul,
                operands,
            }) => {
                operands.push(rhs);
                self
            }
            _ => Binary::new(BinaryOp::Mul).with(self).with(rhs).build(),
        }
    }
}

impl<T: Into<Ast>> BitXor<T> for Ast {
    type Output = Self;

    fn bitxor(mut self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        match &mut self {
            Ast::Binary(Binary {
                operator: BinaryOp::Pow,
                operands,
            }) => {
                operands.push(rhs);
                self
            }
            _ => Binary::new(BinaryOp::Pow).with(self).with(rhs).build(),
        }
    }
}
