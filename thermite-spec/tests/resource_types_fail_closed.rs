use thermite_spec::{validate, SpecError};
use thermite_syntax::parse;

#[test]
fn parsed_resource_declaration_fails_closed_at_spec_boundary() {
    let parsed = parse("resource(heap) struct Grant { value: u64 }");
    assert!(
        parsed.is_clean(),
        "syntax must accept RFC-11: {:?}",
        parsed.errors
    );

    let errors = validate(&parsed.program).expect_err("resource types need provenance validation");
    assert!(errors
        .iter()
        .any(|error| matches!(error, SpecError::UnsupportedResourceTypes { .. })));
}
