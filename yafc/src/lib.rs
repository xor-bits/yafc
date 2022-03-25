#![feature(box_patterns)]
#![feature(drain_filter)]
#![feature(if_let_guard)]

//

pub mod ast;
pub mod eq;
pub mod simplifier;

//

pub fn assert_eq_display<T: PartialEq + std::fmt::Debug + std::fmt::Display>(lhs: T, rhs: T) {
    assert_eq!(lhs, rhs, "\n left: {lhs}\nright: {rhs}")
}
