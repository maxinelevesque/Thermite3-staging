use thermite_lower::{check_program, lower, lower_l1, LowerError};
use thermite_syntax::parse;

#[test]
fn checked_boundary_accepts_flow_while_lowering_remains_fail_closed() {
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

    let checked =
        check_program(&parsed.program).expect("resource declaration and empty flow check");
    assert!(checked.resource_flow().direct_forgets.is_empty());
    assert_resource_refusal(&lower(&parsed.program).unwrap_err());
    assert_resource_refusal(&lower_l1(&parsed.program).unwrap_err());
}
