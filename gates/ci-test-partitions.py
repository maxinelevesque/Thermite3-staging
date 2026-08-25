#!/usr/bin/env python3
"""Generate, check, simulate, and run duration-balanced nextest partitions."""

from __future__ import annotations

import argparse
import math
import statistics
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

try:
    import tomllib
except ImportError:  # pragma: no cover
    tomllib = None


MANIFEST = Path("gates/ci-test-partitions.toml")
EXPLICIT_BUCKETS = 12
CATCH_ALL = 13
BASELINE_RUN = 31837152080
EFFECTIVE_PARALLELISM = 2.17
NOISE_ALLOWANCE = 1.15

COMMITMENT_CASES = [
    "commitment_after_plan_source_mutation_is_atomic",
    "commitment_after_plan_body_mutation_is_atomic",
    "commitment_after_plan_helper_mutation_is_atomic",
    "commitment_after_plan_wrapper_mutation_is_atomic",
    "commitment_before_verus_is_atomic",
    "commitment_after_verus_is_atomic",
    "commitment_after_codegen_is_atomic",
    "commitment_after_artifact_hash_is_atomic",
    "commitment_after_plan_hash_is_atomic",
    "commitment_after_evidence_hash_is_atomic",
    "commitment_after_toolchain_hash_is_atomic",
    "commitment_after_receipt_staging_is_atomic",
]
TV_CASES = [
    f"tv_{phase}_{verdict}_blocks_publication"
    for phase in ("contract", "exec", "body", "loop")
    for verdict in ("divergent", "unsupported", "skipped", "unverifiable")
]


@dataclass(frozen=True, order=True)
class TestId:
    binary: str
    name: str

    @classmethod
    def parse(cls, value: str) -> "TestId":
        binary, sep, name = value.partition(" ")
        if not sep or not binary or not name:
            raise ValueError(f"invalid nextest identity: {value!r}")
        return cls(binary, name)

    def render(self) -> str:
        return f"{self.binary} {self.name}"


@dataclass(frozen=True)
class Assignment:
    test: TestId
    seconds: float
    bucket: int


def repo_root(start: Path) -> Path:
    proc = subprocess.run(
        ["git", "-C", str(start), "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
    )
    if proc.returncode:
        raise RuntimeError(proc.stderr.strip() or "git root unavailable")
    return Path(proc.stdout.strip())


def inventory(root: Path) -> list[TestId]:
    proc = subprocess.run(
        [
            "cargo",
            "nextest",
            "list",
            "--workspace",
            "--color",
            "never",
            "-T",
            "oneline",
            "--cargo-quiet",
            "--cargo-quiet",
        ],
        cwd=root,
        capture_output=True,
        text=True,
    )
    if proc.returncode:
        raise RuntimeError(f"nextest inventory failed: {proc.stderr.strip()}")
    tests = sorted(TestId.parse(line) for line in proc.stdout.splitlines() if line.strip())
    if not tests:
        raise RuntimeError("nextest inventory is empty")
    if len(tests) != len(set(tests)):
        raise RuntimeError("nextest inventory contains duplicate identities")
    return tests


def read_timings(path: Path) -> dict[TestId, float]:
    timings: dict[TestId, float] = {}
    legacy: dict[str, float] = {}
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        seconds_text, sep, identity_text = line.partition("\t")
        if not sep:
            raise ValueError(f"{path}:{number}: expected seconds<TAB>identity")
        seconds = float(seconds_text)
        if not math.isfinite(seconds) or seconds < 0:
            raise ValueError(f"{path}:{number}: invalid duration {seconds_text!r}")
        if identity_text in (
            "forge::verified_build every_injected_commitment_failure_is_atomic",
            "forge::verified_build every_tv_phase_and_nonpass_class_blocks_publication",
        ):
            legacy[identity_text] = seconds
        else:
            test = TestId.parse(identity_text)
            if test in timings:
                raise ValueError(f"{path}:{number}: duplicate timing for {test.render()}")
            timings[test] = seconds

    commitment_total = legacy.get(
        "forge::verified_build every_injected_commitment_failure_is_atomic"
    )
    tv_total = legacy.get(
        "forge::verified_build every_tv_phase_and_nonpass_class_blocks_publication"
    )
    if commitment_total is not None:
        for name in COMMITMENT_CASES:
            timings[TestId("forge::verified_build", name)] = commitment_total / len(
                COMMITMENT_CASES
            )
    if tv_total is not None:
        for name in TV_CASES:
            timings[TestId("forge::verified_build", name)] = tv_total / len(TV_CASES)
    return timings


def allocate(tests: list[TestId], timings: dict[TestId, float]) -> list[Assignment]:
    missing = sorted(set(tests) - set(timings))
    stale = sorted(set(timings) - set(tests))
    if missing or stale:
        details = []
        if missing:
            details.append(f"missing timings: {', '.join(t.render() for t in missing[:10])}")
        if stale:
            details.append(f"stale timings: {', '.join(t.render() for t in stale[:10])}")
        raise ValueError("; ".join(details))

    loads = [0.0] * EXPLICIT_BUCKETS
    result = []
    for test in sorted(tests, key=lambda item: (-timings[item], item)):
        bucket_index = min(range(EXPLICIT_BUCKETS), key=lambda i: (loads[i], i))
        seconds = timings[test]
        result.append(Assignment(test, seconds, bucket_index + 1))
        loads[bucket_index] += seconds
    return sorted(result, key=lambda item: (item.bucket, item.test))


def write_manifest(path: Path, assignments: list[Assignment], source: Path) -> None:
    max_test = max(item.seconds for item in assignments)
    lines = [
        "# Generated by gates/ci-test-partitions.py; review timing and bucket diffs.",
        "schema = 1",
        f"bucket_count = {CATCH_ALL}",
        f"explicit_bucket_count = {EXPLICIT_BUCKETS}",
        f"catch_all_bucket = {CATCH_ALL}",
        f"baseline_run = {BASELINE_RUN}",
        f'effective_parallelism = {EFFECTIVE_PARALLELISM:.2f}',
        f'longest_indivisible_seconds = {max_test:.3f}',
        f'timing_source = "{source.as_posix()}"',
        "",
    ]
    for item in assignments:
        escaped = item.test.render().replace("\\", "\\\\").replace('"', '\\"')
        lines.extend(
            [
                "[[assignment]]",
                f"bucket = {item.bucket}",
                f"seconds = {item.seconds:.3f}",
                f'test = "{escaped}"',
                "",
            ]
        )
    path.write_text("\n".join(lines))


def load_manifest(path: Path) -> tuple[dict, list[Assignment]]:
    if tomllib is None:
        raise RuntimeError("tomllib unavailable; run with the pinned Python")
    with path.open("rb") as handle:
        data = tomllib.load(handle)
    assignments = [
        Assignment(TestId.parse(row["test"]), float(row["seconds"]), int(row["bucket"]))
        for row in data.get("assignment", [])
    ]
    return data, assignments


def validate_manifest(data: dict, assignments: list[Assignment], tests: list[TestId]) -> list[str]:
    errors = []
    if data.get("schema") != 1:
        errors.append("manifest schema must be 1")
    if data.get("bucket_count") != CATCH_ALL:
        errors.append(f"bucket_count must be {CATCH_ALL}")
    if data.get("explicit_bucket_count") != EXPLICIT_BUCKETS:
        errors.append(f"explicit_bucket_count must be {EXPLICIT_BUCKETS}")
    if data.get("catch_all_bucket") != CATCH_ALL:
        errors.append(f"catch_all_bucket must be {CATCH_ALL}")

    identities = [item.test for item in assignments]
    duplicates = sorted({test for test in identities if identities.count(test) > 1})
    stale = sorted(set(identities) - set(tests))
    missing = sorted(set(tests) - set(identities))
    bad_buckets = sorted(
        {
            item.bucket
            for item in assignments
            if not 1 <= item.bucket <= EXPLICIT_BUCKETS
        }
    )
    if duplicates:
        errors.append(f"duplicate assignments: {', '.join(t.render() for t in duplicates[:10])}")
    if stale:
        errors.append(f"stale assignments: {', '.join(t.render() for t in stale[:10])}")
    if missing:
        errors.append(
            "unreviewed catch-all tests require rebalance: "
            + ", ".join(t.render() for t in missing[:10])
        )
    if bad_buckets:
        errors.append(f"invalid explicit buckets: {bad_buckets}")
    return errors


def simulation(assignments: list[Assignment]) -> list[tuple[int, int, float, float, float]]:
    rows = []
    for bucket in range(1, EXPLICIT_BUCKETS + 1):
        members = [item for item in assignments if item.bucket == bucket]
        total = sum(item.seconds for item in members)
        longest = max((item.seconds for item in members), default=0.0)
        predicted = max(longest, total / EFFECTIVE_PARALLELISM)
        rows.append((bucket, len(members), total, longest, predicted))
    return rows


def escape_matcher(value: str) -> str:
    return value.replace("\\", "\\\\").replace(")", "\\)").replace(",", "\\,")


def filter_expression(tests: list[TestId]) -> str:
    if not tests:
        return "not all()"
    return " | ".join(
        f"(binary_id(={escape_matcher(test.binary)}) & test(={escape_matcher(test.name)}))"
        for test in tests
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path)
    sub = parser.add_subparsers(dest="command", required=True)
    generate = sub.add_parser("generate")
    generate.add_argument("--timings", type=Path, required=True)
    generate.add_argument("--out", type=Path, default=MANIFEST)
    sub.add_parser("check")
    sub.add_parser("simulate")
    select = sub.add_parser("select")
    select.add_argument("bucket", type=int)
    run = sub.add_parser("run")
    run.add_argument("bucket", type=int)
    run.add_argument("nextest_args", nargs=argparse.REMAINDER)
    args = parser.parse_args()

    root = repo_root(args.root or Path.cwd())
    manifest_path = root / MANIFEST
    tests = inventory(root)

    if args.command == "generate":
        timings_path = args.timings if args.timings.is_absolute() else root / args.timings
        assignments = allocate(tests, read_timings(timings_path))
        out = args.out if args.out.is_absolute() else root / args.out
        write_manifest(out, assignments, args.timings)
        print(
            f"wrote {out.relative_to(root)}: {len(assignments)} tests in "
            f"{EXPLICIT_BUCKETS} explicit buckets"
        )
        return 0

    data, assignments = load_manifest(manifest_path)
    errors = validate_manifest(data, assignments, tests)
    if args.command == "check":
        if errors:
            print("\n".join(f"partition-check: {error}" for error in errors), file=sys.stderr)
            return 1
        print(f"partition-check: {len(tests)} tests exactly assigned; catch-all bucket present")
        return 0
    if errors:
        print("\n".join(f"partition-check: {error}" for error in errors), file=sys.stderr)
        return 1

    rows = simulation(assignments)
    if args.command == "simulate":
        for bucket, count, total, longest, predicted in rows:
            print(
                f"bucket {bucket}: tests={count} serial={total:.3f}s "
                f"longest={longest:.3f}s predicted={predicted:.3f}s"
            )
        bound = max(item.seconds for item in assignments) * NOISE_ALLOWANCE
        worst = max(row[4] for row in rows)
        print(f"predicted maximum {worst:.3f}s; allowed bound {bound:.3f}s")
        return 0 if worst <= bound else 1

    explicit = {item.test for item in assignments}
    selected = (
        sorted(set(tests) - explicit)
        if args.bucket == CATCH_ALL
        else sorted(item.test for item in assignments if item.bucket == args.bucket)
    )
    if not 1 <= args.bucket <= CATCH_ALL:
        print(f"bucket must be 1..{CATCH_ALL}", file=sys.stderr)
        return 2
    expression = filter_expression(selected)
    if args.command == "select":
        print(expression)
        return 0
    if not selected:
        print(f"partition {args.bucket}: no tests selected")
        return 0
    command = ["cargo", "nextest", "run", "--workspace", "-E", expression]
    command.extend(arg for arg in args.nextest_args if arg != "--")
    print(f"partition {args.bucket}: running {len(selected)} tests", flush=True)
    return subprocess.run(command, cwd=root).returncode


if __name__ == "__main__":
    raise SystemExit(main())
