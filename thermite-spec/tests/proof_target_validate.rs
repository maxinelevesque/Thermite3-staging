//! Issue #41: `proof for f` is an out-of-line reference to an executable
//! function contract. The validator resolves that reference across the complete
//! program, accepts forward references, and rejects absent or wrong-kind roots.

use thermite_spec::{validate, SpecError};

fn parse_ok(source: &str) -> thermite_syntax::Program {
    let parsed = thermite_syntax::parse(source);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    parsed.program
}

#[test]
fn proof_target_accepts_an_existing_fn_in_either_source_order() {
    for source in [
        "fn f(n: u32) -> u32 ! pure requires true ensures result == n { n }\n\
         proof for f { ensures#0 by { omega } }",
        "proof for f { ensures#0 by { omega } }\n\
         fn f(n: u32) -> u32 ! pure requires true ensures result == n { n }",
    ] {
        validate(&parse_ok(source)).expect("proof target should resolve to f");
    }
}

#[test]
fn proof_target_rejects_an_absent_fn() {
    let program = parse_ok("proof for missing { ensures#0 by { omega } }");
    let errors = validate(&program).expect_err("orphan proof must fail validation");
    assert!(errors.iter().any(|error| matches!(
        error,
        SpecError::UnknownProofTarget { target, .. } if target == "missing"
    )));
}

#[test]
fn proof_target_rejects_a_name_of_the_wrong_item_kind() {
    let program = parse_ok(
        "spec fn logical(n: u32) -> u32 measures n { n }\n\
         proof for logical { ensures#0 by { omega } }",
    );
    let errors = validate(&program).expect_err("a spec fn is not a proof target");
    assert!(errors.iter().any(|error| matches!(
        error,
        SpecError::ProofTargetNotFunction { target, found, .. }
            if target == "logical" && *found == "`spec fn`"
    )));
}
