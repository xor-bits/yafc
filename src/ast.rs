use std::{
    fmt::{self, Display},
    ops::{Add, BitXor, Div, Mul, Sub},
};

//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinOp::Add => '+',
                BinOp::Sub => '-',
                BinOp::Mul => '*',
                BinOp::Div => '/',
                BinOp::Pow => '^',
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
}

impl Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnOp::Neg => '-',
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    BinExpr {
        operator: BinOp,
        operands: Box<(AstNode, AstNode)>,
    },
    UnExpr {
        operator: UnOp,
        operand: Box<AstNode>,
    },
    Number(Number),
}

impl Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::BinExpr { operator, operands } => {
                write!(f, "({}{operator}{})", operands.0, operands.1)
            }
            AstNode::UnExpr { operator, operand } => write!(f, "({operator}{operand})"),
            AstNode::Number(Number::Integer(v)) => write!(f, "{v}"),
            AstNode::Number(Number::Decimal(v)) => write!(f, "{v}"),
            AstNode::Number(Number::Var(v)) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Integer(i64),
    Decimal(f64),
    Var(String),
}

impl Add for AstNode {
    type Output = AstNode;

    fn add(self, rhs: Self) -> Self::Output {
        Self::BinExpr {
            operator: BinOp::Add,
            operands: Box::new((self, rhs)),
        }
    }
}

impl Sub for AstNode {
    type Output = AstNode;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::BinExpr {
            operator: BinOp::Sub,
            operands: Box::new((self, rhs)),
        }
    }
}

impl Mul for AstNode {
    type Output = AstNode;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::BinExpr {
            operator: BinOp::Mul,
            operands: Box::new((self, rhs)),
        }
    }
}

impl Div for AstNode {
    type Output = AstNode;

    fn div(self, rhs: Self) -> Self::Output {
        Self::BinExpr {
            operator: BinOp::Div,
            operands: Box::new((self, rhs)),
        }
    }
}

impl BitXor for AstNode {
    type Output = AstNode;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::BinExpr {
            operator: BinOp::Pow,
            operands: Box::new((self, rhs)),
        }
    }
}
