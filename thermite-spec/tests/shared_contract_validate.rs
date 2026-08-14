use thermite_spec::{validate, SpecError};
use thermite_syntax::parse;

#[test]
fn shared_state_is_structurally_rejected_in_requires_and_ensures() {
    for (position, clause) in [
        ("requires", "requires state.n == 0 ensures result < 10"),
        ("ensures", "requires true ensures result == state.n"),
    ] {
        let parsed = parse(&format!(
            "struct State {{ n: u64 }} keeps n < 10
             shared state: State
             lock gate guards state
             fn read_state() -> u64 ! owns(gate), read(state.n)
             {clause}
             {{ holding gate {{ state.n }} }}"
        ));
        assert!(parsed.is_clean(), "{:?}", parsed.errors);
        let errors = validate(&parsed.program).expect_err("shared contract path must reject");
        assert!(
            errors.iter().any(|error| matches!(
                error,
                SpecError::SharedStateInContract {
                    root,
                    position: actual,
                    ..
                } if root == "state" && actual == &position
            )),
            "{errors:?}"
        );
    }
}

#[test]
fn parameter_shadowing_a_shared_root_is_not_a_shared_observation() {
    let parsed = parse(
        "struct State { n: u64 } keeps n < 10
         shared state: State
         fn local(state: State) -> u64 ! pure
         requires state.n < 10 ensures result == state.n
         { state.n }",
    );
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    validate(&parsed.program).expect("the parameter shadows the global shared root");
}
