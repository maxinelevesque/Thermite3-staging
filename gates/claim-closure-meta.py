#!/usr/bin/env python3
"""Run the closed self-governance oracle for claim-closure activation."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


VERSION = "thermite-claim-closure-meta 1"
ROOT = Path(__file__).resolve().parents[1]
TEST_PREFIX = "gates.tests."

CASES = {
    "typed-claim-authority": [
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_shipped_requirement_requires_typed_claim",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_rejects_unknown_typed_claim_kind",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_summary_drift_does_not_change_authoritative_claim_digest",
    ],
    "exact-live-population": [
        f"{TEST_PREFIX}test_claim_closure_author.ClaimClosureAuthorTests.test_exact_population_draft_includes_live_addition_at_materialization",
        f"{TEST_PREFIX}test_completeness_review.ReviewTrackTests.test_missing_shipped_closure_fails",
        f"{TEST_PREFIX}test_completeness_review.ReviewTrackTests.test_v1_staging_cannot_predeclare_v2_claims_or_closures",
    ],
    "closed-witness-mechanisms": [
        f"{TEST_PREFIX}test_claim_closure_red.KnownRedCorpusTests.test_each_semantic_policy_inversion_is_rejected",
        f"{TEST_PREFIX}test_completeness_review.ReviewTrackTests.test_executable_discriminator_runs_same_verifier_on_mutated_oracle",
        f"{TEST_PREFIX}test_completeness_review.ReviewTrackTests.test_exact_population_observation_comes_from_bound_artifact",
        f"{TEST_PREFIX}test_completeness_review.ReviewTrackTests.test_formal_probe_binds_the_kernel_reported_type",
    ],
    "shared-witness-discrimination": [
        f"{TEST_PREFIX}test_completeness_review.ReviewTrackTests.test_shared_witness_membership_is_exact",
    ],
    "content-bound-receipts": [
        f"{TEST_PREFIX}test_completeness_review.ReviewTrackTests.test_stale_receipt_fails",
        f"{TEST_PREFIX}test_claim_closure_author.ClaimClosureAuthorTests.test_executable_draft_round_trips_through_authoritative_gate",
    ],
}


def main(argv: list[str]) -> int:
    if argv == ["--version"]:
        print(
            f"{VERSION}; Python "
            f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}"
        )
        return 0
    if len(argv) != 1:
        return 2
    try:
        oracle = json.loads(Path(argv[0]).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return 2
    if not isinstance(oracle, dict) or set(oracle) != {"cases", "probe", "version"}:
        return 2
    if oracle.get("version") != 1 or oracle.get("probe") != "claim-closure-self-governance":
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
        result = subprocess.run(
            [sys.executable, "-m", "unittest", *CASES[case_id]],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
            timeout=120,
        )
        if result.returncode != expected_exit:
            return 4
    if seen != set(CASES):
        return 4
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
