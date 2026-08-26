#!/usr/bin/env python3
"""Author and audit the frozen claim-closure migration population."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import os
import sys
import tempfile
import tomllib
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from claim_closure_schema import (  # noqa: E402
    claim_expectation_problem,
    claim_subject_problem,
)

REGISTRY = ".design/reqs/registry.toml"
LEDGER = "gates/completeness-review.toml"
DRAFT_DIR = "gates/claim-closure-drafts"
BASELINE_SIZE = 566


def load_review_module():
    path = Path(__file__).with_name("completeness-review.py")
    spec = importlib.util.spec_from_file_location("thermite_claim_closure_review", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


REVIEW = load_review_module()


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


def requirement_rows(root: Path) -> tuple[dict[str, dict], dict]:
    registry = tomllib.loads((root / REGISTRY).read_text(encoding="utf-8"))
    rows = {
        row["id"]: row
        for row in registry.get("requirement", [])
        if isinstance(row, dict) and isinstance(row.get("id"), str)
    }
    return rows, registry


def load_draft_entries(root: Path) -> tuple[list[dict], list[str]]:
    directory = root / DRAFT_DIR
    entries: list[dict] = []
    problems: list[str] = []
    if not directory.exists():
        return entries, problems
    for path in sorted(directory.glob("*.json")):
        try:
            raw = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
            problems.append(f"{path.relative_to(root)}: unreadable draft: {error}")
            continue
        if not isinstance(raw, dict) or raw.get("version") != 1:
            problems.append(f"{path.relative_to(root)}: draft version must be 1")
            continue
        if set(raw) != {"entries", "slice_id", "version"}:
            problems.append(f"{path.relative_to(root)}: draft has unknown top-level fields")
        slice_id = raw.get("slice_id")
        if not isinstance(slice_id, str) or not slice_id:
            problems.append(f"{path.relative_to(root)}: slice_id must be non-empty")
        raw_entries = raw.get("entries")
        if not isinstance(raw_entries, list):
            problems.append(f"{path.relative_to(root)}: entries must be a list")
            continue
        for entry in raw_entries:
            if not isinstance(entry, dict):
                problems.append(f"{path.relative_to(root)}: entry must be an object")
                continue
            copied = dict(entry)
            copied["_draft_path"] = str(path.relative_to(root))
            copied["_slice_id"] = slice_id
            entries.append(copied)
    return entries, problems


def _artifact_paths(closure: dict) -> set[str]:
    artifacts = closure.get("artifacts", [])
    if not isinstance(artifacts, list):
        return set()
    return {value.split("#", 1)[0] for value in artifacts if isinstance(value, str)}


def author_entry(root: Path, entry: dict, row: dict) -> tuple[dict | None, list[str]]:
    req_id = entry.get("requirement_id")
    label = str(req_id or entry.get("_draft_path", "<draft>"))
    problems: list[str] = []
    if set(entry) - {
        "_draft_path",
        "_slice_id",
        "claim",
        "closure",
        "requirement_id",
        "witness_id",
    }:
        problems.append(f"{label}: draft entry has unknown fields")
    claim = entry.get("claim")
    closure_input = entry.get("closure")
    witness_id = entry.get("witness_id")
    if not isinstance(claim, dict) or not isinstance(closure_input, dict):
        return None, [f"{label}: claim and closure must be objects"]
    if not isinstance(witness_id, str) or not witness_id:
        problems.append(f"{label}: witness_id must be non-empty")
    allowed_claim_fields = {
        "expected",
        "kind",
        "reviewed_summary_sha256",
        "subject",
    }
    if set(claim) != allowed_claim_fields:
        problems.append(f"{label}: claim fields must be exact")
    kind = claim.get("kind")
    subject_problem = claim_subject_problem(kind, claim.get("subject"))
    if subject_problem:
        problems.append(f"{label}: {subject_problem}")
    expectation_problem = claim_expectation_problem(kind, claim.get("expected"))
    if expectation_problem:
        problems.append(f"{label}: {expectation_problem}")
    summary = row.get("summary")
    if not isinstance(summary, str) or not summary:
        problems.append(f"{label}: shipped draft row needs a non-empty summary")
        summary = ""
    reviewed = hashlib.sha256(summary.encode("utf-8")).hexdigest()
    if claim.get("reviewed_summary_sha256") != reviewed:
        problems.append(f"{label}: reviewed summary digest is missing or stale")

    closure = dict(closure_input)
    closure.update(
        {
            "requirement_id": req_id,
            "witness_id": witness_id,
            "mechanism": kind,
            "expected": claim.get("expected"),
            "claim_digest": REVIEW.claim_digest(req_id, claim),
            "verifier_version": REVIEW.gate_version(root),
        }
    )
    artifacts = closure.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts:
        problems.append(f"{label}: closure artifacts must be a non-empty list")
    elif any(REVIEW.artifact_digest(root, value) is None for value in artifacts):
        problems.append(f"{label}: closure artifacts must resolve")
    missing_kernel = {REVIEW.GATE, REVIEW.SCHEMA} - _artifact_paths(closure)
    if missing_kernel:
        problems.append(
            f"{label}: closure must bind verification kernel: "
            + ", ".join(sorted(missing_kernel))
        )

    observed: object | None = None
    if kind == "formal_theorem":
        closure["verifier"] = ["builtin:lean_axioms"]
        closure["subject"] = claim.get("subject")
        formal_path = str(claim.get("subject", "")).split("#", 1)[0]
        if formal_path not in _artifact_paths(closure):
            problems.append(f"{label}: theorem module must be content-bound")
        observed, detail = REVIEW.probe_lean_theorem(root, claim.get("subject"))
        if observed is None:
            problems.append(f"{label}: {detail}")
        elif observed != claim.get("expected"):
            problems.append(f"{label}: formal theorem observation differs")
    elif kind == "executable_discriminator":
        argv = closure.get("verifier")
        oracle = closure.get("oracle")
        version_argv = closure.get("tool_version_argv")
        closure["tool_version"] = REVIEW.command_version(root, version_argv)
        if claim.get("subject") != f"oracle:{oracle}":
            problems.append(f"{label}: executable claim does not bind its oracle")
        if claim.get("expected") != ["accepted"]:
            problems.append(f"{label}: executable expectation must be accepted")
        if oracle not in (artifacts or []):
            problems.append(f"{label}: executable oracle must be content-bound")
        oracle_path = REVIEW.bound_input_path(root, oracle)
        if oracle_path is None or oracle_path.suffix != ".json":
            problems.append(f"{label}: executable oracle must be repo-relative JSON")
        else:
            try:
                json.loads(oracle_path.read_text(encoding="utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError):
                problems.append(f"{label}: executable positive oracle must be valid JSON")
        if (
            not isinstance(argv, list)
            or not argv
            or not isinstance(version_argv, list)
            or not version_argv
            or version_argv[0] != argv[0]
        ):
            problems.append(f"{label}: executable verifier/version identity is invalid")
        repo_verifier_inputs = {
            value
            for value in (argv or [])[1:]
            if isinstance(value, str) and (root / value).is_file()
        }
        if not repo_verifier_inputs.issubset(_artifact_paths(closure)):
            problems.append(f"{label}: verifier implementation must be content-bound")
        first = REVIEW.run_bound_verifier(root, argv, oracle)
        second = REVIEW.run_bound_verifier(root, argv, oracle)
        if first != second:
            problems.append(f"{label}: positive verifier is not deterministic")
        if first[0] != 0:
            problems.append(f"{label}: positive oracle must exit 0")
        counterfeits = closure.get("counterfeit")
        if not isinstance(counterfeits, list) or not counterfeits:
            problems.append(f"{label}: executable closure needs counterfeits")
        else:
            for counterfeit in counterfeits:
                if not isinstance(counterfeit, dict):
                    problems.append(f"{label}: counterfeit must be an object")
                    continue
                if set(counterfeit) - {
                    "expected_exit",
                    "from",
                    "mutation",
                    "name",
                    "to",
                }:
                    problems.append(f"{label}: counterfeit has unknown fields")
                if counterfeit.get("mutation") != "replace_text":
                    problems.append(f"{label}: counterfeit must use replace_text")
                expected_exit = counterfeit.get("expected_exit")
                if (
                    not isinstance(expected_exit, int)
                    or expected_exit <= 0
                    or expected_exit >= 127
                ):
                    problems.append(f"{label}: counterfeit exit must be 1..126")
                negative = REVIEW.run_mutated_verifier(root, argv, oracle, counterfeit)
                repeated = REVIEW.run_mutated_verifier(root, argv, oracle, counterfeit)
                if negative != repeated:
                    problems.append(f"{label}: counterfeit is not deterministic")
                if negative[0] != counterfeit.get("expected_exit"):
                    problems.append(f"{label}: counterfeit rejection differs")
                if negative[0] == first[0]:
                    problems.append(f"{label}: counterfeit did not discriminate")
        observed = {
            "result": ["accepted"] if first[0] == 0 else [f"exit:{first[0]}"],
            "output_digest": first[1],
        }
    elif kind == "exact_population":
        closure["verifier"] = ["builtin:exact_population"]
        extractor = closure.get("extractor")
        extractor_subject = None
        if isinstance(extractor, dict):
            extractor_subject = (
                f"regex:{extractor.get('path')}#{extractor.get('pattern')}"
            )
        if claim.get("subject") != extractor_subject:
            problems.append(f"{label}: exact-population claim does not bind extractor")
        if closure.get("population_semantics") not in {
            "closed_case_set",
            "closed_enum",
            "closed_route_set",
            "closed_table",
        }:
            problems.append(f"{label}: exact population semantic class is invalid")
        counterfeit_names = {
            value.get("name")
            for value in closure.get("counterfeit", [])
            if isinstance(value, dict)
        }
        if counterfeit_names != REVIEW.EXACT_POPULATION_MUTATIONS:
            problems.append(f"{label}: exact population mutations are incomplete")
        extracted, detail = REVIEW.extract_population(root, closure.get("extractor"))
        if extracted is None:
            problems.append(f"{label}: {detail}")
        elif extracted != claim.get("expected"):
            problems.append(f"{label}: extracted population differs")
        hostile, detail = REVIEW.hostile_extractor_populations(
            root, closure.get("extractor")
        )
        if hostile is None:
            problems.append(f"{label}: {detail}")
        elif any(values == claim.get("expected") for values in hostile.values()):
            problems.append(f"{label}: hostile population mutation survived")
        observed = extracted
    else:
        problems.append(f"{label}: unknown mechanism")

    if problems or observed is None:
        return None, problems
    closure["discriminator"] = REVIEW.discriminator_digest(root, closure, observed)
    closure["receipt"] = REVIEW.closure_receipt(root, closure, observed=observed)
    return {
        "claim": claim,
        "closure": closure,
        "requirement_id": req_id,
        "slice_id": entry.get("_slice_id"),
    }, []


def check_drafts(root: Path) -> tuple[list[dict], list[str]]:
    rows, _ = requirement_rows(root)
    baseline = set(
        tomllib.loads((root / LEDGER).read_text(encoding="utf-8")).get(
            "baseline_shipped_ids", []
        )
    )
    live_shipped = {
        req_id for req_id, row in rows.items() if row.get("status") == "shipped"
    }
    expected_population = baseline | live_shipped
    entries, problems = load_draft_entries(root)
    authored: list[dict] = []
    seen: set[str] = set()
    for entry in entries:
        req_id = entry.get("requirement_id")
        if not isinstance(req_id, str) or req_id not in rows:
            problems.append(f"{req_id!r}: draft requirement does not resolve")
            continue
        if req_id not in expected_population or rows[req_id].get("status") != "shipped":
            problems.append(f"{req_id}: draft must name a live shipped requirement")
            continue
        if req_id in seen:
            problems.append(f"{req_id}: duplicate draft entry")
            continue
        seen.add(req_id)
        result, entry_problems = author_entry(root, entry, rows[req_id])
        problems.extend(entry_problems)
        if result is not None:
            authored.append(result)

    discriminators: dict[str, str] = {}
    identities: dict[str, str] = {}
    for result in authored:
        closure = result["closure"]
        req_id = result["requirement_id"]
        discriminator = closure.get("discriminator")
        if discriminator in discriminators:
            problems.append(
                f"{req_id}: discriminator duplicates {discriminators[discriminator]}"
            )
        discriminators[discriminator] = req_id
        mechanism = closure.get("mechanism")
        if mechanism == "formal_theorem":
            identity_payload = {"mechanism": mechanism, "verifier": closure.get("verifier")}
        elif mechanism == "executable_discriminator":
            identity_payload = {
                "mechanism": mechanism,
                "oracle": closure.get("oracle"),
                "tool_version": closure.get("tool_version"),
                "verifier": closure.get("verifier"),
            }
        else:
            identity_payload = {
                "extractor": closure.get("extractor"),
                "mechanism": mechanism,
            }
        identity = hashlib.sha256(REVIEW.canonical_json(identity_payload)).hexdigest()
        witness_id = closure.get("witness_id")
        if identity in identities and identities[identity] != witness_id:
            problems.append(f"{req_id}: equivalent witness identity is split")
        identities[identity] = witness_id
    return authored, problems


def toml_value(value: object) -> str:
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(
            f"{key} = {toml_value(item)}" for key, item in value.items()
        ) + " }"
    raise TypeError(f"cannot render TOML value: {value!r}")


def render_materialization(root: Path, authored: list[dict]) -> tuple[str, str]:
    by_id = {result["requirement_id"]: result for result in authored}
    baseline = set(
        tomllib.loads((root / LEDGER).read_text(encoding="utf-8")).get(
            "baseline_shipped_ids", []
        )
    )
    expected_population = baseline | set(shipped_ids(root))
    if set(by_id) != expected_population:
        missing = sorted(expected_population - set(by_id))
        extra = sorted(set(by_id) - expected_population)
        raise ValueError(
            "draft population differs from frozen+live activation set"
            f" (missing={missing}, extra={extra})"
        )

    registry_path = root / REGISTRY
    registry_text = registry_path.read_text(encoding="utf-8")
    if not registry_text.startswith("schema_version = 1\n"):
        raise ValueError("registry must be schema version 1 before activation")
    registry_text = registry_text.replace("schema_version = 1", "schema_version = 2", 1)
    parts = re.split(r"(?=^\[\[requirement\]\]$)", registry_text, flags=re.MULTILINE)
    rendered_parts = [parts[0]]
    for block in parts[1:]:
        match = re.search(r'^id = "([^"]+)"$', block, re.MULTILINE)
        if match is None or match.group(1) not in by_id:
            rendered_parts.append(block)
            continue
        claim = by_id[match.group(1)]["claim"]
        claim_line = "claim = " + toml_value(claim)
        block, count = re.subn(
            r'(^summary = .*?$)',
            lambda summary: summary.group(1) + "\n" + claim_line,
            block,
            count=1,
            flags=re.MULTILINE,
        )
        if count != 1:
            raise ValueError(f"unable to place claim for {match.group(1)}")
        rendered_parts.append(block)
    rendered_registry = "".join(rendered_parts)

    ledger_path = root / LEDGER
    ledger_text = ledger_path.read_text(encoding="utf-8")
    if not ledger_text.startswith("version = 1\n"):
        raise ValueError("ledger must be version 1 before activation")
    ledger_text = ledger_text.replace("version = 1", "version = 2", 1).rstrip() + "\n\n"
    members: dict[tuple[str, str], list[str]] = defaultdict(list)
    for result in authored:
        closure = result["closure"]
        members[(closure["witness_id"], closure["mechanism"])].append(
            result["requirement_id"]
        )
    lines: list[str] = []
    for (witness_id, mechanism), req_ids in sorted(members.items()):
        lines.extend(
            [
                "[[witness]]",
                f"id = {toml_value(witness_id)}",
                f"mechanism = {toml_value(mechanism)}",
                f"members = {toml_value(sorted(req_ids))}",
                "",
            ]
        )
    closure_order = [
        "requirement_id",
        "witness_id",
        "mechanism",
        "population_semantics",
        "discriminator",
        "claim_digest",
        "verifier",
        "verifier_version",
        "tool_version_argv",
        "tool_version",
        "oracle",
        "subject",
        "artifacts",
        "expected",
        "extractor",
        "counterfeit",
        "receipt",
    ]
    for result in sorted(authored, key=lambda value: value["requirement_id"]):
        closure = result["closure"]
        lines.append("[[closure]]")
        for field in closure_order:
            if field in closure:
                lines.append(f"{field} = {toml_value(closure[field])}")
        lines.append("")
    rendered_ledger = ledger_text + "\n".join(lines)
    authoritative_problems = REVIEW.check(
        root,
        backlog_document=tomllib.loads(rendered_ledger),
        registry_document=tomllib.loads(rendered_registry),
    )
    if authoritative_problems:
        raise ValueError(
            "authoritative gate rejects rendered activation: "
            + "; ".join(authoritative_problems)
        )
    return rendered_registry, rendered_ledger


def _replace_with_rollback(path: Path, text: str) -> bytes:
    previous = path.read_bytes()
    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=path.parent, delete=False
    ) as handle:
        handle.write(text)
        temporary = Path(handle.name)
    try:
        os.replace(temporary, path)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise
    return previous


def materialize(root: Path, authored: list[dict]) -> None:
    rendered_registry, rendered_ledger = render_materialization(root, authored)
    registry_path = root / REGISTRY
    ledger_path = root / LEDGER
    previous_ledger = _replace_with_rollback(ledger_path, rendered_ledger)
    try:
        _replace_with_rollback(registry_path, rendered_registry)
    except BaseException:
        ledger_path.write_bytes(previous_ledger)
        raise


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--freeze-baseline", action="store_true")
    mode.add_argument("--check-baseline", action="store_true")
    mode.add_argument("--check-drafts", action="store_true")
    mode.add_argument("--materialize", action="store_true")
    args = parser.parse_args(argv)
    root = Path(args.root).resolve()
    try:
        if args.freeze_baseline:
            freeze_baseline(root)
            print(f"froze {BASELINE_SIZE} shipped requirement IDs")
            return 0
        if args.check_drafts or args.materialize:
            authored, problems = check_drafts(root)
            if problems:
                for problem in problems:
                    print(f"claim-closure-author: {problem}", file=sys.stderr)
                return 1
            if args.materialize:
                materialize(root, authored)
                print(f"materialized {len(authored)} typed claims and closures")
            else:
                rows, _ = requirement_rows(root)
                ledger = tomllib.loads((root / LEDGER).read_text(encoding="utf-8"))
                target = set(ledger.get("baseline_shipped_ids", [])) | {
                    req_id
                    for req_id, row in rows.items()
                    if row.get("status") == "shipped"
                }
                missing = len(target) - len(authored)
                print(
                    f"claim closure drafts: {len(authored)}/{len(target)} valid; "
                    f"{missing} remaining"
                )
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
