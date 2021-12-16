use std::collections::HashMap;

use fuzzy_logic::{
    functions::{DefuzzFactory, MembershipFactory},
    inference::{InferenceMachine, InferenceOptions},
    ops::{MinMaxOps, ZadehOps},
    rules::{And, Is, Rule, RuleSet},
    set::UniversalSet,
};

fn main() {
    // first column of x dest
    let nb_nb = And::new(Is::new("x_dest", "NB"), Is::new("x", "NB"));
    let nb_ns = And::new(Is::new("x_dest", "NB"), Is::new("x", "NS"));
    let nb_z = And::new(Is::new("x_dest", "NB"), Is::new("x", "Z"));
    let nb_ps = And::new(Is::new("x_dest", "NB"), Is::new("x", "PS"));
    let nb_pb = And::new(Is::new("x_dest", "NB"), Is::new("x", "PB"));
    // second column of x dest
    let ns_nb = And::new(Is::new("x_dest", "NS"), Is::new("x", "NB"));
    let ns_ns = And::new(Is::new("x_dest", "NS"), Is::new("x", "NS"));
    let ns_z = And::new(Is::new("x_dest", "NS"), Is::new("x", "Z"));
    let ns_ps = And::new(Is::new("x_dest", "NS"), Is::new("x", "PS"));
    let ns_pb = And::new(Is::new("x_dest", "NS"), Is::new("x", "PB"));

    let rules = vec![
        // first column of x dest
        Rule::new(nb_nb.boxed(), "pitch_output".into(), "Z".into()),
        Rule::new(nb_ns.boxed(), "pitch_output".into(), "NS".into()),
        Rule::new(nb_z.boxed(), "pitch_output".into(), "NB".into()),
        Rule::new(nb_ps.boxed(), "pitch_output".into(), "NB".into()),
        Rule::new(nb_pb.boxed(), "pitch_output".into(), "NB".into()),
        // second column of x dest
        Rule::new(ns_nb.boxed(), "pitch_output".into(), "PS".into()),
        Rule::new(ns_ns.boxed(), "pitch_output".into(), "Z".into()),
        Rule::new(ns_z.boxed(), "pitch_output".into(), "NS".into()),
        Rule::new(ns_ps.boxed(), "pitch_output".into(), "NB".into()),
        Rule::new(ns_pb.boxed(), "pitch_output".into(), "NB".into()),
    ];

    let (x_dest, x, pitch_output) = {
        let x_dest = UniversalSet::new("x_dest")
            .with_domain(vec![-2.0, -1.0, 0.0, 1.0, 2.0])
            .add_set("NB", MembershipFactory::triangular(-2.0, -2.0, -1.0))
            .add_set("NS", MembershipFactory::triangular(-2.0, -1.0, 0.0))
            .add_set("Z", MembershipFactory::triangular(-1.0, 0.0, 1.0))
            .add_set("PS", MembershipFactory::triangular(0.0, 1.0, 2.0))
            .add_set("PB", MembershipFactory::triangular(1.0, 2.0, 2.0));

        let x = UniversalSet::new("x")
            .with_domain(vec![-2.0, -1.0, 0.0, 1.0, 2.0])
            .add_set("NB", MembershipFactory::triangular(-2.0, -2.0, -1.0))
            .add_set("NS", MembershipFactory::triangular(-2.0, -1.0, 0.0))
            .add_set("Z", MembershipFactory::triangular(-1.0, 0.0, 1.0))
            .add_set("PS", MembershipFactory::triangular(0.0, 1.0, 2.0))
            .add_set("PB", MembershipFactory::triangular(1.0, 2.0, 2.0));

        let pitch_output = UniversalSet::new("pitch_output")
        .with_domain(vec![-0.5, -0.25, 0.0, 0.25, 0.5])
            .add_set("NB", MembershipFactory::singleton(-0.5))
            .add_set("NS", MembershipFactory::singleton(-0.25))
            .add_set("Z", MembershipFactory::singleton(0.0))
            .add_set("PS", MembershipFactory::singleton(0.25))
            .add_set("PB", MembershipFactory::singleton(0.5));
        
            (x_dest, x, pitch_output)
    };

    let mut universes = HashMap::default();
    universes.insert("x_dest".into(), x_dest);
    universes.insert("x".into(), x);
    universes.insert("pitch_output".into(), pitch_output);

    let rule_set = RuleSet::new(rules).expect("Valid RuleSet");

    let options = InferenceOptions {
        logic_ops: Box::new(ZadehOps {}),
        set_ops: Box::new(MinMaxOps {}),
        defuzz_func: DefuzzFactory::center_of_mass(),
    };

    let mut inference = InferenceMachine::new(rule_set, universes, options);

    let input = vec![
        // NB
        ("x_dest".into(), -1.9_f32),
        // NS
        ("x".into(), -1.0_f32),
    ]
    .into_iter()
    .collect();


    inference.update(input);

    dbg!(&inference);
    
    // NB of x_dest
    // NS of x
    // Expected: NS
    let output = inference.compute();

    // TODO: Check what's going wrong here.
    // value: NaN
    // Actual: Set: pitch_output: Z UNION pitch_output: NS UNION pitch_output: NB UNION pitch_output: NB UNION pitch_output: NB UNION pitch_output: PS UNION pitch_output: Z UNION pitch_output: NS UNION pitch_output: NB UNION pitch_output: NB value: NaN
    println!("Set: {} value: {}", output.0, output.1)
}
