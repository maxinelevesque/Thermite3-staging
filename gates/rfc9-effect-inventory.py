#!/usr/bin/env python3
"""Deterministic RFC-9 region/effect migration inventory.

Standalone Thermite files and JSON ``program`` fields are parsed as program
text. Rust is reported separately because source files mix executable string
fixtures with prose and expected-output tokens. Every Rust file containing an
effect-shaped token therefore needs an explicit reviewed disposition; a new
unclassified file makes the inventory inconclusive.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

EFFECT = re.compile(r"\b(read|write|net)\(([A-Za-z_][A-Za-z0-9_.]*)\)")
SHARED = re.compile(r"(?m)^\s*shared\s+([A-Za-z_][A-Za-z0-9_]*)\s*:")
BASELINE = "gates/rfc9-effect-inventory.json"

# This is deliberately file-granular: the baseline below freezes the exact atom
# set and site count within each file, while this map records why those residual
# spellings are not an unreviewed Thermite program. Any new file fails closed.
RUST_REVIEW = {
    "forge/src/cache.rs": "non-program cache-key token fixtures",
    "forge/src/check.rs": "RFC-10 shared-state invariant program fixtures",
    "forge/src/cli.rs": "structured diagnostic expected-output token",
    "forge/src/manifest.rs": "non-program manifest tokens and documentation",
    "forge/src/result_arbiter.rs": "non-program certificate effect-row fixture",
    "forge/src/sandbox.rs": "migrated program fixtures plus sandbox token fixtures and documentation",
    "forge/src/vacuity.rs": "migrated maximal-row program fixtures",
    "forge/src/verified_build.rs": "Rust filesystem API calls; not Thermite syntax",
    "forge/src/verified_build/composition.rs": "Rust filesystem API calls; not Thermite syntax",
    "forge/tests/check_conformance.rs": "RFC-10 shared-state invariant conformance fixtures",
    "forge/tests/divergence_cache_effect_row.rs": "migrated programs plus expected-output tokens",
    "forge/tests/divergence_effect_link_string_wrappers.rs": "migrated program fixtures",
    "forge/tests/divergence_provenance.rs": "migrated program fixtures",
    "forge/tests/editor_runs.rs": "expected-output tokens for migrated editor program",
    "forge/tests/effect_link_conformance.rs": "expected-output tokens for migrated conformance program",
    "forge/tests/freestanding_target.rs": "migrated programs plus expected diagnostics",
    "forge/tests/sandbox_conformance.rs": "migrated programs plus expected-output tokens",
    "forge/tests/verified_build.rs": "Rust filesystem API call; not Thermite syntax",
    "thermite-lower/tests/checked_program.rs": "RFC-10 checked-program fixtures",
    "thermite-lower/tests/effects.rs": "migrated programs plus intentional undeclared-root rejection fixture",
    "thermite-lower/tests/equivalence_obligation.rs": "RFC-10 equivalence-obligation fixtures",
    "thermite-lower/tests/rfc10_conformance_matrix.rs": "RFC-10 phase cross-product fixtures",
    "thermite-lower/tests/shared_state_invariants.rs": "RFC-10 shared-state invariant lowering fixtures",
    "thermite-lower/tests/traversal_witness.rs": "RFC-10 traversal-witness fixtures",
    "thermite-skill/src/generate.rs": "generated language documentation tokens",
    "thermite-skill/tests/skill.rs": "generated language documentation expectations",
    "thermite-spec/tests/sealed_validate.rs": "migrated program fixtures",
    "thermite-spec/tests/resource_flow.rs": "RFC-11 resource-flow fixtures with RFC-9 region effect atoms",
    "thermite-spec/tests/shared_contract_validate.rs": "RFC-10 shared-contract validation fixtures",
    "thermite-spec/tests/verified_effect_rows.rs": "RFC-9 program fixtures and expected diagnostics",
    "thermite-syntax/tests/conformance.rs": "parser token fixtures; not complete programs",
    "thermite-syntax/tests/shared_state_invariants.rs": "RFC-10 shared-state invariant parser fixtures",
    "thermite-syntax/tests/verified_effect_rows.rs": "RFC-9 parser program fixtures and expected syntax",
}


def tracked(root: Path) -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z"], capture_output=True, text=True
    )
    if result.returncode:
        raise RuntimeError(result.stderr.strip() or "git ls-files failed")
    return sorted(path for path in result.stdout.split("\0") if path)


def program_facts(text: str) -> tuple[list[str], list[str]]:
    atoms = sorted({match.group(0) for match in EFFECT.finditer(text)})
    roots = {match.group(2).split(".", 1)[0] for match in EFFECT.finditer(text)}
    declared = set(SHARED.findall(text))
    return atoms, sorted(roots - declared)


def json_programs(value: object, address: str = "$"):
    if isinstance(value, dict):
        for key in sorted(value):
            child = value[key]
            child_address = f"{address}.{key}"
            if key == "program" and isinstance(child, str):
                yield child_address, child
            else:
                yield from json_programs(child, child_address)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from json_programs(child, f"{address}[{index}]")


def inventory(root: Path) -> dict[str, object]:
    paths = tracked(root)
    thermite = []
    json_entries = []
    rust = []
    for rel in paths:
        path = root / rel
        if rel.endswith(".th"):
            atoms, missing = program_facts(path.read_text(encoding="utf-8"))
            if atoms:
                thermite.append({"path": rel, "atoms": atoms, "missing_shared": missing})
        elif rel.endswith(".json"):
            try:
                value = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, UnicodeDecodeError, json.JSONDecodeError):
                continue
            for address, text in json_programs(value):
                atoms, missing = program_facts(text)
                if atoms:
                    json_entries.append(
                        {"path": rel, "address": address, "atoms": atoms, "missing_shared": missing}
                    )
        elif rel.endswith(".rs"):
            text = path.read_text(encoding="utf-8")
            matches = list(EFFECT.finditer(text))
            if matches:
                classification = RUST_REVIEW.get(rel)
                if classification is None:
                    raise RuntimeError(
                        f"unclassified Rust effect-shaped tokens in {rel}; "
                        "add a reviewed disposition to RUST_REVIEW"
                    )
                rust.append(
                    {
                        "path": rel,
                        "sites": len(matches),
                        "atoms": sorted({match.group(0) for match in matches}),
                        "classification": classification,
                    }
                )
    return {
        "schema": 2,
        "thermite_programs": thermite,
        "json_program_fields": json_entries,
        "rust_mixed_sites": rust,
        "summary": {
            "tracked_th_files": sum(path.endswith(".th") for path in paths),
            "th_programs_with_state_effects": len(thermite),
            "th_missing_shared": sum(len(item["missing_shared"]) for item in thermite),
            "json_program_fields_with_state_effects": len(json_entries),
            "json_missing_shared": sum(len(item["missing_shared"]) for item in json_entries),
            "rust_files_reviewed": len(rust),
            "rust_reviewed_sites": sum(item["sites"] for item in rust),
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = (args.root or Path(__file__).resolve().parents[1]).resolve()
    try:
        actual = inventory(root)
    except (OSError, RuntimeError) as error:
        print(f"rfc9-effect-inventory: INCONCLUSIVE: {error}", file=sys.stderr)
        return 3
    rendered = json.dumps(actual, indent=2, sort_keys=True) + "\n"
    if args.write:
        (root / BASELINE).write_text(rendered, encoding="utf-8")
        print(f"rfc9-effect-inventory: wrote {BASELINE}")
        return 0
    if not args.check:
        print(rendered, end="")
        return 0
    baseline = root / BASELINE
    try:
        expected = baseline.read_text(encoding="utf-8")
    except OSError as error:
        print(f"rfc9-effect-inventory: missing baseline: {error}", file=sys.stderr)
        return 3
    if rendered != expected:
        print("rfc9-effect-inventory: DRIFT (run without --check and review the diff)")
        return 1
    summary = actual["summary"]
    print(
        "rfc9-effect-inventory: clean: "
        f"{summary['tracked_th_files']} .th files, "
        f"{summary['th_missing_shared']} missing .th roots, "
        f"{summary['json_missing_shared']} missing JSON-program roots, "
        f"{summary['rust_reviewed_sites']} reviewed Rust token sites "
        f"across {summary['rust_files_reviewed']} explicitly classified files"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
