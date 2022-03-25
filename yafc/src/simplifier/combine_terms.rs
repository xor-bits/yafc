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
