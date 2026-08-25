#!/usr/bin/env python3
"""Check and generate the shared AssurancePolicyV2 constructor-pair replay."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

MATRIX = "gates/assurance-v2-replay.json"
ISSUERS = "gates/assurance-v2-issuers.json"
LEAN = "lean/Thermite/AssuranceV2Replay.lean"
FAMILIES = [
    "runtime",
    "bounded",
    "solver_incomplete",
    "solver_complete",
    "lean_empirical",
    "lean_complete",
]
ISSUER_MARKER = re.compile(
    r"^\s*(?://|--)\s*ASSURANCE_V2_ISSUER\s+(\S+)\s+(\S+)\s*$"
)
PREDECESSOR_MARKER = re.compile(
    r"^\s*(?://|--)\s*ASSURANCE_V2_PREDECESSOR\s+(\S+)\s+(\S+)\s*$"
)
CHARACTERIZATION_MARKER = re.compile(
    r"^\s*(?://|--)\s*ASSURANCE_V2_CHARACTERIZATION\s+(\S+)\s+(\S+)\s+(\S+)\s*$"
)


def source_extension_markers(root: Path) -> tuple[
    list[tuple[str, str, str]],
    list[tuple[str, str]],
    list[tuple[str, str, str, str, str]],
]:
    issuers: list[tuple[str, str, str]] = []
    predecessors: list[tuple[str, str]] = []
    characterizations: list[tuple[str, str, str, str, str]] = []
    sources = (
        sorted((root / "forge/src").rglob("*.rs"))
        + sorted((root / "forge/tests").rglob("*.rs"))
        + sorted((root / "lean/Thermite").rglob("*.lean"))
    )
    for path in sources:
        relative = path.relative_to(root).as_posix()
        for line in path.read_text(encoding="utf-8").splitlines():
            if match := ISSUER_MARKER.match(line):
                issuers.append((match.group(1), relative, match.group(2)))
            if match := PREDECESSOR_MARKER.match(line):
                predecessors.append((match.group(1), match.group(2)))
            if match := CHARACTERIZATION_MARKER.match(line):
                characterizations.append(
                    (match.group(1), match.group(2), match.group(3), relative, "")
                )
    # The adjacent test symbol is filled below so a marker cannot nominate an
    # unrelated existing test elsewhere in the file.
    for index, row in enumerate(characterizations):
        family, issuer_path, issuer_symbol, test_path, _ = row
        lines = (root / test_path).read_text(encoding="utf-8").splitlines()
        marker_index = next(
            i
            for i, line in enumerate(lines)
            if CHARACTERIZATION_MARKER.match(line)
            and CHARACTERIZATION_MARKER.match(line).groups()
            == (family, issuer_path, issuer_symbol)
            and not any(existing[:4] == row[:4] for existing in characterizations[:index])
        )
        following = "\n".join(lines[marker_index + 1 : marker_index + 5])
        test_match = re.search(r"\bfn\s+(\w+)\b", following)
        characterizations[index] = (*row[:4], test_match.group(1) if test_match else "")
    return sorted(issuers), sorted(predecessors), sorted(characterizations)


def source_marker_adjacency_errors(root: Path) -> list[str]:
    errors: list[str] = []
    sources = (
        sorted((root / "forge/src").rglob("*.rs"))
        + sorted((root / "forge/tests").rglob("*.rs"))
        + sorted((root / "lean/Thermite").rglob("*.lean"))
    )
    for path in sources:
        lines = path.read_text(encoding="utf-8").splitlines()
        for index, line in enumerate(lines):
            following = "\n".join(lines[index + 1 : index + 5])
            if match := ISSUER_MARKER.match(line):
                symbol = re.escape(match.group(2))
                if not re.search(rf"\bfn\s+{symbol}\b", following):
                    errors.append(
                        f"{path.relative_to(root)}:{index + 1}: issuer marker must be adjacent "
                        f"to fn {match.group(2)}"
                    )
            if match := PREDECESSOR_MARKER.match(line):
                short_name = re.escape(re.split(r"::|\.", match.group(1))[-1])
                if not re.search(rf"\b(?:enum|inductive)\s+{short_name}\b", following):
                    errors.append(
                        f"{path.relative_to(root)}:{index + 1}: predecessor marker must be "
                        f"adjacent to {match.group(1)}"
                    )
            if match := CHARACTERIZATION_MARKER.match(line):
                if not re.search(r"\bfn\s+\w+\b", following):
                    errors.append(
                        f"{path.relative_to(root)}:{index + 1}: characterization marker "
                        "must be adjacent to its test function"
                    )
    return errors


def leq(left: str, right: str) -> bool:
    return (
        (left == "runtime" and right == "runtime")
        or (left == "bounded" and right == "bounded")
        or (
            left == "solver_incomplete"
            and right
            in {
                "solver_incomplete",
                "solver_complete",
                "lean_empirical",
                "lean_complete",
            }
        )
        or (
            left == "solver_complete"
            and right in {"solver_complete", "lean_complete"}
        )
        or (
            left == "lean_empirical"
            and right in {"lean_empirical", "lean_complete"}
        )
        or (left == "lean_complete" and right == "lean_complete")
    )


def lower_bound_frontier(left: str, right: str) -> list[str]:
    def supported(candidate: str) -> bool:
        return leq(candidate, left) and leq(candidate, right)

    return [
        candidate
        for candidate in FAMILIES
        if supported(candidate)
        and not any(
            supported(other)
            and leq(candidate, other)
            and not leq(other, candidate)
            for other in FAMILIES
        )
    ]


def canonical_pairs() -> list[dict[str, object]]:
    return [
        {
            "left": left,
            "right": right,
            "left_le_right": leq(left, right),
            "right_le_left": leq(right, left),
            "lower_bound_frontier": lower_bound_frontier(left, right),
        }
        for left in FAMILIES
        for right in FAMILIES
    ]


def lean_kind(value: str) -> str:
    pieces = value.split("_")
    return "." + pieces[0] + "".join(piece.title() for piece in pieces[1:])


def lean_bool(value: bool) -> str:
    return "true" if value else "false"


def generated(rows: list[dict[str, object]]) -> str:
    rendered = []
    for row in rows:
        frontier = ", ".join(lean_kind(value) for value in row["lower_bound_frontier"])
        rendered.append(
            "  ⟨"
            f"{lean_kind(row['left'])}, {lean_kind(row['right'])}, "
            f"{lean_bool(row['left_le_right'])}, {lean_bool(row['right_le_left'])}, "
            f"[{frontier}]⟩"
        )
    body = ",\n".join(rendered)
    return f'''/- This file is generated by gates/assurance-v2-replay.py. -/
import Thermite.AssurancePolicyV2

namespace Thermite.CertificationMetatheory

def generatedConstructorPairLawsV2 : List ConstructorPairLawV2 := [
{body}
]

theorem generated_replay_matches_symbolic_policy :
    generatedConstructorPairLawsV2 = constructorPairLawsV2 := by decide

theorem generated_replay_covers_six_families_and_thirty_six_pairs :
    allAssuranceKindsV2.length = 6 ∧ generatedConstructorPairLawsV2.length = 36 := by
  decide

def omittedLeanCompleteReplay : List ConstructorPairLawV2 :=
  generatedConstructorPairLawsV2.filter fun row =>
    row.left != .leanComplete && row.right != .leanComplete

theorem omitted_family_mutant_is_rejected :
    omittedLeanCompleteReplay ≠ constructorPairLawsV2 := by decide

def missingCommonLowerBoundReplay : List ConstructorPairLawV2 :=
  generatedConstructorPairLawsV2.map fun row =>
    if row.left = .solverComplete && row.right = .leanEmpirical then
      {{ row with lowerBounds := [] }}
    else row

theorem missing_lower_bound_mutant_is_rejected :
    missingCommonLowerBoundReplay ≠ constructorPairLawsV2 := by decide

end Thermite.CertificationMetatheory
'''


def validate(root: Path, data: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["matrix must be an object"]
    if data.get("version") != 2:
        errors.append("matrix version must be 2")
    if data.get("families") != FAMILIES:
        errors.append("families must be the exact ordered six-family signature")
    rows = data.get("pair")
    if rows != canonical_pairs():
        errors.append("pair matrix must equal the exact symbolic 6x6 policy replay")
    for source, needles in {
        "lean/Thermite/AssurancePolicyV2.lean": [
            "AdmittedRealizedCertificationV2",
            "AssurancePolicyV2",
            "AntichainNF",
            "intersectNF",
            "constructorPairLawsV2",
        ],
        "forge/src/assurance_v2.rs": [
            "ALL_ASSURANCE_KINDS_V2",
            "assurance_kind_leq",
            "lower_bound_frontier",
            "authority_digest",
            "presentation_digest",
        ],
    }.items():
        text = (root / source).read_text(encoding="utf-8")
        for needle in needles:
            if needle not in text:
                errors.append(f"{source}: missing {needle}")
    return errors


def validate_issuers(root: Path, data: object) -> list[str]:
    if not isinstance(data, dict):
        return ["issuer inventory must be an object"]
    errors: list[str] = []
    if data.get("version") != 1:
        errors.append("issuer inventory version must be 1")
    rows = data.get("family")
    if not isinstance(rows, list) or [row.get("kind") for row in rows] != FAMILIES:
        return errors + ["issuer inventory must cover the exact ordered six-family signature"]
    inventoried_issuers: list[tuple[str, str, str]] = []
    inventoried_characterizations: list[tuple[str, str, str, str, str]] = []
    for row in rows:
        issuers = row.get("issuers")
        if not isinstance(issuers, list) or not issuers:
            errors.append(f"{row.get('kind')}: at least one current issuance seam is required")
            continue
        for issuer in issuers:
            try:
                path = issuer["path"]
                symbol = issuer["symbol"]
                test_path = issuer["test_path"]
                characterization_test = issuer["characterization_test"]
                source = (root / path).read_text(encoding="utf-8")
                test_source = (root / test_path).read_text(encoding="utf-8")
            except (KeyError, OSError, TypeError) as error:
                errors.append(f"{row.get('kind')}: invalid issuer: {error}")
                continue
            if f"fn {symbol}" not in source:
                errors.append(f"{row.get('kind')}: missing issuance symbol {path}::{symbol}")
            if f"fn {characterization_test}" not in test_source:
                errors.append(
                    f"{row.get('kind')}: missing characterization test "
                    f"{test_path}::{characterization_test}"
                )
            inventoried_issuers.append((row.get("kind"), path, symbol))
            inventoried_characterizations.append(
                (row.get("kind"), path, symbol, test_path, characterization_test)
            )
    expected_relations = {
        "forge::manifest::AssuranceElement": "compatibility_only",
        "Thermite.CertificationMetatheory.RepresentativePosition": "realizable_counterexample_probe",
        "Thermite.CertificationMetatheory.PolicyPoint": "legacy_self_floor_abstraction",
    }
    relations = data.get("legacy_relation")
    actual_relations = (
        {row.get("carrier"): row.get("classification") for row in relations}
        if isinstance(relations, list)
        else {}
    )
    if actual_relations != expected_relations:
        errors.append("every predecessor policy carrier requires an exact checked disposition")
    source_issuers, source_predecessors, source_characterizations = source_extension_markers(root)
    errors.extend(source_marker_adjacency_errors(root))
    if sorted(inventoried_issuers) != source_issuers:
        errors.append(
            "issuer inventory must equal the bidirectional ASSURANCE_V2_ISSUER source markers"
        )
    if sorted(actual_relations.items()) != source_predecessors:
        errors.append(
            "predecessor inventory must equal the bidirectional ASSURANCE_V2_PREDECESSOR source markers"
        )
    if sorted(inventoried_characterizations) != source_characterizations:
        errors.append(
            "characterization inventory must equal the bidirectional "
            "ASSURANCE_V2_CHARACTERIZATION source markers"
        )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        data = json.loads((root / MATRIX).read_text(encoding="utf-8"))
        errors = validate(root, data)
        issuer_data = json.loads((root / ISSUERS).read_text(encoding="utf-8"))
        errors.extend(validate_issuers(root, issuer_data))
        output = generated(data["pair"]) if not errors else ""
        target = root / LEAN
        if args.write and not errors:
            target.write_text(output, encoding="utf-8")
        elif not errors and (not target.exists() or target.read_text(encoding="utf-8") != output):
            errors.append(f"{LEAN} is stale; run gates/assurance-v2-replay.py --write")
    except (OSError, json.JSONDecodeError, KeyError, TypeError) as error:
        print(f"Assurance V2 replay: INCONCLUSIVE: {error}", file=sys.stderr)
        return 3
    if errors:
        print("Assurance V2 replay: FAIL", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print("Assurance V2 replay: clean: 6 families, 36 constructor pairs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
