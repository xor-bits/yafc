use egg::{define_language, Id, RecExpr, Symbol};
use lalrpop_util::{lalrpop_mod, lexer::Token, ParseError};
use std::{
    fmt,
    ops::{Deref, DerefMut},
};

//

lalrpop_mod!(
    #[allow(clippy::all)]
    grammar
);

//

pub type Num = i64; // todo ratio + bignum

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Mul,
    Pow,
}

impl BinOp {
    fn precedence(self) -> u8 {
        match self {
            BinOp::Add => 4,
            BinOp::Mul => 3,
            BinOp::Pow => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnOp {
    Fac,
}

impl UnOp {
    fn precedence(self) -> u8 {
        match self {
            UnOp::Fac => 1,
        }
    }
}

define_language! {
    pub enum YafcLanguage {
        Num(Num),
        Var(Symbol),

        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "^" = Pow([Id; 2]),

        "!" = Fac(Id),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YafcExpr {
    pub(crate) expr: RecExpr<YafcLanguage>,
    pub(crate) root: Option<Id>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pretty<'a> {
    expr: &'a YafcExpr,
    style: PrettyStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrettyStyle {
    /// a + b * c / d
    #[default]
    Infix,

    /// a + \frac{b \cdot c}{d}
    // TODO:
    LaTeX,
}

impl YafcLanguage {
    pub fn bin(operator: BinOp, operands: [Id; 2]) -> Self {
        match operator {
            BinOp::Add => Self::Add(operands),
            BinOp::Mul => Self::Mul(operands),
            BinOp::Pow => Self::Pow(operands),
        }
    }

    pub fn un(operator: UnOp, operand: Id) -> Self {
        match operator {
            UnOp::Fac => Self::Fac(operand),
        }
    }

    pub fn op_and_prec(&self) -> (char, Option<u8>) {
        use YafcLanguage::*;
        match self {
            Add(_) => ('+', Some(BinOp::Add.precedence())),
            Mul(_) => ('*', Some(BinOp::Mul.precedence())),
            Pow(_) => ('^', Some(BinOp::Pow.precedence())),
            Fac(_) => ('!', Some(UnOp::Fac.precedence())),
            _ => (' ', None),
        }
    }

    pub fn needs_paren(prec: Option<u8>, outer_precedence: Option<u8>) -> bool {
        match (prec, outer_precedence) {
            (Some(prec), Some(outer_precedence)) => prec > outer_precedence,
            _ => false,
        }
    }
}

impl YafcExpr {
    pub fn pretty(&self) -> Pretty {
        self.pretty_opt(<_>::default())
    }

    pub fn pretty_opt(&self, style: PrettyStyle) -> Pretty {
        Pretty { expr: self, style }
    }

    pub fn parse_infix(s: &str) -> Result<Self, ParseError<usize, Token<'_>, String>> {
        let parser = grammar::InputParser::new();
        let mut expr = YafcExpr::new();
        let root = parser.parse(&mut expr, s)?;

        expr.root = Some(root);

        Ok(expr)
    }

    pub(crate) fn new() -> Self {
        Self {
            expr: <_>::default(),
            root: None,
        }
    }

    pub(crate) fn make_num(&mut self, num: Num) -> Id {
        self.add(YafcLanguage::Num(num))
    }

    pub(crate) fn make_var(&mut self, var: &str) -> Id {
        self.add(YafcLanguage::Var(var.into()))
    }

    pub(crate) fn make_add(&mut self, lhs: Id, rhs: Id) -> Id {
        self.add(YafcLanguage::Add([lhs, rhs]))
    }

    pub(crate) fn make_neg(&mut self, v: Id) -> Id {
        let neg_1 = self.make_num(-1);
        self.make_mul(neg_1, v)
    }

    pub(crate) fn make_sub(&mut self, lhs: Id, rhs: Id) -> Id {
        let neg = self.make_neg(rhs);
        self.add(YafcLanguage::Add([lhs, neg]))
    }

    pub(crate) fn make_mul(&mut self, lhs: Id, rhs: Id) -> Id {
        self.add(YafcLanguage::Mul([lhs, rhs]))
    }

    pub(crate) fn make_inv(&mut self, v: Id) -> Id {
        let neg_1 = self.make_num(-1);
        self.make_pow(neg_1, v)
    }

    pub(crate) fn make_div(&mut self, lhs: Id, rhs: Id) -> Id {
        let inv = self.make_inv(rhs);
        self.add(YafcLanguage::Mul([lhs, inv]))
    }

    pub(crate) fn make_pow(&mut self, lhs: Id, rhs: Id) -> Id {
        self.add(YafcLanguage::Pow([lhs, rhs]))
    }

    // pub(crate) fn make_fac(&mut self, v: Id) -> Id {
    //     self.add(YafcLanguage::Fac(v))
    // }
}

impl Deref for YafcExpr {
    type Target = RecExpr<YafcLanguage>;

    fn deref(&self) -> &Self::Target {
        &self.expr
    }
}

impl DerefMut for YafcExpr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.expr
    }
}

impl fmt::Display for YafcExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pretty().fmt(f)
    }
}

impl fmt::Display for Pretty<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let root = self.expr.root.expect("Not evaluated");
        match self.style {
            PrettyStyle::Infix => self.fmt_rec_infix(f, root, None),
            PrettyStyle::LaTeX => self.fmt_rec_latex(f, root, None),
        }
    }
}

impl Pretty<'_> {
    fn fmt_rec_infix(
        &self,
        f: &mut fmt::Formatter,
        i: Id,
        outer_precedence: Option<u8>,
    ) -> fmt::Result {
        use YafcLanguage::*;

        let expr = &self.expr.expr[i];
        let (op, prec) = expr.op_and_prec();
        let needs_paren = YafcLanguage::needs_paren(prec, outer_precedence);

        if needs_paren {
            write!(f, "(")?;
        }

        match expr {
            Num(num) => write!(f, "{num}")?,
            Var(var) => write!(f, "{var}")?,
            Add([lhs, rhs]) | Mul([lhs, rhs]) | Pow([lhs, rhs]) => {
                self.fmt_rec_infix(f, *lhs, prec)?;
                if f.alternate() {
                    write!(f, " {op} ")?;
                } else {
                    write!(f, "{op}")?;
                }
                self.fmt_rec_infix(f, *rhs, prec)?;
            }
            Fac(v) => {
                self.fmt_rec_infix(f, *v, prec)?;
                write!(f, "{op}")?;
            }
        };

        if needs_paren {
            write!(f, ")")?;
        }

        Ok(())
    }

    fn fmt_rec_latex(
        &self,
        f: &mut fmt::Formatter,
        i: Id,
        outer_precedence: Option<u8>,
    ) -> fmt::Result {
        use YafcLanguage::*;

        let expr = &self.expr.expr[i];
        let (op, prec) = expr.op_and_prec();
        let needs_paren = YafcLanguage::needs_paren(prec, outer_precedence);

        let op = match op {
            '+' => "+",
            '*' => "\\cdot",
            '^' => "^",
            '!' => "!",
            _ => "",
        };

        write!(f, "{{")?;
        if needs_paren {
            write!(f, "\\left(")?;
        }

        match expr {
            Num(num) => write!(f, "{num}")?,
            Var(var) => write!(f, "{var}")?,
            Add([lhs, rhs]) | Mul([lhs, rhs]) | Pow([lhs, rhs]) => {
                self.fmt_rec_latex(f, *lhs, prec)?;
                if f.alternate() {
                    write!(f, " {op} ")?;
                } else {
                    write!(f, "{op}")?;
                }
                self.fmt_rec_latex(f, *rhs, prec)?;
            }
            Fac(v) => {
                self.fmt_rec_latex(f, *v, prec)?;
                write!(f, "{op}")?;
            }
        };

        if needs_paren {
            write!(f, "\\right)")?;
        }
        write!(f, "}}")?;

        Ok(())
    }
}
