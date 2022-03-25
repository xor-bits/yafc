use crate::ast::{
    binary::{Binary, BinaryOp},
    Ast,
};

use super::Simplifier;

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
