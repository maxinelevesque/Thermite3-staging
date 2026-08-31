fn pins(text: &str, expected: &[&str]) {
    for pin in expected {
        assert!(text.contains(pin), "missing claim pin {pin}");
    }
}

#[test]
fn option_result_types_are_native_and_conformant() {
    let lower = include_str!("../src/lower.rs");
    pins(
        lower,
        &["Type::Option", "Type::Result", "fn qualify_variant_path"],
    );
    pins(
        include_str!("../../forge/tests/option_result_conformance.rs"),
        &[
            "ac1_option_construct_payload_in_contract_certifies_l3",
            "ac2_result_two_arg_type_construct_payload_certifies_l3",
        ],
    );
}

#[test]
fn recursive_fn_decreases_is_emitted() {
    pins(
        include_str!("../src/lower.rs"),
        &["fn lower_fn(", "fn fn_is_diverge", "decreases"],
    );
    assert!(include_str!("../../forge/tests/recursion_conformance.rs")
        .contains("recursive_fn_with_dec_certifies_l3"));
}

#[test]
fn recursive_termination_has_positive_and_negative_teeth() {
    assert!(include_str!("../../thermite-spec/src/validator.rs").contains("MissingDecreases"));
    pins(
        include_str!("../../forge/tests/recursion_conformance.rs"),
        &[
            "nondecreasing_recursion_is_l0",
            "self_call_without_dec_is_structured_error",
            "diverge_recursion_without_dec_is_l1",
        ],
    );
}

#[test]
fn vec_element_wrappers_are_woven_inner_first() {
    pins(
        include_str!("../src/lower.rs"),
        &[
            "fn collect_vec_elem_types",
            "fn note_vec_elems",
            "fn note_block_vec_elems",
        ],
    );
    assert!(
        include_str!("../../forge/tests/vec_completeness_conformance.rs")
            .contains("vec_nested_borrow_get_certifies_l3")
    );
}

#[test]
fn vec_contract_method_cage_matches_emission() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &["const BUILTIN_METHODS", "\"last\"", "\"contains\""],
    );
    pins(
        include_str!("../src/lower.rs"),
        &["fn emit_one_vec_wrapper", "pub fn last", "pub fn contains"],
    );
}

#[test]
fn local_vec_new_reaches_wrapper_emission() {
    pins(
        include_str!("../src/lower.rs"),
        &["fn is_vec_new", "fn note_block_vec_elems"],
    );
    assert!(
        include_str!("../../forge/tests/vec_completeness_conformance.rs")
            .contains("local_vec_new_no_param_certifies_l3")
    );
}

#[test]
fn noncopy_vec_elements_use_borrowed_access() {
    pins(
        include_str!("../src/lower.rs"),
        &["fn elem_is_copy", "pub fn get(&self"],
    );
    pins(
        include_str!("../../forge/tests/vec_completeness_conformance.rs"),
        &[
            "vec_string_borrow_get_certifies_l3",
            "vec_struct_borrow_get_certifies_l3",
            "vec_nested_borrow_get_certifies_l3",
        ],
    );
}

#[test]
fn vec_wrapper_has_tuple_free_complete_ops() {
    pins(
        include_str!("../src/lower.rs"),
        &[
            "pub fn pop_last",
            "pub fn last",
            "pub fn insert",
            "pub fn remove",
            "pub fn contains",
        ],
    );
    assert!(
        include_str!("../../forge/tests/vec_completeness_conformance.rs")
            .contains("vec_u64_ops_certify_l3")
    );
}

#[test]
fn rfc_frontmatter_schema_is_normative_and_parsed() {
    pins(
        include_str!("../../gates/rfc-check.py"),
        &["def parse_front_matter", "REQUIRED =", "STATUSES ="],
    );
    pins(
        include_str!("../../.design/rfcs/0005-rfc-process.md"),
        &["rfc:", "title:", "status:", "introduces:"],
    );
}

#[test]
fn rfc_gate_rejects_each_malformed_class() {
    pins(
        include_str!("../../gates/rfc-check.py"),
        &[
            "def check",
            "front matter is missing",
            "disagrees with the filename prefix",
            "is already used by",
        ],
    );
    pins(
        include_str!("../../gates/tests/test_rfc_check.py"),
        &[
            "test_missing_front_matter_is_named",
            "test_implemented_is_not_a_status",
            "test_number_must_match_filename",
            "test_two_canonical_rfcs_may_not_share",
        ],
    );
}

#[test]
fn rfc_introduces_resolves_against_registry() {
    pins(
        include_str!("../../gates/rfc-check.py"),
        &[
            "def known_reqs",
            "introduces",
            "which is not in registry.toml",
        ],
    );
    assert!(include_str!("../../gates/tests/test_rfc_check.py")
        .contains("test_unknown_introduced_req_is_rejected"));
}

#[test]
fn whole_program_backends_converge_on_checked_ir() {
    pins(
        include_str!("checked_program.rs"),
        &["compatibility_lowering_routes_reject_the_same_invalid_program"],
    );
    pins(
        include_str!("../../forge/src/check.rs"),
        &["thermite_spec::validate", "run_rfc10_lean_replay"],
    );
}

#[test]
fn canonical_children_have_stable_ids_and_one_inventory() {
    pins(
        include_str!("../../thermite-syntax/src/semantic.rs"),
        &[
            "pub struct NodeId",
            "pub enum SemanticFact",
            "pub enum ChildRole",
            "fn children",
        ],
    );
    assert!(
        include_str!("../../thermite-syntax/tests/semantic_inventory.rs").contains(
            "canonical_inventory_covers_conditions_guards_patterns_and_expression_blocks"
        )
    );
}

#[test]
fn checked_ir_binds_all_rfc10_facts_once() {
    pins(
        include_str!("../src/checked.rs"),
        &[
            "pub struct CheckedProgram",
            "pub fn check_program",
            "Construction is all-or-nothing",
        ],
    );
    pins(
        include_str!("checked_program.rs"),
        &[
            "checked_program_binds_inventory_regions_and_effects_once",
            "checked_holdings_carry_regions_capabilities_transitions_and_close_edges",
        ],
    );
}

#[test]
fn rfc10_conformance_matrix_crosses_all_positions_and_phases() {
    pins(
        include_str!("rfc10_conformance_matrix.rs"),
        &[
            "generated_holding_position_matrix_agrees_across_phases",
            "deep_finite_expression_payload_reaches_every_position",
            "generated_negative_payloads_fail_in_every_compatible_position",
        ],
    );
    assert!(include_str!("../../forge/tests/check_conformance.rs")
        .contains("generated_rfc10_positions_reach_provider_free_forge_check"));
}

#[test]
fn rfc10_delta_and_residual_trust_are_explicit() {
    pins(
        include_str!("../../.design/rfc10-checked-traversal.md"),
        &[
            "Language and assurance delta ledger",
            "Backend completeness",
            "Resource behavior",
            "Residual trust",
        ],
    );
}

#[test]
fn rfc10_evidence_has_kernel_checked_completeness() {
    pins(
        include_str!("../../lean/Thermite/CheckedTraversal.lean"),
        &[
            "theorem verify_complete",
            "theorem verify_iff_supported",
            "theorem produce_complete",
        ],
    );
    assert!(include_str!("traversal_witness.rs")
        .contains("complete_current_inventory_matrix_uses_the_universal_lean_theorem"));
}

#[test]
fn semantic_traversal_is_iterative_and_resource_bounded() {
    pins(
        include_str!("../../thermite-syntax/src/semantic.rs"),
        &[
            "pub struct ResourceLimit",
            "let mut stack",
            "return Err(ResourceLimit",
        ],
    );
    pins(
        include_str!("../../thermite-syntax/tests/semantic_inventory.rs"),
        &[
            "resource_exhaustion_is_structured_and_non_accepting",
            "deep_finite_expression_walk_uses_no_native_recursion",
        ],
    );
}

#[test]
fn every_rfc10_derived_field_has_mutation_controls() {
    pins(
        include_str!("traversal_witness.rs"),
        &[
            "independently_mutated_evidence_is_rejected_by_named_field",
            "lean_rejects_specific_condition_and_match_guard_child_omissions",
        ],
    );
    assert!(include_str!("../../forge/src/check.rs")
        .contains("Lean must reject an omitted canonical call edge"));
}

#[test]
fn lean_derives_semantics_from_neutral_canonical_facts() {
    pins(
        include_str!("../../lean/Thermite/CheckedTraversal.lean"),
        &[
            "def deriveSemantics",
            "def derivedHoldings",
            "def derivedSharedPlaces",
        ],
    );
    pins(
        include_str!("../src/witness.rs"),
        &["CanonicalAstProjection", "canonical_ast_projection"],
    );
}

#[test]
fn holding_semantics_are_uniform_across_executable_blocks() {
    pins(
        include_str!("shared_state_invariants.rs"),
        &[
            "expression_nested_holding_is_inferred_and_closed_at_l3",
            "statement_conditions_obey_holding_inference_and_discipline",
            "executable_holding_closes_once_on_all_rust_exit_kinds",
        ],
    );
}

#[test]
fn l3_requires_kernel_verified_rfc10_replay() {
    pins(
        include_str!("../../forge/src/check.rs"),
        &[
            "run_rfc10_lean_replay",
            "RFC-10 canonical AST projection failed",
        ],
    );
    pins(
        include_str!("../../forge/tests/check_conformance.rs"),
        &[
            "rfc10_shared_state_certifies_through_the_production_route",
            "every_root_corpus_item_preserves_its_frozen_certification_level",
        ],
    );
}

#[test]
fn rust_witness_is_deterministic_and_source_bound() {
    pins(
        include_str!("../src/witness.rs"),
        &[
            "pub struct TraversalWitness",
            "pub fn canonical_json",
            "canonical_ast_sha256",
            "pub fn emit_witness",
        ],
    );
    pins(
        include_str!("traversal_witness.rs"),
        &[
            "witness_is_deterministic_json_and_replays_against_source",
            "witness_is_bound_to_literal_and_binding_changes_not_only_tree_shape",
        ],
    );
}

#[test]
fn stage1_certificate_vocabulary_is_closed_and_total() {
    pins(
        include_str!("../../forge/src/verdict.rs"),
        &[
            "pub enum CertVerdict",
            "pub fn from_engine_verdict",
            "fn all_seven_variants_round_trip",
        ],
    );
    assert!(include_str!("../../forge/src/tv_signal.rs").contains("is_kernel_budget_signal"));
}

#[test]
fn stage1_normative_governance_deliverables_are_present() {
    pins(
        include_str!("../../docs/v2/semantics.md"),
        &[
            "never-converts-silently",
            "covenant-before-burn",
            "The assurance ladder",
        ],
    );
    pins(
        include_str!("../../goal.md"),
        &["R-VERDICT-1", "R-COV-1", "R-GATE-1", "R-SIDE-1", "R-BV-1"],
    );
    pins(
        include_str!("../../forge/src/covenant_engine.rs"),
        &["covenant_gate", "covenant-before-burn"],
    );
}
