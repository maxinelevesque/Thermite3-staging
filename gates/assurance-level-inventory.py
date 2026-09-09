#!/usr/bin/env python3
"""Fail-closed inventory of Rust ``manifest::Level`` compatibility uses.

The scanner removes comments and string/character literals, then inventories
every remaining ``Level`` token and every value-level ``.level`` or
``.compatibility_level()`` access in ``forge/src``.  A reviewed baseline assigns
each stable source occurrence to exactly one of the design's three classes:
``constructor``, ``presentation``, or ``authority_decision``.  Production is
allowed to retain compatibility constructors and presentation reads, but the
authority-decision set must remain empty.

``--write`` preserves prior review classifications and records new sites as
``unclassified``.  It never silently blesses a new occurrence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

BASELINE = "gates/assurance-level-inventory.json"
ALLOWED = {"constructor", "presentation", "authority_decision"}
TOKEN = re.compile(r"\bLevel\b")
VALUE_ACCESS = re.compile(r"\.\s*(?:level\b|compatibility_level\s*\()")
SIGNALS = (("level_token", TOKEN), ("value_access", VALUE_ACCESS))


def level_signals(line: str):
    """Yield every legacy-assurance signal in stable pattern/source order."""
    for signal, pattern in SIGNALS:
        for match in pattern.finditer(line):
            yield signal, match


def tracked_rust(root: Path) -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z", "forge/src/**/*.rs", "forge/src/*.rs"],
        capture_output=True,
        text=True,
    )
    if result.returncode:
        raise RuntimeError(result.stderr.strip() or "git ls-files failed")
    return sorted(set(path for path in result.stdout.split("\0") if path))


def code_only(source: str) -> str:
    """Replace comments and literals with spaces while preserving newlines."""
    out = list(source)
    index = 0
    state = "code"
    raw_hashes = 0
    while index < len(source):
        char = source[index]
        nxt = source[index + 1] if index + 1 < len(source) else ""
        if state == "code":
            if char == "/" and nxt == "/":
                out[index] = out[index + 1] = " "
                index += 2
                state = "line_comment"
                continue
            if char == "/" and nxt == "*":
                out[index] = out[index + 1] = " "
                index += 2
                state = "block_comment"
                continue
            if char == '"':
                out[index] = " "
                index += 1
                state = "string"
                continue
            if char == "'" and index + 2 < len(source):
                # Lifetimes are identifiers; only treat a quote as a character
                # literal when a closing quote exists on this short token.
                closing = source.find("'", index + 1, min(len(source), index + 8))
                if closing != -1 and "\n" not in source[index : closing + 1]:
                    for pos in range(index, closing + 1):
                        out[pos] = " "
                    index = closing + 1
                    continue
            if char == "r":
                match = re.match(r'r(#{0,16})"', source[index:])
                if match:
                    raw_hashes = len(match.group(1))
                    for pos in range(index, index + len(match.group(0))):
                        out[pos] = " "
                    index += len(match.group(0))
                    state = "raw_string"
                    continue
            index += 1
            continue
        if state == "line_comment":
            if char == "\n":
                state = "code"
            else:
                out[index] = " "
            index += 1
            continue
        if state == "block_comment":
            if char == "*" and nxt == "/":
                out[index] = out[index + 1] = " "
                index += 2
                state = "code"
            else:
                if char != "\n":
                    out[index] = " "
                index += 1
            continue
        if state == "string":
            if char == "\\" and nxt:
                out[index] = " "
                if nxt != "\n":
                    out[index + 1] = " "
                index += 2
            elif char == '"':
                out[index] = " "
                index += 1
                state = "code"
            else:
                if char != "\n":
                    out[index] = " "
                index += 1
            continue
        if state == "raw_string":
            closing = '"' + ("#" * raw_hashes)
            if source.startswith(closing, index):
                for pos in range(index, index + len(closing)):
                    out[pos] = " "
                index += len(closing)
                state = "code"
            else:
                if char != "\n":
                    out[index] = " "
                index += 1
    return "".join(out)


def cfg_test_lines(lines: list[str]) -> set[int]:
    """One-based lines belonging to straightforward ``#[cfg(test)]`` items."""
    excluded: set[int] = set()
    index = 0
    while index < len(lines):
        if lines[index].strip() != "#[cfg(test)]":
            index += 1
            continue
        start = index
        item = index + 1
        while item < len(lines) and not lines[item].strip():
            item += 1
        if item >= len(lines):
            excluded.add(start + 1)
            break
        if re.search(r"\bmod\s+tests\b", lines[item]):
            excluded.update(range(start + 1, len(lines) + 1))
            break
        depth = 0
        saw_brace = False
        end = item
        while end < len(lines):
            depth += lines[end].count("{") - lines[end].count("}")
            saw_brace = saw_brace or "{" in lines[end]
            if (saw_brace and depth == 0) or (not saw_brace and ";" in lines[end]):
                break
            end += 1
        excluded.update(range(start + 1, min(end + 2, len(lines) + 1)))
        index = end + 1
    return excluded


def inventory(root: Path, previous: dict[str, str]) -> dict[str, object]:
    sites: list[dict[str, object]] = []
    for relative in tracked_rust(root):
        raw = (root / relative).read_text(encoding="utf-8")
        lines = code_only(raw).splitlines()
        if any(line.strip() == "#![cfg(test)]" for line in lines):
            continue
        excluded = cfg_test_lines(lines)
        occurrence_by_source: dict[tuple[str, str], int] = {}
        for line_number, line in enumerate(lines, 1):
            if line_number in excluded:
                continue
            normalized = " ".join(line.split())
            if not normalized:
                continue
            for signal, match in level_signals(line):
                occurrence_key = (signal, normalized)
                occurrence = occurrence_by_source.get(occurrence_key, 0)
                occurrence_by_source[occurrence_key] = occurrence + 1
                if signal == "level_token":
                    # Preserve the original identities so broadening this gate
                    # does not discard the completed review of every type token.
                    identity_material = f"{relative}\0{normalized}\0{occurrence}".encode()
                else:
                    identity_material = (
                        f"{relative}\0{normalized}\0{signal}\0{occurrence}".encode()
                    )
                identity = hashlib.sha256(identity_material).hexdigest()
                sites.append(
                    {
                        "id": identity,
                        "path": relative,
                        "line": line_number,
                        "column": match.start() + 1,
                        "scope": "production",
                        "signal": signal,
                        "source": normalized,
                        "category": previous.get(identity, "unclassified"),
                    }
                )
    counts = {category: 0 for category in sorted(ALLOWED | {"unclassified"})}
    production_decisions = 0
    for site in sites:
        counts[site["category"]] = counts.get(site["category"], 0) + 1
        if site["scope"] == "production" and site["category"] == "authority_decision":
            production_decisions += 1
    return {
        "schema": 1,
        "sites": sites,
        "summary": {
            "sites": len(sites),
            "categories": counts,
            "production_authority_decisions": production_decisions,
        },
    }


def load_categories(path: Path) -> dict[str, str]:
    if not path.exists():
        return {}
    value = json.loads(path.read_text(encoding="utf-8"))
    return {site["id"]: site["category"] for site in value.get("sites", [])}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = (args.root or Path(__file__).resolve().parents[1]).resolve()
    baseline = root / BASELINE
    try:
        actual = inventory(root, load_categories(baseline))
    except (OSError, RuntimeError, json.JSONDecodeError) as error:
        print(f"assurance-level-inventory: INCONCLUSIVE: {error}", file=sys.stderr)
        return 3
    rendered = json.dumps(actual, indent=2, sort_keys=True) + "\n"
    if args.write:
        baseline.write_text(rendered, encoding="utf-8")
        print(f"assurance-level-inventory: wrote {BASELINE}")
        return 0
    if not args.check:
        print(rendered, end="")
        return 0
    try:
        expected = baseline.read_text(encoding="utf-8")
    except OSError as error:
        print(f"assurance-level-inventory: missing baseline: {error}", file=sys.stderr)
        return 3
    if rendered != expected:
        print("assurance-level-inventory: DRIFT (run --write, classify new sites, and review)")
        return 1
    unclassified = [site for site in actual["sites"] if site["category"] not in ALLOWED]
    decisions = [
        site
        for site in actual["sites"]
        if site["scope"] == "production" and site["category"] == "authority_decision"
    ]
    if unclassified:
        print(f"assurance-level-inventory: {len(unclassified)} unclassified sites", file=sys.stderr)
        return 1
    if decisions:
        print(
            f"assurance-level-inventory: {len(decisions)} production authority decisions remain",
            file=sys.stderr,
        )
        return 1
    print(
        "assurance-level-inventory: clean: "
        f"{len(actual['sites'])} reviewed sites, zero production authority decisions"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
