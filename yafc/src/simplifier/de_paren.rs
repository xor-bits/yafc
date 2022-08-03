use super::Simplifier;
use crate::ast::{
    binary::{Binary, BinaryOp},
    Ast,
};

//

impl Simplifier {
    // remove unnecessary parenthesis
    // example: replace (a+b)+c with a+b+c and so on
    pub fn de_paren(mut ast: Ast) -> Ast {
        // add ops
        if let Ast::Binary(Binary {
            operator: BinaryOp::Add,
            operands,
        }) = ast
        {
            let operands = operands
                .into_iter()
                .flat_map(move |ast| match ast {
                    Ast::Binary(Binary {
                        operator: BinaryOp::Add,
                        operands,
                    }) => operands,
                    ast => vec![ast],
                })
                .collect();

            ast = Binary {
                operator: BinaryOp::Add,
                operands,
            }
            .into();
        }

        // mul ops
        if let Ast::Binary(Binary {
            operator: BinaryOp::Mul,
            operands,
        }) = ast
        {
            let operands = operands
                .into_iter()
                .flat_map(move |ast| match ast {
                    Ast::Binary(Binary {
                        operator: BinaryOp::Mul,
                        operands,
                    }) => operands,
                    ast => vec![ast],
                })
                .collect();

            ast = Binary {
                operator: BinaryOp::Mul,
                operands,
            }
            .into();
        }

        // pow ops
        // a^b^c = a^b^c
        // (a^b)^c = a^(b*c)
        // (a^b^c)^(d^e)^(f^g) = a^(b^c*d^(e*f^g))
        if let Ast::Binary(Binary {
            operator: BinaryOp::Pow,
            operands,
        }) = ast
        {
            ast = operands.into_iter().rev().fold(Ast::Num(1), |last, ast| {
                if last != Ast::Num(1) {
                    match ast {
                        Ast::Binary(Binary {
                            operator: BinaryOp::Pow,
                            mut operands,
                        }) => {
                            let tower_first = operands.remove(0);
                            let tower = Binary {
                                operator: BinaryOp::Pow,
                                operands,
                            }
                            .build();

                            tower_first ^ (tower * last)
                        }
                        ast => ast ^ last,
                    }
                } else {
                    ast
                }
            });
        }

        ast
    }
}

//

#[cfg(test)]
mod tests {
    macro_rules! de_paren_s_assert_eq {
        ($lhs:expr, $rhs:expr) => {
            let lhs = crate::ast::Ast::parse($lhs)
                .unwrap()
                .map(32, crate::simplifier::Simplifier::de_paren);
            let rhs = crate::ast::Ast::parse($rhs).unwrap();
            crate::assert_eq_display!(lhs, rhs);
        };
    }

    #[test]
    pub fn test_de_paren() {
        de_paren_s_assert_eq!("(0 * 1) * (a + b) * 3", "0 * 1 * (a + b) * 3");

        de_paren_s_assert_eq!(
            "(0 * 1 * (a * b * (f * g))) * (a + b + c * d) * 3",
            "0 * 1 * a * b * f * g * (a + b + c * d) * 3"
        );
    }
}
