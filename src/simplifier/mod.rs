use std::collections::HashSet;

use crate::ast::{
    binary::{Binary, BinaryOp},
    unary::{Unary, UnaryOp},
    Ast,
};

//

pub struct Simplifier;

//

impl Simplifier {
    pub fn run(ast: Ast) -> Ast {
        Self::run_once(ast, 0)
    }

    // make simplifier recursive
    fn recurse(mut ast: Ast, depth: usize) -> Ast {
        if let Ast::Binary(Binary { operator, operands }) = ast {
            let operands = operands
                .into_iter()
                .map(|ast| Self::run_once(ast, depth + 1))
                .collect();
            ast = Binary { operator, operands }.into();
            // log::debug!("recurse: {ast} == {ast:?}");
        }
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
            operands: terms,
        }) = ast
        {
            let mut new_terms = vec![];

            // loop through terms
            let mut skipped = HashSet::new();
            for (i, term) in terms.iter().enumerate() {
                if skipped.contains(&i) {
                    continue;
                }

                let mut coeff = Binary::new(BinaryOp::Add);
                let factor = match term {
                    Ast::Binary(Binary {
                        operator: BinaryOp::Mul,
                        operands: factors,
                    }) => {
                        let mut factor = None;
                        // loop through the product in this term
                        for looking_for in factors {
                            // loop through all other terms
                            for (i, new_coeff) in
                                terms.iter().enumerate().skip(i).filter_map(|(i, term)| {
                                    Some((i, Self::term_factor_coeff(term, looking_for)?))
                                })
                            {
                                factor = Some(looking_for.clone());
                                skipped.insert(i);
                                coeff = coeff.with(new_coeff);
                            }

                            // discard those that only have itself to combine with
                            if coeff.operands.len() == 1 {
                                factor = None;
                                coeff.operands.clear();
                            }

                            if factor.is_some() {
                                break;
                            }
                        }

                        factor
                    }
                    pow @ Ast::Binary(Binary { .. }) => {
                        coeff = coeff.with(1);
                        Some(pow.clone())
                    }
                    looking_for => {
                        // loop through all other terms
                        for (i, new_coeff) in
                            terms.iter().enumerate().skip(i).filter_map(|(i, term)| {
                                Some((i, Self::term_factor_coeff(term, looking_for)?))
                            })
                        {
                            skipped.insert(i);
                            coeff = coeff.with(new_coeff);
                        }

                        Some(looking_for.clone())
                        // if !coeff.operands.is_empty() {
                        // } else {
                        //     None
                        // }
                    }
                };

                let coeff = Self::binary_num_ops(coeff.build(), depth);
                log::debug!("coeff {coeff} factor {factor:?}");

                match (factor, coeff) {
                    (Some(factor), Ast::Num(1)) => new_terms.push(factor),
                    (Some(factor), n) => {
                        new_terms.push(Binary::new(BinaryOp::Mul).with(n).with(factor).build())
                    }
                    _ => {}
                }
            }

            ast = Binary {
                operator: BinaryOp::Add,
                operands: new_terms,
            }
            .into();
            // log::debug!("combine_terms: {ast} == {ast:?}");
        }

        ast
    }

    fn term_factor_coeff(term: &Ast, looking_for: &Ast) -> Option<Ast> {
        match term {
            Ast::Binary(Binary {
                operands,
                operator: BinaryOp::Mul,
            }) => {
                let mut first = true;
                let (looking_for, coeff): (Vec<&Ast>, Vec<&Ast>) =
                    operands.iter().partition(|factor| {
                        if first && factor.structural_eq(looking_for) {
                            first = false;
                            true
                        } else {
                            false
                        }
                    });

                if looking_for.len() == 1 {
                    let operator = BinaryOp::Mul;
                    let operands = coeff.into_iter().cloned().collect();

                    Some(Binary { operands, operator }.into())
                } else if looking_for.is_empty() {
                    None
                } else {
                    unreachable!()
                }
            }
            Ast::Binary(Binary {
                operands,
                operator: BinaryOp::Pow,
            }) => match operands.first() {
                Some(first) if first.structural_eq(looking_for) => {
                    let operator = BinaryOp::Pow;
                    let operands = operands
                        .iter()
                        .cloned()
                        .enumerate()
                        .filter_map(|(i, ast)| {
                            // decrease the power by one
                            if i == 1 {
                                match Self::binary_num_ops(
                                    Binary::new(BinaryOp::Add).with(ast).with(-1).into(),
                                    0,
                                ) {
                                    // power of 1 cancels out
                                    Ast::Num(1) => None,
                                    ast => Some(ast),
                                }
                            } else {
                                Some(ast)
                            }
                        })
                        .collect();

                    Some(Binary { operands, operator }.into())
                }
                _ => None,
            },
            other if other.structural_eq(looking_for) => Some(Ast::Num(1)),
            _ => None,
        }
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

    // calculate binary operations immediately calculable
    // example: replace 1+a+2+3 with 6+a
    fn binary_num_ops(mut ast: Ast, _: usize) -> Ast {
        if let Ast::Binary(Binary { operator, operands }) = ast {
            let init = match operator {
                BinaryOp::Add => 0,
                BinaryOp::Mul => 1,
                BinaryOp::Pow => return Binary { operator, operands }.into(),
            };
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

            ast = Binary { operator, operands }.into();
            // log::debug!("binary_num_ops: {ast:?}");
        }

        ast
    }

    fn run_once(mut ast: Ast, depth: usize) -> Ast {
        if depth >= 32 {
            panic!("Recursion depth limit")
        }
        // log::debug!("simplify-init: {ast:?}");

        ast = Self::recurse(ast, depth);
        ast = Self::de_paren(ast, depth);
        ast = Self::combine_terms(ast, depth);
        ast = Self::unary_num_ops(ast, depth);
        ast = Self::binary_num_ops(ast, depth);

        // log::debug!("simplify: {ast} == {ast:?}");

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

    pub fn ast_eq(lhs: Ast, rhs: Ast) {
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

        ast_eq(lhs, rhs);
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

        ast_eq(lhs, rhs);
    }

    #[test]
    pub fn test_term_factor_coeff_first() {
        let term = Binary::new(BinaryOp::Mul)
            .with("x")
            .with("y")
            .with(4)
            .build();
        let coeff = Simplifier::term_factor_coeff(&term, &"y".into());

        assert_eq!(
            coeff,
            Some(Binary::new(BinaryOp::Mul).with("x").with(4).build())
        );
    }

    #[test]
    pub fn test_term_factor_coeff_second() {
        let term = Binary::new(BinaryOp::Mul)
            .with("x")
            .with("xx")
            .with("y")
            .with("yy")
            .with(4)
            .with(0)
            .build();
        let coeff = Simplifier::term_factor_coeff(&term, &"xx".into());

        assert_eq!(
            coeff,
            Some(
                Binary::new(BinaryOp::Mul)
                    .with("x")
                    .with("y")
                    .with("yy")
                    .with(4)
                    .with(0)
                    .build()
            )
        );
    }

    #[test]
    pub fn test_term_factor_coeff_not_found() {
        let term = Binary::new(BinaryOp::Mul)
            .with("x")
            .with("xx")
            .with("y")
            .with("yy")
            .with(4)
            .with(0)
            .build();
        let coeff = Simplifier::term_factor_coeff(&term, &"z".into());

        assert_eq!(coeff, None);
    }

    #[test]
    pub fn test_term_factor_coeff_single() {
        let term = "x".into();
        let coeff = Simplifier::term_factor_coeff(&term, &"x".into());

        assert_eq!(coeff, Some(1.into()));
    }

    #[test]
    pub fn test_term_factor_coeff_power() {
        let term = Binary::new(BinaryOp::Pow).with("x").with(2).build();
        let coeff = Simplifier::term_factor_coeff(&term, &"x".into());

        assert_eq!(coeff, Some("x".into()));
    }

    #[test]
    pub fn test_term_factor_coeff_complex_power() {
        let term = Binary::new(BinaryOp::Pow)
            .with("x")
            .with("y")
            .with("z")
            .build();
        let coeff = Simplifier::term_factor_coeff(&term, &"x".into());

        assert_eq!(
            coeff,
            Some(
                Binary::new(BinaryOp::Pow)
                    .with("x")
                    .with(Binary::new(BinaryOp::Add).with("y").with(-1))
                    .with("z")
                    .build()
            )
        );
    }
}
