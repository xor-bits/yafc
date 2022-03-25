use super::Simplifier;
use crate::ast::{
    binary::{Binary, BinaryOp},
    Ast,
};
use core::fmt;
use std::{fmt::Display, mem};

//

/// 2*x*y^2
/// yields: 2, x, y
pub fn term_factors(term: &Ast) -> impl Iterator<Item = &'_ Ast> {
    struct NestedIter<'a> {
        inner: AstTermFactorIter<'a>,
        outer: Option<AstTermFactorIter<'a>>,
    }

    enum AstTermFactorIter<'a> {
        Nest(Box<NestedIter<'a>>),
        Mul { operands: &'a [Ast] },
        Pow { operand: &'a Ast },
        Other { other: &'a Ast },
        None,
    }

    fn make_iter(ast: &Ast) -> AstTermFactorIter<'_> {
        match ast {
            Ast::Binary(Binary {
                operands,
                operator: BinaryOp::Mul,
            }) => AstTermFactorIter::Mul { operands },
            Ast::Binary(Binary {
                operands,
                operator: BinaryOp::Pow,
            }) => match operands.first() {
                Some(operand) => AstTermFactorIter::Pow { operand },
                None => AstTermFactorIter::None,
            },
            other => AstTermFactorIter::Other { other },
        }
    }

    impl<'a> Iterator for AstTermFactorIter<'a> {
        type Item = &'a Ast;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::Nest(box NestedIter { inner, outer }) => match inner.next() {
                    Some(ast) => Some(ast),
                    None => {
                        match outer.take() {
                            Some(n) => *self = n,
                            None => *self = Self::None,
                        }
                        self.next()
                    }
                },
                Self::Mul { operands } => match operands.split_first() {
                    Some((first, others)) => {
                        *operands = others;

                        let mut tmp = Self::None;
                        mem::swap(&mut tmp, self);
                        *self = Self::Nest(Box::new(NestedIter {
                            inner: make_iter(first),
                            outer: Some(tmp),
                        }));

                        self.next()
                    }
                    None => {
                        *self = Self::None;
                        None
                    }
                },
                Self::Pow { operand } => {
                    *self = make_iter(operand);
                    self.next()
                }
                Self::Other { other } => {
                    let result = Some(*other);
                    *self = Self::None;
                    result
                }
                Self::None => None,
            }
        }
    }

    make_iter(term)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermFactorExtractResult {
    Some {
        /// the one queried
        factor: Ast,
        // others
        coefficient: Ast,
    },
    None {
        original: Ast,
    },
}

impl Display for TermFactorExtractResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TermFactorExtractResult::Some {
                factor,
                coefficient,
            } => write!(f, "[{factor} ; {coefficient}]"),
            TermFactorExtractResult::None { original } => write!(f, "[FAIL ; {original}]"),
        }
    }
}

/// 2*x*y^2, &x
/// yields: Some((2*y^2, x))
///
/// 4*x*y^2, &2
/// yields: Some((2*x*y^2, 2))
///
/// 3*x*y^2, &2
/// yields: None
///
/// x*y^2, &2
/// yields: None
///
/// x*y^2, &z
/// yields: None
pub fn term_factor_extract(term: Ast, factor: &Ast) -> TermFactorExtractResult {
    match term {
        Ast::Binary(Binary {
            mut operands,
            operator: BinaryOp::Mul,
        }) => {
            for (i, operand) in operands.iter().enumerate() {
                if operand.structural_eq(factor) {
                    let factor = operands.remove(i);
                    let coefficient = Binary {
                        operands,
                        operator: BinaryOp::Mul,
                    }
                    .build();
                    return TermFactorExtractResult::Some {
                        factor,
                        coefficient,
                    };
                }

                if let TermFactorExtractResult::Some {
                    factor,
                    coefficient,
                } = term_factor_extract(operand.clone(), factor)
                {
                    let _ = operands.remove(i);
                    let coefficient = Binary {
                        operands,
                        operator: BinaryOp::Mul,
                    }
                    .with(coefficient)
                    .build();
                    return TermFactorExtractResult::Some {
                        factor,
                        coefficient,
                    };
                }
            }

            TermFactorExtractResult::None {
                original: Binary {
                    operands,
                    operator: BinaryOp::Mul,
                }
                .build(),
            }
        }
        Ast::Binary(Binary {
            mut operands,
            operator: BinaryOp::Pow,
        }) => {
            if let Some(first) = operands.first() {
                // x^y / x^z => x^(y-z)

                let base = match factor {
                    Ast::Binary(Binary {
                        operands,
                        operator: BinaryOp::Pow,
                    }) => operands.first(),
                    ast => Some(ast),
                };

                if let Some(true) = base.map(|base| first.structural_eq(base)) {
                    let base = operands.remove(0);

                    let lhs = {
                        if operands.is_empty() {
                            operands.push(1.into());
                        }
                        Binary {
                            operands,
                            operator: BinaryOp::Pow,
                        }
                    };
                    let rhs = match factor.clone() {
                        Ast::Binary(Binary {
                            operator: BinaryOp::Pow,
                            mut operands,
                        }) => {
                            operands.remove(0);
                            Binary {
                                operands,
                                operator: BinaryOp::Pow,
                            }
                        }
                        _ => Binary::new(BinaryOp::Pow).with(1),
                    };

                    let coefficient = Binary::new(BinaryOp::Pow)
                        .with(base.clone())
                        .with(
                            Binary::new(BinaryOp::Add)
                                .with(lhs)
                                .with(Binary::new(BinaryOp::Mul).with(rhs).with(-1).build()),
                        )
                        .build()
                        .map(32, |ast| Simplifier::binary_num_ops(ast, 0));

                    TermFactorExtractResult::Some {
                        factor: base,
                        coefficient,
                    }
                } else {
                    TermFactorExtractResult::None {
                        original: Binary {
                            operands,
                            operator: BinaryOp::Pow,
                        }
                        .build(),
                    }
                }
            } else {
                TermFactorExtractResult::None {
                    original: Binary {
                        operands,
                        operator: BinaryOp::Pow,
                    }
                    .build(),
                }
            }
        }
        other if other.structural_eq(factor) => TermFactorExtractResult::Some {
            factor: other,
            coefficient: Ast::Num(1),
        },
        original => TermFactorExtractResult::None { original },
    }
}

//

#[cfg(test)]
mod test {
    use std::fmt::{Debug, Display};

    use crate::{
        ast::binary::{Binary, BinaryOp},
        simplifier::factorize::{term_factor_extract, term_factors, TermFactorExtractResult},
    };

    pub fn assert_eq<T: Debug + Display + PartialEq>(lhs: T, rhs: T) {
        assert_eq!(lhs, rhs, "\n left: {lhs}\nright: {rhs}")
    }

    #[test]
    pub fn test_term_factors_mul() {
        // 2xy => 2,x,y
        let term = Binary::new(BinaryOp::Mul)
            .with(2)
            .with("x")
            .with("y")
            .build();

        let mut iter = term_factors(&term);

        assert_eq!(iter.next(), Some(&2.into()));
        assert_eq!(iter.next(), Some(&"x".into()));
        assert_eq!(iter.next(), Some(&"y".into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn test_term_factors_pow() {
        // x^y^z => x
        let term = Binary::new(BinaryOp::Pow)
            .with("x")
            .with("y")
            .with("z")
            .build();

        let mut iter = term_factors(&term);

        assert_eq!(iter.next(), Some(&"x".into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn test_term_factors_nest() {
        // 2xy^3 => 2,x,y
        let term = Binary::new(BinaryOp::Mul)
            .with(2)
            .with("x")
            .with(Binary::new(BinaryOp::Pow).with("y").with(3))
            .build();

        let mut iter = term_factors(&term);

        assert_eq!(iter.next(), Some(&2.into()));
        assert_eq!(iter.next(), Some(&"x".into()));
        assert_eq!(iter.next(), Some(&"y".into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn test_term_factors_nest_complex() {
        // ((y^2)^3)*x*(y^3)*(3^z) => y,x,y,3,z
        let term = Binary::new(BinaryOp::Mul)
            .with(
                Binary::new(BinaryOp::Pow)
                    .with(Binary::new(BinaryOp::Pow).with("y").with(2))
                    .with(3),
            )
            .with("x")
            .with(Binary::new(BinaryOp::Pow).with("y").with(3))
            .with(Binary::new(BinaryOp::Mul).with(3).with("z"))
            .build();

        let mut iter = term_factors(&term);

        assert_eq!(iter.next(), Some(&"y".into()));
        assert_eq!(iter.next(), Some(&"x".into()));
        assert_eq!(iter.next(), Some(&"y".into()));
        assert_eq!(iter.next(), Some(&3.into()));
        assert_eq!(iter.next(), Some(&"z".into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn test_term_factor_extract_mul() {
        // 2xy / x => x,2y
        let term = Binary::new(BinaryOp::Mul)
            .with(2)
            .with("x")
            .with("y")
            .build();

        let lhs = term_factor_extract(term, &"x".into());
        let rhs = TermFactorExtractResult::Some {
            factor: "x".into(),
            coefficient: Binary::new(BinaryOp::Mul).with(2).with("y").build(),
        };

        assert_eq(lhs, rhs);
    }

    #[test]
    pub fn test_term_factor_extract_pow() {
        // x^y^z / x => x^(y^z-1)
        let term = Binary::new(BinaryOp::Pow)
            .with("x")
            .with("y")
            .with("z")
            .build();

        let lhs = term_factor_extract(term, &"x".into());
        let rhs = TermFactorExtractResult::Some {
            factor: "x".into(),
            coefficient: Binary::new(BinaryOp::Pow)
                .with("x")
                .with(
                    Binary::new(BinaryOp::Add)
                        .with(Binary::new(BinaryOp::Pow).with("y").with("z"))
                        .with(-1),
                )
                .build(),
        };

        assert_eq(lhs, rhs);
    }

    #[test]
    pub fn test_term_factor_extract_nest() {
        // 2xy^3 / y => 2xy^2
        let term = Binary::new(BinaryOp::Mul)
            .with(2)
            .with("x")
            .with(Binary::new(BinaryOp::Pow).with("y").with(3))
            .build();

        let lhs = term_factor_extract(term, &"y".into());
        let rhs = TermFactorExtractResult::Some {
            factor: "y".into(),
            coefficient: Binary::new(BinaryOp::Mul)
                .with(2)
                .with("x")
                .with(Binary::new(BinaryOp::Pow).with("y").with(2))
                .build(),
        };

        assert_eq(lhs, rhs);
    }

    #[test]
    pub fn test_term_factor_extract_nest_complex_0() {
        // y^3 / y^3 => 1
        let term = Binary::new(BinaryOp::Pow).with("y").with(3).build();

        let factor = term.clone();
        let lhs = term_factor_extract(term, &factor);
        let rhs = TermFactorExtractResult::Some {
            factor: "y".into(),
            coefficient: 1.into(),
        };

        assert_eq(lhs, rhs);
    }

    #[test]
    pub fn test_term_factor_extract_nest_complex_1() {
        // ((y^2)^3)*x*(y^3)*(3^z) / y^3 => ((y^2)^3)*x*(3^z)
        let term = Binary::new(BinaryOp::Mul)
            .with(
                Binary::new(BinaryOp::Pow)
                    .with(Binary::new(BinaryOp::Pow).with("y").with(2))
                    .with(3),
            )
            .with("x")
            .with(Binary::new(BinaryOp::Pow).with("y").with(3))
            .with(Binary::new(BinaryOp::Mul).with(3).with("z"))
            .build();

        let factor = Binary::new(BinaryOp::Pow).with("y").with("3").build();
        let lhs = term_factor_extract(term, &factor);
        let rhs = TermFactorExtractResult::Some {
            factor: "y".into(),
            coefficient: Binary::new(BinaryOp::Mul)
                .with(
                    Binary::new(BinaryOp::Pow)
                        .with(Binary::new(BinaryOp::Pow).with("y").with(2))
                        .with(3),
                )
                .with("x")
                .with(Binary::new(BinaryOp::Mul).with(3).with("z"))
                .build(),
        };

        assert_eq(lhs, rhs);
    }
}
