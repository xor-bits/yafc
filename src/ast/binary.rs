use super::Ast;
use core::fmt;
use std::fmt::Display;

//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binary {
    pub operator: BinaryOp,
    pub operands: Vec<Ast>,
    // true = negative , false = positive
    // sign: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Mul,
    Pow,
}

//

impl Binary {
    pub fn new(operator: BinaryOp) -> Self {
        Self {
            operator,
            operands: vec![],
            // sign: false,
        }
    }

    pub fn with<A: Into<Ast>>(mut self, ast: A) -> Self {
        self.operands.push(ast.into());
        self
    }

    pub fn negate<A: Into<Ast>>(ast: A) -> Self {
        Binary::new(BinaryOp::Mul).with(ast).with(-1)
    }

    pub fn build(self) -> Ast {
        self.into()
    }

    pub fn push<A: Into<Ast>>(&mut self, ast: A) {
        self.operands.push(ast.into());
    }

    pub(super) fn format(
        &self,
        f: &mut fmt::Formatter<'_>,
        outer: Option<BinaryOp>,
    ) -> fmt::Result {
        let len = self.operands.len();
        if len == 0 {
            Ok(())
        } else if len == 1 {
            write!(f, "{}", self.operands.first().unwrap())
        } else {
            let needs_paren = match (self.operator, outer) {
                (BinaryOp::Add, Some(BinaryOp::Add)) => false,
                (BinaryOp::Add, Some(BinaryOp::Mul)) => true,
                (BinaryOp::Add, Some(BinaryOp::Pow)) => true,

                (BinaryOp::Mul, Some(BinaryOp::Add)) => false,
                (BinaryOp::Mul, Some(BinaryOp::Mul)) => false,
                (BinaryOp::Mul, Some(BinaryOp::Pow)) => true,

                (BinaryOp::Pow, Some(BinaryOp::Add)) => false,
                (BinaryOp::Pow, Some(BinaryOp::Mul)) => false,
                (BinaryOp::Pow, Some(BinaryOp::Pow)) => false,

                (BinaryOp::Add, None) => false,
                (BinaryOp::Mul, None) => false,
                (BinaryOp::Pow, None) => false,
            };

            if needs_paren {
                write!(f, "(")?;
            }

            let mut iter = self.operands.iter();
            iter.next().unwrap().format(f, Some(self.operator))?;

            for op in iter {
                if f.alternate() {
                    write!(f, " {} ", self.operator)?;
                } else {
                    write!(f, "{}", self.operator)?;
                }
                op.format(f, Some(self.operator))?;
            }

            if needs_paren {
                write!(f, ")")?;
            }

            Ok(())
        }
    }
}

//

impl From<Binary> for Ast {
    fn from(mut val: Binary) -> Self {
        if val.operands.is_empty() {
            match val.operator {
                BinaryOp::Add => Ast::Num(0),
                BinaryOp::Mul => Ast::Num(1),
                BinaryOp::Pow => Ast::Num(1),
            }
        } else if val.operands.len() == 1 {
            val.operands.pop().unwrap()
        } else {
            Self::Binary(val)
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Pow => write!(f, "^"),
        }
    }
}
