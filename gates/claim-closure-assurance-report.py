#!/usr/bin/env python3
"""Discriminate the layered engineer assurance-report claims."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


VERSION = "thermite-claim-closure-assurance-report 1"
PROBE = "assurance-layered-report"
CASE_TIMEOUT_SECONDS = 360
ROOT = Path(__file__).resolve().parents[1]


def forge_unit(test_name: str) -> list[str]:
    return [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        "forge",
        "--bin",
        "forge",
        test_name,
        "--",
        "--exact",
    ]


CASES = {
    "assurance-report-mixed-live": [
        sys.executable,
        "gates/assurance-report-smoke.py",
    ],
    "assurance-report-layers": forge_unit(
        "assurance_report::tests::one_validated_object_drives_all_disclosure_layers_deterministically"
    ),
    "assurance-report-declaration-population": forge_unit(
        "assurance_report::tests::source_population_includes_certified_effect_and_protocol_declarations"
    ),
    "assurance-report-tamper": forge_unit(
        "assurance_report::tests::population_and_presentation_tampering_fail_validation"
    ),
    "assurance-report-comparison": forge_unit(
        "assurance_report::tests::exact_base_comparison_separates_formal_movements"
    ),
    "assurance-report-movement-vocabulary": forge_unit(
        "assurance_report::tests::comparison_vocabulary_keeps_formal_changes_separate"
    ),
    "assurance-report-historical-policy": forge_unit(
        "assurance_report::tests::historical_and_policy_rejected_portraits_remain_loud_nonclaims"
    ),
    "assurance-report-html": forge_unit(
        "assurance_report::tests::html_escapes_source_controlled_content_and_carries_restrictive_csp"
    ),
    "assurance-report-bounded-json": forge_unit(
        "assurance_report::tests::diagnostic_json_is_bounded_and_rejects_unknown_report_fields"
    ),
    "assurance-report-live-floor": forge_unit(
        "assurance_report::tests::floors_consume_only_live_common_frontier_and_exact_fiber"
    ),
    "assurance-report-ci-contract": [
        sys.executable,
        "-m",
        "unittest",
        "gates.tests.test_ci_workflow_contract",
    ],
    "assurance-report-skill-fresh": [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        "thermite-skill",
        "--test",
        "skill",
        "committed_skill_is_fresh",
        "--",
        "--exact",
    ],
}


def tool_version() -> tuple[int, str]:
    try:
        cargo = subprocess.run(
            ["cargo", "--version"],
            capture_output=True,
            text=True,
            check=True,
            timeout=30,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return 3, ""
    return 0, f"{VERSION}; {cargo}"


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
    if oracle.get("version") != 1 or oracle.get("probe") != PROBE:
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
                timeout=CASE_TIMEOUT_SECONDS,
            )
        except (OSError, subprocess.TimeoutExpired):
            return 3
        if result.returncode != expected_exit:
            return 4
    return 0 if seen == set(CASES) else 4


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
