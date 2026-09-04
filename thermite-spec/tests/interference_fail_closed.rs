use thermite_spec::{validate, InterferenceErrorKind, SpecError};
use thermite_syntax::parse;

#[test]
fn an_unstable_postcondition_fails_closed() {
    let parsed = parse(
        "shared counter: u64\nfn ack(s: &mut u64) -> u64 \
         ! write(counter) requires true ensures result >= final(s) \
         interleaves { asks final(s) >= s; promises final(s) >= s; } \
         { 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = validate(&parsed.program).expect_err("unstable RFC-12 contract must fail closed");
    assert!(errors.iter().any(|error| matches!(
        error,
        SpecError::Interference { kind: InterferenceErrorKind::UnstablePostcondition, function: Some(function), .. } if function == "ack"
    )));
}
