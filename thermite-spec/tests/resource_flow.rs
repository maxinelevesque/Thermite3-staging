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

#[test]
fn early_returns_and_loop_edges_are_checked_independently() {
    validate_source(&format!(
        "{GRANT}fn early(g: Grant, c: bool) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ if c {{ forget(g); return 0; }} forget(g); 0 }}"
    ))
    .expect("each returning edge disposes its own obligation");

    let early_leak = validate_source(&format!(
        "{GRANT}fn early_leak(g: Grant, c: bool) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ if c {{ return 0; }} forget(g); 0 }}"
    ))
    .unwrap_err();
    assert!(has(&early_leak, ResourceFlowErrorKind::Unconsumed));

    validate_source(&format!(
        "{GRANT}fn loop_ok(g: Grant, c: bool) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ while c keeps true measures 1 {{ break; }} forget(g); 0 }}"
    ))
    .expect("a loop preserving the header live set is accepted");

    let loop_mismatch = validate_source(&format!(
        "{GRANT}fn loop_bad(g: Grant, c: bool) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ while c keeps true measures 1 {{ forget(g); continue; }} 0 }}"
    ))
    .unwrap_err();
    assert!(has(&loop_mismatch, ResourceFlowErrorKind::LoopMismatch));
}

#[test]
fn a_declared_diverging_loop_has_no_resource_post_obligation() {
    validate_source(&format!(
        "{GRANT}fn spin(g: Grant) -> u64 ! diverge requires true ensures result == 0 \
         {{ loop keeps true measures 1 {{ continue; }} }}"
    ))
    .expect("a bare loop with no break has no returning resource edge");

    let returning_edge = validate_source(&format!(
        "{GRANT}fn maybe_spin(g: Grant, c: bool) -> u64 ! diverge requires true ensures result == 0 \
         {{ loop keeps true measures 1 {{ if c {{ break; }} continue; }} 0 }}"
    ))
    .unwrap_err();
    assert!(
        has(&returning_edge, ResourceFlowErrorKind::Unconsumed),
        "a reachable break remains an ordinary checked returning edge"
    );
}

#[test]
fn non_resource_projection_assignment_remains_owned_by_existing_checks() {
    validate_source(
        "struct State { n: u64 } keeps n < 10
         shared state: State
         lock gate guards state
         fn update() -> u64 ! owns(gate), read(state.n), write(state.n)
           requires true
           ensures result < 10
         { holding gate { state.n = 0; state.n } }",
    )
    .expect("RFC-11 flow must not reject an ordinary RFC-10 field assignment");
}

#[test]
fn destructuring_replaces_containers_with_component_obligations() {
    validate_source(&format!(
        "{GRANT}\
         resource struct Envelope {{ grant: Grant }}\n\
         resource enum Choice {{ A(Grant), B(Grant) }}\n\
         resource enum OptionalGrant {{ Present(Grant), Absent }}\n\
         fn dispose(g: Grant) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ forget(g); 0 }}\n\
         fn open(e: Envelope) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ match e {{ Envelope {{ grant }} => dispose(grant) }} }}\n\
         fn choose(c: Choice) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ match c {{ A(g) => dispose(g), B(g) => dispose(g) }} }}\n\
         fn optional(c: OptionalGrant) -> u64 ! forgets(heap) requires true ensures true \
         {{ match c {{ Present(g) => dispose(g), Absent => 0 }} }}\n\
         fn rewrap(e: Envelope) -> Envelope ! pure requires true ensures true \
         {{ match e {{ Envelope {{ grant }} => Envelope {{ grant: grant }} }} }}"
    ))
    .expect("destructuring must transfer every resource-bearing component");

    let direct_drop = validate_source(&format!(
        "{GRANT}fn crack(g: Grant) -> u64 ! pure requires true ensures result == 0 \
         {{ match g {{ Grant {{ id }} => id }} }}"
    ))
    .unwrap_err();
    assert!(has(&direct_drop, ResourceFlowErrorKind::ImplicitDrop));
}

#[test]
fn multi_region_forget_prices_every_provenance_atom() {
    let prefix = "resource(heap) struct HeapGrant { id: u64 }\n\
                  resource(device.port) struct PortGrant { id: u64 }\n\
                  resource struct Bundle { heap: HeapGrant, port: PortGrant }\n";
    validate_source(&format!(
        "{prefix}fn discard(b: Bundle) -> u64 ! forgets(heap), forgets(device.port) \
         requires true ensures result == 0 {{ forget(b); 0 }}"
    ))
    .expect("the complete two-region footprint is priced");

    let errors = validate_source(&format!(
        "{prefix}fn discard(b: Bundle) -> u64 ! forgets(heap) \
         requires true ensures result == 0 {{ forget(b); 0 }}"
    ))
    .unwrap_err();
    assert!(has(&errors, ResourceFlowErrorKind::MissingForgetEffect));
}
