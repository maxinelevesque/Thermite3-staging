#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Bidirectional consistency gate for the language-completeness review track."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from claim_closure_schema import (
    MECHANISMS,
    canonical_json,
    claim_digest,
    claim_expectation_problem,
    claim_subject_problem,
)

INVENTORY = "gates/language-completeness-inventory.toml"
BACKLOG = "gates/completeness-review.toml"
GATE = "gates/completeness-review.py"
SCHEMA = "gates/claim_closure_schema.py"
EVIDENCE_SUFFIXES = {".lean", ".rs", ".py", ".sh", ".json", ".toml"}
REF = re.compile(r"^([^#]+)#(.+)$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
EXACT_POPULATION_MUTATIONS = {
    "addition",
    "duplication",
    "omission",
    "substitution",
}
BASELINE_SHIPPED_COUNT = 566


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


def artifact_digest(root: Path, reference: object) -> tuple[str, str] | None:
    if not isinstance(reference, str) or not reference:
        return None
    path_text = reference.split("#", 1)[0]
    relative = Path(path_text)
    if relative.is_absolute() or ".." in relative.parts:
        return None
    path = (root / relative).resolve()
    try:
        path.relative_to(root.resolve())
    except ValueError:
        return None
    if not path.is_file():
        return None
    return reference, hashlib.sha256(path.read_bytes()).hexdigest()


def gate_version(root: Path) -> str | None:
    paths = [root / GATE, root / SCHEMA]
    if any(not path.is_file() for path in paths):
        return None
    payload = [
        (str(path.relative_to(root)), hashlib.sha256(path.read_bytes()).hexdigest())
        for path in paths
    ]
    return hashlib.sha256(canonical_json(payload)).hexdigest()


def _string_list(raw: dict, field: str) -> list[str] | None:
    value = raw.get(field)
    if not isinstance(value, list) or any(not isinstance(v, str) or not v for v in value):
        return None
    return value


def run_verifier(root: Path, argv: object) -> tuple[int, str]:
    if not isinstance(argv, list) or not argv or any(not isinstance(v, str) for v in argv):
        return 127, "invalid verifier argv"
    try:
        result = subprocess.run(
            argv,
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        observation = hashlib.sha256(
            canonical_json(
                {
                    "error": type(error).__name__,
                    "returncode": 127,
                    "stderr": "",
                    "stdout": "",
                }
            )
        ).hexdigest()
        return 127, observation
    observation = hashlib.sha256(
        canonical_json(
            {
                "returncode": result.returncode,
                "stderr": result.stderr,
                "stdout": result.stdout,
            }
        )
    ).hexdigest()
    return result.returncode, observation


def command_version(root: Path, argv: object) -> str | None:
    if not isinstance(argv, list) or not argv or any(not isinstance(v, str) for v in argv):
        return None
    result = subprocess.run(
        argv, cwd=root, capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        return None
    return (result.stdout or result.stderr).strip()


def bound_input_path(root: Path, reference: object) -> Path | None:
    if not isinstance(reference, str) or not reference or "#" in reference:
        return None
    relative = Path(reference)
    if relative.is_absolute() or ".." in relative.parts:
        return None
    path = (root / relative).resolve()
    try:
        path.relative_to(root.resolve())
    except ValueError:
        return None
    return path if path.is_file() else None


def run_bound_verifier(
    root: Path, argv: object, input_reference: object
) -> tuple[int, str]:
    path = bound_input_path(root, input_reference)
    if path is None:
        return 127, "unresolved bound input"
    if not isinstance(argv, list) or not argv or any(not isinstance(v, str) for v in argv):
        return 127, "invalid verifier argv"
    with tempfile.TemporaryDirectory() as tmpdir:
        copied = Path(tmpdir) / path.name
        copied.write_bytes(path.read_bytes())
        return run_verifier(root, [*argv, str(copied)])


def run_mutated_verifier(
    root: Path,
    argv: object,
    oracle_reference: object,
    counterfeit: object,
) -> tuple[int, str]:
    oracle = bound_input_path(root, oracle_reference)
    if oracle is None or not isinstance(counterfeit, dict):
        return 127, "invalid oracle or counterfeit"
    mutation = counterfeit.get("mutation")
    if mutation != "replace_text":
        return 127, "executable counterfeits must use semantic replace_text"
    source = counterfeit.get("from")
    replacement = counterfeit.get("to")
    if (
        not isinstance(source, str)
        or not source
        or not isinstance(replacement, str)
        or source == replacement
    ):
        return 127, "replace_text requires distinct non-empty from/to strings"
    try:
        text = oracle.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return 127, "replace_text oracle must be UTF-8"
    if text.count(source) != 1:
        return 127, "replace_text source must occur exactly once"
    mutated_text = text.replace(source, replacement, 1)
    try:
        json.loads(mutated_text)
    except json.JSONDecodeError:
        return 127, "semantic counterfeit must remain valid JSON"
    payload = mutated_text.encode("utf-8")
    with tempfile.TemporaryDirectory() as tmpdir:
        mutated = Path(tmpdir) / oracle.name
        mutated.write_bytes(payload)
        return run_verifier(root, [*argv, str(mutated)])


def extract_population(root: Path, extractor: object) -> tuple[list[str] | None, str]:
    if not isinstance(extractor, dict):
        return None, "extractor must be a table"
    if extractor.get("kind") != "regex":
        return None, "only the closed regex extractor is accepted"
    path = bound_input_path(root, extractor.get("path"))
    pattern = extractor.get("pattern")
    if path is None or not isinstance(pattern, str) or not pattern:
        return None, "regex extractor path and pattern must resolve"
    try:
        compiled = re.compile(pattern, re.MULTILINE)
    except re.error as error:
        return None, f"invalid extractor regex: {error}"
    text = path.read_text(encoding="utf-8")
    values: list[str] = []
    for match in compiled.finditer(text):
        if match.lastindex != 1:
            return None, "extractor regex must contain exactly one capture group"
        values.append(match.group(1))
    return sorted(values), ""


def hostile_extractor_populations(
    root: Path, extractor: object
) -> tuple[dict[str, list[str]] | None, str]:
    if not isinstance(extractor, dict) or extractor.get("kind") != "regex":
        return None, "hostile mutations require the closed regex extractor"
    path = bound_input_path(root, extractor.get("path"))
    pattern = extractor.get("pattern")
    if path is None or not isinstance(pattern, str) or not pattern:
        return None, "hostile mutation extractor does not resolve"
    try:
        compiled = re.compile(pattern, re.MULTILINE)
    except re.error as error:
        return None, f"invalid extractor regex: {error}"
    source = path.read_text(encoding="utf-8")
    matches = list(compiled.finditer(source))
    if not matches or matches[0].lastindex != 1:
        return None, "hostile mutations require at least one single-capture match"
    first = matches[0]
    sentinel = "__claim_closure_counterfeit__"
    whole = first.group(0)
    capture_start, capture_end = first.span(1)
    match_start, match_end = first.span(0)
    relative_start = capture_start - match_start
    relative_end = capture_end - match_start
    added_match = whole[:relative_start] + sentinel + whole[relative_end:]
    mutated_sources = {
        "addition": source + "\n" + added_match + "\n",
        "duplication": source[:match_end] + "\n" + whole + source[match_end:],
        "omission": source[:match_start] + source[match_end:],
        "substitution": source[:capture_start] + sentinel + source[capture_end:],
    }
    observations: dict[str, list[str]] = {}
    for name, mutated_source in mutated_sources.items():
        values: list[str] = []
        for match in compiled.finditer(mutated_source):
            if match.lastindex != 1:
                return None, "mutated extractor lost its single-capture shape"
            values.append(match.group(1))
        observations[name] = sorted(values)
    return observations, ""


def probe_lean_theorem(root: Path, subject: object) -> tuple[list[str] | None, str]:
    if not isinstance(subject, str) or "#" not in subject:
        return None, "formal theorem subject must be path#declaration"
    path_text, declaration = subject.split("#", 1)
    path = root / path_text
    if not path.is_file() or path.suffix != ".lean" or not declaration:
        return None, "formal theorem subject must resolve to a Lean file and declaration"
    try:
        rel = path.relative_to(root / "lean").with_suffix("")
    except ValueError:
        return None, "formal theorem modules must live under lean/"
    module = ".".join(rel.parts)
    if shutil.which("lake") is None:
        return None, "lake is unavailable"
    version_result = subprocess.run(
        ["lake", "--version"],
        cwd=root / "lean",
        capture_output=True,
        text=True,
        check=False,
    )
    if version_result.returncode != 0:
        return None, "lake version probe failed"
    tool_version = (version_result.stdout or version_result.stderr).strip()
    with tempfile.TemporaryDirectory() as tmpdir:
        probe = Path(tmpdir) / "ClaimClosureProbe.lean"
        probe.write_text(
            f"import {module}\n#check {declaration}\n#print axioms {declaration}\n",
            encoding="utf-8",
        )
        result = subprocess.run(
            ["lake", "env", "lean", str(probe)],
            cwd=root / "lean",
            capture_output=True,
            text=True,
            check=False,
        )
    if result.returncode != 0:
        return None, f"Lean theorem probe failed: {result.stderr.strip()}"
    output = result.stdout + result.stderr
    axiom_marker = f"'{declaration}'"
    marker_index = output.find(axiom_marker)
    if marker_index < 0:
        return None, "Lean theorem probe emitted no declaration-specific axiom report"
    normalized_type = " ".join(output[:marker_index].split())
    if not normalized_type:
        return None, "Lean theorem probe emitted no declaration type"
    type_digest = hashlib.sha256(normalized_type.encode("utf-8")).hexdigest()
    if "does not depend on any axioms" in output:
        axioms: list[str] = []
    else:
        match = re.search(r"depends on axioms:\s*\[(.*?)\]", output, re.DOTALL)
        if match is None:
            return None, "Lean theorem probe emitted no parseable axiom report"
        axioms = sorted(
            value.strip() for value in match.group(1).split(",") if value.strip()
        )
    return [
        f"declaration:{declaration}",
        f"type_sha256:{type_digest}",
        "axioms:" + ",".join(axioms),
        f"tool:{tool_version}",
    ], output


def discriminator_digest(root: Path, closure: dict, observed: object) -> str | None:
    artifacts = _string_list(closure, "artifacts")
    if artifacts is None:
        return None
    artifact_digests = [artifact_digest(root, reference) for reference in artifacts]
    if any(value is None for value in artifact_digests):
        return None
    payload = {
        "artifacts": artifact_digests,
        "counterfeits": closure.get("counterfeit"),
        "mechanism": closure.get("mechanism"),
        "observed": observed,
        "oracle": closure.get("oracle"),
        "population_semantics": closure.get("population_semantics"),
        "subject": closure.get("subject"),
        "extractor": closure.get("extractor"),
        "tool_version": closure.get("tool_version"),
        "verifier": closure.get("verifier"),
    }
    return hashlib.sha256(canonical_json(payload)).hexdigest()


def closure_receipt_payload(
    root: Path,
    closure: dict,
    *,
    observed: str,
) -> dict | None:
    artifacts = _string_list(closure, "artifacts")
    expected = _string_list(closure, "expected")
    if artifacts is None or expected is None:
        return None
    artifact_digests = [artifact_digest(root, reference) for reference in artifacts]
    if any(value is None for value in artifact_digests):
        return None
    return {
        "artifacts": artifact_digests,
        "claim_digest": closure.get("claim_digest"),
        "counterfeits": closure.get("counterfeit", []),
        "discriminator": closure.get("discriminator"),
        "expected": expected,
        "mechanism": closure.get("mechanism"),
        "observed": observed,
        "oracle": closure.get("oracle"),
        "population_semantics": closure.get("population_semantics"),
        "subject": closure.get("subject"),
        "extractor": closure.get("extractor"),
        "requirement_id": closure.get("requirement_id"),
        "verifier": closure.get("verifier"),
        "verifier_version": closure.get("verifier_version"),
        "tool_version": closure.get("tool_version"),
        "tool_version_argv": closure.get("tool_version_argv"),
        "witness_id": closure.get("witness_id"),
    }


def closure_receipt(root: Path, closure: dict, *, observed: str) -> str | None:
    payload = closure_receipt_payload(root, closure, observed=observed)
    if payload is None:
        return None
    return hashlib.sha256(canonical_json(payload)).hexdigest()


def check(
    root: Path,
    *,
    backlog_document: dict | None = None,
    registry_document: dict | None = None,
) -> list[str]:
    try:
        inventory = tomllib.loads((root / INVENTORY).read_text(encoding="utf-8"))
        backlog = backlog_document or tomllib.loads(
            (root / BACKLOG).read_text(encoding="utf-8")
        )
        registry = registry_document or tomllib.loads(
            (root / ".design/reqs/registry.toml").read_text(encoding="utf-8")
        )
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [f"track input unreadable: {error}"]
    backlog_version = backlog.get("version")
    if backlog_version not in {1, 2}:
        return ["backlog version must be 1 or 2"]

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

    raw_requirements = registry.get("requirement", [])
    if not isinstance(raw_requirements, list):
        return ["requirement registry must contain a requirement list"]
    shipped: dict[str, dict] = {}
    for raw_req in raw_requirements:
        if not isinstance(raw_req, dict) or raw_req.get("status") != "shipped":
            continue
        req_id = raw_req.get("id")
        if not isinstance(req_id, str) or not req_id:
            problems.append("shipped registry row has no requirement ID")
            continue
        shipped[req_id] = raw_req

    baseline = backlog.get("baseline_shipped_ids")
    if not isinstance(baseline, list) or any(not isinstance(v, str) for v in baseline):
        problems.append("baseline_shipped_ids must be a list of requirement IDs")
        baseline = []
    if len(baseline) != len(set(baseline)):
        problems.append("baseline_shipped_ids must not contain duplicates")
    if len(baseline) != BASELINE_SHIPPED_COUNT:
        problems.append(
            "baseline_shipped_ids must freeze exactly "
            f"{BASELINE_SHIPPED_COUNT} IDs, found {len(baseline)}"
        )
    unknown_baseline = sorted(set(baseline) - set(shipped))
    if unknown_baseline:
        problems.append(
            "baseline shipped IDs are no longer shipped: " + ", ".join(unknown_baseline)
        )

    # Version 1 freezes and audits the migration population while the existing
    # structural registry remains authoritative. Opting the ledger and registry
    # into version 2 is atomic: every shipped typed claim and closure below then
    # becomes mandatory, with no partial-enforcement escape hatch.
    if backlog_version == 1:
        if registry.get("schema_version") != 1:
            problems.append("version-1 closure ledger requires registry schema version 1")
        if backlog.get("witness") or backlog.get("closure"):
            problems.append("version-1 closure ledger must not predeclare v2 witnesses or closures")
        if any(
            isinstance(row, dict) and "claim" in row
            for row in raw_requirements
        ):
            problems.append("version-1 registry must not predeclare v2 typed claims")
        return problems
    if registry.get("schema_version") != 2:
        problems.append("version-2 closure ledger requires registry schema version 2")

    raw_witnesses = backlog.get("witness", [])
    if not isinstance(raw_witnesses, list):
        problems.append("witness must be a list of tables")
        raw_witnesses = []
    witnesses: dict[str, dict] = {}
    for witness in raw_witnesses:
        if not isinstance(witness, dict):
            problems.append("witness entries must be tables")
            continue
        witness_id = witness.get("id")
        if not isinstance(witness_id, str) or not witness_id:
            problems.append("witness id must be a non-empty string")
            continue
        if witness_id in witnesses:
            problems.append(f"duplicate witness id: {witness_id}")
        witnesses[witness_id] = witness
        if witness.get("mechanism") not in MECHANISMS:
            problems.append(f"{witness_id}: unknown witness mechanism")
        members = _string_list(witness, "members")
        if members is None:
            problems.append(f"{witness_id}: members must be a non-empty string list")
        elif len(members) != len(set(members)):
            problems.append(f"{witness_id}: members must be exact and unique")

    raw_closures = backlog.get("closure", [])
    if not isinstance(raw_closures, list):
        problems.append("closure must be a list of tables")
        raw_closures = []
    closures: dict[str, dict] = {}
    witness_members_from_closures: dict[str, set[str]] = {}
    witness_discriminators: dict[str, set[str]] = {}
    global_discriminators: dict[str, str] = {}
    witness_identity_owners: dict[str, str] = {}
    verifier_cache: dict[tuple[str, ...], tuple[int, str]] = {}

    for closure in raw_closures:
        if not isinstance(closure, dict):
            problems.append("closure entries must be tables")
            continue
        req_id = closure.get("requirement_id")
        if not isinstance(req_id, str) or not req_id:
            problems.append("closure requirement_id must be a non-empty string")
            continue
        if req_id in closures:
            problems.append(f"{req_id}: duplicate closure entry")
        closures[req_id] = closure
        raw_req = shipped.get(req_id)
        if raw_req is None:
            problems.append(f"{req_id}: closure does not name a shipped requirement")
            continue

        mechanism = closure.get("mechanism")
        if mechanism not in MECHANISMS:
            problems.append(f"{req_id}: unknown closure mechanism {mechanism!r}")
            continue
        common_fields = {
            "artifacts",
            "claim_digest",
            "counterfeit",
            "discriminator",
            "expected",
            "mechanism",
            "receipt",
            "requirement_id",
            "verifier",
            "verifier_version",
            "witness_id",
        }
        mechanism_fields = {
            "formal_theorem": {"subject"},
            "executable_discriminator": {
                "oracle",
                "tool_version",
                "tool_version_argv",
            },
            "exact_population": {"extractor", "population_semantics"},
        }
        unknown_fields = sorted(
            set(closure) - common_fields - mechanism_fields[mechanism]
        )
        if unknown_fields:
            problems.append(
                f"{req_id}: closure contains mechanism-incompatible field(s): "
                + ", ".join(unknown_fields)
            )
        claim = raw_req.get("claim")
        digest = claim_digest(req_id, claim)
        if digest is None:
            problems.append(f"{req_id}: shipped requirement has no well-typed claim")
            continue
        subject_problem = claim_subject_problem(claim.get("kind"), claim.get("subject"))
        if subject_problem is not None:
            problems.append(f"{req_id}: {subject_problem}")
            continue
        expectation_problem = claim_expectation_problem(
            claim.get("kind"), claim.get("expected")
        )
        if expectation_problem is not None:
            problems.append(f"{req_id}: {expectation_problem}")
            continue
        if isinstance(claim, dict) and claim.get("kind") != mechanism:
            problems.append(f"{req_id}: claim kind and closure mechanism differ")
        if closure.get("claim_digest") != digest:
            problems.append(f"{req_id}: closure claim digest is missing or stale")

        witness_id = closure.get("witness_id")
        if not isinstance(witness_id, str) or witness_id not in witnesses:
            problems.append(f"{req_id}: closure witness_id does not resolve")
            continue
        witness = witnesses[witness_id]
        if witness.get("mechanism") != mechanism:
            problems.append(f"{req_id}: closure mechanism differs from its witness")
        witness_members_from_closures.setdefault(witness_id, set()).add(req_id)

        artifacts = _string_list(closure, "artifacts")
        if artifacts is None or not artifacts:
            problems.append(f"{req_id}: closure must bind at least one artifact")
        elif any(artifact_digest(root, value) is None for value in artifacts):
            problems.append(f"{req_id}: one or more closure artifacts do not resolve")
        expected = _string_list(closure, "expected")
        if expected is None or not expected:
            problems.append(f"{req_id}: closure expected observations are required")
        elif isinstance(claim, dict) and expected != claim.get("expected"):
            problems.append(f"{req_id}: closure expectation differs from its typed claim")
        verifier_version = closure.get("verifier_version")
        active_gate_version = gate_version(root)
        if verifier_version != active_gate_version:
            problems.append(f"{req_id}: verifier_version is missing or stale")
        artifact_paths = {
            value.split("#", 1)[0] for value in (artifacts or [])
        }
        missing_kernel = sorted({GATE, SCHEMA} - artifact_paths)
        if missing_kernel:
            problems.append(
                f"{req_id}: closure must content-bind its verification kernel: "
                + ", ".join(missing_kernel)
            )

        observed: object = None
        if mechanism == "formal_theorem":
            if closure.get("verifier") != ["builtin:lean_axioms"]:
                problems.append(f"{req_id}: formal theorem must use the built-in Lean prober")
            subject = claim.get("subject") if isinstance(claim, dict) else None
            if closure.get("subject") != subject:
                problems.append(f"{req_id}: formal closure subject differs from its typed claim")
            formal_path = subject.split("#", 1)[0] if isinstance(subject, str) else ""
            if formal_path not in artifact_paths:
                problems.append(f"{req_id}: theorem module must be a content-bound artifact")
            formal_observed, detail = probe_lean_theorem(root, subject)
            if formal_observed is None:
                problems.append(f"{req_id}: {detail}")
                continue
            observed = formal_observed
            if observed != expected:
                problems.append(f"{req_id}: Lean theorem observation differs from expectation")
        elif mechanism == "executable_discriminator":
            argv = closure.get("verifier")
            oracle = closure.get("oracle")
            if not isinstance(argv, list) or not argv or any(
                not isinstance(v, str) or not v for v in argv
            ):
                problems.append(f"{req_id}: verifier must be a non-empty argv list")
                continue
            oracle_path = bound_input_path(root, oracle)
            if oracle_path is None or oracle_path.suffix != ".json":
                problems.append(f"{req_id}: executable oracle must be repo-relative JSON")
                continue
            try:
                json.loads(oracle_path.read_text(encoding="utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError):
                problems.append(f"{req_id}: executable positive oracle must be valid JSON")
                continue
            version_argv = closure.get("tool_version_argv")
            if (
                not isinstance(version_argv, list)
                or not version_argv
                or version_argv[0] != argv[0]
            ):
                problems.append(f"{req_id}: tool-version probe must share verifier identity")
            elif command_version(root, version_argv) != closure.get("tool_version"):
                problems.append(f"{req_id}: executable verifier tool version is stale")
            repo_verifier_inputs = {
                value
                for value in argv[1:]
                if isinstance(value, str) and (root / value).is_file()
            }
            if not repo_verifier_inputs.issubset(artifact_paths):
                problems.append(f"{req_id}: verifier implementation must be content-bound")
            if not isinstance(claim, dict) or claim.get("subject") != f"oracle:{oracle}":
                problems.append(f"{req_id}: executable claim must bind its oracle input")
            if expected != ["accepted"]:
                problems.append(f"{req_id}: executable positive observation must be accepted")
            if oracle not in (artifacts or []):
                problems.append(f"{req_id}: oracle must be a content-bound artifact")
            key = (*argv, str(oracle))
            result = verifier_cache.get(key)
            if result is None:
                result = run_bound_verifier(root, argv, oracle)
                verifier_cache[key] = result
            returncode, observation_digest = result
            repeated = run_bound_verifier(root, argv, oracle)
            if repeated != result:
                problems.append(f"{req_id}: positive verifier is not deterministic")
            observed = ["accepted"] if returncode == 0 else [f"exit:{returncode}"]
            if observed != expected:
                problems.append(f"{req_id}: positive oracle observation differs from expectation")
            if returncode != 0:
                problems.append(f"{req_id}: positive oracle must exit 0")
            counterfeits = closure.get("counterfeit")
            if not isinstance(counterfeits, list) or not counterfeits:
                problems.append(f"{req_id}: executable discriminator needs named counterfeits")
            else:
                names: set[str] = set()
                for counterfeit in counterfeits:
                    if not isinstance(counterfeit, dict):
                        problems.append(f"{req_id}: counterfeit entries must be tables")
                        continue
                    name = counterfeit.get("name")
                    unknown_counterfeit_fields = sorted(
                        set(counterfeit)
                        - {"expected_exit", "from", "mutation", "name", "to"}
                    )
                    if unknown_counterfeit_fields:
                        problems.append(
                            f"{req_id}: counterfeit contains unknown field(s): "
                            + ", ".join(unknown_counterfeit_fields)
                        )
                    expected_exit = counterfeit.get("expected_exit")
                    if not isinstance(name, str) or not name or name in names:
                        problems.append(f"{req_id}: counterfeit names must be non-empty and unique")
                        continue
                    names.add(name)
                    if counterfeit.get("mutation") != "replace_text":
                        problems.append(
                            f"{req_id}/{name}: counterfeit must use semantic replace_text"
                        )
                    if (
                        not isinstance(expected_exit, int)
                        or expected_exit <= 0
                        or expected_exit >= 127
                    ):
                        problems.append(f"{req_id}/{name}: counterfeit must expect rejection")
                        continue
                    first_counterfeit = run_mutated_verifier(
                        root, argv, oracle, counterfeit
                    )
                    repeated_counterfeit = run_mutated_verifier(
                        root, argv, oracle, counterfeit
                    )
                    ccode, _ = first_counterfeit
                    if ccode != expected_exit:
                        problems.append(
                                f"{req_id}/{name}: counterfeit returned {ccode}, expected {expected_exit}"
                            )
                    if ccode == returncode:
                        problems.append(f"{req_id}/{name}: counterfeit did not discriminate")
                    if first_counterfeit != repeated_counterfeit:
                        problems.append(f"{req_id}/{name}: counterfeit verifier is not deterministic")
            observed = {"result": observed, "output_digest": observation_digest}
        else:
            if closure.get("verifier") != ["builtin:exact_population"]:
                problems.append(f"{req_id}: exact population must use the built-in verifier")
            if closure.get("population_semantics") not in {
                "closed_case_set",
                "closed_enum",
                "closed_route_set",
                "closed_table",
            }:
                problems.append(
                    f"{req_id}: exact population must name a closed-set semantic class"
                )
            extractor = closure.get("extractor")
            extracted, detail = extract_population(root, extractor)
            if extracted is None:
                problems.append(f"{req_id}: {detail}")
                continue
            if not isinstance(extractor, dict):
                problems.append(f"{req_id}: exact population extractor is invalid")
                continue
            extractor_subject = (
                f"regex:{extractor.get('path')}#{extractor.get('pattern')}"
            )
            if not isinstance(claim, dict) or claim.get("subject") != extractor_subject:
                problems.append(f"{req_id}: exact-population claim does not bind its extractor")
            if extractor.get("path") not in artifact_paths:
                problems.append(f"{req_id}: extractor input must be a content-bound artifact")
            if extracted != expected:
                problems.append(f"{req_id}: extracted population differs from expectation")
            mutations = closure.get("counterfeit")
            if isinstance(mutations, list):
                for mutation in mutations:
                    if isinstance(mutation, dict) and set(mutation) != {"name"}:
                        problems.append(
                            f"{req_id}: exact-population counterfeit accepts only `name`"
                        )
            names = {
                value.get("name")
                for value in mutations
                if isinstance(value, dict) and isinstance(value.get("name"), str)
            } if isinstance(mutations, list) else set()
            if names != EXACT_POPULATION_MUTATIONS:
                problems.append(
                    f"{req_id}: exact population must name addition, duplication, omission, and substitution mutations"
                )
            hostile_observations, hostile_detail = hostile_extractor_populations(
                root, extractor
            )
            if hostile_observations is None:
                problems.append(f"{req_id}: {hostile_detail}")
            else:
                for name, population in hostile_observations.items():
                    if population == expected:
                        problems.append(f"{req_id}: hostile {name} mutation was not rejected")
            observed = extracted

        expected_discriminator = discriminator_digest(root, closure, observed)
        discriminator = closure.get("discriminator")
        if expected_discriminator is None or discriminator != expected_discriminator:
            problems.append(f"{req_id}: per-claim discriminator is missing or stale")
        elif discriminator in witness_discriminators.setdefault(witness_id, set()):
            problems.append(
                f"{req_id}: discriminator result is reused within {witness_id}"
            )
        else:
            witness_discriminators[witness_id].add(discriminator)
            previous_req = global_discriminators.get(discriminator)
            if previous_req is not None:
                problems.append(
                    f"{req_id}: discriminator result is already used by {previous_req}"
                )
            global_discriminators[discriminator] = req_id

        if mechanism == "formal_theorem":
            witness_identity_payload = {
                "mechanism": mechanism,
                "verifier": closure.get("verifier"),
            }
        elif mechanism == "executable_discriminator":
            witness_identity_payload = {
                "mechanism": mechanism,
                "oracle": closure.get("oracle"),
                "tool_version": closure.get("tool_version"),
                "verifier": closure.get("verifier"),
            }
        else:
            witness_identity_payload = {
                "extractor": closure.get("extractor"),
                "mechanism": mechanism,
            }
        witness_identity = hashlib.sha256(
            canonical_json(witness_identity_payload)
        ).hexdigest()
        previous_witness = witness_identity_owners.get(witness_identity)
        if previous_witness is not None and previous_witness != witness_id:
            problems.append(
                f"{req_id}: equivalent witness identity is split across "
                f"{previous_witness} and {witness_id}"
            )
        witness_identity_owners[witness_identity] = witness_id

        receipt = closure_receipt(root, closure, observed=observed)
        if receipt is None:
            problems.append(f"{req_id}: closure receipt inputs are incomplete")
        elif closure.get("receipt") != receipt:
            problems.append(f"{req_id}: closure receipt is missing or stale")

    expected_population = set(shipped) | set(baseline)
    missing_closures = sorted(expected_population - set(closures))
    extra_closures = sorted(set(closures) - expected_population)
    if missing_closures:
        problems.append("shipped requirements lack closure: " + ", ".join(missing_closures))
    if extra_closures:
        problems.append("closure contains non-population IDs: " + ", ".join(extra_closures))

    for witness_id, witness in witnesses.items():
        declared = set(witness.get("members", []))
        actual = witness_members_from_closures.get(witness_id, set())
        if declared != actual:
            problems.append(
                f"{witness_id}: declared members differ from exact closure membership"
            )
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
