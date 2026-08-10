#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Run the front-end-driven rewriter over a tree and report what it reaches.

    uv run --python 3.11 dev/thermite3-migrate/coverage.py <tree>

Measure a pinned upstream on a `git archive` export of the pin, never on
`.build/thermite`: counting through a clone this repository writes into is what
put two of Bulla's probe files into published corpus figures.

Two populations, needing different verdicts:

  `.th` files            every one should migrate; a decline is a blocker
  `.rs` string literals  a fragment migrates if it parses as Thermite, and the
                         corpus deliberately holds templates, expected Verus
                         output and fixtures that are invalid on purpose

A decline carrying no clause keyword costs nothing, so the number that matters
is declines that do carry one.
"""

import collections
import importlib.util
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent
BIN = HERE / "target/release/thermite3-migrate"

sys.path.insert(0, str(HERE))
from unescape import unescape  # noqa: E402

_spec = importlib.util.spec_from_file_location("tm", HERE / "thermite-migrate.py")
tm = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(tm)

V2KW = re.compile(r"(?<![A-Za-z0-9_@])(req|ens|fx|inv|dec)(?![A-Za-z0-9_])")
DECLARES = r"(fn|struct|enum|protocol|lemma|prop|spec)\s"


def migrate(text):
    p = subprocess.run([str(BIN), "to-v3"], input=text, capture_output=True, text=True)
    return p.returncode, p.stdout, p.stderr


def survives(out):
    """v2 clause keywords left after migration, ignoring comments and strings."""
    s = re.sub(r"//[^\n]*", "", out)
    s = re.sub(r'"(?:[^"\\]|\\.)*"', '""', s)
    return V2KW.findall(s)


def main(root):
    root = Path(root)
    if not BIN.exists():
        sys.exit(f"build it first: cargo build --release --manifest-path {HERE}/thmig/Cargo.toml")

    ok, declined, left = [], [], []
    for f in sorted(root.rglob("*.th")):
        rc, out, err = migrate(f.read_text(errors="replace"))
        rel = f.relative_to(root)
        if rc:
            declined.append((rel, err.strip().splitlines()[0] if err else ""))
        else:
            ok.append(rel)
            if survives(out):
                left.append(rel)

    print("### .th corpus")
    print(f"  migrated, no clause keyword surviving : {len(ok) - len(left)}")
    print(f"  migrated but a keyword survived       : {len(left)}")
    print(f"  declined                              : {len(declined)}")
    for r, e in declined:
        print(f"     DECLINED  {r}  ({e})")
    for r in left:
        print(f"     SURVIVED  {r}")

    lit_ok = lit_free = 0
    need = []
    sites = collections.Counter()
    files = set()
    for f in sorted(root.rglob("*.rs")):
        src = f.read_text(errors="replace")
        for s, e, sep in tm._rust_literals(src):
            body = src[s:e]
            if not re.search(r"(^|" + re.escape(sep) + r")\s*(pub\s+)?(spec\s+)?" + DECLARES, body):
                continue
            # The parser needs the literal's VALUE, not its source spelling. A
            # `\`-continuation is the one that bites: leaving it in feeds stray
            # backslashes to the lexer and turns 153 clean migrations into
            # declines that look like the corpus's fault.
            value, _ = unescape(body, raw=(sep == "\n"))
            rc, out, err = migrate(value)
            if rc == 0:
                lit_ok += 1
                files.add(f.relative_to(root))
                counted = subprocess.run(
                    [str(BIN), "count"], input=value, capture_output=True, text=True
                ).stdout.strip().splitlines()
                for line in counted:
                    k, v = line.split()
                    sites[k] += int(v)
            elif V2KW.search(re.sub(r"//[^\n]*", "", value)):
                need.append((f.relative_to(root), value.strip()[:100]))
            else:
                lit_free += 1

    n = sum(sites[k] for k in ("ens", "req", "fx", "dec", "inv"))
    print("\n### `.th` fragments in Rust string literals")
    print(f"  migrated            : {lit_ok}, carrying {n} clause sites across {len(files)} files")
    print(f"  declined, no clause keyword (free)  : {lit_free}")
    print(f"  declined, clause-bearing (review)   : {len(need)}")
    for r, t in need:
        print(f"     {r}\n        {t!r}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else ".")
