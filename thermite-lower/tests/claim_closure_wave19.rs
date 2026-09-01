fn pins(text: &str, expected: &[&str]) {
    for pin in expected {
        assert!(text.contains(pin), "missing claim pin {pin}");
    }
}

#[test]
fn tv_exec_reference_encoder_is_independent_bounded_and_fail_closed() {
    pins(
        include_str!("../../thermite-tv/src/exec_encode.rs"),
        &[
            "pub fn exec_ref_value",
            "Expr::Binary",
            "Expr::Index",
            "Expr::Cast",
            "RefEncodeError::Unsupported(node_kind(other))",
        ],
    );
    let manifest = include_str!("../../thermite-tv/Cargo.toml");
    let dependencies = manifest
        .split_once("[dependencies]")
        .expect("thermite-tv manifest has dependencies")
        .1;
    assert!(
        !dependencies.contains("thermite-lower"),
        "exec reference encoder depends on production lowering"
    );
}

#[test]
fn tv_exec_obligation_wraps_production_in_an_exec_fn_equivalence_contract() {
    pins(
        include_str!("../../thermite-tv/src/obligation.rs"),
        &[
            "pub struct ExecObligationFrame",
            "pub fn exec_equivalence_obligation",
            "let reference = exec_ref_value",
            "fn tv_exec_wrap(",
            "ensures result == ",
        ],
    );
}

#[test]
fn tv_exec_generator_is_deterministic_typed_bounded_and_well_framed() {
    pins(
        include_str!("../../thermite-tv/src/gen.rs"),
        &[
            "pub struct ExecClause",
            "const EXEC_SCALAR_BOUND: u128 = 1000",
            "const EXEC_MAX_DEPTH: usize = 2",
            "pub fn gen_exec_exprs(seed: u64, n: usize)",
            "req: scope.req()",
            "slice_params: scope.slice_params()",
        ],
    );
}

#[test]
fn tv_exec_teeth_accept_faithful_and_catch_four_infidelity_classes() {
    pins(
        include_str!("../../thermite-tv/tests/exec_teeth.rs"),
        &[
            "fn assert_faithful_verifies",
            "fn assert_infidel_caught",
            "fn e1_cast_paren_faithful_verifies",
            "fn e2_cast_lt_infidel_caught",
            "fn e3_wrong_op_infidel_caught",
            "fn e4_index_infidel_caught",
        ],
    );
}

#[test]
fn tv_exec_forge_plugin_joins_lowering_reference_discharge_and_total_verdicts() {
    pins(
        include_str!("../../forge/src/exec_tv.rs"),
        &[
            "pub enum ExecVerdict",
            "Faithful",
            "Divergent { detail: String }",
            "Unverifiable { reason: String }",
            "Skipped { reason: String }",
            "pub fn run_generated",
            "gen_exec_exprs(seed, n)",
            "pub fn exec_tv_file",
            "thermite_lower::lower_exec_expr",
            "exec_equivalence_obligation",
        ],
    );
    pins(
        include_str!("../../forge/tests/exec_tv_conformance.rs"),
        &[
            "fn generated_exec_run_all_faithful",
            "every CHECKED generated exec expr must be faithful",
        ],
    );
}

#[test]
fn tv_loop_subset_recognizer_is_single_while_and_fail_closed() {
    pins(
        include_str!("../../thermite-tv/src/exec_stmt_encode.rs"),
        &[
            "fn recognize_v1_loop",
            "Stmt::Loop(loop_node)",
            "LoopKind::While(_)",
            "loop_node.invs.is_empty()",
            "RefEncodeError::Unsupported",
        ],
    );
    pins(
        include_str!("../../thermite-tv/tests/loop_teeth.rs"),
        &[
            "fn l4_loop_kind_is_skipped",
            "fn l4_break_body_is_skipped",
            "fn l4_mid_body_return_is_skipped",
            "fn l4_nested_loop_is_skipped",
        ],
    );
}

#[test]
fn tv_loop_reference_computes_entry_preservation_and_exit_pieces() {
    pins(
        include_str!("../../thermite-tv/src/exec_stmt_encode.rs"),
        &[
            "pub struct LoopObligations",
            "pub entry_pred: String",
            "pub cond: String",
            "pub keeps: String",
            "pub step_cells: Vec<String>",
            "pub inv_at_step: String",
            "pub fn loop_ref_obligations",
        ],
    );
    pins(
        include_str!("../../thermite-tv/tests/loop_teeth.rs"),
        &[
            "fn l0_loop_ref_obligations_match_hand_derived",
            "assert_eq!(obs.inv_at_step",
        ],
    );
}

#[test]
fn tv_loop_obligation_emitters_cover_entry_preservation_and_exit() {
    pins(
        include_str!("../../thermite-tv/src/obligation.rs"),
        &[
            "pub fn loop_entry_obligation",
            "proof fn tv_loop_entry(",
            "pub fn loop_preservation_obligation",
            "fn tv_loop_step(",
            "pub fn loop_exit_obligation",
            "proof fn tv_loop_exit(",
        ],
    );
    pins(
        include_str!("../../thermite-tv/tests/loop_teeth.rs"),
        &[
            "fn l1_entry_obligation_verifies",
            "fn l1_preservation_obligation_verifies",
            "fn l1_exit_obligation_verifies",
            "fn l2_broken_preservation_caught",
            "fn l3_wrong_exit_characterization_caught",
        ],
    );
}

#[test]
fn verified_self_verification_architecture_binds_verus_core_to_rust_mirrors() {
    pins(
        include_str!("../../thermite-verified/src/lib.rs"),
        &[
            "#[cfg(verus_keep_ghost)]",
            "mod verus_core",
            "verus! {",
            "pub fn subsumes_masks",
            "pub fn should_emit_external_body",
            "pub fn aggregate_level",
            "pub fn meets_floor_60",
        ],
    );
}

#[test]
fn verified_tier1_coverage_has_finite_targets_and_production_anchors() {
    let verified = include_str!("../../thermite-verified/src/lib.rs");
    pins(
        verified,
        &[
            "pub fn subsumes_masks",
            "pub fn ladder_action_l3_tag",
            "pub fn io_allow",
            "pub fn should_emit_external_body",
            "pub fn aggregate_level",
            "pub fn meets_floor_60",
        ],
    );
    pins(
        include_str!("../../forge/src/manifest.rs"),
        &["aggregate_project_min_matches_proved_aggregate_level_over_all_level_lists"],
    );
}

#[test]
fn verified_tier_boundaries_exclude_io_and_external_body_from_the_core() {
    let verified = include_str!("../../thermite-verified/src/lib.rs");
    let core = verified
        .split_once("mod verus_core")
        .expect("verified core module exists")
        .1;
    pins(
        core,
        &[
            "verus! {",
            "ensures r == spec_subsumes",
            "proof fn zero_scored_never_passes",
        ],
    );
    assert!(
        !core.contains("std::fs"),
        "verified core acquired filesystem I/O"
    );
    assert!(
        !core.contains("std::process"),
        "verified core acquired process I/O"
    );
    assert!(
        !core
            .lines()
            .any(|line| line.trim_start().starts_with("#[verifier::external_body]")),
        "verified core acquired an external-body escape"
    );
}

#[test]
fn verified_honesty_runs_no_cheating_and_has_mutation_teeth() {
    pins(
        include_str!("../../thermite-verified/tests/verus_verify.rs"),
        &[
            ".arg(\"--no-cheating\")",
            "fn verified_core_passes_verus_no_cheating",
            "fn broken_subsumes_fails_verification",
            "fn broken_ladder_action_counterexample_degrades_fails",
            "fn broken_io_allow_xor_fails_monotone",
            "fn broken_should_emit_external_body_true_fails",
            "fn broken_aggregate_max_fails_le_all",
            "fn broken_meets_floor_drops_scored_guard_fails",
        ],
    );
}

#[test]
fn verified_effect_subsumption_is_proved_and_exhaustively_anchored() {
    pins(
        include_str!("../../thermite-verified/src/lib.rs"),
        &[
            "pub fn subsumes_masks",
            "let missing = callee & !caller",
            "pub fn spec_subsumes_mask",
            "pub fn subsumes(caller: u16, callee: u16)",
            "ensures r == spec_subsumes(caller, callee)",
        ],
    );
    pins(
        include_str!("../../thermite-lower/tests/effects_verified.rs"),
        &[
            "thermite_verified::subsumes_masks",
            "fn verified_spec_is_not_vacuous",
        ],
    );
}

#[test]
fn verified_ci_gauntlet_invokes_the_real_verus_driver_and_requires_zero_errors() {
    pins(
        include_str!("../../thermite-verified/tests/verus_verify.rs"),
        &[
            "fn run_verus",
            ".arg(\"--no-cheating\")",
            ".arg(\"--crate-type=lib\")",
            "fn verified_core_passes_verus_no_cheating",
            "output.contains(\"verified, 0 errors\")",
        ],
    );
}

#[test]
fn verified_degrade_anti_cheat_maps_counterexamples_to_hard_failure() {
    pins(
        include_str!("../../thermite-verified/src/lib.rs"),
        &[
            "pub fn ladder_action_l3_tag",
            "L3Tag::Counterexample => LadderAction::HardFail",
            "pub fn ladder_action_l2_tag",
            "L2Tag::Counterexample => LadderAction::HardFail",
            "proof fn anti_cheat_holds_for_all_verdicts",
        ],
    );
    pins(
        include_str!("../../forge/src/degrade.rs"),
        &[
            "fn ladder_action_l3_equals_verified_tag_over_all_verdicts",
            "fn ladder_action_l2_equals_verified_tag_over_all_verdicts",
        ],
    );
}

#[test]
fn verified_sandbox_allowlist_is_bounded_deny_by_default_and_anchored() {
    pins(
        include_str!("../../thermite-verified/src/lib.rs"),
        &[
            "pub fn widen(i: u16)",
            "pub fn io_allow(fx: u16)",
            "proof fn io_allow_within_io_bits",
            "ensures (io_allow(fx) & !0x1Fu32) == 0u32",
        ],
    );
    pins(
        include_str!("../../forge/src/sandbox.rs"),
        &["fn syscall_allowlist_matches_proved_io_allow_over_all_512_masks"],
    );
}

#[test]
fn verified_boundary_honesty_never_launders_regular_functions() {
    pins(
        include_str!("../../thermite-verified/src/lib.rs"),
        &[
            "pub fn should_emit_external_body",
            "has_boundary || has_slag",
            "proof fn regular_fn_never_external_body",
        ],
    );
    pins(
        include_str!("../../thermite-lower/tests/boundary_gate_verified.rs"),
        &[
            "fn lower_fn_emits_external_body_iff_proved_predicate",
            "fn regular_fn_is_fully_proved_never_external_body",
        ],
    );
}

#[test]
fn verified_aggregate_level_is_the_attained_project_minimum() {
    pins(
        include_str!("../../thermite-verified/src/lib.rs"),
        &[
            "pub fn aggregate_level(levels: &[Level])",
            "acc = min2(acc, levels[i])",
            "proof fn aggregate_le_all",
            "proof fn aggregate_is_attained",
        ],
    );
    pins(
        include_str!("../../forge/src/manifest.rs"),
        &["aggregate_project_min_matches_proved_aggregate_level_over_all_level_lists"],
    );
}

#[test]
fn verified_mutation_floor_rejects_zero_scored_and_matches_production() {
    pins(
        include_str!("../../thermite-verified/src/lib.rs"),
        &[
            "pub fn meets_floor_60(killed: usize, scored: usize)",
            "s > 0 && k * 100 >= s * 60",
            "proof fn zero_scored_never_passes",
        ],
    );
    pins(
        include_str!("../../forge/src/mutation.rs"),
        &["fn zero_scored_never_passes_on_both_representations"],
    );
}

#[test]
fn verus_error_count_is_structured_optional_and_never_synthesized() {
    pins(
        include_str!("../../forge/src/verified_build.rs"),
        &[
            "pub struct VerusEvidence",
            "pub errors: Option<u64>",
            "fn parse_verus_summary",
            "summary.get(\"errors\").and_then(|v| v.as_u64())",
        ],
    );
}

#[test]
fn verus_failure_diagnostics_claim_only_known_numeric_counts() {
    pins(
        include_str!("../../forge/src/verified_build.rs"),
        &[
            "fn verus_failure_detail",
            "Some(errors) => format!",
            "None => format!",
            "fn verus_failure_detail_claims_only_structured_counts",
            "assert!(!unknown.contains(\"errors=\"))",
        ],
    );
    pins(
        include_str!("../../forge/tests/verus_error_accounting.rs"),
        &["fn frontend_rejection_omits_an_unknown_error_count_and_publishes_nothing"],
    );
}

#[test]
fn verus_unknown_counts_fail_closed_for_compile_publication_and_replay() {
    let build = include_str!("../../forge/src/verified_build.rs");
    pins(
        build,
        &[
            "let success = output.status.success() && reported_success && errors == Some(0)",
            "compiled.evidence.errors != Some(0)",
            "verus.errors != Some(0)",
            "parse_verus_summary(&verus.stdout) != (true, Some(0))",
        ],
    );
    pins(
        include_str!("../../forge/tests/verus_error_accounting.rs"),
        &["assert!(!bundle.exists(), \"frontend rejection published a bundle\")"],
    );
}
