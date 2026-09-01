use thermite_spec::{validate, ResourceFlowErrorKind, SpecError};
use thermite_syntax::parse;

fn validate_source(source: &str) -> Result<(), Vec<SpecError>> {
    let parsed = parse(source);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    validate(&parsed.program)
}

const GRANT: &str = "resource(heap) struct Grant { id: u64 }\n";

fn has(errors: &[SpecError], kind: ResourceFlowErrorKind) -> bool {
    errors
        .iter()
        .any(|error| matches!(error, SpecError::ResourceFlow { kind: found, .. } if *found == kind))
}

#[test]
fn forget_and_transfer_discharge_live_resources() {
    validate_source(&format!(
        "{GRANT}\
         fn dispose(g: Grant) -> u64\n\
           ! forgets(heap)\n\
           requires true\n\
           ensures result == 0\n\
         {{ forget(g); 0 }}\n\
         fn forward(g: Grant) -> u64\n\
           ! forgets(heap)\n\
           requires true\n\
           ensures result == 0\n\
         {{ dispose(g) }}\n\
         fn identity(g: Grant) -> Grant\n\
           ! pure\n\
           requires true\n\
           ensures true\n\
         {{ g }}"
    ))
    .expect("forget, call transfer, and return transfer must be accepted");
}

#[test]
fn forget_requires_the_complete_declared_effect_footprint() {
    let errors = validate_source(&format!(
        "{GRANT}\
         fn dispose(g: Grant) -> u64\n\
           ! pure\n\
           requires true\n\
           ensures result == 0\n\
         {{ forget(g); 0 }}"
    ))
    .unwrap_err();
    assert!(has(&errors, ResourceFlowErrorKind::MissingForgetEffect));
}

#[test]
fn leaks_double_consumption_copy_and_overwrite_are_rejected() {
    let leak = validate_source(&format!(
        "{GRANT}fn leak(g: Grant) -> u64 ! pure requires true ensures result == 0 {{ 0 }}"
    ))
    .unwrap_err();
    assert!(has(&leak, ResourceFlowErrorKind::Unconsumed));

    let double = validate_source(&format!(
        "{GRANT}fn twice(g: Grant) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ forget(g); forget(g); 0 }}"
    ))
    .unwrap_err();
    assert!(has(&double, ResourceFlowErrorKind::UseAfterMove));

    let copy = validate_source(&format!(
        "{GRANT}fn compare(g: Grant) -> bool ! pure requires true ensures true \
         {{ let same: bool = g == g; same }}"
    ))
    .unwrap_err();
    assert!(has(&copy, ResourceFlowErrorKind::Copy));

    let overwrite = validate_source(&format!(
        "{GRANT}fn replace(g: Grant) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ let mut slot: Grant = g; slot = Grant {{ id: 1 }}; forget(slot); 0 }}"
    ))
    .unwrap_err();
    assert!(has(&overwrite, ResourceFlowErrorKind::ImplicitDrop));
}

#[test]
fn branch_joins_require_the_same_live_set() {
    validate_source(&format!(
        "{GRANT}fn balanced(g: Grant, c: bool) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ if c {{ forget(g); }} else {{ forget(g); }} 0 }}"
    ))
    .expect("both branches consume the same obligation");

    let errors = validate_source(&format!(
        "{GRANT}fn unbalanced(g: Grant, c: bool) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ if c {{ forget(g); }} else {{ }} 0 }}"
    ))
    .unwrap_err();
    assert!(has(&errors, ResourceFlowErrorKind::BranchMismatch));
}

#[test]
fn borrows_do_not_consume_but_by_value_calls_do() {
    validate_source(&format!(
        "{GRANT}\
         fn inspect(g: &Grant) -> u64 ! pure requires true ensures result == 0 {{ 0 }}\n\
         fn owner(g: Grant) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ inspect(&g); forget(g); 0 }}"
    ))
    .expect("an explicit borrow leaves the resource live for its final disposition");
}
