#!/usr/bin/env python3
"""Discriminate exact project-assurance composition claims."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


VERSION = "thermite-claim-closure-assurance-composition 1"
PROBE = "assurance-project-composition"
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
    "assurance-project-population": forge_unit(
        "assurance_v2::tests::project_population_is_an_exact_total_partition"
    ),
    "assurance-portfolio-lift": forge_unit(
        "assurance_v2::tests::portfolio_lift_requires_every_source_ordered_clause"
    ),
    "assurance-project-frontiers": forge_unit(
        "assurance_v2::tests::common_and_evidence_frontiers_answer_different_questions"
    ),
    "assurance-project-transports": forge_unit(
        "assurance_v2::tests::unavailable_clause_and_item_transport_contribute_empty_not_omission"
    ),
    "assurance-project-lift": forge_unit(
        "assurance_v2::tests::project_lift_requires_complete_evidence_and_separate_completeness_premises"
    ),
    "assurance-noitems": forge_unit(
        "assurance_v2::tests::empty_population_is_no_items_not_top"
    ),
    "assurance-composition-lean": [
        "bash",
        "-c",
        'cd lean && exec lake build "$@"',
        "lean-build",
        "Thermite.AssuranceComposition",
    ],
    "assurance-composition-replay": [
        sys.executable,
        "gates/assurance-v2-replay.py",
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
        lean = subprocess.run(
            ["lake", "env", "lean", "--version"],
            cwd=ROOT / "lean",
            capture_output=True,
            text=True,
            check=True,
            timeout=30,
        ).stdout.strip().replace("\n", " ")
    except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return 3, ""
    return 0, f"{VERSION}; {cargo}; {lean}"


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
