#!/usr/bin/env python3
"""Discriminate the checked assurance-transport claims."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


VERSION = "thermite-claim-closure-assurance-transport 1"
PROBE = "assurance-transport"
CASE_TIMEOUT_SECONDS = 360
ROOT = Path(__file__).resolve().parents[1]


CASES = {
    "assurance-transport-rust": [
        "cargo",
        "test",
        "--quiet",
        "--locked",
        "-p",
        "forge",
        "--bin",
        "forge",
        "assurance_transport::tests::",
    ],
    "assurance-transport-lean": [
        "bash",
        "-c",
        'cd lean && exec lake build "$@"',
        "lean-build",
        "Thermite.AssuranceTransport",
        "Thermite.AssuranceTransportReplay",
    ],
    "assurance-transport-replay": [
        sys.executable,
        "gates/assurance-transport-replay.py",
    ],
    "assurance-transport-level-boundary": [
        sys.executable,
        "-m",
        "unittest",
        "gates.tests.test_assurance_transport_replay.AssuranceTransportReplayTest.test_transport_decisions_do_not_read_archival_levels",
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
