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
DISPOSITIONS = {"issue_48", "bounded_exclusion", "completeness_review"}
INVENTORY = "gates/language-completeness-inventory.toml"


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
        if gap.get("disposition") not in DISPOSITIONS:
            errors.append(f"{ident}: invalid or missing disposition")
        if not gap.get("observed") or not gap.get("claimed") or not gap.get("trust_consequence"):
            errors.append(f"{ident}: observed, claimed, and trust_consequence are required")
        for path in gap.get("evidence", []):
            if not existing_path(root, path):
                errors.append(f"{ident}: missing evidence path {path}")

    claims = data.get("claim", [])
    claim_ids = [row.get("id") for row in claims]
    if len(claim_ids) != len(set(claim_ids)):
        errors.append("claim ids must be unique")
    for claim in claims:
        ident = claim.get("id", "<missing id>")
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
    args = parser.parse_args()
    root = args.root.resolve()
    inventory_path = root / args.inventory
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
