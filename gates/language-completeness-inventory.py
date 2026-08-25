#!/usr/bin/env python3
"""Fail-closed inventory for the versioned language-completeness project.

The AST side is derived from public enums in thermite-syntax/src/ast.rs.  The
checked ledger must name every enum variant exactly once.  Documentation claims
and RFC-3 increments are intentionally explicit: their anchors and evidence are
checked, while gaps must have one durable disposition.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import tomllib
from pathlib import Path

STAGES = (
    "parser",
    "validator",
    "canonical_semantics",
    "checked_ir",
    "lowering",
    "proof_route",
    "policy",
    "certification",
)
SUPPORT = {"supported", "partial", "excluded", "not_applicable"}
DISPOSITIONS = {"current_project", "bounded_exclusion", "completeness_review"}
INVENTORY = "gates/language-completeness-inventory.toml"
MATRIX = "gates/language-support-matrix.json"


def strip_comments(text: str) -> str:
    return re.sub(r"//[^\n]*|/\*.*?\*/", "", text, flags=re.S)


def enum_variants(text: str) -> set[str]:
    """Return `Enum::Variant` names without depending on Rust parser tooling."""
    text = strip_comments(text)
    found: set[str] = set()
    header = re.compile(r"\bpub\s+enum\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{")
    for match in header.finditer(text):
        enum = match.group(1)
        start = match.end()
        depth = 1
        paren = bracket = 0
        token: list[str] = []
        variants: list[str] = []
        i = start
        while i < len(text) and depth:
            ch = text[i]
            if ch == "{" and paren == bracket == 0:
                depth += 1
            elif ch == "}" and paren == bracket == 0:
                depth -= 1
                if depth == 0:
                    if token:
                        variants.append("".join(token))
                    break
            elif ch == "(":
                paren += 1
            elif ch == ")":
                paren -= 1
            elif ch == "[":
                bracket += 1
            elif ch == "]":
                bracket -= 1
            elif ch == "," and depth == 1 and paren == bracket == 0:
                variants.append("".join(token))
                token = []
                i += 1
                continue
            token.append(ch)
            i += 1
        for raw in variants:
            name = re.match(r"\s*([A-Z][A-Za-z0-9_]*)", raw)
            if name:
                found.add(f"{enum}::{name.group(1)}")
    return found


def existing_path(root: Path, value: str) -> bool:
    path = value.split("#", 1)[0]
    return (root / path).exists()


def support_matrix(data: dict) -> dict:
    profiles = {row["id"]: row for row in data.get("profile", [])}
    constructs = data.get("construct", []) or data.get("profile", [{}])[0].get("construct", [])
    return {
        "version": 1,
        "stages": list(STAGES),
        "constructs": [
            {"id": row["id"], **{stage: profiles[row["profile"]][stage] for stage in STAGES}}
            for row in sorted(constructs, key=lambda item: item["id"])
        ],
        "claims": [
            {
                "id": row["id"], "source": row["source"],
                **{stage: profiles[row["profile"]][stage] for stage in STAGES},
            }
            for row in sorted(data.get("claim", []), key=lambda item: item["id"])
        ],
    }


def matrix_text(data: dict) -> str:
    return json.dumps(support_matrix(data), indent=2) + "\n"


def check(root: Path, inventory_path: Path) -> list[str]:
    errors: list[str] = []
    data = tomllib.loads(inventory_path.read_text(encoding="utf-8"))
    source = root / data.get("ast_source", "")
    if not source.is_file():
        return [f"ast_source does not exist: {source.relative_to(root)}"]

    actual = enum_variants(source.read_text(encoding="utf-8"))
    # TOML keys following [[profile]] belong to that table.  Keeping the large
    # construct array beside its sole profile makes the relationship visible.
    rows = data.get("construct", []) or data.get("profile", [{}])[0].get("construct", [])
    ids = [row.get("id") for row in rows]
    duplicates = sorted({item for item in ids if ids.count(item) > 1})
    if duplicates:
        errors.append(f"duplicate construct rows: {', '.join(duplicates)}")
    declared = {item for item in ids if isinstance(item, str)}
    missing = sorted(actual - declared)
    stale = sorted(declared - actual)
    if missing:
        errors.append(f"unclassified AST constructs: {', '.join(missing)}")
    if stale:
        errors.append(f"stale AST constructs: {', '.join(stale)}")

    profiles = {row.get("id"): row for row in data.get("profile", [])}
    for row in rows:
        ident = row.get("id", "<missing id>")
        profile = profiles.get(row.get("profile"))
        if profile is None:
            errors.append(f"{ident}: unknown profile {row.get('profile')!r}")
            continue
        for stage in STAGES:
            if profile.get(stage) not in SUPPORT:
                errors.append(f"{ident}: profile {profile.get('id')} has invalid {stage}")
        evidence = row.get("evidence", profile.get("evidence", []))
        if not evidence:
            errors.append(f"{ident}: no evidence")
        for path in evidence:
            if not existing_path(root, path):
                errors.append(f"{ident}: missing evidence path {path}")

    gaps = data.get("gap", [])
    gap_ids = [row.get("id") for row in gaps]
    if len(gap_ids) != len(set(gap_ids)):
        errors.append("gap ids must be unique")
    gap_by_id = {row.get("id"): row for row in gaps}
    for gap in gaps:
        ident = gap.get("id", "<missing id>")
        if gap.get("status") == "open" and gap.get("disposition") not in DISPOSITIONS:
            errors.append(f"{ident}: invalid or missing disposition")
        if not gap.get("observed") or not gap.get("claimed") or not gap.get("trust_consequence"):
            errors.append(f"{ident}: observed, claimed, and trust_consequence are required")
        stages = gap.get("stages", [])
        if not stages or any(stage not in STAGES for stage in stages):
            errors.append(f"{ident}: stages must name one or more known stages")
        if not gap.get("counterexample"):
            errors.append(f"{ident}: smallest counterexample is required")
        for path in gap.get("evidence", []):
            if not existing_path(root, path):
                errors.append(f"{ident}: missing evidence path {path}")
        for path in gap.get("resolution_evidence", []):
            if not existing_path(root, path):
                errors.append(f"{ident}: missing resolution evidence path {path}")

    claims = data.get("claim", [])
    claim_ids = [row.get("id") for row in claims]
    if len(claim_ids) != len(set(claim_ids)):
        errors.append("claim ids must be unique")
    for claim in claims:
        ident = claim.get("id", "<missing id>")
        profile = profiles.get(claim.get("profile"))
        if profile is None:
            errors.append(f"{ident}: unknown or missing stage profile {claim.get('profile')!r}")
        claim_source = root / claim.get("source", "")
        if not claim_source.is_file():
            errors.append(f"{ident}: missing source {claim.get('source')}")
            continue
        if claim.get("anchor", "") not in claim_source.read_text(encoding="utf-8"):
            errors.append(f"{ident}: anchor not found in {claim.get('source')}")
        digest = hashlib.sha256(claim_source.read_bytes()).hexdigest()
        if claim.get("source_sha256") != digest:
            errors.append(
                f"{ident}: source changed; reviewed sha256 is "
                f"{claim.get('source_sha256')!r}, current is {digest}"
            )
        for path in claim.get("evidence", []):
            if not existing_path(root, path):
                errors.append(f"{ident}: missing evidence path {path}")
        gap_id = claim.get("gap")
        if gap_id is not None and gap_id not in gap_by_id:
            errors.append(f"{ident}: unknown gap {gap_id}")

    required_sources: set[str] = set()
    for pattern in data.get("authoritative_sources", []):
        matches = sorted(root.glob(pattern))
        if not matches:
            errors.append(f"authoritative source pattern matches nothing: {pattern}")
        required_sources.update(str(path.relative_to(root)) for path in matches if path.is_file())
    claimed_sources = {row.get("source") for row in claims}
    core_sources = {source for source in required_sources if not source.startswith(".design/rfcs/")}
    missing_core = sorted(core_sources - claimed_sources)
    if missing_core:
        errors.append(f"authoritative sources have no reviewed claim group: {', '.join(missing_core)}")

    for gap in gaps:
        ident = gap.get("id", "<missing id>")
        status = gap.get("status")
        if status == "resolved":
            if gap.get("disposition") is not None:
                errors.append(f"{ident}: resolved gap must not retain an open disposition")
            if not gap.get("resolution_evidence"):
                errors.append(f"{ident}: resolved gap requires resolution_evidence")
        elif status == "open":
            disposition = gap.get("disposition")
            if disposition == "current_project" and not gap.get("tracking_requirement"):
                errors.append(f"{ident}: current_project requires tracking_requirement")
            elif disposition == "current_project" and gap["tracking_requirement"] not in (
                root / ".design/reqs/registry.toml"
            ).read_text(encoding="utf-8"):
                errors.append(f"{ident}: tracking_requirement does not resolve")
            elif disposition == "bounded_exclusion" and not gap.get("boundary"):
                errors.append(f"{ident}: bounded_exclusion requires boundary")
            elif disposition == "completeness_review" and not re.match(
                r"^https://github\.com/[^/]+/[^/]+/issues/\d+$", gap.get("issue", "")
            ):
                errors.append(f"{ident}: completeness_review requires a GitHub issue URL")
            detail_fields = {
                "current_project": {"tracking_requirement"},
                "bounded_exclusion": {"boundary"},
                "completeness_review": {"issue"},
            }.get(disposition, set())
            stale_details = ({"tracking_requirement", "boundary", "issue"} - detail_fields) & gap.keys()
            if stale_details:
                errors.append(f"{ident}: disposition has stale detail fields {sorted(stale_details)}")
        else:
            errors.append(f"{ident}: status must be open or resolved")

    matrix_ready = all(row.get("profile") in profiles for row in rows + claims)
    if matrix_ready:
        matrix_path = root / MATRIX
        expected_matrix = matrix_text(data)
        if not matrix_path.is_file() or matrix_path.read_text(encoding="utf-8") != expected_matrix:
            errors.append(f"{MATRIX}: missing or stale generated support matrix")

    increments = data.get("rfc3_increment", [])
    expected = {f"R2-{number}" for number in range(1, 10)}
    increment_ids = {row.get("id") for row in increments}
    if increment_ids != expected:
        errors.append(
            "RFC-3 increments must be exactly R2-1..R2-9; "
            f"missing={sorted(expected - increment_ids)}, stale={sorted(increment_ids - expected)}"
        )
    for row in increments:
        ident = row.get("id", "<missing id>")
        if row.get("status") not in {"absent", "partial", "shipped"}:
            errors.append(f"{ident}: invalid status")
        if not row.get("evidence"):
            errors.append(f"{ident}: no evidence")
        for path in row.get("evidence", []):
            if not existing_path(root, path):
                errors.append(f"{ident}: missing evidence path {path}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--inventory", default=INVENTORY)
    parser.add_argument("--dump-ast", action="store_true")
    parser.add_argument("--write-matrix", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    inventory_path = root / args.inventory
    if args.write_matrix:
        data = tomllib.loads(inventory_path.read_text(encoding="utf-8"))
        (root / MATRIX).write_text(matrix_text(data), encoding="utf-8")
        print(f"wrote {MATRIX}")
        return 0
    if args.dump_ast:
        for item in sorted(enum_variants((root / "thermite-syntax/src/ast.rs").read_text())):
            print(item)
        return 0
    try:
        errors = check(root, inventory_path)
    except (OSError, tomllib.TOMLDecodeError) as error:
        print(f"language completeness inventory: INCONCLUSIVE: {error}", file=sys.stderr)
        return 3
    if errors:
        print("language completeness inventory: FAIL", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    data = tomllib.loads(inventory_path.read_text(encoding="utf-8"))
    print(
        "language completeness inventory: clean: "
        f"{len(data.get('construct', []) or data['profile'][0]['construct'])} constructs, "
        f"{len(data['claim'])} claim groups, "
        f"{len(data['gap'])} gaps, {len(data['rfc3_increment'])} RFC-3 increments"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
