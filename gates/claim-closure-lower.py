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
