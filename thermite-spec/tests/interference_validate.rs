use thermite_spec::{
    check_interference, check_interference_for_conflicts, validate, InterferenceError,
    InterferenceErrorKind, InterferenceReport, InterferenceRequirement, RegionIndex,
};
use thermite_syntax::{parse, Item, Program, RegionPath};

fn count_fn(name: &str, asks: &str, promises: &str) -> String {
    format!(
        "fn {name}(s: &mut u64) -> u64 \
         ! write(counter) requires true ensures final(s) >= 0 \
         interleaves {{ asks {asks}; promises {promises}; }} {{ 0 }}"
    )
}

fn check_declared_pairs(program: &Program) -> Result<InterferenceReport, Vec<InterferenceError>> {
    let regions = RegionIndex::build(program).expect("test shared-state metadata resolves");
    let mut requirements = Vec::new();
    for item in &program.items {
        let Item::Concurrent(composition) = item else {
            continue;
        };
        for left in 0..composition.roots.len() {
            for right in left + 1..composition.roots.len() {
                requirements.push(InterferenceRequirement {
                    composition: composition.name.clone(),
                    left_root: composition.roots[left].clone(),
                    right_root: composition.roots[right].clone(),
                    overlap: Some(RegionPath::from("counter")),
                });
            }
        }
    }
    check_interference_for_conflicts(program, &requirements, &regions)
}

#[test]
fn monotone_preorder_peers_discharge_both_ordered_obligations() {
    let source = format!(
        "shared counter: u64\nconcurrent pair {{ left, right }}\n{}\n{}",
        count_fn("left", "final(s) >= s", "final(s) >= s"),
        count_fn("right", "s <= final(s)", "s <= final(s)")
    );
    let parsed = parse(&source);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    validate(&parsed.program).expect("compatible monotone peers validate");
    let report = check_declared_pairs(&parsed.program).expect("checked report");
    assert_eq!(report.functions.len(), 2);
    assert_eq!(report.obligations.len(), 2);
    assert_eq!(report.obligations[0].guarantor, "left");
    assert_eq!(report.obligations[0].relying, "right");
    assert_eq!(report.obligations[1].guarantor, "right");
    assert_eq!(report.obligations[1].relying, "left");
}

#[test]
fn participant_local_parameter_names_resolve_to_the_same_shared_identity() {
    let parsed = parse(
        "shared counter: u64\n\
         concurrent pair { left, right }\n\
         fn left(a: &mut u64) -> u64 ! write(counter) requires true ensures final(a) >= 0 \
           interleaves { asks final(a) >= a; promises final(a) >= a; } { 0 }\n\
         fn right(b: &mut u64) -> u64 ! write(counter) requires true ensures final(b) >= 0 \
           interleaves { asks final(b) >= b; promises final(b) >= b; } { 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let report = check_declared_pairs(&parsed.program).expect("aliases resolve to shared identity");
    assert_eq!(report.obligations.len(), 2);
    for contract in report.functions.values() {
        assert!(contract
            .asks
            .atoms
            .iter()
            .all(|atom| atom.place == "counter"));
    }
}

#[test]
fn exact_step_and_epoch_equality_are_not_preorder_envelopes() {
    for relation in ["final(s) == s + 1", "final(s) == s"] {
        let parsed = parse(&count_fn("bad", relation, "final(s) >= s"));
        assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
        let errors = check_interference(&parsed.program).expect_err("relation must fail");
        assert!(errors
            .iter()
            .any(|error| error.kind == InterferenceErrorKind::UnsupportedRelation));
    }
}

#[test]
fn persistent_bit_bool_and_count_relations_are_closed_while_other_mutations_reject() {
    for source in [
        count_fn("count", "final(s) >= s", "final(s) >= s"),
        "shared bits: u64\nfn grow_bits(s: &mut u64) -> u64 ! write(bits) requires true ensures true \
         interleaves { asks final(s) | s == final(s); promises final(s) | s == final(s); } { 0 }"
            .to_string(),
        "shared flag: bool\nfn raise(s: &mut bool) -> bool ! write(flag) requires true ensures true \
         interleaves { asks !s || final(s); promises !s || final(s); } { true }"
            .to_string(),
    ] {
        let parsed = parse(&source);
        assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
        check_interference(&parsed.program).expect("closed persistent relation must validate");
    }

    for relation in ["final(s) == 0", "final(s) + 1 >= s", "final(s) != s"] {
        let parsed = parse(&count_fn("bad", relation, "final(s) >= s"));
        assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
        let errors = check_interference(&parsed.program).expect_err("mutation is outside v1");
        assert!(errors
            .iter()
            .any(|error| error.kind == InterferenceErrorKind::UnsupportedRelation));
    }

    let parsed = parse(
        "shared counter: u64\nfn bad(owned: u64) -> u64 ! write(counter) requires true ensures true \
         interleaves { asks final(owned) >= owned; promises final(owned) >= owned; } { 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = check_interference(&parsed.program).expect_err("owned state must not resolve");
    assert!(errors
        .iter()
        .any(|error| error.kind == InterferenceErrorKind::UnresolvedStateIdentity));
}

#[test]
fn clause_local_validator_does_not_invent_conflicts_for_disjoint_composition() {
    let parsed = parse(
        "shared left_state: u64\n\
         shared right_state: u64\n\
         #[boundary(\"ext::left\")] fn left() -> u64 ! write(left_state) requires true ensures true;\n\
         #[boundary(\"ext::right\")] fn right() -> u64 ! write(right_state) requires true ensures true;\n\
         concurrent pair { left, right }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    validate(&parsed.program)
        .expect("SpecTherm pre-validation must preserve disjoint RFC-9 compositions");
}

#[test]
fn incompatible_peers_and_missing_contracts_fail_closed() {
    let incompatible = format!(
        "shared counter: u64\nconcurrent pair {{ count, bits }}\n{}\n{}",
        count_fn("count", "final(s) >= s", "final(s) >= s"),
        count_fn(
            "bits",
            "final(s) | s == final(s)",
            "final(s) | s == final(s)"
        )
    );
    let parsed = parse(&incompatible);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = check_declared_pairs(&parsed.program).expect_err("peers must be incompatible");
    assert!(errors
        .iter()
        .any(|error| error.kind == InterferenceErrorKind::IncompatiblePeer));

    let missing = format!(
        "shared counter: u64\nconcurrent pair {{ count, plain }}\n{}\nfn plain() -> u64 ! write(counter) requires true ensures true {{ 0 }}",
        count_fn("count", "final(s) >= s", "final(s) >= s")
    );
    let parsed = parse(&missing);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = check_declared_pairs(&parsed.program).expect_err("missing contract must fail");
    assert!(errors
        .iter()
        .any(|error| error.kind == InterferenceErrorKind::MissingContract));
}

#[test]
fn handler_obligations_follow_priority_in_one_direction() {
    let source = format!(
        "shared counter: u64\nhandlers {{ low at 1, high at 2 }}\n{}\n{}",
        count_fn("low", "final(s) >= s", "final(s) >= s"),
        count_fn("high", "final(s) >= s", "final(s) >= s")
    );
    let parsed = parse(&source);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let report = check_declared_pairs(&parsed.program).expect("handler relations validate");
    assert_eq!(report.obligations.len(), 1);
    assert_eq!(report.obligations[0].guarantor, "high");
    assert_eq!(report.obligations[0].relying, "low");
}

#[test]
fn duplicate_participants_and_ambiguous_handler_priorities_fail() {
    let duplicate = format!(
        "shared counter: u64\nconcurrent pair {{ same, same }}\n{}",
        count_fn("same", "final(s) >= s", "final(s) >= s")
    );
    let parsed = parse(&duplicate);
    let errors = check_interference(&parsed.program).expect_err("duplicate must fail");
    assert!(errors
        .iter()
        .any(|error| error.kind == InterferenceErrorKind::DuplicateParticipant));

    let priorities = format!(
        "shared counter: u64\nhandlers {{ left at 1, right at 1 }}\n{}\n{}",
        count_fn("left", "final(s) >= s", "final(s) >= s"),
        count_fn("right", "final(s) >= s", "final(s) >= s")
    );
    let parsed = parse(&priorities);
    let errors = check_interference(&parsed.program).expect_err("equal priorities must fail");
    assert!(errors
        .iter()
        .any(|error| error.kind == InterferenceErrorKind::InvalidHandlerPriority));
}
