#!/usr/bin/env python3
"""Discriminate the complete Tier-A relational-frame implementation slice."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


VERSION = "thermite-claim-closure-relational-frame 1"
PROBE = "tier-a-relational-frame"
CASES = (
    ("relational-semantic-derivation", 0),
    ("relational-maximal-effect-coverage", 0),
    ("relational-semantic-carrier", 0),
    ("relational-row-composition", 0),
    ("relational-universal-theorem", 0),
    ("relational-artifact-witness", 0),
    ("relational-semantic-transport", 0),
    ("relational-certificate-projections", 0),
    ("relational-mutation-evidence", 0),
    ("relational-research-gates", 0),
    ("relational-no-new-syntax", 0),
    ("relational-vertical-slice", 0),
)

LOWER_UNIT = ("cargo", "test", "--locked", "-p", "thermite-lower", "relational_frame", "--lib")
LOWER_INTEGRATION = (
    "cargo", "test", "--locked", "-p", "thermite-lower", "--test", "relational_frame_witness"
)
FORGE = ("cargo", "test", "--locked", "-p", "forge", "--bin", "forge", "relational_")
SYNTAX = ("cargo", "test", "--locked", "-p", "thermite-syntax")
LEAN = ("bash", "gates/lean-axiom-probe.sh")

COMMANDS_BY_CASE = {
    "relational-semantic-derivation": (LEAN,),
    "relational-maximal-effect-coverage": (LEAN, LOWER_UNIT),
    "relational-semantic-carrier": (LEAN,),
    "relational-row-composition": (LEAN, LOWER_UNIT),
    "relational-universal-theorem": (LEAN,),
    "relational-artifact-witness": (LOWER_UNIT, LOWER_INTEGRATION),
    "relational-semantic-transport": (LEAN, LOWER_INTEGRATION),
    "relational-certificate-projections": (FORGE,),
    "relational-mutation-evidence": (LOWER_UNIT, LOWER_INTEGRATION, FORGE),
    "relational-research-gates": (LEAN, LOWER_UNIT, FORGE),
    "relational-no-new-syntax": (SYNTAX, FORGE),
    "relational-vertical-slice": (LEAN, LOWER_INTEGRATION, FORGE),
}


def load_oracle(path: str) -> tuple[tuple[str, int], ...] | None:
    try:
        oracle = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return None
    if not isinstance(oracle, dict) or set(oracle) != {"cases", "probe", "version"}:
        return None
    if oracle.get("version") != 1 or oracle.get("probe") != PROBE:
        return None
    cases = oracle.get("cases")
    if not isinstance(cases, list):
        return None
    observed: list[tuple[str, int]] = []
    for case in cases:
        if not isinstance(case, dict) or set(case) != {"expected_exit", "id"}:
            return None
        case_id = case.get("id")
        expected_exit = case.get("expected_exit")
        if not isinstance(case_id, str) or not isinstance(expected_exit, int):
            return None
        observed.append((case_id, expected_exit))
    return tuple(observed)


def main(argv: list[str]) -> int:
    if argv == ["--version"]:
        print(VERSION)
        return 0
    if len(argv) != 3 or argv[0] != "--claim":
        return 2
    claim_id = argv[1]
    if claim_id not in COMMANDS_BY_CASE:
        return 2
    observed = load_oracle(argv[2])
    if observed is None:
        return 2
    if observed != CASES:
        return 4
    for command in COMMANDS_BY_CASE[claim_id]:
        try:
            result = subprocess.run(
                command,
                check=False,
                timeout=900,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
        except (OSError, subprocess.TimeoutExpired):
            return 3
        if result.returncode != 0:
            return 3
    print(claim_id)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
