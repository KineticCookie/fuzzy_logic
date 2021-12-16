#![warn(missing_docs)]
//! The implementation of the fuzzy logic inference.
//!
//! Provides structs, which defines fuzzy sets and rules.
//! Rules are constructed with logical operations but implementation of operations is chosen by user.
//!
//! User is available to implement his own functions and operations.
pub mod functions;
pub mod inference;
pub mod ops;
pub mod rules;
pub mod set;

#[cfg(test)]
mod test {
    use crate::functions::MembershipFactory;
    #[test]
    fn fuzzy_logic() {
        let mem = MembershipFactory::triangular(-15.0, -15.0, 22.0);
        mem(-15.0); // -> 1.0
    }
}
