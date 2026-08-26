#!/usr/bin/env python3
"""Deterministic verifier for the claim-closure kernel's known-red corpus."""

from __future__ import annotations

import json
import sys
from pathlib import Path

EXPECTED_KEYS = {
    "executable_owner_is_content_bound",
    "raw_provenance_closes_semantic_claim",
    "shared_witness_membership_is_exact",
    "typed_claim_is_authoritative",
}


def main(argv: list[str]) -> int:
    if len(argv) != 1:
        return 64
    try:
        raw = json.loads(Path(argv[0]).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return 65
    if not isinstance(raw, dict) or set(raw) != {"properties", "version"}:
        return 10
    properties = raw.get("properties")
    if raw.get("version") != 1 or not isinstance(properties, dict):
        return 10
    if set(properties) != EXPECTED_KEYS:
        return 10
    if properties["executable_owner_is_content_bound"] is not True:
        return 11
    if properties["raw_provenance_closes_semantic_claim"] is not False:
        return 7
    if properties["shared_witness_membership_is_exact"] is not True:
        return 8
    if properties["typed_claim_is_authoritative"] is not True:
        return 9
    print("accepted")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
