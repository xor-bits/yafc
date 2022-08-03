use self::{
    binary::{Binary, BinaryOp},
    unary::{Unary, UnaryOp},
};
use core::fmt;
use lalrpop_util::{lalrpop_mod, lexer::Token, ParseError};
use std::fmt::{Debug, Display};

//

pub use grammar::InputParser;

//

pub mod binary;
pub mod build;
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
    pub fn parse(input: &str) -> Result<Ast, ParseError<usize, Token, &str>> {
        InputParser::new().parse(input)
    }

    pub fn format(&self, f: &mut fmt::Formatter<'_>, outer: Option<BinaryOp>) -> fmt::Result {
        match self {
            Ast::Num(v) => write!(f, "{v}"),
            Ast::Var(v) => write!(f, "{v}"),
            Ast::Binary(v) => v.format(f, outer),
            Ast::Unary(v) => write!(f, "{v}"),
        }
    }

    pub fn recurse<F: FnMut(&Self)>(&self, limit: usize, mut f: F) {
        self.recurse_with(limit, &mut f);
    }

    pub fn map<F: FnMut(Self) -> Self>(self, limit: usize, mut f: F) -> Self {
        self.recurse_mut_with(limit, &mut f)
    }

    pub(crate) fn from_pre_post(pre: Vec<&str>, mut ast: Ast, post: Option<UnaryOp>) -> Self {
        let negate_count = pre
            .into_iter()
            .flat_map(|pre| pre.chars())
            .filter(|c| *c == '-')
            .count();
        let negative = negate_count % 2 == 1;

        if let Some(post) = post {
            ast = Ast::unary(post, ast);
        }
        if negative {
            ast = Binary::negate(ast).build();
        }

        ast
    }

    fn recurse_with<F: FnMut(&Self)>(&self, limit: usize, f: &mut F) {
        if limit == 0 {
            log::warn!("Recursion depth limit");
            return;
        }
        let next_limit = limit - 1;

        f(self);
        match self {
            Ast::Binary(Binary { operands, .. }) => {
                for operand in operands {
                    operand.recurse(next_limit, &mut *f)
                }
            }
            Ast::Unary(Unary { operand, .. }) => operand.recurse(next_limit, f),
            _ => {}
        }
    }

    fn recurse_mut_with<F: FnMut(Self) -> Self>(mut self, limit: usize, f: &mut F) -> Self {
        if limit == 0 {
            log::warn!("Recursion depth limit");
            return self;
        }
        let next_limit = limit - 1;

        self = match self {
            Ast::Binary(Binary {
                mut operands,
                operator,
            }) => {
                operands = operands
                    .into_iter()
                    .map(|operand| operand.recurse_mut_with(next_limit, &mut *f))
                    .collect();
                Ast::Binary(Binary { operands, operator })
            }
            Ast::Unary(Unary {
                mut operand,
                operator,
            }) => {
                operand = Box::new(operand.recurse_mut_with(next_limit, f));
                Ast::Unary(Unary { operand, operator })
            }
            ast => ast,
        };
        f(self)
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

impl From<char> for Ast {
    fn from(val: char) -> Self {
        Self::Var(val.to_string())
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format(f, None)
    }
}

//

#[macro_export]
macro_rules! build_ast {
    (+ $($e:expr)+) => {
        Binary::new(BinaryOp::Add)$(.with($e))+.build()
    };

    (* $($e:expr)+) => {
        Binary::new(BinaryOp::Mul)$(.with($e))+.build()
    };

    (^ $($e:expr)+) => {
        Binary::new(BinaryOp::Pow)$(.with($e))+.build()
    };
}
