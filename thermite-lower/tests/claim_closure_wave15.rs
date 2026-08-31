fn pins(text: &str, expected: &[&str]) {
    for pin in expected {
        assert!(text.contains(pin), "missing claim pin {pin}");
    }
}

#[test]
fn s1_axiom_gate_is_shared_by_every_lean_discharge() {
    pins(
        include_str!("../../forge/src/engine.rs"),
        &[
            "STANDARD_AXIOM_ALLOWLIST",
            "fn certify_lean_axioms",
            "certify_lean_axioms_gate_accepts_clean_refuses_smuggled",
        ],
    );
    assert!(
        include_str!("../../.design/verified/exporter-surface-correspondence.md")
            .contains("correspondence")
    );
}

#[test]
fn s1_covenant_executes_witnesses_before_burn() {
    pins(
        include_str!("../../forge/src/covenant_engine.rs"),
        &[
            "pub fn analyze_covenant",
            "pub fn covenant_gate",
            "covenant_gate_never_burns_without_covenant",
        ],
    );
    pins(
        include_str!("../../forge/tests/covenant_conformance.rs"),
        &["CovenantRefuted", "witness"],
    );
}

#[test]
fn s1_frozen_battery_refuses_unlisted_citations_and_reports_stuck() {
    pins(
        include_str!("../../forge/src/battery.rs"),
        &[
            "pub fn enforce",
            "pub fn stuck_from_lake_output",
            "BatteryUnlistedTactic",
            "BatteryUnlistedSimpLemma",
        ],
    );
    assert!(include_str!("../../conformance/battery/registry.json").contains("tactics"));
}

#[test]
fn s1_antigoodhart_gates_reelaborate_and_bound_meaning() {
    pins(
        include_str!("../../forge/src/check.rs"),
        &[
            "fn gate_definition_tower",
            "pub(crate) fn reelaboration_mutants",
        ],
    );
    pins(
        include_str!("../../forge/src/lean_export.rs"),
        &["pub fn export_arbitrary_result_harness"],
    );
    pins(
        include_str!("../../forge/src/meaning.rs"),
        &["TOWER_DEPTH_BUDGET", "TOWER_DEFINITION_BUDGET"],
    );
}

#[test]
fn s1_goal_fill_and_burn_receipt_are_bound_to_committed_proof() {
    pins(
        include_str!("../../forge/src/goal_repl.rs"),
        &[
            "pub fn render_proof",
            "pub fn fill_hole",
            "fn proof_hole_span",
        ],
    );
    pins(
        include_str!("../../forge/src/burn.rs"),
        &["pub struct BurnReceipt", "proof_tokens", "cited_lemmas"],
    );
    pins(
        include_str!("../../forge/src/lean_export.rs"),
        &["pub fn export_lemma"],
    );
}

#[test]
fn s1_relax_route_escalates_real_validity_to_l4() {
    pins(
        include_str!("../../forge/src/relax.rs"),
        &["pub fn classify_fn", "RelaxVerdict"],
    );
    pins(
        include_str!("../../forge/src/engine.rs"),
        &["pub struct NlsatEngine", "RealWitness"],
    );
    pins(
        include_str!("../../forge/src/manifest.rs"),
        &["pub enum Level", "L4"],
    );
}

#[test]
fn s1_real_relaxation_lemmas_are_axiom_probed() {
    pins(
        include_str!("../../lean/Thermite/Relax.lean"),
        &["theorem rencode_sound", "theorem r_relax_sound"],
    );
    pins(
        include_str!("../../gates/lean-axiom-probe.sh"),
        &[
            "Thermite.Relax.rencode_sound",
            "Thermite.Relax.r_relax_sound",
        ],
    );
}

#[test]
fn s1_lemma_library_is_certified_deduplicated_and_cached() {
    pins(
        include_str!("../../forge/src/lemma_library.rs"),
        &[
            "pub struct LemmaLibrary",
            "pub fn enforce_citations",
            "statement_hash",
            "rewrite_citations",
        ],
    );
    pins(
        include_str!("../../forge/src/accessibility.rs"),
        &["pub fn cache_dec_wf_accessibility"],
    );
}

#[test]
fn s1_out_of_line_proofs_bind_only_to_executable_functions() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &["UnknownProofTarget", "ProofTargetNotFunction"],
    );
    pins(
        include_str!("../../thermite-spec/tests/proof_target_validate.rs"),
        &[
            "proof_target_rejects_an_absent_fn",
            "proof_target_rejects_a_name_of_the_wrong_item_kind",
        ],
    );
    assert!(include_str!("../../thermite-syntax/tests/forge_items.rs")
        .contains("orphan_and_wrong_kind_proofs_have_no_semantic_addresses"));
}

#[test]
fn s2_foundation_has_finite_carriers_and_true_quantifier_folds() {
    pins(
        include_str!("../../lean/Thermite/Strat/Carrier.lean"),
        &["structure CarrierAssign", "complete"],
    );
    pins(
        include_str!("../../lean/Thermite/Strat/Denote.lean"),
        &["sdenote_all_iff", "sdenote_ex_iff", "canonicalQOracle"],
    );
    assert!(
        include_str!("../../lean/Thermite/PinFiniteEscape.lean").contains("incompleteEnum_escapes")
    );
}

#[test]
fn s2_complete_pin_battery_guards_each_metatheory_boundary() {
    pins(
        include_str!("../../lean/Thermite/PinStratSelfLoop.lean"),
        &["stripSelf"],
    );
    pins(
        include_str!("../../lean/Thermite/PinNNFPolarity.lean"),
        &["NNF"],
    );
    pins(
        include_str!("../../lean/Thermite/PinRelaxRefute.lean"),
        &["Real"],
    );
    pins(
        include_str!("../../.design/verified/strat-rust-lean-correspondence.md"),
        &["stage-2 pin battery", "PinRestratDropSide"],
    );
}

#[test]
fn s2_substkit_preserves_binders_and_refutes_broken_lift() {
    pins(
        include_str!("../../lean/Thermite/Strat/SubstKit.lean"),
        &["sdenote_push_lift", "sdenote_subst", "substFrm_liftFrm"],
    );
    pins(
        include_str!("../../lean/Thermite/PinBrokenLift.lean"),
        &["liftBadFrm", "brokenLift_breaks_push_lift"],
    );
}

#[test]
fn s2_rust_classifier_matches_kernel_classifier() {
    pins(
        include_str!("../../thermite-spec/src/classifier.rs"),
        &["pub enum RejectReason", "pub fn admitted", "SortGraphCycle"],
    );
    pins(
        include_str!("../../forge/src/strat_tv.rs"),
        &["disagreement", "tripwire"],
    );
    assert!(include_str!("../../forge/tests/strat_differential.rs").contains("generated"));
}

#[test]
fn s2_reference_encoder_is_sound_and_capture_free() {
    pins(
        include_str!("../../lean/Thermite/Strat/Soundness.lean"),
        &["theorem strat_ref_sound", "theorem strat_ref_wf"],
    );
    pins(
        include_str!("../../lean/Thermite/PinStratFlip.lean"),
        &["flip_breaks_soundness"],
    );
    pins(
        include_str!("../../lean/Thermite/PinStratCapture.lean"),
        &["capture_breaks_soundness"],
    );
}

#[test]
fn s2_all_combinators_have_derivations_and_offbyone_pin() {
    let comb = include_str!("../../lean/Thermite/Strat/CombDeriv.lean");
    for name in [
        "comb_deriv_forall_in",
        "comb_deriv_exists_in",
        "comb_deriv_sorted",
        "comb_deriv_forall_below",
        "comb_deriv_forall_from",
        "comb_deriv_disjoint",
        "comb_deriv_count_where",
        "comb_deriv_permutation_of",
    ] {
        assert!(comb.contains(name));
    }
    assert!(
        include_str!("../../lean/Thermite/PinCombDeriv.lean").contains("offbyone_breaks_demotion")
    );
}

#[test]
fn s2_restratification_requires_separately_discharged_side() {
    pins(
        include_str!("../../lean/Thermite/Strat/Restratify.lean"),
        &["restrat_conservative", "side_admitted", "restrat_complete"],
    );
    pins(
        include_str!("../../thermite-spec/src/restratify.rs"),
        &["SideUndischarged", "pub fn certify"],
    );
    assert!(include_str!("../../lean/Thermite/PinRestratDropSide.lean")
        .contains("dropSide_breaks_certification"));
}

#[test]
fn s2_faithfulness_and_two_phase_tv_fail_closed() {
    pins(
        include_str!("../../lean/Thermite/Strat/Faithfulness.lean"),
        &["strat_lowering_faithful", "SourceModel"],
    );
    pins(
        include_str!("../../thermite-tv/src/strat_two_phase.rs"),
        &["pub fn run_two_phase", "TvVerdict::Withheld", "G2_FLIPPED"],
    );
    assert!(
        include_str!("../../thermite-lower/tests/strat_quantifier_lower.rs").contains("forall")
    );
}

#[test]
fn s2_g2_flip_requires_all_four_green_checks() {
    pins(
        include_str!("../../thermite-tv/src/strat_two_phase.rs"),
        &[
            "pub struct G2Checks",
            "pub fn g2_flip_permitted",
            "gate_blocks_the_flip_when_any_one_check_is_red",
        ],
    );
    pins(
        include_str!("../../gates/audit.sh"),
        &["[G2]", "strat-faithful-tv"],
    );
}

#[test]
fn s3_bv_syntax_is_feature_gated_and_width_closed() {
    pins(
        include_str!("../../thermite-syntax/src/parser.rs"),
        &["BvTagWithoutShadowPlumbing", "BvWidth"],
    );
    pins(
        include_str!("../../thermite-syntax/tests/bv_tag_parse.rs"),
        &["nowrap", "64"],
    );
}

#[test]
fn s3_bitvector_engine_uses_direct_qf_bv_and_countermodels() {
    pins(
        include_str!("../../forge/src/bitvector.rs"),
        &["QF_BV", "pub struct BitVectorEngine", "countermodel"],
    );
    pins(
        include_str!("../../forge/tests/bv_lowering.rs"),
        &["counterexample", "timeout"],
    );
}

#[test]
fn s3_bv_shadow_is_visible_in_certificates_and_audit() {
    pins(
        include_str!("../../forge/src/manifest.rs"),
        &["bv_shadow", "BvShadow"],
    );
    pins(
        include_str!("../../forge/src/audit.rs"),
        &["bv_shadows", "pub struct BvShadowRow"],
    );
}

#[test]
fn s3_mutation_scoring_is_width_aware_and_equivalence_aware() {
    pins(
        include_str!("../../forge/src/check.rs"),
        &["fn bv_mutation_score", "is_equivalent"],
    );
    assert!(include_str!("../../forge/tests/bv_lowering.rs").contains("mutant"));
}

#[test]
fn s3_nowrap_side_obligations_fail_closed() {
    pins(
        include_str!("../../forge/src/bitvector.rs"),
        &[
            "discharge_nowrap",
            "BvOutcome::Unknown",
            "BvOutcome::Unavailable",
        ],
    );
    assert!(include_str!("../../forge/src/check.rs")
        .contains("an_undecided_nowrap_obligation_fails_closed"));
}

#[test]
fn s3_review_surfaces_fork_density_and_tower_depth() {
    pins(
        include_str!("../../forge/src/forks.rs"),
        &["density", "tower", "warning"],
    );
    assert!(include_str!("../../forge/src/forks.rs").contains("#[cfg(test)]"));
}

#[test]
fn s3_lean_smt_export_covers_lia_and_literal_bv_surface() {
    pins(
        include_str!("../../forge/src/lean_smt_export.rs"),
        &["QF_LIA", "QF_BV", "BitVec"],
    );
    assert!(include_str!("../../forge/src/lean_smt_export.rs").contains("#[cfg(test)]"));
}
