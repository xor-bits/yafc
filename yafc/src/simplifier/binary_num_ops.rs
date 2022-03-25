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
    /// :-:|:-:
    /// 1+a+2+3 | 6+a
    /// 0+a | a
    /// 1*a*2*3 | 6*a
    /// 1*a | a
    /// 1^a^2^3 | 1
    /// a^2^3 | a^8
    /// a^0 | 1
    /// a^1 | a
    pub fn binary_num_ops(mut ast: Ast) -> Ast {
        if let Ast::Binary(Binary { operator, operands }) = ast {
            ast = match operator {
                // power op has to be handled differently
                // because there the order of the operands matter
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

                    Binary { operator, operands }.build()
                }
            };

            // log::debug!("binary_num_ops: {ast:?}");
        }

        ast
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_eq_display,
        ast::binary::{Binary, BinaryOp},
        build_ast,
        simplifier::Simplifier,
    };

    #[test]
    fn test_binary_num_ops_examples() {
        let lhs = build_ast!(+ 1 "a" 2 3);
        let rhs = build_ast!(+ 6 "a");
        assert_eq_display!(&Simplifier::binary_num_ops(lhs), &rhs);

        let lhs = build_ast!(+ 0 "a");
        let rhs = build_ast!(+ "a");
        assert_eq_display!(Simplifier::binary_num_ops(lhs), rhs);

        let lhs = build_ast!(* 1 "a" 2 3);
        let rhs = build_ast!(* 6 "a");
        assert_eq_display!(Simplifier::binary_num_ops(lhs), rhs);

        let lhs = build_ast!(* 1 "a");
        let rhs = build_ast!(*"a");
        assert_eq_display!(Simplifier::binary_num_ops(lhs), rhs);

        let lhs = build_ast!(^ 1 "a" 2 3);
        let rhs = build_ast!(^ 1);
        assert_eq_display!(Simplifier::binary_num_ops(lhs), rhs);

        let lhs = build_ast!(^ "a" 2 3);
        let rhs = build_ast!(^ "a" 8);
        assert_eq_display!(Simplifier::binary_num_ops(lhs), rhs);

        let lhs = build_ast!(^ "a" 0);
        let rhs = build_ast!(^ 1);
        assert_eq_display!(Simplifier::binary_num_ops(lhs), rhs);

        let lhs = build_ast!(^ "a" 1);
        let rhs = build_ast!(^ "a");
        assert_eq_display!(Simplifier::binary_num_ops(lhs), rhs);
    }
}
