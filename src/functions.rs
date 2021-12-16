//! This module defines types and structures for fuzzy logic functions.
//!
//! Module contains implementation of membership functions and defuzzification functions.
//! Also contains factory methods to create most used functions.

use crate::set::Set;

/// Used to calculate the membership of the given item.
/// All membership functions must be this type.
pub type MembershipFunction = dyn Fn(f32) -> f32;

/// Used to defuzzificate the fuzzy logic inference result.
/// All defuzzification functions must be this type.
pub type DefuzzFunc = dyn Fn(&Set) -> f32;

/// Defines methods to create most used membership functions.
///
/// # Examples
/// Create triangular function:
///
/// ```rust
/// use fuzzy_logic::functions::MembershipFactory;
///
/// let mem = MembershipFactory::triangular(-15.0, -15.0, 22.0);
/// mem(-15.0); // -> 1.0
/// ```
pub struct MembershipFactory;

impl MembershipFactory {
    /// Creates triangular function.
    pub fn triangular(a: f32, b: f32, c: f32) -> Box<MembershipFunction> {
        Box::new(move |x: f32| {
            if a <= x && x <= b {
                1.0 - (b - x) / (b - a)
            } else if b <= x && x <= c {
                1.0 - (x - b) / (c - b)
            } else {
                0.0
            }
        })
    }

    /// Creates trapezoidal function.
    pub fn trapezoidal(a: f32, b: f32, c: f32, d: f32) -> Box<MembershipFunction> {
        Box::new(move |x: f32| {
            if x < a {
                0.0
            } else if x <= b {
                (x - a) / (b - a)
            } else if x <= c {
                1.0
            } else if x <= d {
                (d - x) / (d - c)
            } else {
                0.0
            }
        })
    }

    /// Creates sigmoidal function.
    pub fn sigmoidal(a: f32, c: f32) -> Box<MembershipFunction> {
        Box::new(move |x: f32| 1.0 / (1.0 + (-1.0 * a * (x - c)).exp()))
    }

    /// Creates gaussian function.
    pub fn gaussian(a: f32, b: f32, c: f32) -> Box<MembershipFunction> {
        Box::new(move |x: f32| a * (-1.0 * ((x - b).powi(2) / (2.0 * c.powi(2)))).exp())
    }

    /// Creates a singleton function
    // If value == x returns 1.0 (part of the set)
    // If value != x returns 0.0 (not part of the set)
    pub fn singleton(value: f32) -> Box<MembershipFunction> {
        Box::new(move |x: f32| if value == x { 1.0 } else { 0.0 })
    }
}

/// Defines methods to create most used defuzzification functions.
///
/// # Examples
/// Create function which calculates center of mass:
///
/// ```rust
/// use fuzzy_logic::{
///     functions::{DefuzzFactory, MembershipFactory},
///     set::Set
/// };
///
/// let mem = MembershipFactory::triangular(-15.0, -15.0, 22.0);
/// let df = DefuzzFactory::center_of_mass();
/// let set = Set::new_with_mem("Test".to_string(), mem);
/// df(&set);
/// ```
pub struct DefuzzFactory;

impl DefuzzFactory {
    /// Creates function which calculates center of mass.
    pub fn center_of_mass() -> Box<DefuzzFunc> {
        Box::new(|s: &Set| {
            let sum = s.cache.borrow().iter().fold(0.0, |acc, (_, &v)| acc + v);
            let prod_sum = s
                .cache
                .borrow()
                .iter()
                .fold(0.0, |acc, (&k, &v)| acc + k.into_inner() * v);
            prod_sum / sum
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f32;

    #[test]
    fn sigmoidal() {
        let steepness = 2.0;
        for i in -100..100 {
            let midpoint = i as f32;
            let f = MembershipFactory::sigmoidal(steepness, midpoint);
            let diff = (0.5 - f(midpoint)).abs();
            assert!(diff <= f32::EPSILON);
        }
    }
}
