#!/usr/bin/env python3
"""Author and audit the frozen claim-closure migration population."""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path

REGISTRY = ".design/reqs/registry.toml"
LEDGER = "gates/completeness-review.toml"
BASELINE_SIZE = 566


def shipped_ids(root: Path) -> list[str]:
    registry = tomllib.loads((root / REGISTRY).read_text(encoding="utf-8"))
    values = sorted(
        row["id"]
        for row in registry.get("requirement", [])
        if row.get("status") == "shipped"
    )
    if len(values) != len(set(values)):
        raise ValueError("shipped requirement IDs are not unique")
    return values


def freeze_baseline(root: Path) -> None:
    values = shipped_ids(root)
    if len(values) != BASELINE_SIZE:
        raise ValueError(
            f"refusing to freeze {len(values)} shipped IDs; expected {BASELINE_SIZE}"
        )
    path = root / LEDGER
    text = path.read_text(encoding="utf-8")
    if "baseline_shipped_ids" in text:
        raise ValueError("baseline_shipped_ids is already present")
    first, separator, rest = text.partition("\n")
    if first.strip() != "version = 1" or not separator:
        raise ValueError("expected an unmigrated version-1 completeness ledger")
    rendered = ["version = 1", "", "baseline_shipped_ids = ["]
    rendered.extend(f'  "{value}",' for value in values)
    rendered.extend(["]", "", rest])
    path.write_text("\n".join(rendered), encoding="utf-8")


def check_baseline(root: Path) -> list[str]:
    ledger = tomllib.loads((root / LEDGER).read_text(encoding="utf-8"))
    baseline = ledger.get("baseline_shipped_ids")
    current = shipped_ids(root)
    problems: list[str] = []
    if not isinstance(baseline, list) or any(not isinstance(v, str) for v in baseline):
        return ["baseline_shipped_ids is absent or malformed"]
    if len(baseline) != BASELINE_SIZE:
        problems.append(f"baseline has {len(baseline)} IDs, expected {BASELINE_SIZE}")
    if baseline != sorted(baseline):
        problems.append("baseline IDs are not in canonical sorted order")
    if len(baseline) != len(set(baseline)):
        problems.append("baseline IDs are not unique")
    missing = sorted(set(baseline) - set(current))
    if missing:
        problems.append("baseline IDs are no longer shipped: " + ", ".join(missing))
    return problems


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--freeze-baseline", action="store_true")
    mode.add_argument("--check-baseline", action="store_true")
    args = parser.parse_args(argv)
    root = Path(args.root).resolve()
    try:
        if args.freeze_baseline:
            freeze_baseline(root)
            print(f"froze {BASELINE_SIZE} shipped requirement IDs")
            return 0
        problems = check_baseline(root)
    except (OSError, KeyError, ValueError, tomllib.TOMLDecodeError) as error:
        print(f"claim-closure-author: {error}", file=sys.stderr)
        return 1
    if problems:
        for problem in problems:
            print(f"claim-closure-author: {problem}", file=sys.stderr)
        return 1
    print(f"claim closure baseline: {BASELINE_SIZE} IDs, clean")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
