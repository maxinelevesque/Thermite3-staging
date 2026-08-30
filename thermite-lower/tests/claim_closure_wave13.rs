fn assert_pins(haystack: &str, pins: &[&str]) {
    for pin in pins {
        assert!(haystack.contains(pin), "missing claim pin {pin}");
    }
}

#[test]
fn l1_dec_scope_is_runtime_honest() {
    let source = include_str!("../src/l1.rs");
    assert_pins(
        source,
        &[
            "fn lower_loop_l1_with_provider",
            "thermite_check!",
            "invariant",
        ],
    );
    let tests = include_str!("l1_conformance.rs");
    assert_pins(
        tests,
        &[
            "no_syscall_sandbox_and_no_dec_guarantee",
            "!emitted.contains(\"thermite_check!(\\\"dec\\\"\")",
        ],
    );
}

#[test]
fn l1_effect_scope_stays_compile_time_only() {
    let source = include_str!("../src/l1.rs");
    assert_pins(source, &["pub fn lower_l1", "Result<String, LowerError>"]);
    let tests = include_str!("l1_conformance.rs");
    assert_pins(
        tests,
        &[
            "no_syscall_sandbox_and_no_dec_guarantee",
            "forbidden in [\"syscall\", \"sandbox\", \"seccomp\", \"fx pure\"]",
        ],
    );
}

#[test]
fn l1_enum_match_and_is_have_plain_rust_lowering() {
    let source = include_str!("../src/l1.rs");
    assert_pins(
        source,
        &[
            "fn lower_enum_l1",
            "fn lower_match_exec",
            "fn lower_pattern_exec",
            "matches!",
        ],
    );
    assert!(include_str!("adt_lower_conformance.rs").contains("shape_l1_compiles_and_runs"));
}

#[test]
fn l1_ergonomic_desugars_use_existing_runtime_nodes() {
    let source = include_str!("../src/l1.rs");
    assert_pins(
        source,
        &[
            "fn lower_match_exec",
            "fn lower_loop_l1_with_provider",
            "Expr::TupleProj",
            "Expr::Is",
        ],
    );
    assert!(include_str!("../../forge/tests/ergonomics_conformance.rs")
        .contains("req5_while_let_desugars_to_while_is_variant"));
}

#[test]
fn l1_errors_are_structured_not_toolchain_panics() {
    let source = include_str!("../src/l1.rs");
    assert_pins(source, &["pub fn lower_l1", "Result<String, LowerError>"]);
    assert!(include_str!("l1_conformance.rs").contains("unsupported_construct_is_err_not_panic"));
}

#[test]
fn l1_golden_runs_and_its_negative_contract_fires() {
    let golden = include_str!("../../tests/golden/l1/sum.l1.rs");
    assert_pins(golden, &["thermite_check!", "fn sum"]);
    let tests = include_str!("l1_conformance.rs");
    assert_pins(
        tests,
        &[
            "sum_l1_compiles_and_runs",
            "negative_fixture_fires_violation",
        ],
    );
}

#[test]
fn l1_match_guards_are_emitted_and_walked() {
    let source = include_str!("../src/l1.rs");
    assert_pins(
        source,
        &[
            "fn lower_match_exec",
            "arm.guard",
            "collect_combinators_in_expr",
            "rename_params_in_expr",
        ],
    );
    assert!(include_str!("../../forge/tests/ergonomics_conformance.rs")
        .contains("req3_guarded_match_certifies_l3"));
}

#[test]
fn l1_or_patterns_emit_native_alternatives() {
    let source = include_str!("../src/l1.rs");
    assert_pins(
        source,
        &[
            "fn lower_pattern_exec",
            "Pattern::Or(alts)",
            "join(\" | \")",
        ],
    );
    assert!(include_str!("../../forge/tests/ergonomics_conformance.rs")
        .contains("req4_or_pattern_certifies_l3"));
}

#[test]
fn l1_recursive_adts_emit_box_and_deref() {
    let source = include_str!("../src/l1.rs");
    assert_pins(
        source,
        &["pub(crate) fn lower_type", "Box<", "Expr::Deref(inner)"],
    );
    assert!(include_str!("adt_lower_conformance.rs")
        .contains("list_sum_lowers_recursive_box_and_verifies_l3"));
}

#[test]
fn l1_string_parse_and_vec_runtime_twins_are_present() {
    let source = include_str!("../src/l1.rs");
    assert_pins(
        source,
        &[
            "fn emit_string_runtime_l1",
            "fn emit_vec_runtime_l1",
            "fn parse_u64(s: TString) -> Option<u64>",
        ],
    );
    assert!(include_str!("../../forge/tests/acceptance_programs.rs")
        .contains("calculator_string_parse_builds_and_runs_end_to_end"));
}

#[test]
fn l1_spec_functions_have_executable_lowering() {
    let source = include_str!("../src/l1.rs");
    assert_pins(
        source,
        &["pub(crate) fn lower_spec_fn_l1", "fn slice_fold_body_l1"],
    );
    assert!(include_str!("l1_conformance.rs").contains("sum_l1_compiles_and_runs"));
}

#[test]
fn l1_struct_invariants_are_always_active() {
    let source = include_str!("../src/l1.rs");
    assert_pins(
        source,
        &[
            "fn lower_struct_l1",
            "well_formed",
            "invariant_struct_names",
            "thermite_check!",
        ],
    );
    assert!(include_str!("adt_lower_conformance.rs")
        .contains("struct_invariant_combinator_is_emitted_and_runs_at_l1"));
}

#[test]
fn l2_bound_string_states_bounded_assurance() {
    let source = include_str!("../src/l2.rs");
    assert_pins(
        source,
        &[
            "pub fn bound_string",
            "slice <= {SLICE_BOUND}, unwind {unwind}",
            "bound_string_states_the_caveat",
        ],
    );
}

#[test]
fn l2_lowering_is_deterministic_by_construction_and_test() {
    let source = include_str!("../src/l2.rs");
    assert_pins(source, &["pub(crate) const SLICE_BOUND", "fn unwind_bound"]);
    assert!(include_str!("l2_conformance.rs").contains("lowering_is_deterministic"));
    for forbidden in ["SystemTime", "thread_rng", "rand::"] {
        assert!(
            !source.contains(forbidden),
            "nondeterministic L2 input {forbidden}"
        );
    }
}

#[test]
fn l2_ergonomics_reuses_l1_after_desugaring() {
    let source = include_str!("../src/l2.rs");
    assert_pins(
        source,
        &["lower_spec_fn_l1", "lower_fn_body_exec", "pub fn lower_l2"],
    );
    assert!(!source.contains("Pattern::Or"));
    assert!(!source.contains("MatchArm"));
}

#[test]
fn l2_errors_are_structured_not_panics() {
    let source = include_str!("../src/l2.rs");
    assert_pins(source, &["pub fn lower_l2", "Result<String, LowerError>"]);
    assert!(include_str!("l2_conformance.rs").contains("unlowerable_is_err_not_panic"));
}

#[test]
fn l2_emits_per_function_kani_harnesses() {
    let source = include_str!("../src/l2.rs");
    assert_pins(
        source,
        &[
            "pub fn lower_l2",
            "fn emit_harness",
            "#[kani::proof]",
            "kani::assume",
        ],
    );
    assert!(include_str!("l2_conformance.rs").contains("sum_harness_verifies_to_bound"));
}

#[test]
fn l2_symbolic_inputs_are_type_driven() {
    let source = include_str!("../src/l2.rs");
    assert_pins(
        source,
        &[
            "fn infer_symbolic_input",
            "SLICE_BOUND",
            "kani::any()",
            "len <= N",
        ],
    );
    assert!(include_str!("l2_conformance.rs").contains("bound_is_type_derived_not_name_derived"));
}

#[test]
fn l2_unwind_bounds_follow_loop_shape() {
    let source = include_str!("../src/l2.rs");
    assert_pins(
        source,
        &[
            "fn unwind_bound",
            "fn has_unconditional_loop",
            "SLICE_BOUND + 1",
            "SLICE_BOUND + 2",
            "unwind_bound_is_shape_keyed",
        ],
    );
}

#[test]
fn map_unsupported_shapes_return_lower_error() {
    let source = include_str!("../src/lower.rs");
    assert_pins(
        source,
        &[
            "pub(crate) fn tmap_name",
            "fn tmap_type_suffix",
            "Result<String, LowerError>",
        ],
    );
    assert!(include_str!("../../gates/anti-pattern-gate.py").contains("panic"));
}

#[test]
fn map_remove_returns_prior_value_and_preserves_absence() {
    let source = include_str!("../src/lower.rs");
    assert_pins(
        source,
        &[
            "pub fn remove(&mut self, k:",
            "result: Option<",
            "self.data.remove(i)",
            "return Some",
        ],
    );
    assert!(include_str!("../../forge/tests/map_conformance.rs")
        .contains("ac1_2_3_map_wrapper_roundtrip_and_absent_none_verify_l3"));
}

#[test]
fn map_type_ripples_through_l3_l1_and_consumers() {
    let lower = include_str!("../src/lower.rs");
    let l1 = include_str!("../src/l1.rs");
    assert_pins(
        lower,
        &[
            "Type::Map",
            "fn emit_map_wrappers",
            "pub(crate) fn tmap_name",
        ],
    );
    assert_pins(l1, &["Type::Map", "fn emit_map_runtime_l1"]);
    assert!(include_str!("../../forge/src/check.rs").contains("Type::Map"));
    assert!(include_str!("../../thermite-skill/src/generate.rs").contains("Type::Map"));
}

#[test]
fn map_bounded_traversal_has_checked_index_accessors() {
    let source = include_str!("../src/lower.rs");
    assert_pins(
        source,
        &[
            "pub fn key_at",
            "pub fn value_at",
            "spec_key_at",
            "spec_value_at",
            "requires i < self.data.len()",
        ],
    );
    assert!(include_str!("../../forge/tests/map_conformance.rs")
        .contains("ac1_map_kv_corpus_lowering_verifies_under_real_verus"));
}

#[test]
fn map_wrapper_is_bounded_vec_of_pairs_with_full_surface() {
    let source = include_str!("../src/lower.rs");
    assert_pins(
        source,
        &[
            "fn emit_map_wrappers",
            "vstd::vec::Vec<(",
            "contains_key",
            "pub fn insert",
            "pub fn remove",
            "pub fn key_at",
            "pub fn value_at",
        ],
    );
    assert!(include_str!("../../forge/tests/map_conformance.rs")
        .contains("ac1_map_kv_builds_and_runs_insert_get_yields_value"));
}

#[test]
fn option_result_parse_emission_is_gated_and_verified() {
    let source = include_str!("../src/lower.rs");
    assert_pins(
        source,
        &[
            "pub(crate) fn program_uses_parse",
            "fn emit_parse_defs",
            "pub fn parse_u64(s: &TString) -> (result: Option<u64>)",
        ],
    );
    assert!(
        include_str!("../../forge/tests/option_result_conformance.rs")
            .contains("ac4_parse_u64_lowering_verifies_under_real_verus")
    );
}
