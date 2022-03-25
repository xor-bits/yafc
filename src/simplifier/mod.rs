use self::factorize::{term_factor_extract, term_factors, TermFactorExtractResult};
use crate::ast::{
    binary::{Binary, BinaryOp},
    unary::{Unary, UnaryOp},
    Ast,
};
use std::mem;

//

mod de_paren;
mod factorize;

//

pub struct Simplifier;

//

impl Simplifier {
    pub fn run(mut ast: Ast) -> Ast {
        ast = ast.map(32, Self::de_paren);
        ast = ast.map(32, Self::combine_terms);
        ast = ast.map(32, Self::unary_num_ops);
        ast = ast.map(32, Self::binary_num_ops);

        ast
    }

    // combine terms
    // example: x + x = 2 * x
    fn combine_terms(mut ast: Ast) -> Ast {
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

            // reverse new_terms to make it be more consistent with the original TODO: VecDeque
            new_terms.reverse();

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
    fn unary_num_ops(mut ast: Ast) -> Ast {
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
    fn binary_num_ops(mut ast: Ast) -> Ast {
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
                                if other == BinaryOp::Add {
                                    result += n
                                } else {
                                    result *= n
                                }
                                false
                            }
                            _ => true,
                        })
                        .collect();

                    if result != init {
                        // TODO: VecDeque
                        operands.insert(0, Ast::Num(result));
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
    use crate::{
        assert_eq_display,
        ast::binary::{Binary, BinaryOp},
    };

    #[test]
    pub fn test_combine_terms() {
        // y * x * 2 + x + x * 2 + 3
        // ==
        // 3 + (3 + 2 * y) * x
        let ast = Binary::new(BinaryOp::Add)
            .with(Binary::new(BinaryOp::Mul).with("y").with("x").with(2))
            .with("x")
            .with(Binary::new(BinaryOp::Mul).with("x").with(2))
            .with(3)
            .build();
        let lhs = Simplifier::combine_terms(ast).map(32, Simplifier::binary_num_ops);
        let rhs = Binary::new(BinaryOp::Add)
            .with(3)
            .with(
                Binary::new(BinaryOp::Mul)
                    .with(
                        Binary::new(BinaryOp::Add)
                            .with(3)
                            .with(Binary::new(BinaryOp::Mul).with(2).with("y")),
                    )
                    .with("x"),
            )
            .build();

        assert_eq_display(&lhs, &rhs);
    }
}
