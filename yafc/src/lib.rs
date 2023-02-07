pub mod ast;
pub mod simplifier;

//

#[macro_export]
macro_rules! assert_eq_display {
    ($lhs:expr, $rhs:expr) => {
        let lhs = $lhs;
        let rhs = $rhs;
        assert_eq!(lhs, rhs, "\n left: {lhs}\nright: {rhs}")
    };
}
