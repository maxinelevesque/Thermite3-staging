#!/usr/bin/env python3
"""Run the closed core-L3-lowering claim oracle."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


VERSION = "thermite-claim-closure-lower 1"
ROOT = Path(__file__).resolve().parents[1]


def integration(test_target: str, test_name: str) -> list[str]:
    return [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        "thermite-lower",
        "--test",
        test_target,
        test_name,
        "--",
        "--exact",
    ]


def package_integration(package: str, test_target: str, test_name: str) -> list[str]:
    return [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        package,
        "--test",
        test_target,
        test_name,
        "--",
        "--exact",
    ]


def package_bin_unit(package: str, bin_target: str, test_name: str) -> list[str]:
    return [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        package,
        "--bin",
        bin_target,
        test_name,
        "--",
        "--exact",
    ]


def python_unittest(test_name: str) -> list[str]:
    return [sys.executable, "-m", "unittest", test_name]


CASES = {
    "boundary-dispatch-matches-verified-predicate": integration(
        "boundary_gate_verified", "lower_fn_emits_external_body_iff_proved_predicate"
    ),
    "regular-function-never-external-body": integration(
        "boundary_gate_verified", "regular_fn_is_fully_proved_never_external_body"
    ),
    "boundary-predicate-verifies-no-cheating": [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        "thermite-verified",
        "--test",
        "verus_verify",
        "verified_core_passes_verus_no_cheating",
        "--",
        "--exact",
    ],
    "boundary-true-mutation-fails": [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        "thermite-verified",
        "--test",
        "verus_verify",
        "broken_should_emit_external_body_true_fails",
        "--",
        "--exact",
    ],
    "adt-struct-verifies": integration(
        "adt_lower_conformance", "bank_account_lowers_struct_invariant_and_verifies_l3"
    ),
    "adt-struct-invariant-combinator": integration(
        "adt_lower_conformance", "struct_invariant_combinator_is_emitted_and_runs_at_l1"
    ),
    "invbind-unary-verifies": integration(
        "adt_lower_conformance", "unary_struct_invariant_binds_fields_and_verifies_l3"
    ),
    "invbind-forge-check-battery": package_integration(
        "forge", "struct_invariant_receiver", "check_and_battery_accept_unary_struct_invariant"
    ),
    "adt-enum-verifies": integration(
        "adt_lower_conformance", "shape_lowers_enum_match_is_and_verifies_l3"
    ),
    "adt-recursive-verifies": integration(
        "adt_lower_conformance", "list_sum_lowers_recursive_box_and_verifies_l3"
    ),
    "adt-recursive-predicates-verify": integration(
        "divergence_adt_fold_generality", "adt_predicates_keep_their_declared_bool_return"
    ),
    "adt-error-is-structured": [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        "thermite-lower",
        "--lib",
        "lower::tests::loop_body_is_err_not_silent",
        "--",
        "--exact",
    ],
    "adt-validation-never-panics": package_integration(
        "thermite-spec", "adt_validate", "adt_validation_never_panics"
    ),
    "adt-accepted-programs-validate": package_integration(
        "thermite-spec", "adt_validate", "adt_corpus_programs_validate_clean"
    ),
    "adt-rejects-are-exact": package_integration(
        "thermite-spec", "adt_validate", "adt_reject_cases_yield_exact_error"
    ),
    "scheme-spec-functions-verify": integration(
        "adt_schemes_conformance", "list_fold_lowers_to_generated_schemes_and_verifies_l3"
    ),
    "scheme-induction-law-cited": integration(
        "adt_schemes_conformance",
        "multiplier_instance_cites_the_generated_law_no_fresh_induction",
    ),
    "scheme-induction-premise-load-bearing": integration(
        "adt_schemes_conformance", "negative_control_premise_removed_fails_verus"
    ),
    "scheme-rejects-are-structured": package_integration(
        "thermite-spec", "scheme_validate", "reject_cases_yield_the_oracle_error"
    ),
    "scheme-unsupported-lowering-is-error": integration(
        "l1_conformance", "unsupported_construct_is_err_not_panic"
    ),
    "collections-vec-wrapper-verifies": integration(
        "collections_conformance", "vec_demo_lowers_wrapper_and_verifies_l3"
    ),
    "collections-vec-view-verifies": integration(
        "collections_conformance", "vec_view_supports_spec_indexing_and_combinators"
    ),
    "collections-vec-view-forge-certifies": package_integration(
        "forge", "divergence_collections", "vec_view_index_and_combinator_certify_l3"
    ),
    "collections-unsupported-element-errors": integration(
        "collections_conformance", "unsupported_vec_element_is_structured_error_not_panic"
    ),
    "bytes-eq-defs-and-citation": integration(
        "bytes_eq_conformance", "bytes_eq_demo_emits_def_and_lemmas_and_citation"
    ),
    "bytes-eq-emission-gate": integration(
        "bytes_eq_conformance", "non_bytes_eq_program_does_not_emit_bytes_eq"
    ),
    "bytes-eq-verifies": integration(
        "bytes_eq_conformance", "bytes_eq_demo_verifies_l3_under_real_verus"
    ),
    "bytes-eq-content-mutant-fails": integration(
        "bytes_eq_conformance", "bytes_eq_demo_content_mutant_fails_real_verus"
    ),
    "bytes-eq-l1-twin-runs": integration(
        "divergence_bytes_eq_l1_empty_window",
        "bytes_eq_l1_twin_empty_window_matches_certified_spec_value",
    ),
    "string-byte-builder-certifies": package_integration(
        "forge", "string_format_conformance", "ac6_byte_builder_certifies_l3_alloc"
    ),
    "string-format-roundtrip-certifies": package_integration(
        "forge", "string_format_conformance", "ac7_to_string_round_trip_certifies_l3"
    ),
    "string-format-overclaim-rejected": package_integration(
        "forge", "string_format_conformance", "ac7_overclaimed_round_trip_is_rejected"
    ),
    "string-parse-u64-verifies": package_integration(
        "forge", "option_result_conformance", "ac4_parse_u64_lowering_verifies_under_real_verus"
    ),
    "string-parse-u64-mutant-fails": package_integration(
        "forge", "option_result_conformance", "ac4_broken_parse_u64_body_fails_real_verus"
    ),
    "string-substring-predicates-certify": package_integration(
        "forge", "string_search_conformance", "ac9_predicates_certify_l3_pure"
    ),
    "string-substring-mutant-fails": package_integration(
        "forge", "string_search_conformance", "ac9_broken_starts_with_fails_real_verus"
    ),
    "string-find-certifies": package_integration(
        "forge", "string_search_conformance", "ac10_find_certifies_l3_with_pinned_some"
    ),
    "string-split-verifies": package_integration(
        "forge", "string_search_conformance", "ac11_split_count_bound_verifies_under_real_verus"
    ),
    "string-split-mutant-fails": package_integration(
        "forge", "string_search_conformance", "ac11_broken_split_fails_real_verus"
    ),
    "string-trim-verifies": package_integration(
        "forge", "string_search_conformance", "ac12_trim_verifies_under_real_verus"
    ),
    "string-spec-scanning-certifies": package_integration(
        "forge", "spec_fn_string_param", "string_scanning_spec_fn_lowers_and_certifies_l3"
    ),
    "string-spec-scanning-mutant-rejected": package_integration(
        "forge", "spec_fn_string_param", "broken_scan_body_rejects_below_l3"
    ),
    "optres-option-builtins-certify": package_integration(
        "forge", "option_result_conformance", "ac1_option_construct_payload_in_contract_certifies_l3"
    ),
    "optres-result-builtins-certify": package_integration(
        "forge", "option_result_conformance", "ac2_result_two_arg_type_construct_payload_certifies_l3"
    ),
    "optres-payload-mutant-rejected": package_integration(
        "forge", "option_result_conformance", "ac3_broken_some_under_payload_ens_is_rejected"
    ),
    "mutation-frozen-set-observable": package_bin_unit(
        "forge", "forge", "mutation::tests::frozen_set_and_order_for_small_fn"
    ),
    "mutation-order-is-deterministic": package_bin_unit(
        "forge", "forge", "mutation::tests::generate_is_deterministic"
    ),
    "mutation-order-is-capped": package_bin_unit(
        "forge", "forge", "mutation::tests::capped_at_mutant_cap"
    ),
    "mutation-preserves-contract": package_bin_unit(
        "forge", "forge", "mutation::tests::mutant_keeps_contract_changes_only_body"
    ),
    "mutation-polarity-is-inverted": package_bin_unit(
        "forge", "forge", "mutation::tests::classify_polarity_is_inverted"
    ),
    "mutation-floor-core": package_bin_unit(
        "forge", "forge", "mutation::tests::score_ratio_floor_and_string"
    ),
    "mutation-floor-zero-backstop": package_bin_unit(
        "forge", "forge", "mutation::tests::empty_score_is_below_floor"
    ),
    "mutation-floor-live-reject": package_integration(
        "forge", "mutation_conformance", "reject_fixture_scores_below_floor_and_is_gated_weak_contract"
    ),
    "mutation-floor-configurable": package_integration(
        "forge", "mutation_conformance", "floor_is_configurable_weak_fixture_certifies_under_low_floor"
    ),
    "mutation-cert-fields-graduate": package_bin_unit(
        "forge", "forge", "manifest::tests::with_mutation_score_graduates_fields_and_stays_oracle_excluded"
    ),
    "mutation-cert-fields-reject": package_bin_unit(
        "forge", "forge", "manifest::tests::rejected_weak_contract_carries_cause_ratio_and_survivor"
    ),
    "mutation-live-post-l3-score": package_integration(
        "forge", "mutation_conformance", "accept_fixtures_score_at_or_above_floor_and_certify_l3"
    ),
    "mutation-cache-role-isolated": package_bin_unit(
        "forge", "forge", "cache::tests::auxiliary_query_roles_cannot_alias_the_main_item_keyspace"
    ),
    "mutation-live-ratio-deterministic": package_integration(
        "forge", "mutation_conformance", "kill_ratio_is_deterministic_across_two_runs"
    ),
    "mutation-match-guard-observable": package_bin_unit(
        "forge", "forge", "mutation::tests::match_guard_expression_is_in_mutation_walk"
    ),
    "degrade-proved-stays-l3": package_bin_unit(
        "forge", "forge", "degrade::tests::proved_certifies_l3_no_lower_rung"
    ),
    "degrade-timeout-reaches-l2": package_bin_unit(
        "forge", "forge", "degrade::tests::timeout_then_l2_verified_certifies_l2_degraded"
    ),
    "degrade-underbound-reaches-l1": package_bin_unit(
        "forge", "forge", "degrade::tests::l2_under_bound_drops_to_l1"
    ),
    "degrade-lowered-fields": package_bin_unit(
        "forge", "forge", "degrade::tests::degraded_l2_carries_flag_and_reason"
    ),
    "degrade-l3-counterexample-hard-fails": package_bin_unit(
        "forge", "forge", "degrade::tests::counterexample_never_degrades"
    ),
    "degrade-l2-counterexample-hard-fails": package_bin_unit(
        "forge", "forge", "degrade::tests::l2_counterexample_never_drops_to_l1"
    ),
    "degrade-l2-environment-error-propagates": package_bin_unit(
        "forge", "forge", "degrade::tests::l2_environment_error_is_not_a_degrade"
    ),
    "degrade-l1-environment-error-propagates": package_bin_unit(
        "forge", "forge", "degrade::tests::l1_environment_error_is_not_a_degrade"
    ),
    "degrade-ladder-deterministic": package_bin_unit(
        "forge", "forge", "degrade::tests::ladder_is_deterministic"
    ),
    "degrade-manifest-minimum": package_bin_unit(
        "forge", "forge", "degrade::tests::aggregate_is_min_over_functions"
    ),
    "degrade-manifest-hard-fail": package_bin_unit(
        "forge", "forge", "degrade::tests::hard_fail_caps_project_at_failure"
    ),
    "degrade-manifest-all-l3": package_bin_unit(
        "forge", "forge", "degrade::tests::all_l3_is_project_l3_no_lowering"
    ),
    "degrade-scope-end-to-end": package_bin_unit(
        "forge", "forge", "degrade::tests::aggregate_project_scope_all_end_to_end"
    ),
    "degrade-scope-boundary": package_bin_unit(
        "forge", "forge", "degrade::tests::aggregate_project_scope_any_to_boundary_lists_crossings"
    ),
    "degrade-scope-empty": package_bin_unit(
        "forge", "forge", "degrade::tests::aggregate_project_scope_empty_is_end_to_end"
    ),
    "degrade-cli-assurance-headline": package_integration(
        "forge", "degrade_conformance", "human_output_shows_project_assurance_headline"
    ),
    "degrade-audit-assurance-manifest": package_integration(
        "forge", "audit_conformance", "corpus_empty_tcb"
    ),
    "degrade-audit-deterministic": package_integration(
        "forge", "audit_conformance", "audit_is_deterministic"
    ),
    "audit-schema-v1": package_bin_unit(
        "forge", "forge", "audit::tests::pure_project_has_empty_slag_and_boundary_tcb"
    ),
    "audit-command-json": package_integration(
        "forge", "audit_conformance", "corpus_empty_tcb"
    ),
    "audit-pure-projection": package_bin_unit(
        "forge", "forge", "audit::tests::same_assurance_different_trust_bases_survive_audit_projection"
    ),
    "audit-project-assurance": package_bin_unit(
        "forge", "forge", "audit::tests::lowered_assurance_listed_in_project_section"
    ),
    "audit-deterministic-unit": package_bin_unit(
        "forge", "forge", "audit::tests::manifest_is_deterministic"
    ),
    "audit-deterministic-live": package_integration(
        "forge", "audit_conformance", "audit_is_deterministic"
    ),
    "audit-lean-membership-auto": package_bin_unit(
        "forge", "forge", "audit::tests::probe_pure_int_tail_is_auto"
    ),
    "audit-lean-membership-live": package_integration(
        "forge", "audit_conformance", "lean_fragment_tier_auto"
    ),
    "audit-lean-probe-agrees": package_bin_unit(
        "forge", "forge", "audit::tests::probe_agrees_with_direct_export_item"
    ),
    "audit-lean-probe-no-lake": package_integration(
        "forge", "audit_conformance", "lean_fragment_present_without_lake"
    ),
    "audit-refusal-boundary": package_bin_unit(
        "forge", "forge", "audit::tests::probe_boundary_is_not_pure_contract"
    ),
    "audit-refusal-live": package_integration(
        "forge", "audit_conformance", "lean_fragment_refusal_boundary"
    ),
    "audit-informational-compat-unit": package_bin_unit(
        "forge", "forge", "audit::tests::pre_amendment_v1_deserializes_into_typed_manifest"
    ),
    "audit-informational-compat-live": package_integration(
        "forge", "audit_conformance", "pre_amendment_v1_document_still_deserializes"
    ),
    "build-pipeline-library": package_integration(
        "forge", "build_conformance", "sum_builds_as_library"
    ),
    "build-rustc-hard-fail": package_integration(
        "forge", "build_conformance", "uncompilable_lowering_is_nonzero_exit"
    ),
    "build-artifact-library": package_integration(
        "forge", "build_conformance", "sum_builds_as_library"
    ),
    "build-artifact-entry": package_integration(
        "forge", "build_conformance", "sum_runs"
    ),
    "build-l1-checks-structural": package_integration(
        "forge", "build_conformance", "checks_are_baked_in"
    ),
    "build-l1-checks-runtime": package_integration(
        "forge", "build_conformance", "ens_violation_fires_at_runtime"
    ),
    "build-manifest-shape": package_integration(
        "forge", "build_conformance", "sum_runs"
    ),
    "build-manifest-reproducible": package_integration(
        "forge", "build_conformance", "rebuilt_library_is_byte_identical"
    ),
    "build-entry-sandbox": package_integration(
        "forge", "sandbox_conformance", "pure_runs_clean"
    ),
    "build-out-runnable": package_integration(
        "forge", "build_conformance", "out_places_runnable_binary"
    ),
    "build-out-error": package_integration(
        "forge", "build_conformance", "out_bad_path_is_structured_error"
    ),
    "build-kernel-target": package_integration(
        "forge", "freestanding_target", "pure_fn_builds_no_std_freestanding_rlib"
    ),
    "build-std-target-default": package_integration(
        "forge", "freestanding_target", "default_target_source_is_byte_identical_to_no_target_flag"
    ),
    "build-kernel-nostd-unit": package_bin_unit(
        "forge", "forge", "build::tests::kernel_emit_source_carries_no_std_prelude"
    ),
    "build-kernel-nostd-live": package_integration(
        "forge", "freestanding_target", "pure_fn_builds_no_std_freestanding_rlib"
    ),
    "kernel-fx-read-reject": package_integration(
        "forge", "freestanding_target", "ambient_read_fx_fn_is_refused"
    ),
    "kernel-fx-multi-reject": package_integration(
        "forge", "freestanding_target", "ambient_write_net_term_fx_refuse_identically"
    ),
    "kernel-entry-reject": package_integration(
        "forge", "freestanding_target", "freestanding_target_with_entry_is_usage_error"
    ),
    "kernel-l1-checks-live": package_integration(
        "forge", "freestanding_target", "l1_checks_emitted_verbatim_in_kernel_source"
    ),
    "kernel-l3-check-golden": package_integration(
        "forge", "check_conformance", "sum_cert_matches_golden_deterministic_subset"
    ),
    "kernel-l3-std-default": package_integration(
        "forge", "freestanding_target", "default_target_source_is_byte_identical_to_no_target_flag"
    ),
    "audit-tcb-nonempty": package_integration(
        "forge", "audit_conformance", "slag_boundary_tcb"
    ),
    "manifest-assurance-minimum": package_bin_unit(
        "forge", "forge", "manifest::tests::aggregate_headline_is_min_over_functions"
    ),
    "manifest-assurance-hard-fail": package_bin_unit(
        "forge", "forge", "manifest::tests::aggregate_hard_fail_is_project_failure"
    ),
    "manifest-scope-oracle-visible": package_bin_unit(
        "forge", "forge", "manifest::tests::assurance_scope_is_additive_normalized_and_golden_stable"
    ),
    "manifest-scope-live": package_integration(
        "forge", "composition_basis_conformance", "project_aggregation_is_the_honest_min_over_parts"
    ),
    "manifest-boundary-live": package_integration(
        "forge", "boundary_conformance", "foreign_id_certifies_l1_boundary_not_l3"
    ),
    "manifest-boundary-target-substitution": package_bin_unit(
        "forge", "forge", "manifest::tests::rfc3_coordinates::ffi_artifact_rejects_boundary_target_substitution"
    ),
    "manifest-schema-shape": package_bin_unit(
        "forge", "forge", "manifest::tests::schema_matches_appendix_a"
    ),
    "manifest-produced-fields-shape": package_bin_unit(
        "forge", "forge", "manifest::tests::schema_matches_appendix_a"
    ),
    "manifest-produced-effects": package_bin_unit(
        "forge", "forge", "manifest::tests::effects_of_covers_every_variant"
    ),
    "manifest-forward-declared-oracle": package_bin_unit(
        "forge", "forge", "manifest::tests::oracle_ignores_forward_declared_and_time"
    ),
    "manifest-suggested-move-reserved": package_bin_unit(
        "forge", "forge", "manifest::tests::suggested_move_is_reserved_absence"
    ),
    "manifest-obligation-results": package_bin_unit(
        "forge", "forge", "manifest::tests::obligation_results_present"
    ),
    "manifest-solver-time-excluded": package_bin_unit(
        "forge", "forge", "manifest::tests::oracle_ignores_forward_declared_and_time"
    ),
    "manifest-serde-roundtrip": package_bin_unit(
        "forge", "forge", "manifest::tests::golden_deterministic_subset_round_trips"
    ),
    "manifest-serde-deterministic": package_bin_unit(
        "forge", "forge", "manifest::tests::serialization_is_deterministic"
    ),
    "manifest-slag-meta-live": package_bin_unit(
        "forge", "forge", "manifest::tests::slag_l1_cert_shape"
    ),
    "manifest-reject-reason-live": package_bin_unit(
        "forge", "forge", "manifest::tests::rejected_cert_carries_cause_and_is_not_l3"
    ),
    "manifest-cache-provenance-live": package_bin_unit(
        "forge", "forge", "manifest::tests::cached_field_is_additive_and_oracle_excluded"
    ),
    "manifest-profile-classification": package_bin_unit(
        "forge", "forge", "check::tests::failure_with_profile_report_classifies_as_timeout"
    ),
    "manifest-profile-parser": package_bin_unit(
        "forge", "forge", "profile::tests::parse_profile_hand_derived_fields"
    ),
    "manifest-profile-suggested-move": package_bin_unit(
        "forge", "forge", "profile::tests::suggested_move_is_top_prompt"
    ),
    "check-pipeline-success": package_integration(
        "forge", "check_conformance", "sum_cert_matches_golden_deterministic_subset"
    ),
    "check-pipeline-failure": package_integration(
        "forge", "check_conformance", "broken_contract_is_reported_failure_with_counterexample"
    ),
    "check-scratch-stem": package_bin_unit(
        "forge", "forge", "check::tests::crate_stem_has_no_dot_and_is_valid"
    ),
    "check-scratch-success-cleanup": package_integration(
        "forge", "scratch_cleanup", "success_path_leaves_no_scratch_orphan"
    ),
    "check-scratch-error-cleanup": package_integration(
        "forge", "scratch_cleanup", "error_path_leaves_no_scratch_orphan"
    ),
    "check-exit-success": package_bin_unit(
        "forge", "forge", "check::tests::parseable_success_is_l3_cert"
    ),
    "check-exit-failure": package_bin_unit(
        "forge", "forge", "check::tests::parseable_failure_is_reported_cert_with_counterexample"
    ),
    "check-exit-unparseable": package_bin_unit(
        "forge", "forge", "check::tests::unparseable_output_is_verus_output_error"
    ),
    "check-exit-vir-error": package_bin_unit(
        "forge", "forge", "check::tests::vir_error_is_verus_output_error"
    ),
    "check-obligation-witness": package_bin_unit(
        "forge", "forge", "check::tests::parseable_failure_is_reported_cert_with_counterexample"
    ),
    "check-level-success-failure": package_bin_unit(
        "forge", "forge", "check::tests::verus_assembly_uses_pre_discharge_artifact_on_success_and_failure"
    ),
    "check-verus-absent-cold-cache": package_integration(
        "forge", "cache_conformance", "cold_cache_with_verus_unavailable_is_environment_error"
    ),
    "check-deterministic-golden": package_integration(
        "forge", "check_conformance", "sum_cert_matches_golden_deterministic_subset"
    ),
    "check-vacuity-reject": package_integration(
        "forge", "vacuity_slag_conformance", "triage_rejects_match_oracle_cause"
    ),
    "check-vacuity-pass": package_integration(
        "forge", "vacuity_slag_conformance", "triage_accepts_pass_triage"
    ),
    "check-slag-l1-live": package_integration(
        "forge", "vacuity_slag_conformance", "slag_accepts_certify_l1_slag_true"
    ),
    "check-slag-reject-live": package_integration(
        "forge", "vacuity_slag_conformance", "slag_rejects_match_oracle_cause"
    ),
    "check-boundary-l1-live": package_integration(
        "forge", "boundary_conformance", "foreign_id_certifies_l1_boundary_not_l3"
    ),
    "check-boundary-vacuous-reject": package_integration(
        "forge", "boundary_conformance", "boundary_vacuous_contract_is_rejected"
    ),
    "check-diverge-l1-unit": package_bin_unit(
        "forge", "forge", "check::tests::production_l1_gate_preserves_runtime_route_classification"
    ),
    "check-diverge-l1-live": package_integration(
        "forge", "break_continue_conformance", "diverge_loop_with_break_and_continue_caps_at_l1"
    ),
    "check-diverge-nondiverge-reject": package_integration(
        "forge", "editor_runs", "non_diverge_weak_contract_still_rejects_l0_weakcontract"
    ),
    "check-cache-hit-equal": package_integration(
        "forge", "cache_conformance", "second_run_is_a_cache_hit_with_equal_deterministic_fields"
    ),
    "check-cache-solver-skip": package_integration(
        "forge", "cache_conformance", "cache_hit_serves_l3_with_verus_unavailable"
    ),
    "check-cache-changed-body-miss": package_integration(
        "forge", "cache_conformance", "changed_body_is_a_cache_miss"
    ),
    "check-cache-version-inputs": package_bin_unit(
        "forge", "forge", "cache::tests::key_changes_when_any_input_changes"
    ),
    "cache-round-trip": package_bin_unit(
        "forge", "forge", "cache::tests::round_trip_load_store"
    ),
    "cache-corrupt-miss": package_bin_unit(
        "forge", "forge", "cache::tests::corrupt_entry_is_a_miss"
    ),
    "cache-default-location": package_bin_unit(
        "forge", "forge", "cache::tests::default_cache_dir_is_under_target"
    ),
    "cache-item-locality": package_bin_unit(
        "forge", "forge", "check::tests::cache_key_is_local_to_the_item"
    ),
    "profile-schema-parse": package_bin_unit(
        "forge", "forge", "profile::tests::parse_profile_hand_derived_fields"
    ),
    "profile-trigger-parse": package_bin_unit(
        "forge", "forge", "profile::tests::parse_profile_reconstructs_trigger_from_carets"
    ),
    "profile-render-prompts": package_bin_unit(
        "forge", "forge", "profile::tests::render_prompts_names_bottleneck"
    ),
    "profile-render-deterministic": package_bin_unit(
        "forge", "forge", "profile::tests::render_is_deterministic"
    ),
    "cli-output-stream-discipline": package_integration(
        "forge", "check_conformance", "missing_file_is_usage_error_nonzero"
    ),
    "cli-exit-code-classes": package_bin_unit(
        "forge", "forge", "cli::tests::errors_map_to_environment_exit_code"
    ),
    "cli-empty-l2-fails": package_integration(
        "forge", "check_conformance", "explicit_l2_empty_certificate_array_is_not_success"
    ),
    "cli-command-parses": package_bin_unit(
        "forge", "forge", "cli::tests::parses_new_and_check"
    ),
    "cli-command-usage-errors": package_bin_unit(
        "forge", "forge", "cli::tests::usage_errors"
    ),
    "cli-hand-parser-rlimit": package_bin_unit(
        "forge", "forge", "cli::tests::parses_rlimit_flag"
    ),
    "cli-hand-parser-level": package_bin_unit(
        "forge", "forge", "cli::tests::parses_level_flag"
    ),
    "cli-error-aggregation": package_bin_unit(
        "forge", "forge", "cli::tests::aggregation_preserves_inner_diagnostics"
    ),
    "cli-assurance-headline": package_bin_unit(
        "forge", "forge", "cli::tests::render_assurance_shows_headline_and_lowered_flags"
    ),
    "cli-assurance-failure": package_bin_unit(
        "forge", "forge", "cli::tests::render_assurance_shows_failed_headline"
    ),
    "cli-new-scaffold": package_bin_unit(
        "forge", "forge", "cli::tests::scaffold_writes_layout_and_refuses_clobber"
    ),
    "repair-upgrade-budget": package_bin_unit(
        "forge", "forge", "repair::tests::escalation_upgrades_at_the_proving_budget"
    ),
    "repair-bounded-ladder": package_bin_unit(
        "forge", "forge", "repair::tests::escalation_is_bounded_and_terminates"
    ),
    "repair-counterexample-no-retry": package_bin_unit(
        "forge", "forge", "repair::tests::counterexample_is_never_retried"
    ),
    "repair-reject-no-retry": package_bin_unit(
        "forge", "forge", "repair::tests::rejects_are_never_retried"
    ),
    "repair-classification": package_bin_unit(
        "forge", "forge", "repair::tests::classify_routes_timeout_vs_falsity"
    ),
    "repair-deterministic": package_bin_unit(
        "forge", "forge", "repair::tests::escalation_is_deterministic"
    ),
    "repair-environment-error": package_bin_unit(
        "forge", "forge", "repair::tests::environment_error_propagates"
    ),
    "repair-report-noop": package_integration(
        "forge", "repair_conformance", "corpus_sum_is_a_repair_noop"
    ),
    "repair-report-hard-fail": package_integration(
        "forge", "repair_conformance", "counterexample_is_never_upgraded"
    ),
    "review-spec-layer": package_bin_unit(
        "forge", "forge", "review::tests::sum_intent_reviewable_no_bodies"
    ),
    "review-pre-screen": package_bin_unit(
        "forge", "forge", "review::tests::rejected_fn_flagged_not_surfaced"
    ),
    "review-command-shellout": package_integration(
        "forge", "review_conformance", "reviewer_shellout_attaches_verdict"
    ),
    "review-command-failure": package_integration(
        "forge", "review_conformance", "reviewer_failure_is_error_not_panic"
    ),
    "review-deterministic": package_bin_unit(
        "forge", "forge", "review::tests::artifact_is_deterministic"
    ),
    "review-dual-emission": package_integration(
        "forge", "review_conformance", "corpus_sum_intent_reviewable_no_bodies"
    ),
    "review-intent-prompt": package_bin_unit(
        "forge", "forge", "review::tests::sum_intent_reviewable_no_bodies"
    ),
    "review-match-guard": package_bin_unit(
        "forge", "forge", "review::tests::match_guard_callee_is_in_reviewed_spec_surface"
    ),
    "review-verdict-record": package_bin_unit(
        "forge", "forge", "review::tests::verdict_attaches_to_separate_record"
    ),
    "closure-callgraph-transitive": package_bin_unit(
        "forge", "forge", "closure::tests::transitive_boundary_chain_is_to_boundary"
    ),
    "closure-callgraph-cycle": package_bin_unit(
        "forge", "forge", "closure::tests::mutual_recursion_terminates_and_is_end_to_end"
    ),
    "closure-cert-field": package_bin_unit(
        "forge", "forge", "manifest::tests::assurance_scope_is_additive_normalized_and_golden_stable"
    ),
    "closure-deterministic": package_bin_unit(
        "forge", "forge", "closure::tests::classification_is_deterministic"
    ),
    "closure-reachable-deterministic": package_bin_unit(
        "forge", "forge", "closure::tests::reachable_fns_is_deterministic"
    ),
    "closure-match-guard": package_bin_unit(
        "forge", "forge", "closure::tests::match_guard_call_affects_scope_and_reachability"
    ),
    "closure-project-end-to-end": package_bin_unit(
        "forge", "forge", "degrade::tests::aggregate_project_scope_all_end_to_end"
    ),
    "closure-project-boundary": package_bin_unit(
        "forge", "forge", "degrade::tests::aggregate_project_scope_any_to_boundary_lists_crossings"
    ),
    "closure-scope-pure": package_bin_unit(
        "forge", "forge", "closure::tests::pure_caller_of_spec_fn_is_end_to_end"
    ),
    "closure-scope-boundary": package_bin_unit(
        "forge", "forge", "closure::tests::direct_boundary_caller_is_to_boundary"
    ),
    "closure-scope-slag": package_bin_unit(
        "forge", "forge", "closure::tests::slag_in_closure_is_to_boundary"
    ),
    "manifest-level-ord": package_bin_unit(
        "forge", "forge", "manifest::tests::level_ord_is_the_ladder_ordering"
    ),
    "vacuity-gate-reject": package_integration(
        "forge", "vacuity_slag_conformance", "triage_rejects_match_oracle_cause"
    ),
    "vacuity-gate-accept": package_integration(
        "forge", "vacuity_slag_conformance", "triage_accepts_pass_triage"
    ),
    "vacuity-ens-true": package_bin_unit(
        "forge", "forge", "vacuity::tests::ens_literal_true_rejected_a"
    ),
    "vacuity-ens-identity": package_bin_unit(
        "forge", "forge", "vacuity::tests::ens_identity_rejected_a"
    ),
    "vacuity-ens-omits-result": package_bin_unit(
        "forge", "forge", "vacuity::tests::ens_omits_result_rejected_b"
    ),
    "vacuity-ens-equals-req": package_bin_unit(
        "forge", "forge", "vacuity::tests::ens_eq_req_rejected_c"
    ),
    "vacuity-ens-req-conjunct": package_bin_unit(
        "forge", "forge", "vacuity::tests::ens_conjunct_req_rejected_c"
    ),
    "vacuity-match-guard-result": package_bin_unit(
        "forge", "forge", "vacuity::tests::match_guard_result_mention_passes_b"
    ),
    "vacuity-maximal-reject": package_bin_unit(
        "forge", "forge", "vacuity::tests::maximal_fx_no_slag_rejected_d"
    ),
    "vacuity-maximal-slag": package_bin_unit(
        "forge", "forge", "vacuity::tests::maximal_fx_with_slag_passes_d"
    ),
    "vacuity-slag-still-rejects": package_bin_unit(
        "forge", "forge", "vacuity::tests::slag_does_not_excuse_vacuous_ens"
    ),
    "solver-vacuity-harness": package_bin_unit(
        "forge", "forge", "vacuity_solver::tests::vacuity_harness_assumes_req_asserts_false"
    ),
    "solver-tautology-harness": package_bin_unit(
        "forge", "forge", "vacuity_solver::tests::tautology_harness_reuses_lowered_contract"
    ),
    "solver-verdict-proved": package_bin_unit(
        "forge", "forge", "vacuity_solver::tests::proved_summary_is_detected"
    ),
    "solver-verdict-failed": package_bin_unit(
        "forge", "forge", "vacuity_solver::tests::failed_summary_is_clean"
    ),
    "solver-cause-tags": package_bin_unit(
        "forge", "forge", "vacuity_solver::tests::cause_tags_are_the_solver_namespace"
    ),
    "check-outcome-proved": package_bin_unit(
        "forge", "forge", "check::tests::parseable_success_is_l3_cert"
    ),
    "check-outcome-counterexample": package_bin_unit(
        "forge", "forge", "check::tests::parseable_failure_is_reported_cert_with_counterexample"
    ),
    "check-outcome-timeout": package_bin_unit(
        "forge", "forge", "check::tests::failure_with_profile_report_classifies_as_timeout"
    ),
    "check-profile-live-timeout": package_integration(
        "forge", "profile_conformance", "forced_low_rlimit_timeout_carries_profile_when_emitted"
    ),
    "check-timeout-distinct-counterexample": package_integration(
        "forge", "profile_conformance", "broken_contract_is_counterexample_not_timeout"
    ),
    "check-solver-vacuity-reject": package_integration(
        "forge", "solver_vacuity_conformance", "solver_rejects_match_oracle_cause_and_field"
    ),
    "check-solver-vacuity-clean": package_integration(
        "forge", "solver_vacuity_conformance", "corpus_accepts_pass_both_checks_and_still_certify_l3"
    ),
    "check-solver-vacuity-compile-error": package_bin_unit(
        "forge", "forge", "vacuity_solver::tests::compile_error_summary_is_forge_error_not_clean"
    ),
    "check-solver-vacuity-vir-error": package_bin_unit(
        "forge", "forge", "vacuity_solver::tests::vir_error_is_handled_forge_error_not_clean"
    ),
    "check-solver-vacuity-unparseable": package_bin_unit(
        "forge", "forge", "vacuity_solver::tests::unparseable_output_has_no_summary"
    ),
    "check-ergonomics-guard-deps": package_integration(
        "forge", "ergonomics_conformance", "req3_guarded_match_certifies_l3"
    ),
    "check-ergonomics-or-pattern-deps": package_integration(
        "forge", "ergonomics_conformance", "req4_or_pattern_certifies_l3"
    ),
    "engine-attribution-roundtrip": package_integration(
        "forge", "engine_attribution", "engine_attribution_is_additive_and_round_trips"
    ),
    "engine-attribution-verus-golden": package_integration(
        "forge", "engine_attribution", "engine_verus_flag_is_byte_identical_oracle"
    ),
    "engine-disagreement-alarm": package_bin_unit(
        "forge", "forge", "engine::tests::proven_refuted_disagreement_halts"
    ),
    "engine-disagreement-benign": package_bin_unit(
        "forge", "forge", "engine::tests::proven_unknown_is_benign"
    ),
    "engine-ladder-discipline": package_bin_unit(
        "forge", "forge", "engine::tests::verdict_ladder_action_follows_req3"
    ),
    "engine-proved-map": package_bin_unit(
        "forge", "forge", "engine::tests::proved_maps_to_proven"
    ),
    "engine-timeout-map": package_bin_unit(
        "forge", "forge", "engine::tests::timeout_maps_to_unknown"
    ),
    "engine-fast-unknown": package_bin_unit(
        "forge", "forge", "engine::tests::witnessless_counterexample_remaps_to_unknown"
    ),
    "engine-fast-unknown-narrow": package_bin_unit(
        "forge", "forge", "engine::tests::incompleteness_discriminator_is_narrow"
    ),
    "engine-type-error-refuted": package_bin_unit(
        "forge", "forge", "engine::tests::type_error_counterexample_stays_refuted"
    ),
    "engine-interactive-path": package_bin_unit(
        "forge", "forge", "engine::tests::interactive_proof_path_is_beside_source"
    ),
    "engine-interactive-stale": package_bin_unit(
        "forge", "forge", "engine::tests::interactive_stale_hash_is_unknown_never_reused"
    ),
    "engine-interactive-sorry": package_bin_unit(
        "forge", "forge", "engine::tests::interactive_sorry_file_is_unknown_never_proven"
    ),
    "engine-interactive-filled": package_bin_unit(
        "forge", "forge", "engine::tests::interactive_filled_valid_proof_replays_proven"
    ),
    "engine-interactive-command-token": package_bin_unit(
        "forge", "forge", "engine::tests::proof_term_command_token_scans_position_independently"
    ),
    "engine-interface-verus-slots": package_bin_unit(
        "forge", "forge", "engine::tests::verus_engine_fills_four_slots"
    ),
    "engine-interface-lean-slots": package_bin_unit(
        "forge", "forge", "engine::tests::lean_engine_fills_trust_and_evidence_slots"
    ),
    "engine-interface-golden": package_integration(
        "forge", "engine_interface", "sum_cert_oracle_identical_post_engine_refactor"
    ),
    "engine-lean-mutation-outcome": package_bin_unit(
        "forge", "forge", "engine::tests::lean_mutant_outcome_follows_req9"
    ),
    "engine-lean-mutation-untested": package_bin_unit(
        "forge", "forge", "engine::tests::lean_mutation_tally_does_not_inflate_on_untested"
    ),
    "engine-lean-mutation-floor": package_bin_unit(
        "forge", "forge", "engine::tests::lean_tally_floor_gate"
    ),
    "engine-default-order": package_bin_unit(
        "forge", "forge", "engine::tests::default_engine_order_is_verus_only_and_lean_is_explicit"
    ),
    "engine-explicit-flag": package_integration(
        "forge", "engine_attribution", "engine_flag_parsing"
    ),
    "goal-battery-source": package_bin_unit(
        "forge", "forge", "goal_repl::tests::battery_view_reads_contract_quality"
    ),
    "goal-battery-check": package_integration(
        "forge", "goal_repl", "battery_view_matches_check_verdicts"
    ),
    "goal-deterministic-render": package_bin_unit(
        "forge", "forge", "goal_repl::tests::goal_render_is_deterministic"
    ),
    "goal-structured-error": package_bin_unit(
        "forge", "forge", "goal_repl::tests::edit_bad_address_is_structured_error"
    ),
    "goal-edit-splice": package_integration(
        "forge", "goal_repl", "edit_splices_clause_and_rechecks"
    ),
    "goal-edit-address-error": package_integration(
        "forge", "goal_repl", "edit_bad_address_is_honest_error"
    ),
    "goal-fill-close": package_integration(
        "forge", "goal_repl_fill", "fill_closing_the_hole_certifies_l3"
    ),
    "goal-fill-next": package_integration(
        "forge", "goal_repl_fill", "fill_introducing_new_holes_re_presents_them"
    ),
    "goal-fill-non-hole": package_integration(
        "forge", "goal_repl_fill", "fill_on_a_non_hole_address_is_an_honest_error"
    ),
    "goal-dialogue": package_integration(
        "forge", "goal_repl_fill", "ac6_binary_search_dialogue_structural_oracle"
    ),
    "goal-render-discharged": package_bin_unit(
        "forge", "forge", "goal_repl::tests::goal_render_discharged"
    ),
    "goal-render-counterexample": package_bin_unit(
        "forge", "forge", "goal_repl::tests::goal_render_counterexample"
    ),
    "goal-hole-parser": package_integration(
        "forge", "goal_repl_fill", "fn_body_hole_parses_clean_and_records_the_hole"
    ),
    "goal-hole-address": package_integration(
        "forge", "goal_repl_fill", "hole_address_resolves_and_bad_hole_address_is_structured_error"
    ),
    "goal-hole-order": package_integration(
        "forge", "goal_repl_fill", "holes_in_nested_blocks_are_accepted_in_document_order"
    ),
    "goal-open-hole-reject": package_integration(
        "forge", "goal_repl_fill", "holed_item_never_certifies_open_hole_l0_no_verus"
    ),
    "ci-before-after-metrics": python_unittest(
        "gates.tests.test_ci_workflow_contract.CiWorkflowContractTests."
        "test_before_after_report_separates_live_metrics"
    ),
    "ci-deterministic-lpt": python_unittest(
        "gates.tests.test_ci_test_partitions.PartitionTests."
        "test_lpt_allocation_is_deterministic_and_complete"
    ),
    "ci-thirteen-buckets": python_unittest(
        "gates.tests.test_ci_workflow_contract.CiWorkflowContractTests."
        "test_thirteen_duration_buckets_match_the_manifest"
    ),
    "ci-gate-fanout": python_unittest(
        "gates.tests.test_ci_workflow_contract.CiWorkflowContractTests."
        "test_gate_fanout_and_stable_aggregates_are_closed"
    ),
    "ci-matrix-inventory": package_integration(
        "forge", "verified_build", "parallelized_case_inventories_are_frozen"
    ),
    "ci-coverage-duplicate": python_unittest(
        "gates.tests.test_ci_test_partitions.PartitionTests."
        "test_duplicate_assignment_fails_closed"
    ),
    "ci-coverage-catch-all": python_unittest(
        "gates.tests.test_ci_test_partitions.PartitionTests."
        "test_deleted_assignment_enters_catch_all_and_fails_review_gate"
    ),
    "ci-proof-tool-parity": python_unittest(
        "gates.tests.test_ci_workflow_contract.CiWorkflowContractTests."
        "test_test_partitions_restore_every_required_proof_tool"
    ),
    "ci-separate-landing": python_unittest(
        "gates.tests.test_ci_workflow_contract.CiWorkflowContractTests."
        "test_ci_optimization_landed_after_rfc10_without_rewriting_it"
    ),
    "ci-stable-aggregate-contract": python_unittest(
        "gates.tests.test_ci_workflow_contract.CiWorkflowContractTests."
        "test_gate_fanout_and_stable_aggregates_are_closed"
    ),
    "ci-stable-aggregate-fail-closed": python_unittest(
        "gates.tests.test_ci_aggregate.AggregateTests.test_failure_fails"
    ),
    "ci-timing-artifacts": python_unittest(
        "gates.tests.test_ci_workflow_contract.CiWorkflowContractTests."
        "test_timing_artifacts_are_published_even_on_failure"
    ),
    "ci-timing-status-preserved": python_unittest(
        "gates.tests.test_time_command.TimeCommandTests."
        "test_records_success_and_failure_without_masking_status"
    ),
    "clause-address-parser-overflow": package_integration(
        "thermite-syntax",
        "forge_items",
        "proof_clause_ordinal_overflow_is_rejected_without_truncation",
    ),
    "clause-address-selector-bound": package_bin_unit(
        "forge", "forge", "check::tests::clause_selector_conversion_is_checked_and_item_bound"
    ),
    "clause-aggregation-heterogeneous": package_bin_unit(
        "forge",
        "forge",
        "check::tests::heterogeneous_clause_portfolio_preserves_coordinates_and_rejects_splicing",
    ),
    "clause-aggregation-homogeneous": package_bin_unit(
        "forge",
        "forge",
        "check::tests::homogeneous_portfolio_derives_singular_coordinates_only_on_final_acceptance",
    ),
    "clause-audit-wire-authority": package_bin_unit(
        "forge",
        "forge",
        "check::tests::heterogeneous_clause_portfolio_preserves_coordinates_and_rejects_splicing",
    ),
    "clause-cache-verus-freshness": package_bin_unit(
        "forge",
        "forge",
        "manifest::tests::main_cache_replay_requires_the_fresh_artifact_without_restoring_authority",
    ),
    "clause-evidence-atomic-splice": package_bin_unit(
        "forge",
        "forge",
        "check::tests::heterogeneous_clause_portfolio_preserves_coordinates_and_rejects_splicing",
    ),
    "clause-producer-total-prefix": package_bin_unit(
        "forge",
        "forge",
        "check::tests::ac5_real_mixed_producer_preserves_typed_failures_and_exact_prefix",
    ),
    "clause-producer-two-author": package_bin_unit(
        "forge",
        "forge",
        "check::tests::two_author_clauses_bind_each_proof_and_burn_only_to_its_address",
    ),
    "cache-key-pure": package_bin_unit(
        "forge", "forge", "cache::tests::cache_key_is_pure"
    ),
    "cache-key-all-inputs": package_bin_unit(
        "forge", "forge", "cache::tests::key_changes_when_any_input_changes"
    ),
    "cache-key-boundaries": package_bin_unit(
        "forge", "forge", "cache::tests::length_prefixing_prevents_boundary_collision"
    ),
    "frame-signatures": integration(
        "claim_closure_core", "frame_and_function_signatures_are_observable"
    ),
    "type-lowering": integration("claim_closure_core", "type_lowering_is_observable"),
    "exec-expressions": integration(
        "claim_closure_core", "exec_expression_lowering_is_observable"
    ),
    "statements-loops": integration(
        "claim_closure_core", "statement_and_loop_contracts_are_observable"
    ),
    "spec-seq-views": integration(
        "claim_closure_core", "spec_seq_views_are_observable"
    ),
    "combinator-definitions": integration(
        "claim_closure_core", "discovered_combinator_definitions_are_observable"
    ),
    "proof-aid-bounded-multiply": integration(
        "req_bounded_mul_aid", "sq_emits_req_bounded_mul_aid_with_hand_derived_bound"
    ),
    "proof-aid-renamed-fold": integration(
        "divergence_lower", "divergence_push_lemma_shape_derives_new_specfn_name"
    ),
    "proof-aid-coverage-split": integration(
        "divergence_lower", "divergence_coverage_split_shape_derives_new_names"
    ),
    "golden-sum-verifies": integration("lower_conformance", "sum_emitted_verifies"),
    "golden-binary-search-verifies": integration(
        "lower_conformance", "binary_search_emitted_verifies"
    ),
    "structured-unsupported-error": [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        "thermite-lower",
        "--lib",
        "lower::tests::loop_body_is_err_not_silent",
        "--",
        "--exact",
    ],
    "equivalent-mutant-verifies": integration(
        "equivalence_obligation", "equivalent_early_return_verifies"
    ),
    "distinguishing-mutant-fails": integration(
        "equivalence_obligation", "distinguishing_offbyone_fails"
    ),
    "non-scalar-equivalence-unsupported": integration(
        "equivalence_obligation", "non_scalar_return_is_unsupported"
    ),
}


def tool_version() -> tuple[int, str]:
    verus = os.environ.get("VERUS_BIN") or shutil.which("verus")
    if not verus:
        return 3, ""
    try:
        cargo = subprocess.run(
            ["cargo", "--version"], capture_output=True, text=True, check=True
        ).stdout.strip()
        verus_version = subprocess.run(
            [verus, "--version"], capture_output=True, text=True, check=True
        ).stdout.strip().replace("\n", " ")
    except (OSError, subprocess.CalledProcessError):
        return 3, ""
    return 0, f"{VERSION}; {cargo}; {verus_version}"


def main(argv: list[str]) -> int:
    if argv == ["--version"]:
        status, version = tool_version()
        if status == 0:
            print(version)
        return status
    if len(argv) != 1:
        return 2
    try:
        oracle = json.loads(Path(argv[0]).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return 2
    if not isinstance(oracle, dict) or set(oracle) != {"cases", "probe", "version"}:
        return 2
    if oracle.get("version") != 1 or oracle.get("probe") != "thermite-lower-core":
        return 2
    cases = oracle.get("cases")
    if not isinstance(cases, list) or not cases:
        return 2
    seen: set[str] = set()
    for case in cases:
        if not isinstance(case, dict) or set(case) != {"expected_exit", "id"}:
            return 2
        case_id = case.get("id")
        expected_exit = case.get("expected_exit")
        if (
            not isinstance(case_id, str)
            or case_id not in CASES
            or case_id in seen
            or not isinstance(expected_exit, int)
        ):
            return 4
        seen.add(case_id)
        try:
            result = subprocess.run(
                CASES[case_id],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
                timeout=180,
            )
        except (OSError, subprocess.TimeoutExpired):
            return 3
        if result.returncode != expected_exit:
            return 4
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
