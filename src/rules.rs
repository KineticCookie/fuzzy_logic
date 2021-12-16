//! This module contains implementation of the rules evaluation.
//!
//! Fuzzy set operations and fuzzy logic operations are defined here.
//!
//! User can implement his own operations by implementing `LogicOps` or `SetOps` traits.

use crate::{inference::InferenceContext, set::Set};

use std::{cell::RefCell, collections::HashMap, fmt};

/// Abstraction over rule's expression.
pub trait Expression: fmt::Debug {
    /// Evaluates the expression with given `InferenceContext`.
    fn eval(&self, context: &InferenceContext) -> f32;
    /// Return the string representation of the expression.
    fn to_string(&self) -> String;
}

/// 'Is' expression calculates membership of the given variable.
#[derive(Debug, Clone)]
pub struct Is {
    /// Variable in which membership we're interested.
    variable: String,
    /// Where to calculate the membership.
    set: String,
}

impl Is {
    /// Constructs `Is` expression.
    pub fn new(variable: &str, set: &str) -> Is {
        Is {
            variable: variable.into(),
            set: set.into(),
        }
    }
}

impl Expression for Is {
    /// Returns membership of given value.
    fn eval(&self, context: &InferenceContext) -> f32 {
        // TODO: replace with `.get()`
        let value = context.values[&self.variable];

        let universe = context
            .universes
            .get(&self.variable)
            .expect(&format!("{} does not exists", &self.variable));
        let set = universe
            .sets
            .get(&self.set)
            .expect(&format!("{} does not exists", &self.set));
        set.check(value)
    }
    /// String representation of the current `Is` expression.
    fn to_string(&self) -> String {
        format!("(is {} {})", self.variable, self.set)
    }
}

/// 'And' expression calculates AND logical operation with given implementation.
#[derive(Debug, Clone)]
pub struct And<L, R>
where
    L: Expression,
    R: Expression,
{
    /// Left operand.
    left: L,
    /// Right operand.
    right: R,
}

impl<L: Expression, R: Expression> And<L, R> {
    /// Constructs `And` expression.
    pub fn new(left: L, right: R) -> And<L, R> {
        And { left, right }
    }

    /// Creates a `Box<Self>` for convenience when creating [`Rule`]s
    pub fn boxed(self) -> Box<And<L, R>> {
        Box::new(self)
    }
}

impl<L: Expression, R: Expression> Expression for And<L, R> {
    /// Gets 'and' implementation from `context` and returns its value.
    fn eval(&self, context: &InferenceContext) -> f32 {
        let left_result = self.left.eval(context);
        let right_result = self.right.eval(context);
        (*context.options.logic_ops).and(left_result, right_result)
    }
    /// String representation of the current `And` expression.
    fn to_string(&self) -> String {
        format!("(and {} {})", self.left.to_string(), self.right.to_string())
    }
}

/// 'Or' expression calculates OR logical operation with given implementation.
#[derive(Debug, Clone)]
pub struct Or<L, R>
where
    L: Expression,
    R: Expression,
{
    /// Left operand.
    left: L,
    /// Right operand.
    right: R,
}

impl<L: Expression, R: Expression> Or<L, R> {
    /// Constructs `Or` expression.
    pub fn new(left: L, right: R) -> Or<L, R> {
        Or { left, right }
    }
}

impl<L: Expression, R: Expression> Expression for Or<L, R> {
    /// Gets 'or' implementation from `context` and returns its value.
    fn eval(&self, context: &InferenceContext) -> f32 {
        let left_result = self.left.eval(context);
        let right_result = self.right.eval(context);
        (*context.options.logic_ops).or(left_result, right_result)
    }

    /// String representation of the current `Or` expression.
    fn to_string(&self) -> String {
        format!("(or {} {})", self.left.to_string(), self.right.to_string())
    }
}

/// 'Not' expression calculates NOT logical operation with given implementation.
#[derive(Debug)]
pub struct Not {
    /// Expression to calculate.
    expression: Box<dyn Expression>,
}

impl Not {
    /// Constructs `Not` expression.
    pub fn new(expression: Box<dyn Expression>) -> Not {
        Not {
            expression: expression,
        }
    }
}

impl Expression for Not {
    /// Gets 'not' implementation from `context` and returns its value.
    fn eval(&self, context: &InferenceContext) -> f32 {
        let value = (*self.expression).eval(context);
        (*context.options.logic_ops).not(value)
    }

    /// String representation of the current `Not` expression.
    fn to_string(&self) -> String {
        format!("(not {})", (*self.expression).to_string())
    }
}

/// Describes fuzzy inference rule.
#[derive(Debug)]
pub struct Rule {
    /// Root of the evaluation tree.
    condition: Box<dyn Expression>,
    /// IF ... THEN `result_set`.
    result_set: String,
    /// The universe of `result_set`.
    result_universe: String,
}

impl Rule {
    /// Constructs the new rule with given arguments.
    pub fn new(
        condition: Box<dyn Expression>,
        result_universe: String,
        result_set: String,
    ) -> Rule {
        Rule {
            condition,
            result_set,
            result_universe,
        }
    }

    /// Computes the current rule. Returns the fuzzy set as the result.
    pub fn compute(&self, context: &InferenceContext) -> Set {
        let expression_result = (*self.condition).eval(context);
        let universe = context
            .universes
            .get(&self.result_universe)
            .expect(&format!("{} does not exists", &self.result_universe));

        let set = universe
            .sets
            .get(&self.result_set)
            .expect(&format!("{} does not exists", &self.result_set));

        let result_values = set
            .cache
            .borrow()
            .iter()
            .filter_map(|(&key, &value)| {
                if value <= expression_result {
                    Some((key, value))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, f32>>();

        Set::new_with_domain(
            format!("{}: {}", &self.result_universe, &self.result_set),
            RefCell::new(result_values),
        )
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(Rule {}:{} if:{})",
            &self.result_universe,
            &self.result_set,
            &(*self.condition).to_string()
        )
    }
}

/// Contains all the rules. Evaluates them.
#[derive(Debug)]
pub struct RuleSet {
    /// Vector with rules.
    rules: Vec<Rule>,
}

impl RuleSet {
    /// Constructs the `RuleSet` with given `Rule`s
    pub fn new(rules: Vec<Rule>) -> Result<RuleSet, String> {
        let mut rules_iter = rules.iter();

        let first_rule_universe = rules_iter
            .next()
            .map(|rule| rule.result_universe.clone())
            .ok_or_else(|| "empty Rules".to_string())?;

        // Check if all the other rules have the same universe
        for rule in rules_iter {
            if first_rule_universe != rule.result_universe {
                return Err(format!(
                    "Rules are in different result universes({} and {})",
                    &first_rule_universe, &rule.result_universe
                ));
            }
        }
        return Ok(RuleSet { rules });
    }

    /// Computes all rules. Resulting fuzzy sets are then united and returned.
    pub fn compute_all(&self, context: &InferenceContext) -> Set {
        let mut result_set = self.rules[0].compute(context);

        for rule in &self.rules[1..self.rules.len()] {
            let mut result = rule.compute(context);
            result_set = (*context.options.set_ops).union(&mut result_set, &mut result);
        }
        result_set
    }
}

impl fmt::Display for RuleSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for rule in &self.rules {
            s = s + &format!("\t{}\n", rule);
        }
        write!(f, "(RuleSet\n{})", s)
    }
}
