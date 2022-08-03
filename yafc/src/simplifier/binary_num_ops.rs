use super::Simplifier;
use crate::{
    ast::{
        binary::{Binary, BinaryOp},
        Ast,
    },
    build_ast,
};

//

impl Simplifier {
    /// calculate binary operations immediately calculable
    ///
    /// examples:
    ///
    /// replace | with
    ///      :-:|:-:
    /// 1+a+2+3 | 6+a
    ///     0+a | a
    /// 1*a*2*3 | 6*a
    ///     1*a | a
    /// 1^a^2^3 | 1
    ///   a^2^3 | a^8
    ///     a^0 | 1
    ///     a^1 | a
    pub fn binary_num_ops(mut ast: Ast) -> Ast {
        if let Ast::Binary(Binary { operator, operands }) = ast {
            ast = match operator {
                // power op has to be handled differently
                // because here the order of _operands_ matter
                BinaryOp::Pow => {
                    operands
                        .into_iter()
                        .rev()
                        .fold(Ast::Num(1), |acc, ast| match (acc, ast) {
                            (Ast::Num(a), Ast::Num(b)) => {
                                if let Ok(c) = a.try_into() {
                                    Ast::Num(b.pow(c))
                                } else {
                                    build_ast!(^ b a)
                                }
                            }
                            (Ast::Num(0), _) => Ast::Num(1),
                            (Ast::Num(1), ast) => ast,
                            (_, Ast::Num(0)) => Ast::Num(0),
                            (_, Ast::Num(1)) => Ast::Num(1),
                            (rhs, lhs) => build_ast!(^ lhs rhs),
                        })
                }
                // otherwise, all numbers are just collected
                BinaryOp::Mul => {
                    let multiplier: i64 = Self::collect_numbers(&operands).product();
                    let operands = Self::collect_non_numbers(operands);

                    if multiplier == 0 {
                        return Ast::Num(0);
                    }

                    Binary {
                        operator,
                        operands: (multiplier != 1)
                            .then_some(Ast::Num(multiplier))
                            .into_iter()
                            .chain(operands)
                            .collect(),
                    }
                    .build()
                }
                BinaryOp::Add => {
                    let sum: i64 = Self::collect_numbers(&operands).sum();
                    let operands = Self::collect_non_numbers(operands);

                    Binary {
                        operator,
                        operands: (sum != 0)
                            .then_some(Ast::Num(sum))
                            .into_iter()
                            .chain(operands)
                            .collect(),
                    }
                    .build()
                }
            };

            // log::debug!("binary_num_ops: {ast:?}");
        }

        ast
    }

    fn collect_numbers(operands: &[Ast]) -> impl Iterator<Item = i64> + '_ {
        operands.iter().filter_map(|ast| match ast {
            Ast::Num(n) => Some(*n),
            _ => None,
        })
    }

    fn collect_non_numbers(operands: Vec<Ast>) -> impl Iterator<Item = Ast> {
        operands.into_iter().filter_map(|ast| match ast {
            Ast::Num(..) => None,
            ast => Some(ast),
        })
    }
}

#[cfg(test)]
mod tests {
    macro_rules! binary_num_ops_assert_eq {
        ($lhs:expr, $rhs:expr) => {
            let lhs = crate::ast::Ast::parse($lhs)
                .unwrap()
                .map(32, crate::simplifier::Simplifier::binary_num_ops);
            let rhs = crate::ast::Ast::parse($rhs).unwrap();
            crate::assert_eq_display!(lhs, rhs);
        };
    }

    #[test]
    fn test_binary_num_ops_examples() {
        binary_num_ops_assert_eq!("1 + a + 2 + 3", "6 + a");
        binary_num_ops_assert_eq!("0 + a", "a");

        binary_num_ops_assert_eq!("1 * a * 2 * 3", "6 * a");
        binary_num_ops_assert_eq!("1 * a", "a");
        binary_num_ops_assert_eq!("0 * a", "0");

        binary_num_ops_assert_eq!("1 ^ a ^ 2 ^ 3", "1");
        binary_num_ops_assert_eq!("a ^ 2 ^ 3", "a ^ 8");
        binary_num_ops_assert_eq!("a ^ 0", "1");
        binary_num_ops_assert_eq!("a ^ 1", "a");
    }
}
