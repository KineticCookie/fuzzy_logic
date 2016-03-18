extern crate ordered_float;

use std::fmt;
use std::f32;
use std::collections::HashMap;
use membership::MembershipFunction;

use self::ordered_float::OrderedFloat;

pub struct Set {
    pub name: String,
    pub membership: Option<Box<MembershipFunction>>,
    pub cache: HashMap<OrderedFloat<f32>, f32>,
}

impl Set {
    pub fn new_with_mem(name: String, membership: Box<MembershipFunction>) -> Set {
        Set {
            name: name,
            membership: Some(membership),
            cache: HashMap::new(),
        }
    }
    pub fn new_with_domain(name: String, cache: HashMap<OrderedFloat<f32>, f32>) -> Set {
        Set {
            name: name,
            membership: None,
            cache: cache,
        }
    }

    pub fn new_empty() -> Set {
        Set::new_with_domain("Empty".to_string(), HashMap::new())
    }

    pub fn check(&mut self, x: f32) -> f32 {
        let ordered = OrderedFloat(x);
        let func = self.membership.as_ref();
        let mut mem = 0.0;
        {
            let value = self.cache.entry(ordered).or_insert(match func {
                Some(f) => f(x),
                None => unreachable!(),
            });
            mem = *value;
        }
        if mem <= 0.0 {
            self.cache.remove(&ordered);
        }
        mem
    }
}

impl fmt::Debug for Set {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for (k, v) in &self.cache {
            s = s + &format!("k:{} v:{}\n", k, v);
        }
        write!(f, "Set {{ name: {}\ncache: {} }}", self.name, s)
    }
}

#[derive(Debug)]
pub struct UniversalSet {
    name: String,
    domain: Vec<f32>,
    sets: HashMap<String, Set>,
}

impl UniversalSet {
    pub fn new(name: String) -> UniversalSet {
        UniversalSet {
            name: name,
            domain: Vec::new(),
            sets: HashMap::new(),
        }
    }

    pub fn set_domain(&mut self, domain: Vec<f32>) {
        self.domain = domain;
    }

    pub fn create_set(&mut self, name: String, membership: Box<MembershipFunction>) {
        self.sets.entry(name.clone()).or_insert(Set {
            name: name,
            membership: Some(membership),
            cache: HashMap::new(),
        });
    }

    pub fn memberships(&mut self, x: f32) -> HashMap<String, f32> {
        self.sets
            .iter_mut()
            .map(|(name, set)| (name.clone(), set.check(x)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // TODO make test
    fn initial() {
        unimplemented!();
    }
}