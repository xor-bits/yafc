use super::Ast;
use core::fmt;
use std::fmt::Display;

//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unary {
    pub operator: UnaryOp,
    pub operand: Box<Ast>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Fac,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryAffix {
    Postfix,
    Prefix,
}

//

impl Unary {
    pub fn build(self) -> Ast {
        self.into()
    }
}

impl UnaryOp {
    pub const fn affix(&self) -> UnaryAffix {
        match self {
            UnaryOp::Fac => UnaryAffix::Postfix,
        }
    }
}

impl Ast {
    pub fn unary(operator: UnaryOp, operand: Ast) -> Ast {
        Ast::Unary(Unary {
            operator,
            operand: Box::new(operand),
        })
    }
}

//

impl From<Unary> for Ast {
    fn from(val: Unary) -> Self {
        Self::Unary(val)
    }
}

impl Display for Unary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.operator.affix() == UnaryAffix::Prefix {
            write!(f, "{}{}", self.operator, self.operand)
        } else {
            write!(f, "{}{}", self.operand, self.operator)
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Fac => write!(f, "!"),
        }
    }
}
