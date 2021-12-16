//! This module contains implementation of the fuzzy set and universal set.
//!
//! Fuzzy set is the basis of fuzzy logic.
//! Given as a part of the universal set with the membership function.

use crate::functions::MembershipFunction;
use std::{cell::RefCell, collections::HashMap, f32, fmt};

use ordered_float::OrderedFloat;

/// Fuzzy set itself.
pub struct Set {
    /// Name of the fuzzy set.
    pub name: String,
    /// Membership function.
    pub membership: Option<Box<MembershipFunction>>,
    /// Cache with calculated memberships.
    pub cache: RefCell<HashMap<OrderedFloat<f32>, f32>>,
}

impl Set {
    /// Constructs the new `Set` with given membership function.
    /// Don't create sets with this method. Use `UniversalSet`.
    pub fn new_with_mem(name: String, membership: Box<MembershipFunction>) -> Set {
        Set {
            name,
            membership: Some(membership),
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Constructs the new `Set` with given cache function.
    /// This cover the cases, where membership function is not available. E.g. result of an operation.
    pub fn new_with_domain(name: String, cache: RefCell<HashMap<OrderedFloat<f32>, f32>>) -> Set {
        Set {
            name,
            membership: None,
            cache,
        }
    }

    /// Returns the membership of item.
    /// If already computed -- returns from cache.
    /// Elsewise -- calculates from function, and if `value > 0` then caches it.
    pub fn check(&self, x: f32) -> f32 {
        let ordered = OrderedFloat(x);
        let func = self.membership.as_ref();
        let mut cache = self.cache.borrow_mut();

        let mem = *cache.entry(ordered).or_insert(match func {
            Some(f) => f(x),
            None => 0.0,
        });
     
        if mem <= 0.0 {
            cache.remove(&ordered);
        }
        mem
    }
}

impl fmt::Debug for Set {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for (k, v) in self.cache.borrow().iter() {
            s = s + &format!("k:{} v:{}\n", k, v);
        }

        write!(f, "Set {{ name: {}\ncache: {} }}", self.name, s)
    }
}

#[derive(Debug)]
/// Universal set for fuzzy sets.
pub struct UniversalSet {
    /// Name of the universal set.
    name: String,
    /// Domain.
    domain: Vec<f32>,
    /// Children fuzzy sets.
    pub sets: HashMap<String, Set>, // TODO
}

impl UniversalSet {
    /// Constructs the new empty universal set.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            domain: Vec::new(),
            sets: HashMap::new(),
        }
    }

    /// Sets the domain of the universal set.
    #[deprecated = "Use chainable method with_domain()"]
    pub fn set_domain(&mut self, domain: Vec<f32>) {
        self.domain = domain;
    }

    /// Chain-able method for overriding the `domain` of the universal set
    pub fn with_domain(mut self, domain: Vec<f32>) -> Self {
        self.domain = domain;
        self
    }

    /// Constructs the child fuzzy set with given membership.
    pub fn create_set(&mut self, name: &str, membership: Box<MembershipFunction>) {
        if !self.sets.contains_key(name) {
            let set = Set {
                name: name.to_string(),
                membership: Some(membership),
                cache: RefCell::new(HashMap::new()),
            };
            for i in &self.domain {
                set.check(*i);
            }
            self.sets.insert(name.into(), set);
        }
    }

    /// Chain-able method for adding new `Set`s.
    pub fn add_set(mut self, name: &str, membership: Box<MembershipFunction>) -> Self {
        self.create_set(name, membership);
        self
    }

    /// Computes memberships from all children fuzzy sets.
    pub fn memberships(&self, x: f32) -> HashMap<String, f32> {
        self.sets
            .iter()
            .map(|(name, set)| (name.clone(), set.check(x)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::functions::MembershipFactory;

    use super::*;

    #[test]
    fn test_universal_set() {
        let x_dest = UniversalSet::new("x_dest")
            .add_set("NB", MembershipFactory::triangular(-2.0, -2.0, -1.0))
            .add_set("NS", MembershipFactory::triangular(-2.0, -1.0, 0.0))
            .add_set("Z", MembershipFactory::triangular(-1.0, 0.0, 1.0))
            .add_set("PS", MembershipFactory::triangular(0.0, 1.0, 2.0))
            .add_set("PB", MembershipFactory::triangular(1.0, 2.0, 2.0));

        let get_membership_for = -1.5_f32;
        let actual = x_dest.memberships(get_membership_for);

        let expected = vec![
            ("NB".into(), 0.5),
            ("NS".into(), 0.5),
            ("Z".into(), 0.),
            ("PS".into(), 0.),
            ("PB".into(), 0.),
        ]
        .into_iter()
        .collect::<HashMap<String, f32>>();

        for (set_name, membership) in actual {
            match expected.get(&set_name) {
                // assert values only if expected values are set
                Some(expected_membership) => assert_relative_eq!(*expected_membership, membership),
                // otherwise expect `0`
                None => assert_relative_eq!(membership, 0.),
            }
        }
    }
}
