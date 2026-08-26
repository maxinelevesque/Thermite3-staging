#!/usr/bin/env python3
"""Shared closed vocabulary for typed claim-closure payloads."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

MECHANISMS = {
    "formal_theorem",
    "executable_discriminator",
    "exact_population",
}

FORMAL_SUBJECT = re.compile(r"^(?P<path>[^#]+\.lean)#(?P<declaration>[A-Za-z_][A-Za-z0-9_'.]*)$")
ORACLE_SUBJECT = re.compile(r"^oracle:(?P<path>[^#]+)$")
REGEX_SUBJECT = re.compile(r"^regex:(?P<path>[^#]+)#(?P<pattern>.+)$")


def canonical_json(value: object) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")


def claim_digest(requirement_id: str, claim: object) -> str | None:
    if not isinstance(claim, dict):
        return None
    kind = claim.get("kind")
    subject = claim.get("subject")
    expected = claim.get("expected")
    if not isinstance(kind, str) or not isinstance(subject, str):
        return None
    if not isinstance(expected, list) or any(not isinstance(v, str) for v in expected):
        return None
    payload = {
        "expected": expected,
        "kind": kind,
        "requirement_id": requirement_id,
        "subject": subject,
    }
    return hashlib.sha256(canonical_json(payload)).hexdigest()


def _repo_relative(path_text: str) -> bool:
    path = Path(path_text)
    return bool(path_text) and not path.is_absolute() and ".." not in path.parts


def claim_subject_problem(kind: object, subject: object) -> str | None:
    """Return a stable diagnostic when a mechanism subject is outside the grammar."""
    if kind not in MECHANISMS:
        return "claim kind is outside the closed mechanism vocabulary"
    if not isinstance(subject, str) or not subject:
        return "claim subject must be a non-empty string"
    if kind == "formal_theorem":
        match = FORMAL_SUBJECT.fullmatch(subject)
        if match is None or not _repo_relative(match.group("path")):
            return "formal theorem subject must be repo-relative .lean path#declaration"
    elif kind == "executable_discriminator":
        match = ORACLE_SUBJECT.fullmatch(subject)
        if (
            match is None
            or not _repo_relative(match.group("path"))
            or not match.group("path").endswith(".json")
        ):
            return "executable subject must be oracle:<repo-relative-json-path>"
    else:
        match = REGEX_SUBJECT.fullmatch(subject)
        if match is None or not _repo_relative(match.group("path")):
            return "exact-population subject must be regex:<repo-relative-path>#<pattern>"
        try:
            compiled = re.compile(match.group("pattern"), re.MULTILINE)
        except re.error as error:
            return f"exact-population subject has invalid regex: {error}"
        if compiled.groups != 1:
            return "exact-population regex must contain exactly one capture group"
        if not match.group("pattern").startswith("^") or not match.group("pattern").endswith("$"):
            return "exact-population regex must be a whole-line ^...$ extractor"
    return None


def claim_expectation_problem(kind: object, expected: object) -> str | None:
    if not isinstance(expected, list) or any(
        not isinstance(value, str) or not value for value in expected
    ):
        return "claim expectation must be a non-empty string list"
    if len(expected) != len(set(expected)):
        return "claim expectation must not contain duplicates"
    if kind == "exact_population" and len(expected) < 2:
        return "exact-population claims require at least two decision-relevant members"
    return None
