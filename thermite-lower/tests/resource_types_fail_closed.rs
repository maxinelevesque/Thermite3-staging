use thermite_lower::{check_program, lower, lower_l1, LowerError};
use thermite_syntax::parse;

#[test]
fn checked_and_lowering_boundaries_reject_provenance_only_resource_programs() {
    let parsed = parse("resource(heap) struct Grant { id: u64 }");
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);

    let assert_resource_refusal = |error: &LowerError| match error {
        LowerError::Unsupported { what, .. } => {
            assert!(
                what.contains("ownership flow"),
                "unexpected refusal: {what}"
            )
        }
        other => panic!("unexpected refusal: {other:?}"),
    };

    let errors = check_program(&parsed.program).unwrap_err();
    assert_resource_refusal(&errors[0]);
    assert_resource_refusal(&lower(&parsed.program).unwrap_err());
    assert_resource_refusal(&lower_l1(&parsed.program).unwrap_err());
}
