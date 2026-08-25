#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Bidirectional consistency gate for the language-completeness review track."""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

INVENTORY = "gates/language-completeness-inventory.toml"
BACKLOG = "gates/completeness-review.toml"
EVIDENCE_SUFFIXES = {".lean", ".rs", ".py", ".sh", ".json", ".toml"}
REF = re.compile(r"^([^#]+)#(.+)$")


def resolves_evidence(root: Path, reference: object) -> bool:
    if not isinstance(reference, str):
        return False
    match = REF.match(reference)
    if not match:
        return False
    path = root / match.group(1)
    return (
        path.is_file()
        and path.suffix in EVIDENCE_SUFFIXES
        and match.group(2) in path.read_text(encoding="utf-8")
    )


def check(root: Path) -> list[str]:
    try:
        inventory = tomllib.loads((root / INVENTORY).read_text(encoding="utf-8"))
        backlog = tomllib.loads((root / BACKLOG).read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [f"track input unreadable: {error}"]
    if backlog.get("version") != 1:
        return ["backlog version must be 1"]

    gaps = {gap.get("id"): gap for gap in inventory.get("gap", [])}
    items = backlog.get("item", [])
    problems: list[str] = []
    item_ids = [item.get("id") for item in items]
    if len(item_ids) != len(set(item_ids)):
        problems.append("backlog item ids must be unique")

    by_gap: dict[str, list[dict]] = {}
    for item in items:
        by_gap.setdefault(item.get("gap"), []).append(item)

    review_gaps = {
        gap_id: gap for gap_id, gap in gaps.items()
        if gap.get("disposition") == "completeness_review" or gap.get("review_item")
    }
    for gap_id, gap in review_gaps.items():
        matches = by_gap.get(gap_id, [])
        if len(matches) != 1:
            problems.append(f"{gap_id}: expected exactly one backlog item, found {len(matches)}")
            continue
        item = matches[0]
        if gap.get("issue") and item.get("issue") != gap.get("issue"):
            problems.append(f"{gap_id}: backlog issue does not match gap disposition")
        if gap.get("review_item") and item.get("id") != gap.get("review_item"):
            problems.append(f"{gap_id}: resolved gap points at a different review item")
        if gap.get("status") == "open" and item.get("status") != "open":
            problems.append(f"{gap_id}: open gap requires an open backlog item")
        if gap.get("status") == "resolved" and item.get("status") != "closed":
            problems.append(f"{gap_id}: resolved gap requires a closed backlog item")

    for item in items:
        ident = item.get("id", "<missing id>")
        gap_id = item.get("gap")
        if gap_id not in review_gaps:
            problems.append(f"{ident}: orphan backlog item for {gap_id!r}")
            continue
        if item.get("status") not in {"open", "closed"}:
            problems.append(f"{ident}: status must be open or closed")
        if not re.match(r"^https://github\.com/[^/]+/[^/]+/issues/\d+$", item.get("issue", "")):
            problems.append(f"{ident}: issue must be a durable GitHub issue URL")
        evidence = item.get("closure_evidence", [])
        if item.get("status") == "closed":
            if not evidence:
                problems.append(f"{ident}: closed item requires executable/formal closure_evidence")
            for reference in evidence:
                if not resolves_evidence(root, reference):
                    problems.append(f"{ident}: closure evidence does not resolve: {reference!r}")
        elif evidence:
            problems.append(f"{ident}: open item must not predeclare closure evidence")
    return problems


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".")
    args = parser.parse_args(argv)
    problems = check(Path(args.root).resolve())
    if problems:
        for problem in problems:
            print(f"completeness-review: {problem}", file=sys.stderr)
        return 1
    print("completeness review track: clean")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
