#!/usr/bin/env python3
"""Marshal claim-specific JSON cases through the shipped syntax implementation."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

VERSION = "thermite-claim-closure-exec 2"
PROBES = {
    "thermite-syntax-ast-operators": "ast-operators",
    "thermite-syntax-integer-tokens": "integers",
    "thermite-syntax-parse-edges": "parse-edges",
    "thermite-syntax-parse-expressions": "parse-expressions",
    "thermite-syntax-parse-fidelity": "parse-fidelity",
    "thermite-syntax-parse-items": "parse-items",
    "thermite-syntax-token-stream": "tokens",
}
PROBE_ARGV = [
    "cargo",
    "run",
    "--quiet",
    "--locked",
    "-p",
    "thermite-syntax",
    "--example",
    "claim-closure-probe",
]


def main(argv: list[str]) -> int:
    if argv == ["--version"]:
        try:
            rustc = subprocess.run(
                ["rustc", "--version"], capture_output=True, text=True, check=True
            ).stdout.strip()
        except (OSError, subprocess.CalledProcessError):
            return 3
        print(f"{VERSION}; {rustc}")
        return 0
    if len(argv) != 1:
        return 2
    try:
        oracle = json.loads(Path(argv[0]).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return 2
    if not isinstance(oracle, dict) or set(oracle) != {"cases", "probe", "version"}:
        return 2
    probe = oracle.get("probe")
    if oracle.get("version") != 1 or probe not in PROBES:
        return 2
    cases = oracle.get("cases")
    if not isinstance(cases, list) or not cases:
        return 2
    for case in cases:
        if not isinstance(case, dict) or set(case) not in (
            {"expected", "source"},
            {"expected", "source_path"},
        ):
            return 2
        source = case.get("source")
        source_path = case.get("source_path")
        expected = case.get("expected")
        if source_path is not None:
            relative = Path(source_path) if isinstance(source_path, str) else Path("..")
            if relative.is_absolute() or ".." in relative.parts:
                return 2
            try:
                source = relative.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError):
                return 2
        if not isinstance(source, str) or not isinstance(expected, dict):
            return 2
        try:
            result = subprocess.run(
                [*PROBE_ARGV, PROBES[probe]],
                input=source,
                capture_output=True,
                text=True,
                check=False,
                timeout=120,
            )
        except (OSError, subprocess.TimeoutExpired):
            return 3
        if result.returncode != 0:
            return 3
        try:
            observed = json.loads(result.stdout)
        except json.JSONDecodeError:
            return 3
        if observed != expected:
            return 4
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
