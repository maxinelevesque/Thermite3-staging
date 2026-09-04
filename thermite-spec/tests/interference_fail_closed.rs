use thermite_spec::{validate, SpecError};
use thermite_syntax::parse;

#[test]
fn parsed_interference_cannot_cross_the_unimplemented_spec_boundary() {
    let parsed = parse(
        "fn ack(s: &mut u64) -> u64 \
         ! pure requires true ensures result >= final(s) \
         interleaves { asks final(s) >= s; promises final(s) >= s; } \
         { 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = validate(&parsed.program).expect_err("RFC-12 must fail closed");
    assert!(errors.iter().any(|error| matches!(
        error,
        SpecError::UnsupportedInterferenceClauses { function, .. } if function == "ack"
    )));
}
