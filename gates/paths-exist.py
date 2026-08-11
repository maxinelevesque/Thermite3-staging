#!/usr/bin/env python3
"""Check that every repo-relative path referenced by the control files exists.

Scans the executable surface — CI workflows, the ``Makefile``, the
``justfile``, the shell gates and dev scripts, the Python gates, and Rust
source — for references into the tracked tree, and fails on any that resolve
to nothing. Comments are stripped first: the gate checks what runs, and a
historical path named in prose is the frozen-record convention, not a defect.

Measured evidence (RFC-18 §4): the layout move shipped three breaks that CI
caught and the local suite structurally could not — ``gates/g3.sh`` still
invoked ``scripts/lean-axiom-probe.sh`` (exit 127), a Rust test failed on
``scripts/audit.sh`` NotFound, and a stale ``include_str!`` path in
``forge/src/epr_reconstruct.rs`` failed four jobs as a block. Each is a
repo-relative reference whose target had moved.

What is checked, per file kind:

* ``.github/workflows/*.yml``, ``Makefile``, ``justfile``, ``gates/`` and
  ``dev/`` shell scripts, top-level ``gates/*.py``: any token starting with a
  known tree prefix (``gates/``, ``dev/``, ``conformance/``, ...) after
  comment stripping. ``scripts/`` and ``tooling/`` are deliberately in the
  prefix list: they no longer exist, so a live reference to either is
  exactly the stale-layout class this gate exists for.
* Rust source: ``include_str!``/``include_bytes!`` literals resolved against
  the containing file (the compile-time semantics), and any string literal
  that IS a tree path (starts with a known prefix, after ``./``/``../``
  stripping). Literals that merely mention a path inside prose are not
  checked — "install lean/elan" is a sentence, not a reference.

A missing path that git ignores is skipped: a reference to a generated
location (``lean/.lake/...``) is legitimate and its absence from a fresh
tree is normal. ``gates/tests/`` fixtures are excluded — they write their
paths into temp dirs.

* 0: every referenced path resolves
* 1: at least one referenced path does not exist
* 3: the check could not run reliably

Usage: ``uv run python gates/paths-exist.py [--root <repo-toplevel>]``
"""

import re
import subprocess
import sys
from pathlib import Path


# Project settings

# Top-level tree prefixes a reference can start with. scripts/ and tooling/
# are the pre-RFC-18 layout: they intentionally stay listed so a surviving
# live reference to either fails rather than being unmatchable.
KNOWN_PREFIXES = (
    "gates/",
    "dev/",
    "conformance/",
    "tests/",
    "lean/",
    ".design/",
    ".github/",
    "opt-in/",
    "docs/",
    "scripts/",
    "tooling/",
    "forge/",
    "thermite-syntax/",
    "thermite-spec/",
    "thermite-lower/",
    "thermite-skill/",
    "thermite-tv/",
    "thermite-verified/",
    "thermite-test-utils/",
)

# Exit codes (the doc-drift REQ-9 convention).
EXIT_OK = 0
EXIT_FAIL = 1
EXIT_INCONCLUSIVE = 3

MISSING = "MISSING-PATH"

_PREFIX_ALT = "|".join(re.escape(p) for p in KNOWN_PREFIXES)
TOKEN_RE = re.compile(r"(?:%s)[A-Za-z0-9._/\-*]*" % _PREFIX_ALT)
INCLUDE_RE = re.compile(r'include_(?:str|bytes)!\s*\(\s*"([^"]+)"')
STRING_RE = re.compile(r'"((?:[^"\\\n]|\\.)*)"')


class EnvironmentError3(Exception):
    """The check could not determine the answer; maps to exit 3."""


# --- enumeration ---------------------------------------------------------


def _tracked_files(root):
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


def _scan_set(tracked):
    """(relpath, kind) pairs for every file this gate reads."""
    out = []
    for f in tracked:
        if f.startswith(".github/workflows/") and f.endswith((".yml", ".yaml")):
            out.append((f, "hash"))
        elif f in ("Makefile", "justfile"):
            out.append((f, "hash"))
        elif f.startswith(("gates/", "dev/")) and f.endswith((".sh", ".env")):
            out.append((f, "hash"))
        elif (
            f.startswith("gates/")
            and not f.startswith("gates/tests/")
            and f.endswith(".py")
        ):
            out.append((f, "hash"))
        elif f.endswith(".rs"):
            out.append((f, "rust"))
    return out


# --- comment stripping ---------------------------------------------------


def _strip_hash_comments(text):
    """Drop '#' comments outside quotes, line by line."""
    lines = []
    for line in text.splitlines():
        out = []
        quote = None
        for ch in line:
            if quote:
                out.append(ch)
                if ch == quote:
                    quote = None
                continue
            if ch in "\"'":
                quote = ch
                out.append(ch)
                continue
            if ch == "#":
                break
            out.append(ch)
        lines.append("".join(out))
    return "\n".join(lines)


def _strip_rust_comments(text):
    """Drop // and /* */ comments; keep string contents intact."""
    out = []
    i = 0
    n = len(text)
    in_string = False
    in_block = 0
    in_line = False
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if in_line:
            if c == "\n":
                in_line = False
                out.append(c)
            i += 1
            continue
        if in_block:
            if c == "/" and i > 0 and text[i - 1] == "*":
                in_block -= 1
            if c == "\n":
                out.append(c)
            i += 1
            continue
        if in_string:
            out.append(c)
            if c == "\\" and nxt:
                out.append(nxt)
                i += 2
                continue
            if c == '"':
                in_string = False
            i += 1
            continue
        if c == '"':
            in_string = True
            out.append(c)
            i += 1
            continue
        if c == "/" and nxt == "/":
            in_line = True
            i += 2
            continue
        if c == "/" and nxt == "*":
            in_block += 1
            i += 2
            continue
        out.append(c)
        i += 1
    return "".join(out)


# --- reference extraction ------------------------------------------------


def _clean_token(tok):
    return tok.rstrip(".,;:'\")]}").rstrip("/")


def _accept_context(text, start):
    """Reject substring hits and absolute paths like /dev/null.

    A token is a reference when it starts a word. A preceding '/' is allowed
    only when the '/' itself follows a variable or path segment (e.g.
    "$tmp_dir/gates/x.py"); a '/' after whitespace or a redirection is an
    absolute path, which is never a tree reference.
    """
    if start == 0:
        return True
    prev = text[start - 1]
    if prev.isalnum() or prev in "._-":
        return False
    if prev == "/":
        if start < 2:
            return False
        before = text[start - 2]
        return before.isalnum() or before in "})_"
    return True


def _hash_file_refs(text):
    """(token, None) refs from a '#'-commented file, root-relative."""
    cleaned = _strip_hash_comments(text)
    refs = []
    for m in TOKEN_RE.finditer(cleaned):
        if not _accept_context(cleaned, m.start()):
            continue
        tok = _clean_token(m.group(0))
        if not tok or "$" in tok or "{" in tok:
            continue
        refs.append(tok)
    return refs


def _rust_file_refs(relpath, text):
    """(token, base) refs from a Rust file.

    include_str!/include_bytes! resolve against the containing file's
    directory. Other string literals are checked only when the whole literal
    is a tree path; those resolve against the repo root, with the containing
    crate root as a fallback (cargo tests run from the manifest dir).
    """
    cleaned = _strip_rust_comments(text)
    file_dir = str(Path(relpath).parent)
    includes = []
    include_literals = set()
    for m in INCLUDE_RE.finditer(cleaned):
        includes.append((m.group(1), file_dir))
        include_literals.add(m.group(1))

    literals = []
    for m in STRING_RE.finditer(cleaned):
        lit = m.group(1)
        if lit in include_literals:
            continue
        s = lit
        while s.startswith("./"):
            s = s[2:]
        while s.startswith("../"):
            s = s[3:]
        if not s or not s.startswith(KNOWN_PREFIXES):
            continue
        if s != _clean_token(s) + "/" and s != _clean_token(s):
            s = _clean_token(s)
        if not s or "$" in s or "{" in s or " " in s:
            continue
        literals.append(s)
    return includes, literals


# --- resolution ----------------------------------------------------------


def _tracked_glob_match(tok, tracked):
    pat = re.escape(tok).replace(r"\*\*", ".*").replace(r"\*", "[^/]*")
    rx = re.compile("^" + pat + "(/|$)")
    return any(rx.match(t) for t in tracked)


def _ignored(root, paths):
    """The subset of `paths` that .gitignore covers (absence is normal)."""
    if not paths:
        return set()
    try:
        proc = subprocess.run(
            ["git", "-C", str(root), "check-ignore", "--stdin", "-z"],
            input="\0".join(paths) + "\0",
            capture_output=True,
            text=True,
        )
    except (FileNotFoundError, PermissionError, OSError) as exc:
        raise EnvironmentError3(f"git could not be invoked: {exc}") from exc
    # 0: some ignored; 1: none ignored; anything else is an error.
    if proc.returncode not in (0, 1):
        raise EnvironmentError3(
            f"git check-ignore failed: {proc.stderr.strip()}"
        )
    return {p for p in proc.stdout.split("\0") if p}


def evaluate(root):
    tracked = _tracked_files(root)
    scan = _scan_set(tracked)

    # (token, source-file, base-dir or None) — base None means root-relative
    # with the containing crate root as a fallback.
    candidates = []
    for relpath, kind in scan:
        try:
            text = (root / relpath).read_text(encoding="utf-8",
                                              errors="replace")
        except OSError as exc:
            raise EnvironmentError3(f"unreadable: {relpath}: {exc}")
        if kind == "hash":
            for tok in _hash_file_refs(text):
                candidates.append((tok, relpath, None))
        else:
            includes, literals = _rust_file_refs(relpath, text)
            for tok, base in includes:
                candidates.append((tok, relpath, base))
            for tok in literals:
                candidates.append((tok, relpath, None))

    unresolved = {}
    for tok, source, base in candidates:
        if base is not None:
            target = (root / base / tok).resolve()
            if target.exists():
                continue
            unresolved.setdefault(f"{base}/{tok}", set()).add(source)
            continue
        crate = source.split("/", 1)[0]
        if "*" in tok:
            if _tracked_glob_match(tok, tracked):
                continue
            if _tracked_glob_match(f"{crate}/{tok}", tracked):
                continue
        else:
            if (root / tok).exists():
                continue
            if (root / crate / tok).exists():
                continue
        unresolved.setdefault(tok, set()).add(source)

    ignored = _ignored(root, sorted(unresolved))
    findings = []
    for tok in sorted(unresolved):
        if tok in ignored:
            continue
        sources = sorted(unresolved[tok])
        shown = ", ".join(sources[:3]) + (" ..." if len(sources) > 3 else "")
        findings.append(f"{MISSING} {tok}  (referenced by: {shown})")

    if findings:
        findings.append(f"paths-exist: {len(findings)} missing path(s)")
        return EXIT_FAIL, findings
    return EXIT_OK, [
        f"paths-exist: ok — {len(candidates)} references across "
        f"{len(scan)} files all resolve"
    ]


# --- entry ---------------------------------------------------------------


def _parse_args(argv):
    root = None
    args = list(argv)
    while args:
        a = args.pop(0)
        if a == "--root":
            if not args:
                print("paths-exist: --root requires a value", file=sys.stderr)
                sys.exit(EXIT_INCONCLUSIVE)
            root = Path(args.pop(0))
        elif a in ("-h", "--help"):
            print(__doc__)
            sys.exit(EXIT_OK)
        else:
            print(f"paths-exist: unknown argument {a!r}", file=sys.stderr)
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
            print(f"paths-exist: git could not be invoked: {exc}",
                  file=sys.stderr)
            sys.exit(EXIT_INCONCLUSIVE)
        if proc.returncode != 0 or not proc.stdout.strip():
            print("paths-exist: not inside a git repository (pass --root)",
                  file=sys.stderr)
            sys.exit(EXIT_INCONCLUSIVE)
        root = Path(proc.stdout.strip())

    try:
        exit_code, lines = evaluate(root)
    except EnvironmentError3 as exc:
        print(f"paths-exist: INCONCLUSIVE — {exc}", file=sys.stderr)
        sys.exit(EXIT_INCONCLUSIVE)
    for line in lines:
        print(line)
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
