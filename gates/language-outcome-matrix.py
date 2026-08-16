#!/usr/bin/env python3
"""Fail-closed schema and coverage gate for issue #48's AC-7 matrix."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

STAGES = {
    "parser", "validator", "canonical_semantics", "checked_ir", "lowering",
    "proof_route", "policy", "certification",
}
OUTCOMES = {
    "success", "unsupported_language", "invalid_source", "unsupported_policy",
    "resource_exhausted", "tool_unavailable", "tool_incompatible",
    "counterexample", "proof_failure", "soundness_alarm",
}
MATRIX = "gates/language-outcome-matrix.json"


def check(root: Path, path: Path) -> list[str]:
    data = json.loads(path.read_text(encoding="utf-8"))
    errors: list[str] = []
    if data.get("version") != 1:
        errors.append("matrix version must be 1")
    vocabulary_sources = {
        "forge/src/outcome_matrix.rs": {
            "success": "Success", "unsupported_language": "UnsupportedLanguage",
            "invalid_source": "InvalidSource", "unsupported_policy": "UnsupportedPolicy",
            "resource_exhausted": "ResourceExhausted", "tool_unavailable": "ToolUnavailable",
            "tool_incompatible": "ToolIncompatible", "counterexample": "Counterexample",
            "proof_failure": "ProofFailure", "soundness_alarm": "SoundnessAlarm",
        },
        "lean/Thermite/LanguageCompleteness.lean": {
            outcome: outcome.split("_")[0] + "".join(part.title() for part in outcome.split("_")[1:])
            for outcome in OUTCOMES
        },
    }
    for source, spellings in vocabulary_sources.items():
        text = (root / source).read_text(encoding="utf-8")
        for outcome, spelling in spellings.items():
            if spelling not in text:
                errors.append(f"{source}: missing {outcome} constructor {spelling}")
    rows = data.get("case")
    if not isinstance(rows, list) or not rows:
        return errors + ["case must be a non-empty array"]
    ids = [row.get("id") for row in rows if isinstance(row, dict)]
    if len(ids) != len(rows) or any(not item for item in ids):
        errors.append("every case requires an id")
    if len(ids) != len(set(ids)):
        errors.append("case ids must be unique")
    seen_stages = {row.get("stage") for row in rows if isinstance(row, dict)}
    seen_outcomes = {row.get("expected") for row in rows if isinstance(row, dict)}
    if seen_stages != STAGES:
        errors.append(f"stage coverage must be exact; missing={sorted(STAGES-seen_stages)}, stale={sorted(seen_stages-STAGES)}")
    if seen_outcomes != OUTCOMES:
        errors.append(f"outcome coverage must be exact; missing={sorted(OUTCOMES-seen_outcomes)}, stale={sorted(seen_outcomes-OUTCOMES)}")
    for row in rows:
        if not isinstance(row, dict):
            errors.append("every case must be an object")
            continue
        ident = row.get("id", "<missing>")
        if not isinstance(row.get("program"), str) or not row["program"]:
            errors.append(f"{ident}: representative program is required")
        expected = row.get("expected")
        facts = row.get("facts", {})
        if not isinstance(facts, dict) or any(key not in OUTCOMES - {"success"} for key in facts):
            errors.append(f"{ident}: facts contain an unknown outcome")
            continue
        asserted = [key for key, value in facts.items() if value is True]
        if any(value is not True for value in facts.values()):
            errors.append(f"{ident}: fact values must be true when present")
        if expected == "success" and asserted:
            errors.append(f"{ident}: success cannot carry terminal facts")
        elif expected != "success" and asserted != [expected]:
            errors.append(f"{ident}: expected outcome must be its sole terminal fact")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--matrix", default=MATRIX)
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        errors = check(root, root / args.matrix)
    except (OSError, json.JSONDecodeError) as error:
        print(f"language outcome matrix: INCONCLUSIVE: {error}", file=sys.stderr)
        return 3
    if errors:
        print("language outcome matrix: FAIL", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print("language outcome matrix: clean: 8 stages, 10 outcomes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
