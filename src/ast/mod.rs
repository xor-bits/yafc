use self::{
    binary::{Binary, BinaryOp},
    unary::{Unary, UnaryOp},
};
use core::fmt;
use lalrpop_util::lalrpop_mod;
use std::fmt::{Debug, Display};

//

pub mod binary;
pub mod unary;

//

lalrpop_mod!(pub grammar);

//

type Num = i64;

//

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    Num(Num),
    Var(String),
    Binary(Binary),
    Unary(Unary),
}

//

impl Ast {
    fn format(&self, f: &mut fmt::Formatter<'_>, outer: Option<BinaryOp>) -> fmt::Result {
        match self {
            Ast::Num(v) => write!(f, "{v}"),
            Ast::Var(v) => write!(f, "{v}"),
            Ast::Binary(v) => v.format(f, outer),
            Ast::Unary(v) => write!(f, "{v}"),
        }
    }
}

impl From<Num> for Ast {
    fn from(val: Num) -> Self {
        Self::Num(val)
    }
}

impl From<String> for Ast {
    fn from(val: String) -> Self {
        Self::Var(val)
    }
}

impl From<&str> for Ast {
    fn from(val: &str) -> Self {
        Self::Var(val.to_string())
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format(f, None)
    }
}
