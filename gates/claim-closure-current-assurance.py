#!/usr/bin/env python3
"""Discriminate the production current-assurance authority migration."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


VERSION = "thermite-claim-closure-current-assurance 1"
PROBE = "current-assurance-authority"
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
    "current-assurance-level-inventory": [
        sys.executable,
        "gates/assurance-level-inventory.py",
        "--check",
    ],
    "current-assurance-level-is-presentation": forge_unit(
        "manifest::tests::current_assurance::compatibility_level_edit_cannot_change_current_authority"
    ),
    "current-assurance-envelope": forge_unit(
        "manifest::tests::current_assurance::current_envelope_rejects_compatibility_level_and_unknown_schema"
    ),
    "current-assurance-historical-inspection": forge_unit(
        "manifest::tests::current_assurance::unversioned_levels_are_inspect_only_with_specific_warnings"
    ),
    "current-assurance-portfolio-splice": forge_unit(
        "check::tests::heterogeneous_clause_portfolio_preserves_coordinates_and_rejects_splicing"
    ),
    "current-assurance-persisted-artifact-replay": forge_unit(
        "manifest::tests::rfc3_coordinates::main_cache_replay_requires_the_fresh_artifact_without_restoring_authority"
    ),
    "current-assurance-audit-presentation": forge_unit(
        "audit::tests::migrated_l1_audit_derives_presentation_after_legacy_level_substitution"
    ),
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
