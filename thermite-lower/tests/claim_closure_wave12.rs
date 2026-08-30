#[test]
fn maximal_row_policy_is_owned_by_forge_not_the_lowerer() {
    let lowerer = include_str!("../src/effects.rs");
    let entry = lowerer
        .split("pub fn check_effects")
        .nth(1)
        .unwrap()
        .split("pub fn analyze_effects")
        .next()
        .unwrap();
    assert!(entry.contains("crate::CheckedProgram::build(program).map(|_| ())"));
    assert!(!entry.contains("vacuity"));
    assert!(!entry.contains("slag"));

    let forge = include_str!("../../forge/src/vacuity.rs");
    for required in ["maximal", "slag", "triage"] {
        assert!(
            forge.contains(required),
            "missing forge policy pin {required}"
        );
    }
}

#[test]
fn effect_checker_has_no_runtime_sandbox_emission_surface() {
    let source = include_str!("../src/effects.rs");
    let entry = source
        .split("pub fn check_effects")
        .nth(1)
        .unwrap()
        .split("pub fn analyze_effects")
        .next()
        .unwrap();
    assert!(entry.contains("Result<(), Vec<LowerError>>"));
    for forbidden in ["syscall", "seccomp", "sandbox", "codegen"] {
        assert!(
            !entry.contains(forbidden),
            "effect checker unexpectedly owns runtime concern {forbidden}"
        );
    }
    let sandbox = include_str!("../../forge/src/sandbox.rs");
    assert!(sandbox.contains("Sandbox"));
    assert!(sandbox.contains("syscall"));
}

#[test]
fn ergonomic_desugars_reach_existing_lowerer_nodes_without_new_runtime_forms() {
    let conformance = include_str!("../../forge/tests/ergonomics_conformance.rs");
    for required in [
        "req1_tuple_destructure_desugars_to_temp_plus_projections",
        "req2_for_range_certifies_l3",
        "req5_if_let_certifies_l3",
        "req5_while_let_certifies_l3",
        "req5_while_let_desugars_to_while_is_variant",
    ] {
        assert!(
            conformance.contains(required),
            "missing ergonomic desugar witness {required}"
        );
    }
    let lowering = include_str!("../src/lower.rs");
    for existing_node in [
        "Expr::TupleProj",
        "LoopKind::While",
        "Expr::Match",
        "Expr::Is",
    ] {
        assert!(
            lowering.contains(existing_node),
            "missing established lowering node {existing_node}"
        );
    }
}

#[test]
fn every_holding_exit_normalizes_close_before_provider_release() {
    let implementation = include_str!("../src/locks.rs");
    for required in [
        "normalize_holding_closes",
        "normalize_holding_fallthrough",
        "append_close_calls",
        "Stmt::Return",
        "Stmt::Break | Stmt::Continue",
        "drop guard",
    ] {
        assert!(
            implementation.contains(required),
            "missing holding-close implementation pin {required}"
        );
    }
    let conformance = include_str!("shared_state_invariants.rs");
    for required in [
        "via_return",
        "via_break",
        "via_continue",
        "catch_unwind",
        "restoration before release",
        "__thermite_close_gate",
    ] {
        assert!(
            conformance.contains(required),
            "missing holding-close edge witness {required}"
        );
    }
}
