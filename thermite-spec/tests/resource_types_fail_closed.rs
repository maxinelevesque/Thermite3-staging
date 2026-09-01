use thermite_spec::{validate, SpecError};
use thermite_syntax::parse;

#[test]
fn parsed_resource_declaration_is_accepted_by_provenance_boundary() {
    let parsed = parse("resource(heap) struct Grant { value: u64 }");
    assert!(
        parsed.is_clean(),
        "syntax must accept RFC-11: {:?}",
        parsed.errors
    );

    validate(&parsed.program).expect("direct provenance is now checked");
}

#[test]
fn parsed_forget_surface_fails_closed_until_resource_semantics_land() {
    let parsed = parse(
        "fn release(grant: Grant) -> u64\n\
         ! forgets(heap)\n\
         requires true\n\
         ensures result == 0\n\
         { forget(grant); 0 }",
    );
    assert!(
        parsed.is_clean(),
        "syntax must accept RFC-11 forget forms: {:?}",
        parsed.errors
    );

    let errors = validate(&parsed.program).expect_err("forget needs resource-flow validation");
    assert!(errors
        .iter()
        .any(|error| matches!(error, SpecError::UnsupportedResourceTypes { .. })));
}
