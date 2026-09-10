#!/usr/bin/env python3
"""Validate the checked cross-version/procedure assurance replay matrix."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

MATRIX = "gates/assurance-transport-replay.json"
OUTCOMES = {
    "strengthened",
    "weakened",
    "incomparable",
    "changed_fiber",
    "missing_classification",
}


def canonical(value: object) -> bytes:
    return json.dumps(value, separators=(",", ":"), sort_keys=True).encode()


def classify(case: dict[str, str]) -> str:
    if case["relation"] == "missing":
        return "missing_classification"
    if case["relation"] == "incompatible":
        return "incomparable"
    if case["fiber"] != "same":
        return "changed_fiber"
    if case["boundary"] == "weaker":
        return "weakened"
    return "strengthened"


def validate(root: Path) -> list[str]:
    data = json.loads((root / MATRIX).read_text(encoding="utf-8"))
    errors: list[str] = []
    if data.get("version") != 1:
        errors.append("matrix version must be 1")
    if set(data.get("outcomes", [])) != OUTCOMES:
        errors.append("matrix outcomes must cover the exact five transport outcomes")
    cases = data.get("case")
    if not isinstance(cases, list) or len(cases) != 7:
        errors.append("matrix must contain exactly seven named cases")
        return errors
    ids = [case.get("id") for case in cases]
    if len(set(ids)) != len(ids):
        errors.append("case IDs must be unique")
    for case in cases:
        if classify(case) != case.get("outcome"):
            errors.append(f"{case.get('id')}: outcome does not match checked rule")
    if {case["outcome"] for case in cases} != OUTCOMES:
        errors.append("cases must exercise every transport outcome")
    first = hashlib.sha256(canonical(data)).hexdigest()
    second = hashlib.sha256(canonical(json.loads(canonical(data)))).hexdigest()
    if first != second:
        errors.append("canonical replay is not byte-stable")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    errors = validate(args.root.resolve())
    if errors:
        print("Assurance transport replay: FAIL")
        for error in errors:
            print(f"  - {error}")
        return 1
    print("Assurance transport replay: clean: 7 cases, 5 outcomes, canonical bytes stable")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
