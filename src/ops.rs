//! This module contains implementation of the fuzzy logic operations.
//!
//! Fuzzy set operations and fuzzy logic operations are defined here.
//!
//! User can implement his own operations by implementing `LogicOps` or `SetOps` traits.
use crate::set::Set;
use std::{cell::RefCell, collections::HashMap, fmt::Debug};

/// Abstraction over set operations. Doesn't contain default implementation.
pub trait SetOps: Debug {
    /// Union of fuzzy sets.
    fn union(&self, left: &mut Set, right: &mut Set) -> Set;
    /// Intersection of fuzzy sets.
    fn intersect(&self, left: &mut Set, right: &mut Set) -> Set;
}

/// Implementation of commonly used minimax set operations.
#[derive(Debug)]
pub struct MinMaxOps;

impl SetOps for MinMaxOps {
    /// Union of fuzzy sets.
    ///
    /// Values with highest memberships are copied to the result set.
    fn union(&self, left: &mut Set, right: &mut Set) -> Set {
        let mut result = HashMap::new();
        for (k, v) in left.cache.borrow().iter() {
            let right_mem = right.check(k.into_inner());
            result.insert(*k, v.max(right_mem));
        }
        for (k, v) in right.cache.borrow().iter() {
            if result.contains_key(k) {
                continue;
            }
            let left_mem = left.check(k.into_inner());
            result.insert(*k, v.max(left_mem));
        }
        Set::new_with_domain(
            format!("{} UNION {}", left.name, right.name),
            RefCell::new(result),
        )
    }

    /// Intersection of fuzzy sets.
    ///
    /// Values with lowest memberships are copied to the result set.
    fn intersect(&self, left: &mut Set, right: &mut Set) -> Set {
        let mut result = HashMap::new();
        for (k, v) in left.cache.borrow().iter() {
            let right_mem = right.check(k.into_inner());
            if right_mem > 0.0 {
                result.insert(*k, v.min(right_mem));
            }
        }
        Set::new_with_domain(
            format!("{} INTERSECT {}", left.name, right.name),
            RefCell::new(result),
        )
    }
}

/// Abstraction over fuzzy logic operations. Doesn't contain default implementation.
pub trait LogicOps: Debug {
    /// Fuzzy logic AND operation.
    fn and(&self, left: f32, right: f32) -> f32;
    /// Fuzzy logic OR operation.
    fn or(&self, left: f32, right: f32) -> f32;
    /// Fuzzy logic NOT operation.
    fn not(&self, value: f32) -> f32;
}

/// Implementation of commonly used Zadeh fuzzy logic operations.
#[derive(Debug)]
pub struct ZadehOps;

impl LogicOps for ZadehOps {
    /// Fuzzy logic AND operation.
    ///
    /// Returns minimum of arguments.
    ///
    /// # Examples
    /// The classical AND operator's truth table:
    ///
    /// ```rust
    /// use fuzzy_logic::ops::{LogicOps, ZadehOps};
    ///
    /// let ops = ZadehOps{};
    ///
    /// ops.and(0.0, 0.0); //-> 0.0
    /// ops.and(0.0, 1.0); //-> 0.0
    /// ops.and(1.0, 0.0); //-> 0.0
    /// ops.and(1.0, 1.0); //-> 1.0
    /// ```
    fn and(&self, left: f32, right: f32) -> f32 {
        left.min(right)
    }

    /// Fuzzy logic AND operation.
    ///
    /// Returns maximum of arguments.
    ///
    /// # Examples
    /// The classical OR operator's truth table:
    ///
    /// ```rust
    /// use fuzzy_logic::ops::{LogicOps, ZadehOps};
    ///
    /// let ops = ZadehOps{};
    ///
    /// ops.or(0.0, 0.0); //-> 0.0
    /// ops.or(0.0, 1.0); //-> 1.0
    /// ops.or(1.0, 0.0); //-> 1.0
    /// ops.or(1.0, 1.0); //-> 1.0
    /// ```
    fn or(&self, left: f32, right: f32) -> f32 {
        left.max(right)
    }

    /// Fuzzy logic AND operation.
    ///
    /// Returns inversed logical value.
    /// # Examples
    /// The classical NOT operator's truth table:
    ///
    /// ```rust
    /// use fuzzy_logic::ops::{LogicOps, ZadehOps};
    ///
    /// let ops = ZadehOps{};
    ///
    /// ops.not(0.0); //-> 1.0
    /// ops.not(1.0); //-> 0.0
    /// ```
    fn not(&self, value: f32) -> f32 {
        1.0 - value
    }
}
