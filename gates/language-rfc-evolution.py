#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Fail-closed language-fragment evolution discipline for RFCs after AC-11."""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

MODES = {"unaffected", "tracked"}
CHANGES = {"preserve", "expand", "narrow"}
REQUIRED = {
    "preserve": ("preservation", "negative_witness"),
    "expand": ("classifier", "inclusion", "support_matrix", "negative_witness"),
    "narrow": (
        "classifier", "compatibility_break", "counterexample",
        "support_matrix", "negative_witness",
    ),
}
REF = re.compile(r"^([^#]+)#(.+)$")


def front_value(path: Path, key: str) -> str | None:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---\n", 4)
    if end < 0:
        return None
    match = re.search(rf"(?m)^{re.escape(key)}:\s*(\S+)\s*$", text[4:end])
    return match.group(1) if match else None


def resolves(root: Path, reference: object) -> bool:
    if not isinstance(reference, str):
        return False
    match = REF.match(reference)
    if not match:
        return False
    path = root / match.group(1)
    return path.is_file() and match.group(2) in path.read_text(encoding="utf-8")


def check(root: Path) -> list[str]:
    manifest_path = root / "gates" / "language-rfc-evolution.toml"
    try:
        data = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [f"manifest unreadable: {error}"]

    if data.get("version") != 1:
        return ["manifest version must be 1"]
    baseline = data.get("baseline_rfc")
    entries = data.get("evolution", [])
    if not isinstance(baseline, list) or not all(isinstance(x, str) for x in baseline):
        return ["baseline_rfc must be a list of filenames"]
    if not isinstance(entries, list):
        return ["evolution must be an array of tables"]

    rfc_dir = root / ".design" / "rfcs"
    files = {path.name: path for path in rfc_dir.glob("*.md")}
    problems: list[str] = []
    stale = sorted(set(baseline) - set(files))
    if stale:
        problems.append(f"baseline names missing RFCs: {', '.join(stale)}")

    by_rfc: dict[str, list[dict]] = {}
    for entry in entries:
        if not isinstance(entry, dict) or not isinstance(entry.get("rfc"), str):
            problems.append("every evolution entry must name an RFC filename")
            continue
        by_rfc.setdefault(entry["rfc"], []).append(entry)

    for name, path in sorted(files.items()):
        if name in baseline:
            continue
        mode = front_value(path, "language-evolution")
        if mode not in MODES:
            problems.append(
                f"{name}: front matter must declare language-evolution: "
                "unaffected or tracked"
            )
            continue
        changes = by_rfc.get(name, [])
        if mode == "unaffected" and changes:
            problems.append(f"{name}: unaffected RFC has evolution entries")
        if mode == "tracked" and not changes:
            problems.append(f"{name}: tracked RFC has no fragment evolution entries")

    for name in sorted(by_rfc):
        if name not in files:
            problems.append(f"{name}: evolution entry names no RFC")
        elif name in baseline:
            problems.append(f"{name}: baseline RFC may not have a new evolution entry")

    seen: set[tuple[str, str]] = set()
    for name, changes in by_rfc.items():
        for entry in changes:
            fragment = entry.get("fragment")
            change = entry.get("change")
            key = (name, str(fragment))
            if not isinstance(fragment, str) or not fragment:
                problems.append(f"{name}: evolution entry must name a fragment")
                continue
            if key in seen:
                problems.append(f"{name}: duplicate evolution for fragment {fragment}")
            seen.add(key)
            if change not in CHANGES:
                problems.append(f"{name}/{fragment}: change must be preserve, expand, or narrow")
                continue
            if change == "narrow" and entry.get("inclusion"):
                problems.append(
                    f"{name}/{fragment}: narrowing cannot claim ordinary inclusion; "
                    "record a compatibility_break"
                )
            for field in REQUIRED[change]:
                reference = entry.get(field)
                if not resolves(root, reference):
                    problems.append(
                        f"{name}/{fragment}: {change} requires resolving {field}=path#token"
                    )
    return problems


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".")
    args = parser.parse_args(argv)
    problems = check(Path(args.root).resolve())
    if problems:
        for problem in problems:
            print(f"language-rfc-evolution: {problem}", file=sys.stderr)
        return 1
    print("language RFC evolution: clean")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
