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
                            let first = operands.remove(0);
                            Binary::new(BinaryOp::Pow)
                                .with(first)
                                .with(
                                    Binary::new(BinaryOp::Mul)
                                        .with(Binary {
                                            operator: BinaryOp::Pow,
                                            operands,
                                        })
                                        .with(last),
                                )
                                .build()
                        }
                        ast => Binary::new(BinaryOp::Pow).with(ast).with(last).build(),
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
    use crate::{
        assert_eq_display,
        ast::binary::{Binary, BinaryOp},
        simplifier::Simplifier,
    };

    #[test]
    pub fn test_de_paren() {
        // lhs: (0*1)*(a+b)*3
        // rhs: 0*1*(a+b)*3

        let lhs = Binary::new(BinaryOp::Mul)
            .with(Binary::new(BinaryOp::Mul).with(0).with(1))
            .with(Binary::new(BinaryOp::Add).with("a").with("b"))
            .with(3)
            .build();
        let rhs = Binary::new(BinaryOp::Mul)
            .with(0)
            .with(1)
            .with(Binary::new(BinaryOp::Add).with("a").with("b"))
            .with(3)
            .build();

        assert_eq_display(&Simplifier::de_paren(lhs), &rhs);
    }

    #[test]
    pub fn test_de_paren_rec() {
        // lhs: (0*1*(a*b*(f*g)))*(a+b+c*d)*3
        // rhs: 0*1*a*b*f*g*(a+b+c*d)*3

        let lhs = Binary::new(BinaryOp::Mul)
            .with(
                Binary::new(BinaryOp::Mul).with(0).with(1).with(
                    Binary::new(BinaryOp::Mul)
                        .with("a")
                        .with("b")
                        .with(Binary::new(BinaryOp::Mul).with("f").with("g")),
                ),
            )
            .with(
                Binary::new(BinaryOp::Add)
                    .with("a")
                    .with("b")
                    .with(Binary::new(BinaryOp::Mul).with("c").with("d")),
            )
            .with(3)
            .build();
        let rhs = Binary::new(BinaryOp::Mul)
            .with(0)
            .with(1)
            .with("a")
            .with("b")
            .with("f")
            .with("g")
            .with(
                Binary::new(BinaryOp::Add)
                    .with("a")
                    .with("b")
                    .with(Binary::new(BinaryOp::Mul).with("c").with("d")),
            )
            .with(3)
            .build();

        assert_eq_display(&lhs.map(32, Simplifier::de_paren), &rhs);
    }
}
