extern crate ordered_float;
extern crate num_cpus;
extern crate scoped_threadpool;

use self::scoped_threadpool::Pool;

use inference::InferenceContext;
use set::Set;

use std::fmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc::channel;

pub trait Expression {
    fn eval(&self, context: &InferenceContext) -> f32;
    fn to_string(&self) -> String;
}

pub struct Is {
    variable: String,
    set: String,
}

impl Is {
    pub fn new(variable: String, set: String) -> Is {
        Is {
            variable: variable,
            set: set,
        }
    }
}

impl Expression for Is {
    fn eval(&self, context: &InferenceContext) -> f32 {
        let value = context.values[&self.variable];
        let universe = context.universes
                                  .get(&self.variable)
                                  .expect(&format!("{} is not exists", &self.variable));
        let set = universe.sets
                              .get(&self.set)
                              .expect(&format!("{} is not exists", &self.set));
        set.check(value)
    }
    fn to_string(&self) -> String {
        format!("(is {} {})", self.variable, self.set)
    }
}

pub struct And<L, R>
    where L: Expression,
          R: Expression
{
    left: L,
    right: R,
}

impl<L: Expression, R: Expression> And<L, R> {
    pub fn new(left: L, right: R) -> And<L, R> {
        And {
            left: left,
            right: right,
        }
    }
}

impl<L: Expression, R: Expression> Expression for And<L, R> {
    fn eval(&self, context: &InferenceContext) -> f32 {
        let left_result = self.left.eval(context);
        let right_result = self.right.eval(context);
        (*context.options.logic_ops).and(left_result, right_result)
    }
    fn to_string(&self) -> String {
        format!("(and {} {})", self.left.to_string(), self.right.to_string())
    }
}

pub struct Or<L, R>
    where L: Expression,
          R: Expression
{
    left: L,
    right: R,
}

impl<L: Expression, R: Expression> Or<L, R> {
    pub fn new(left: L, right: R) -> Or<L, R> {
        Or {
            left: left,
            right: right,
        }
    }
}

impl<L: Expression, R: Expression> Expression for Or<L, R> {
    fn eval(&self, context: &InferenceContext) -> f32 {
        let left_result = self.left.eval(context);
        let right_result = self.right.eval(context);
        (*context.options.logic_ops).or(left_result, right_result)
    }
    fn to_string(&self) -> String {
        format!("(or {} {})", self.left.to_string(), self.right.to_string())
    }
}

pub struct Not {
    expression: Box<Expression>,
}

impl Not {
    fn new(expression: Box<Expression>) -> Not {
        Not { expression: expression }
    }
}

impl Expression for Not {
    fn eval(&self, context: &InferenceContext) -> f32 {
        let value = (*self.expression).eval(context);
        (*context.options.logic_ops).not(value)
    }
    fn to_string(&self) -> String {
        format!("(not {})", (*self.expression).to_string())
    }
}

pub struct Rule {
    condition: Box<Expression>,
    result_set: String,
    result_universe: String,
}

impl Rule {
    pub fn new(condition: Box<Expression>, result_universe: String, result_set: String) -> Rule {
        Rule {
            condition: condition,
            result_set: result_set,
            result_universe: result_universe,
        }
    }
    pub fn compute(&self, context: &InferenceContext) -> Set {
        let expression_result = (*self.condition).eval(context);
        let universe = context.universes
                              .get(&self.result_universe)
                              .expect(&format!("{} is not exists", &self.result_universe));
        let set = universe.sets
                          .get(&self.result_set)
                          .expect(&format!("{} is not exists", &self.result_set));
        let result_values = set.cache.borrow()
                               .iter()
                               .filter_map(|(&key, &value)| {
                                   if value <= expression_result {
                                       Some((key, value))
                                   } else {
                                       None
                                   }
                               })
                               .collect::<HashMap<_, f32>>();
        Set::new_with_domain(format!("{}: {}", &self.result_universe, &self.result_set),
                             RefCell::new(result_values))
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "(Rule {}:{} if:{})",
               &self.result_universe,
               &self.result_set,
               &(*self.condition).to_string())
    }
}

pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    pub fn new(rules: Vec<Rule>) -> Result<RuleSet, String> {
        let rule_universe = rules[0].result_universe.clone();
        for rule in &rules {
            if rule_universe != rule.result_universe {
                return Err(format!("Rules are in different result universes({} and {})",
                                   &rule_universe,
                                   &rule.result_universe));
            }
        }
        return Ok(RuleSet { rules: rules });
    }
    pub fn compute_all(&self, context: &InferenceContext) -> Set {
        let mut result_set = self.rules[0].compute(context);
        for rule in &self.rules[1..self.rules.len()] {
            let mut result = rule.compute(context);
            result_set = (*context.options.set_ops).union(&mut result_set, &mut result);
        }
        result_set
    }

    #[cfg(feature = "async_rules")]
    pub fn compute_all_async(&self, context: &InferenceContext) -> Set  {
        let mut pool = Pool::new(num_cpus::get() as u32);
        pool.scoped(|scope| {
            let mut counter = self.rules.len();
            let (tx, rx) = channel();
            for rule in self.rules.iter() {
                let tx = tx.clone();
                scope.execute(move || {
                    let rule_result = rule.compute(context);
                    tx.send(rule_result).unwrap();
                });
            }
            let mut result = Set::empty();
            while counter != 0 {
                result = (*context.options.set_ops).union(&mut result, &mut rx.recv().unwrap());
                counter -= 1;
            }
            result
        })
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
