use crate::ast::{
    binary::{Binary, BinaryOp},
    unary::Unary,
    Ast,
};

//

impl Ast {
    pub fn structural_eq(&self, other: &Ast) -> bool {
        match (self, other) {
            // consts and vars
            (Ast::Num(a), Ast::Num(b)) => a == b,
            (Ast::Var(a), Ast::Var(b)) => a == b,

            // order of operations DO matter
            (
                Ast::Binary(Binary {
                    operator: a,
                    operands: c,
                }),
                Ast::Binary(Binary {
                    operator: b,
                    operands: d,
                }),
            ) if a == b && a == &BinaryOp::Pow => {
                c.iter().zip(d.iter()).all(|(c, d)| c.structural_eq(d))
            }

            // order of operations DON't matter
            (
                Ast::Binary(Binary {
                    operator: a,
                    operands: c,
                }),
                Ast::Binary(Binary {
                    operator: b,
                    operands: d,
                }),
            ) if a == b => c.iter().all(|c| d.iter().any(|d| c.structural_eq(d))),

            // simple unary op
            (
                Ast::Unary(Unary {
                    operator: a,
                    operand: c,
                }),
                Ast::Unary(Unary {
                    operator: b,
                    operand: d,
                }),
            ) if a == b => c.structural_eq(d),

            // any other wont eq
            _ => false,
        }
    }

    /* pub fn mathematical_eq(&self, other: &Ast) -> bool {
        todo!()
    } */
}

//

#[cfg(test)]
mod test {
    use crate::ast::{
        binary::{Binary, BinaryOp},
        unary::{Unary, UnaryOp},
    };

    #[test]
    pub fn structural_eq_bin() {
        let a = Binary::new(BinaryOp::Add).with(2).with("a").with(4).build();
        let b = Binary::new(BinaryOp::Add).with(4).with(2).with("a").build();

        assert!(a.structural_eq(&b));

        let a = Binary::new(BinaryOp::Add).with(2).with("a").with(4).build();
        let b = Binary::new(BinaryOp::Add).with(4).with(2).with("a").build();

        assert!(a.structural_eq(&b));
    }

    #[test]
    pub fn structural_eq_bin_0() {
        let a = Binary::new(BinaryOp::Mul).with(0).with("a").with(4).build();
        let b = Binary::new(BinaryOp::Mul).with(0).with(4).with("a").build();

        assert!(a.structural_eq(&b));

        let a = Binary::new(BinaryOp::Mul).with(0).with("b").with(4).build();
        let b = Binary::new(BinaryOp::Mul).with(0).with(4).with("a").build();

        assert!(!a.structural_eq(&b));
    }

    #[test]
    pub fn structural_eq_bin_pow() {
        let a = Binary::new(BinaryOp::Pow).with("a").with(5).with(4).build();
        let b = Binary::new(BinaryOp::Pow).with("a").with(5).with(4).build();

        assert!(a.structural_eq(&b));

        let a = Binary::new(BinaryOp::Pow).with("a").with(5).with(4).build();
        let b = Binary::new(BinaryOp::Pow).with("a").with(4).with(5).build();

        assert!(!a.structural_eq(&b));
    }

    #[test]
    pub fn structural_eq_bin_nest() {
        let a = Binary::new(BinaryOp::Mul)
            .with(Binary::new(BinaryOp::Add).with(2).with("h").with(0).build())
            .with("a")
            .with(4)
            .build();
        let b = Binary::new(BinaryOp::Mul)
            .with(4)
            .with("a")
            .with(Binary::new(BinaryOp::Add).with(0).with("h").with(2).build())
            .build();

        assert!(a.structural_eq(&b));

        let a = Binary::new(BinaryOp::Mul)
            .with(Binary::new(BinaryOp::Add).with(2).with("h").with(0).build())
            .with("a")
            .with(4)
            .build();
        let b = Binary::new(BinaryOp::Mul)
            .with(4)
            .with("a")
            .with(Binary::new(BinaryOp::Pow).with(0).with("h").with(2).build())
            .build();

        assert!(!a.structural_eq(&b));
    }

    #[test]
    pub fn structural_eq_un() {
        let a = Unary {
            operator: UnaryOp::Fac,
            operand: Box::new(Binary::new(BinaryOp::Pow).with(2).with(3).build()),
        }
        .build();
        let b = Unary {
            operator: UnaryOp::Fac,
            operand: Box::new(Binary::new(BinaryOp::Pow).with(2).with(3).build()),
        }
        .build();

        assert!(a.structural_eq(&b));
    }
}
