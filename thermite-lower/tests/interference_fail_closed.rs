use thermite_lower::{check_program, lower, LowerError};
use thermite_syntax::parse;

#[test]
fn validated_relations_cannot_fall_back_to_pre_rfc12_lowering() {
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
    let error = lower(&parsed.program).expect_err("lowering must fail closed");
    assert!(matches!(
        error,
        LowerError::Unsupported { what, .. } if what.contains("RFC-12")
    ));
}
