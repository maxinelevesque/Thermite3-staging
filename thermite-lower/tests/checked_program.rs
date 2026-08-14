use thermite_lower::{
    analyze_effects, check_program, lower, lower_l1, lower_l2, lower_l3_library, CheckedProgram,
    L3LibraryTarget, LowerError,
};
use thermite_syntax::{parse, WorkBudget};

const VALID: &str = "fn f(x: u64) -> u64 ! pure requires true ensures result == x { x }";

#[test]
fn checked_program_binds_inventory_regions_and_effects_once() {
    let parsed = parse(VALID);
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    let checked = check_program(&parsed.program).expect("checked construction");
    assert!(!checked.inventory().kinds.is_empty());
    assert_eq!(
        checked.effects(),
        &analyze_effects(&parsed.program).unwrap()
    );
    assert_eq!(checked.source(), &parsed.program);
}

#[test]
fn resource_limit_is_non_certifying_at_the_checked_boundary() {
    let parsed = parse(VALID);
    let errors = CheckedProgram::build_with_budget(&parsed.program, WorkBudget(1))
        .expect_err("one node cannot inventory this program");
    assert!(matches!(
        errors.as_slice(),
        [LowerError::ResourceLimit {
            budget: 1,
            required_at_least: 2
        }]
    ));
}

#[test]
fn compatibility_lowering_routes_reject_the_same_invalid_program() {
    let parsed = parse(
        "struct Counter { n: u64 } keeps n < 10
         shared counter: Counter
         lock counter_lock guards counter
         fn read() -> u64 ! read(counter.n) requires true ensures result < 10
         { counter.n }",
    );
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    let expected = check_program(&parsed.program).unwrap_err()[0].to_string();
    assert_eq!(lower(&parsed.program).unwrap_err().to_string(), expected);
    assert_eq!(lower_l1(&parsed.program).unwrap_err().to_string(), expected);
    assert!(lower_l2(&parsed.program)
        .unwrap_err()
        .to_string()
        .contains("RFC-10 shared-state L2 Kani harness"));
    assert_eq!(
        lower_l3_library(&parsed.program, &[], L3LibraryTarget::Std)
            .unwrap_err()
            .to_string(),
        expected
    );
}

#[test]
fn checked_holdings_carry_regions_capabilities_transitions_and_close_edges() {
    let parsed = parse(
        "struct State { n: u64 } keeps n < 10
         shared left: State
         shared right: State
         lock outer guards left
         lock inner guards right after outer
         fn f() -> u64 ! owns(outer), owns(inner), read(right.n)
           requires true ensures result < 10
         { holding outer { holding inner { return right.n; } } 0 }",
    );
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    let checked = check_program(&parsed.program).expect("checked construction");
    assert_eq!(checked.holdings().len(), 2);
    let outer = &checked.holdings()[0];
    let inner = &checked.holdings()[1];
    assert_eq!(outer.guarded_region.to_string(), "left");
    assert_eq!(inner.guarded_region.to_string(), "right");
    assert_eq!(inner.incoming_held, ["outer"]);
    assert_eq!(inner.outgoing_held, ["outer"]);
    assert!(inner.capability.starts_with("capability@"));
    assert!(inner.close_edges.iter().any(|edge| {
        edge.reason == thermite_lower::CloseReason::Return
            && edge.inner_to_outer == ["inner", "outer"]
    }));
    let access = checked
        .shared_places()
        .iter()
        .find(|place| place.path.to_string() == "right.n")
        .expect("resolved shared access");
    assert_eq!(access.authorizing_locks, ["inner"]);
}
