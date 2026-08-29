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
