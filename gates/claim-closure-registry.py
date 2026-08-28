#!/usr/bin/env python3
"""Run the closed registry/control-plane claim oracle."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


VERSION = "thermite-claim-closure-registry 1"
ROOT = Path(__file__).resolve().parents[1]

TEST_PREFIX = "gates.tests."
CASES = {
    "live-registry-clean": [sys.executable, "gates/req-registry.py", "--check"],
    "live-legacy-status-clean": [sys.executable, "gates/req-status.py"],
    "stable-id-duplicate-rejected": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_rejects_duplicate_requirement_id",
    ],
    "undeclared-status-rejected": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_rejects_unknown_status",
    ],
    "unresolved-file-evidence-rejected": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_rejects_unresolved_file_evidence",
    ],
    "blocked-status-requires-typed-blocker": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_blocked_requires_issue_blocker",
    ],
    "unresolved-command-evidence-rejected": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_command_evidence_rejects_unresolved_executable",
    ],
    "closed-live-blocker-rejected": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_live_issue_adapter_rejects_closed_github_blocker",
    ],
    "generated-view-round-trip": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_valid_registry_writes_and_checks_generated_view",
    ],
    "stale-generated-view-rejected": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_check_detects_stale_generated_view",
    ],
    "legacy-conflict-tripwire": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_status.ReqStatusOracleTest.test_exact_req_label_conflict_fails",
    ],
    "legacy-inventory-normalized": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_status.ReqStatusOracleTest.test_json_inventory_is_normalized",
    ],
    "generated-reference-region-round-trip": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_reference_list_writes_rust_doc_comment_region",
    ],
    "generated-replacement-satisfies-mapping": [
        sys.executable,
        "-m",
        "unittest",
        f"{TEST_PREFIX}test_req_registry.ReqRegistryOracleTest.test_legacy_mapping_accepts_generated_replacement_region",
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
    if oracle.get("version") != 1 or oracle.get("probe") != "req-registry-control-plane":
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
                timeout=60,
            )
        except (OSError, subprocess.TimeoutExpired):
            return 3
        if result.returncode != expected_exit:
            return 4
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
