use self::factorize::{term_factor_extract, term_factors, TermFactorExtractResult};
use crate::ast::{
    binary::{Binary, BinaryOp},
    unary::{Unary, UnaryOp},
    Ast,
};
use std::mem;

//

mod factorize;

//

pub struct Simplifier;

//

impl Simplifier {
    pub fn run(mut ast: Ast) -> Ast {
        ast = ast.map(32, |ast| Self::de_paren(ast, 0));
        ast = ast.map(32, |ast| Self::combine_terms(ast, 0));
        ast = ast.map(32, |ast| Self::unary_num_ops(ast, 0));
        ast = ast.map(32, |ast| Self::binary_num_ops(ast, 0));

        ast
    }

    // remove unnecessary parenthesis
    // example: replace (a+b)+c with a+b+c and so on
    fn de_paren(mut ast: Ast, _: usize) -> Ast {
        if let Ast::Binary(Binary { operator, operands }) = ast {
            let operands = operands
                .into_iter()
                .flat_map(move |ast| match ast {
                    Ast::Binary(Binary {
                        operator: b,
                        operands,
                    }) if operator == b => operands,
                    ast => vec![ast],
                })
                .collect();

            ast = Binary { operator, operands }.into();
            // log::debug!("de_paren: {ast} == {ast:?}");
        }
        ast
    }

    // combine terms
    // example: x + x = 2 * x
    fn combine_terms(mut ast: Ast, depth: usize) -> Ast {
        if let Ast::Binary(Binary {
            operator: BinaryOp::Add,
            operands: mut terms,
        }) = ast
        {
            let mut new_terms = vec![];

            while let Some(last) = terms.last().cloned() {
                let first_non_0_common_factor = term_factors(&last)
                    .map(|factor| {
                        let mut coeff = Binary::new(BinaryOp::Add);

                        terms.drain_filter(|ast| {
                            let mut tmp = Ast::Num(0);
                            mem::swap(&mut tmp, ast);
                            match term_factor_extract(tmp, factor) {
                                TermFactorExtractResult::Some { coefficient, .. } => {
                                    coeff.push(coefficient);
                                    true
                                }
                                TermFactorExtractResult::None { mut original } => {
                                    // swap back
                                    mem::swap(&mut original, ast);
                                    false
                                }
                            }
                        });

                        assert!(!coeff.operands.is_empty());

                        (factor.clone(), coeff)
                    })
                    .next();

                if let Some((factor, coeff)) = first_non_0_common_factor {
                    new_terms.push(Binary::new(BinaryOp::Mul).with(coeff).with(factor).build());
                } else {
                    new_terms.push(terms.pop().unwrap());
                }
            }

            ast = Binary {
                operator: BinaryOp::Add,
                operands: new_terms,
            }
            .build();
        }

        ast
    }

    // calculate unary operations
    // example: replace 4! with 24
    fn unary_num_ops(mut ast: Ast, _: usize) -> Ast {
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

    /// calculate binary operations immediately calculable
    ///
    /// examples:
    ///
    /// replace | with
    /// :-:|:-:
    /// 1+a+2+3 | 6+a
    /// 0+a | a
    /// 1*a*2*3 | 6*a
    /// 1*a | a
    /// 1^a^2^3 | 1
    /// a^2^3 | a^8
    /// a^0 | 1
    /// a^1 | a
    fn binary_num_ops(mut ast: Ast, _: usize) -> Ast {
        if let Ast::Binary(Binary { operator, operands }) = ast {
            let operands = match operator {
                // power op has to be handled differently
                // because there the order of the operands matter
                BinaryOp::Pow => operands.into_iter().fold(vec![], |mut acc, ast| {
                    let x = (acc.last_mut(), ast);
                    match x {
                        (Some(Ast::Num(a)), Ast::Num(b)) => {
                            if let Ok(b) = b.try_into() {
                                *a = a.pow(b)
                            } else {
                                acc.push(Ast::Num(b))
                            }
                        }
                        (Some(ast @ Ast::Num(0)), _) => *ast = Ast::Num(0),
                        (Some(ast @ Ast::Num(1)), _) => *ast = Ast::Num(1),
                        (Some(ast), Ast::Num(0)) => *ast = Ast::Num(1),
                        (Some(_), Ast::Num(1)) => {}
                        (Some(_) | None, ast) => acc.push(ast),
                    }
                    acc
                }),
                // otherwise, all numbers are just collected
                other => {
                    let init = if other == BinaryOp::Add { 0 } else { 1 };
                    let mut result = init;
                    let mut operands: Vec<Ast> = operands
                        .into_iter()
                        .filter(|ast| match ast {
                            Ast::Num(n) => {
                                match operator {
                                    BinaryOp::Add => result += n,
                                    BinaryOp::Mul => result *= n,
                                    _ => unreachable!(),
                                };
                                false
                            }
                            _ => true,
                        })
                        .collect();

                    if result != init {
                        operands.push(Ast::Num(result));
                    }
                    operands
                }
            };

            ast = Binary { operator, operands }.into();
            // log::debug!("binary_num_ops: {ast:?}");
        }

        ast
    }
}

//

#[cfg(test)]
mod test {
    use super::Simplifier;
    use crate::ast::{
        binary::{Binary, BinaryOp},
        Ast,
    };

    pub fn ast_eq(lhs: &Ast, rhs: &Ast) {
        assert_eq!(lhs, rhs, "\nleft: {lhs}\nright: {rhs}")
    }

    #[test]
    pub fn test_de_paren() {
        let ast = Binary::new(BinaryOp::Mul)
            .with(Binary::new(BinaryOp::Mul).with(0).with(1))
            .with(Binary::new(BinaryOp::Add).with("a").with("b"))
            .with(3)
            .build();
        let lhs = Simplifier::de_paren(ast, 0);
        let rhs = Binary::new(BinaryOp::Mul)
            .with(0)
            .with(1)
            .with(Binary::new(BinaryOp::Add).with("a").with("b"))
            .with(3)
            .build();

        ast_eq(&lhs, &rhs);
    }

    #[test]
    pub fn test_combine_terms() {
        // y * x * 2 + x + x * 2 + 3
        // ==
        // (y * 2 + 3) * x + 3
        let ast = Binary::new(BinaryOp::Add)
            .with(Binary::new(BinaryOp::Mul).with("y").with("x").with(2))
            .with("x")
            .with(Binary::new(BinaryOp::Mul).with("x").with(2))
            .with(3)
            .build();
        let lhs = Simplifier::combine_terms(ast, 0);
        let rhs = Binary::new(BinaryOp::Add)
            .with(
                Binary::new(BinaryOp::Mul)
                    .with(
                        Binary::new(BinaryOp::Add)
                            .with(Binary::new(BinaryOp::Mul).with("y").with(2))
                            .with(3),
                    )
                    .with("x"),
            )
            .with(3)
            .build();

        ast_eq(&lhs, &rhs);
    }
}
