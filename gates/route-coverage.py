#!/usr/bin/env python3
"""Check that every route resolves and every gated source file is routed.

Routes come from ``gates/routes.toml``. This gate closes the two coverage
holes RFC-18 §4 names as residual trust:

* **Route → tree**: a route whose ``crate_pattern`` matches zero tracked
  files is DEAD — its target moved or was deleted, and the pins it feeds
  digest an empty set. A route added ahead of its file (the manifest-time
  convention: the route exists so the spec-discipline hook routes the
  builder's first edit) declares ``unbuilt = true`` and is exempt until the
  file lands, at which point the flag itself goes stale and is flagged.
  A route's ``design`` doc and each ``reference`` entry must also resolve.
* **Tree → route**: every tracked file the spec-discipline hook would gate
  (a ``.rs`` file under a gated crate's ``src/``) must match at least one
  route. Measured evidence for this direction: ``gates/g4.sh`` silently lost
  its route when it left the ``scripts/g4-*`` glob, and the loss could only
  surface as an R-XLATE-2 block on the next edit (RFC-18 §4). Unrouted files
  that predate this gate live in ``gates/route-coverage-burndown.txt``; the
  list only shrinks, and a stale entry is itself a finding.

Results are sorted by class and path:

* 0: every route resolves and every gated file is routed
* 1: a dead route, stale flag, missing design/reference, or unrouted file
* 3: the check could not run reliably

Usage: ``uv run python gates/route-coverage.py [--root <repo-toplevel>]``
"""

import re
import subprocess
import sys
from pathlib import Path

try:
    import tomllib  # Python 3.11+
except ImportError:  # pragma: no cover - exercised via the env-failure path
    tomllib = None


# Project settings

# Repo-relative path to the route table (the enumeration source).
ROUTES_RELPATH = "gates/routes.toml"

# The gated-file predicate. Keep in sync with gates/spec-discipline.py —
# this gate is the static sweep of the same rule that hook enforces per-edit.
TARGET_CRATE_PREFIXES = ("thermite-",)
TARGET_CRATE_EXACT = ("forge",)
EXCLUDED_CRATES = ("thermite-test-utils",)
TARGET_EXTENSION = ".rs"

# The burn-down list: gated files that predate this gate and carry no route,
# one repo-relative path per line ('#' comments allowed). Absent means empty.
# Entries only ever leave the list — an entry that becomes routed or leaves
# the tree is flagged, and a NEW gated file gets a route, not a line there.
BURNDOWN_RELPATH = "gates/route-coverage-burndown.txt"

# Defect classes: the literal tokens the report emits and the oracle asserts.
DEAD_ROUTE = "DEAD-ROUTE"
STALE_UNBUILT = "STALE-UNBUILT"
MISSING_DESIGN = "MISSING-DESIGN"
MISSING_REFERENCE = "MISSING-REFERENCE"
UNROUTED = "UNROUTED"
STALE_KNOWN_UNROUTED = "STALE-KNOWN-UNROUTED"

# Exit codes (the doc-drift REQ-9 convention).
EXIT_OK = 0
EXIT_FAIL = 1
EXIT_INCONCLUSIVE = 3


class EnvironmentError3(Exception):
    """The check could not determine the answer; maps to exit 3."""


# --- enumeration --------------------------------------------------------


def _tracked_files(root):
    """The tracked tree, from git; raise exit-3 when git cannot answer."""
    try:
        proc = subprocess.run(
            ["git", "-C", str(root), "ls-files", "-z"],
            capture_output=True,
            text=True,
        )
    except (FileNotFoundError, PermissionError, OSError) as exc:
        raise EnvironmentError3(f"git could not be invoked: {exc}") from exc
    if proc.returncode != 0:
        raise EnvironmentError3(
            f"git ls-files failed in {root}: {proc.stderr.strip()}"
        )
    return [f for f in proc.stdout.split("\0") if f]


def _load_routes(root):
    if tomllib is None:
        raise EnvironmentError3(
            "no TOML reader (tomllib is 3.11+, this is "
            f"{sys.version_info.major}.{sys.version_info.minor}); "
            "run under the pinned interpreter: uv run python "
            "gates/route-coverage.py"
        )
    p = root / ROUTES_RELPATH
    if not p.is_file():
        raise EnvironmentError3(f"route table not found: {ROUTES_RELPATH}")
    try:
        with open(p, "rb") as f:
            data = tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise EnvironmentError3(
            f"route table unreadable ({ROUTES_RELPATH}): {exc}"
        )
    routes = data.get("route", [])
    if not isinstance(routes, list) or not routes:
        raise EnvironmentError3(
            f"route table {ROUTES_RELPATH} yielded zero routes — nothing to "
            "check; an empty enumeration source is INCONCLUSIVE, not a pass"
        )
    return routes


# --- pattern matching (the spec-discipline semantics) --------------------


def glob_to_regex(pattern):
    """Convert a simple glob (supporting * and **) to a regex."""
    out = []
    i = 0
    while i < len(pattern):
        c = pattern[i]
        if c == "*":
            if i + 1 < len(pattern) and pattern[i + 1] == "*":
                out.append(".*")
                i += 2
                if i < len(pattern) and pattern[i] == "/":
                    i += 1
            else:
                out.append("[^/]*")
                i += 1
        elif c == "?":
            out.append("[^/]")
            i += 1
        elif c in ".^$+(){}|\\[]":
            out.append("\\" + c)
            i += 1
        else:
            out.append(c)
            i += 1
    return "^" + "".join(out) + "$"


def match_pattern(file_path, pattern):
    if pattern == file_path:
        return True
    if "*" in pattern or "?" in pattern:
        return re.match(glob_to_regex(pattern), file_path) is not None
    return False


def is_gated_path(rel_path):
    """True iff the spec-discipline hook would gate this file."""
    if not rel_path.endswith(TARGET_EXTENSION):
        return False
    parts = rel_path.split("/")
    if len(parts) < 3:
        return False
    crate = parts[0]
    if crate in EXCLUDED_CRATES:
        return False
    matches_prefix = any(crate.startswith(p) for p in TARGET_CRATE_PREFIXES)
    matches_exact = crate in TARGET_CRATE_EXACT
    if not (matches_prefix or matches_exact):
        return False
    if "src" not in parts[1:]:
        return False
    return True


# --- evaluation ----------------------------------------------------------


def _load_burndown(root):
    p = root / BURNDOWN_RELPATH
    if not p.is_file():
        return []
    entries = []
    try:
        for line in p.read_text(encoding="utf-8").splitlines():
            line = line.split("#", 1)[0].strip()
            if line:
                entries.append(line)
    except OSError as exc:
        raise EnvironmentError3(
            f"burn-down list unreadable ({BURNDOWN_RELPATH}): {exc}"
        )
    return entries


def evaluate(root):
    tracked = _tracked_files(root)
    tracked_set = set(tracked)
    routes = _load_routes(root)
    burndown = _load_burndown(root)

    findings = []

    for i, route in enumerate(routes):
        pattern = route.get("crate_pattern")
        design = route.get("design")
        if not isinstance(pattern, str) or not pattern:
            raise EnvironmentError3(
                f"route table {ROUTES_RELPATH}: route entry #{i} has no "
                "usable `crate_pattern`"
            )
        if not isinstance(design, str) or not design:
            raise EnvironmentError3(
                f"route table {ROUTES_RELPATH}: route entry #{i} has no "
                "usable `design`"
            )
        unbuilt = bool(route.get("unbuilt", False))
        matched = any(match_pattern(f, pattern) for f in tracked)

        if not matched and not unbuilt:
            findings.append(
                (DEAD_ROUTE, pattern,
                 f"{DEAD_ROUTE} {pattern} matches no tracked file "
                 f"(design: {design}); if the file moved, follow it; if the "
                 f"route is ahead of the build, declare `unbuilt = true`")
            )
        if matched and unbuilt:
            findings.append(
                (STALE_UNBUILT, pattern,
                 f"{STALE_UNBUILT} {pattern} declares `unbuilt = true` but "
                 f"matches a tracked file — the file landed; drop the flag")
            )
        if design not in tracked_set:
            findings.append(
                (MISSING_DESIGN, design,
                 f"{MISSING_DESIGN} {design} is not tracked "
                 f"(route: {pattern})")
            )
        for ref in route.get("reference", []):
            ok = ref in tracked_set or any(
                t.startswith(ref.rstrip("/") + "/") for t in tracked
            )
            if not ok:
                findings.append(
                    (MISSING_REFERENCE, ref,
                     f"{MISSING_REFERENCE} {ref} names no tracked file or "
                     f"directory (route: {pattern})")
                )

    patterns = [r.get("crate_pattern") for r in routes]
    burndown_set = set(burndown)
    for f in tracked:
        if not is_gated_path(f):
            continue
        routed = any(match_pattern(f, p) for p in patterns)
        if not routed and f not in burndown_set:
            findings.append(
                (UNROUTED, f,
                 f"{UNROUTED} {f} is gated by spec-discipline but matches no "
                 f"route; add a route to {ROUTES_RELPATH} (the edit hook "
                 f"will block this file until one exists)")
            )

    for f in sorted(burndown_set):
        if f not in tracked_set:
            findings.append(
                (STALE_KNOWN_UNROUTED, f,
                 f"{STALE_KNOWN_UNROUTED} {f} left the tree; remove it from "
                 f"{BURNDOWN_RELPATH}")
            )
        elif any(match_pattern(f, p) for p in patterns):
            findings.append(
                (STALE_KNOWN_UNROUTED, f,
                 f"{STALE_KNOWN_UNROUTED} {f} is now routed; remove it from "
                 f"{BURNDOWN_RELPATH}")
            )

    findings.sort(key=lambda t: (t[0], t[1]))
    lines = [msg for _, _, msg in findings]
    if findings:
        classes = sorted({cls for cls, _, _ in findings})
        lines.append(
            f"route-coverage: {len(findings)} finding(s) "
            f"({', '.join(classes)})"
        )
        return EXIT_FAIL, lines
    lines.append(
        f"route-coverage: ok — {len(routes)} routes resolve, "
        f"{sum(1 for f in tracked if is_gated_path(f))} gated files covered "
        f"({len(burndown)} on the burn-down list)"
    )
    return EXIT_OK, lines


# --- entry ---------------------------------------------------------------


def _parse_args(argv):
    root = None
    args = list(argv)
    while args:
        a = args.pop(0)
        if a == "--root":
            if not args:
                print("route-coverage: --root requires a value", file=sys.stderr)
                sys.exit(EXIT_INCONCLUSIVE)
            root = Path(args.pop(0))
        elif a in ("-h", "--help"):
            print(__doc__)
            sys.exit(EXIT_OK)
        else:
            print(f"route-coverage: unknown argument {a!r}", file=sys.stderr)
            sys.exit(EXIT_INCONCLUSIVE)
    return root


def main(argv=None):
    root = _parse_args(sys.argv[1:] if argv is None else argv)
    if root is None:
        try:
            proc = subprocess.run(
                ["git", "rev-parse", "--show-toplevel"],
                capture_output=True,
                text=True,
            )
        except (FileNotFoundError, PermissionError, OSError) as exc:
            print(f"route-coverage: git could not be invoked: {exc}",
                  file=sys.stderr)
            sys.exit(EXIT_INCONCLUSIVE)
        if proc.returncode != 0 or not proc.stdout.strip():
            print("route-coverage: not inside a git repository "
                  "(pass --root)", file=sys.stderr)
            sys.exit(EXIT_INCONCLUSIVE)
        root = Path(proc.stdout.strip())

    try:
        exit_code, lines = evaluate(root)
    except EnvironmentError3 as exc:
        print(f"route-coverage: INCONCLUSIVE — {exc}", file=sys.stderr)
        sys.exit(EXIT_INCONCLUSIVE)
    for line in lines:
        print(line)
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
