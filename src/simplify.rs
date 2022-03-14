use crate::ast::{AstNode, BinOp, Number, UnOp};

//

struct Simplifier {}

impl Simplifier {
    fn matcher(pattern: &'static str) -> &'static str {
        match pattern {
            "a+0" => "a",
            "0+a" => "a",
            "a*0" => "0",
            "0*a" => "0",
            "a^0" => "1",
            "0^a" => "0",

            "a+a" => "2*a",
            "a*a" => "a^2",
            "a*b+a*c" => "a*(b+c)",
            "a+a*b" => "a*(1+b)",

            other => other,
        }
    }

    fn run_once(ast: AstNode) -> AstNode {
        const ZERO_I: AstNode = AstNode::Number(Number::Integer(0));
        const ONE_I: AstNode = AstNode::Number(Number::Integer(1));

        match ast {
            // 0.0 == 0
            AstNode::Number(Number::Decimal(a)) if a.abs() <= f64::EPSILON => ZERO_I,

            // a + 0 = a
            AstNode::BinExpr {
                operator: BinOp::Add,
                operands: box (AstNode::Number(Number::Decimal(a)), b),
            } if a.abs() <= f64::EPSILON => b,
            AstNode::BinExpr {
                operator: BinOp::Add,
                operands: box (a, AstNode::Number(Number::Decimal(b))),
            } if b.abs() <= f64::EPSILON => a,
            AstNode::BinExpr {
                operator: BinOp::Add,
                operands: box (AstNode::Number(Number::Integer(0)), b),
            } => b,
            AstNode::BinExpr {
                operator: BinOp::Add,
                operands: box (a, AstNode::Number(Number::Integer(0))),
            } => a,

            // a * 0 = 0
            AstNode::BinExpr {
                operator: BinOp::Mul,
                operands: box (AstNode::Number(Number::Decimal(a)), _),
            } if a.abs() <= f64::EPSILON => ZERO_I,
            AstNode::BinExpr {
                operator: BinOp::Mul,
                operands: box (_, AstNode::Number(Number::Decimal(b))),
            } if b.abs() <= f64::EPSILON => ZERO_I,
            AstNode::BinExpr {
                operator: BinOp::Mul,
                operands: box (AstNode::Number(Number::Integer(0)), _),
            } => ZERO_I,
            AstNode::BinExpr {
                operator: BinOp::Mul,
                operands: box (_, AstNode::Number(Number::Integer(0))),
            } => ZERO_I,

            // a^0 = 1 (where a can also be 0, so 0^0=1)
            AstNode::BinExpr {
                operator: BinOp::Pow,
                operands: box (_, AstNode::Number(Number::Decimal(b))),
            } if b.abs() <= f64::EPSILON => ONE_I,
            AstNode::BinExpr {
                operator: BinOp::Pow,
                operands: box (_, AstNode::Number(Number::Integer(0))),
            } => ONE_I,

            // 0^b = 0 (where b cannot be 0, so 0^0=1)
            AstNode::BinExpr {
                operator: BinOp::Pow,
                operands: box (AstNode::Number(Number::Decimal(a)), _),
            } if a.abs() <= f64::EPSILON => ZERO_I,
            AstNode::BinExpr {
                operator: BinOp::Pow,
                operands: box (AstNode::Number(Number::Integer(0)), _),
            } => ZERO_I,

            other => other,
        }
    }
}

// 0 + a = a

//

impl AstNode {
    pub fn eval(mut self) -> Result<AstNode, &'static str> {
        self = Simplifier::run_once(self);

        Ok(match self {
            AstNode::BinExpr { operator, operands } => {
                let (lhs, rhs) = *operands;

                let (operator, rhs) = match operator {
                    BinOp::Sub => (
                        BinOp::Add,
                        AstNode::BinExpr {
                            operator: BinOp::Mul,
                            operands: Box::new((rhs, AstNode::Number(Number::Integer(-1)))),
                        },
                    ),
                    BinOp::Div => (
                        BinOp::Mul,
                        AstNode::BinExpr {
                            operator: BinOp::Pow,
                            operands: Box::new((rhs, AstNode::Number(Number::Integer(-1)))),
                        },
                    ),
                    other => (other, rhs),
                };

                enum T {
                    A((i64, i64)),
                    B((f64, f64)),
                    C((AstNode, AstNode)),
                }
                let sides = match (lhs.eval()?, rhs.eval()?) {
                    (
                        AstNode::Number(Number::Integer(lhs)),
                        AstNode::Number(Number::Integer(rhs)),
                    ) => T::A((lhs, rhs)),
                    (
                        AstNode::Number(Number::Decimal(lhs)),
                        AstNode::Number(Number::Integer(rhs)),
                    ) => T::B((lhs, rhs as f64)),
                    (
                        AstNode::Number(Number::Integer(lhs)),
                        AstNode::Number(Number::Decimal(rhs)),
                    ) => T::B((lhs as f64, rhs)),
                    (
                        AstNode::Number(Number::Decimal(lhs)),
                        AstNode::Number(Number::Decimal(rhs)),
                    ) => T::B((lhs, rhs as f64)),
                    operands => T::C(operands),
                };

                match (sides, operator) {
                    (T::A((lhs, rhs)), BinOp::Add) => AstNode::Number(Number::Integer(lhs + rhs)),
                    (T::B((lhs, rhs)), BinOp::Add) => AstNode::Number(Number::Decimal(lhs + rhs)),

                    (T::A((lhs, rhs)), BinOp::Sub) => AstNode::Number(Number::Integer(lhs - rhs)),
                    (T::B((lhs, rhs)), BinOp::Sub) => AstNode::Number(Number::Decimal(lhs - rhs)),

                    (T::A((lhs, rhs)), BinOp::Mul) => AstNode::Number(Number::Integer(lhs * rhs)),
                    (T::B((lhs, rhs)), BinOp::Mul) => AstNode::Number(Number::Decimal(lhs * rhs)),

                    (T::A((lhs, rhs)), BinOp::Div) => {
                        if let Some(res) = lhs.checked_div(rhs) {
                            AstNode::Number(Number::Integer(res))
                        } else {
                            return Err("division by zero");
                        }
                    }
                    (T::B((lhs, rhs)), BinOp::Div) => {
                        if rhs != 0.0 {
                            AstNode::Number(Number::Decimal(lhs / rhs))
                        } else {
                            return Err("division by zero");
                        }
                    }

                    (T::A((lhs, rhs)), BinOp::Pow) => {
                        if let Ok(rhs) = rhs.try_into() {
                            AstNode::Number(Number::Integer(lhs.pow(rhs)))
                        } else {
                            AstNode::Number(Number::Decimal((lhs as f64).powi(rhs as _)))
                        }
                    }
                    (T::B((lhs, rhs)), BinOp::Pow) => {
                        AstNode::Number(Number::Decimal(lhs.powf(rhs)))
                    }

                    (T::C(operands), _) => AstNode::BinExpr {
                        operator,
                        operands: Box::new(operands),
                    },
                }
            }
            AstNode::UnExpr { operator, operand } => match (operator, operand.eval()?) {
                (UnOp::Neg, AstNode::Number(Number::Decimal(v))) => {
                    AstNode::Number(Number::Decimal(-v))
                }
                (UnOp::Neg, AstNode::Number(Number::Integer(v))) => {
                    AstNode::Number(Number::Integer(-v))
                }
                (operator, operand) => AstNode::UnExpr {
                    operand: Box::new(operand),
                    operator,
                },
            },
            other => other,
        })
    }
}
