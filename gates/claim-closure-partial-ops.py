#!/usr/bin/env python3
"""Discriminate partial-operator obligations with the real Verus controls."""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

VERSION = "thermite-claim-closure-partial-ops 1"
SUITE = "forge-partial-operator-controls"
CONTROLS = (
    ("guarded-remainder", "rem_with_nonzero_req_certifies_l3", "L3"),
    ("unguarded-remainder", "rem_without_nonzero_req_is_l0", "L0"),
    ("guarded-shifts", "shifts_and_bitwise_certify_l3", "L3"),
    ("unguarded-left-shift", "shift_without_bound_is_l0", "L0"),
)
PASS_RE = re.compile(r"test result: ok\. 1 passed; 0 failed; 0 ignored;")


def verus_binary() -> Path | None:
    configured = os.environ.get("VERUS_BIN")
    if configured and Path(configured).is_file():
        return Path(configured).resolve()
    discovered = shutil.which("verus")
    if discovered:
        return Path(discovered).resolve()
    fallback = Path.home() / ".local" / "bin" / "verus"
    return fallback.resolve() if fallback.is_file() else None


def version_line(binary: Path) -> str | None:
    try:
        result = subprocess.run(
            [str(binary), "--version"],
            capture_output=True,
            text=True,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if result.returncode != 0:
        return None
    return (result.stdout or result.stderr).strip()


def main(argv: list[str]) -> int:
    binary = verus_binary()
    if binary is None:
        return 3
    verus_version = version_line(binary)
    if verus_version is None:
        return 3
    if argv == ["--version"]:
        print(f"{VERSION}; {verus_version}")
        return 0
    if len(argv) != 1:
        return 2
    try:
        oracle = json.loads(Path(argv[0]).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return 2
    if not isinstance(oracle, dict) or set(oracle) != {"cases", "suite", "version"}:
        return 2
    if oracle.get("version") != 1 or oracle.get("suite") != SUITE:
        return 2
    cases = oracle.get("cases")
    if not isinstance(cases, list):
        return 2
    observed_controls: list[tuple[str, str, str]] = []
    for case in cases:
        if not isinstance(case, dict) or set(case) != {
            "control",
            "expected_level",
            "test",
        }:
            return 2
        values = (case["control"], case["test"], case["expected_level"])
        if not all(isinstance(value, str) for value in values):
            return 2
        observed_controls.append(values)
    if tuple(observed_controls) != CONTROLS:
        return 4

    environment = dict(os.environ)
    environment["VERUS_BIN"] = str(binary)
    for _, test, _ in CONTROLS:
        try:
            result = subprocess.run(
                [
                    "cargo",
                    "test",
                    "--locked",
                    "-p",
                    "forge",
                    "--test",
                    "operators_conformance",
                    test,
                    "--",
                    "--exact",
                ],
                capture_output=True,
                text=True,
                check=False,
                timeout=180,
                env=environment,
            )
        except (OSError, subprocess.TimeoutExpired):
            return 3
        transcript = result.stdout + result.stderr
        if result.returncode != 0 or PASS_RE.search(transcript) is None:
            return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
