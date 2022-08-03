use super::{factorize::term_factors, Simplifier};
use crate::{
    ast::{
        binary::{Binary, BinaryOp},
        Ast,
    },
    simplifier::factorize::{term_factor_extract, TermFactorExtractResult},
};
use std::mem;

//

impl Simplifier {
    // combine terms
    // example: x + x = 2 * x
    pub fn combine_terms(mut ast: Ast) -> Ast {
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
}

//

#[cfg(test)]
mod test {
    macro_rules! combine_terms_assert_eq {
        ($lhs:expr, $rhs:expr) => {
            let lhs = crate::ast::Ast::parse($lhs)
                .unwrap()
                .map(32, crate::simplifier::Simplifier::combine_terms);
            let rhs = crate::ast::Ast::parse($rhs).unwrap();
            crate::assert_eq_display!(lhs, rhs);
        };
    }

    #[test]
    pub fn test_combine_terms() {
        combine_terms_assert_eq!("y * x * 2 + x + x * 2 + 3", "(y * 2 + 1 + 2) * x + 1 * 3");
    }
}
