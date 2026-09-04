use thermite_lower::{
    check_program, lower, lower_l1_artifact, lower_l2, lower_l3_artifact, LowerError,
};
use thermite_syntax::parse;

#[test]
fn validated_relations_are_bound_into_l1_and_l3_but_not_silently_accepted_by_l2() {
    let parsed = parse(
        "shared counter: u64\nfn grow(s: &mut u64) -> u64 \
         ! write(counter) requires true ensures final(s) >= 0 \
         interleaves { asks final(s) >= s; promises final(s) >= s; } \
         { 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let checked =
        check_program(&parsed.program).expect("relational report enters checked boundary");
    assert_eq!(checked.interference().functions.len(), 1);
    lower(&parsed.program).expect("L3 source emission preserves executable behavior");
    let l1 = lower_l1_artifact(&parsed.program, "grow").expect("checked L1 artifact");
    assert!(l1.interference_witness().is_some());
    assert!(l1.wrapper_identity().contains(":interference-sha256:"));
    let l3 = lower_l3_artifact(&parsed.program, "grow").expect("checked L3 artifact");
    assert!(l3.interference_witness().is_some());
    assert!(l3.query_identity().contains(":interference-sha256:"));

    let error = lower_l2(&parsed.program).expect_err("L2 has no RFC-12 evidence consumer");
    assert!(matches!(
        error,
        LowerError::Unsupported { what, .. } if what.contains("RFC-12")
    ));
}
