# thermite3-migrate

Rewrites Thermite 2 clause syntax to Thermite 3 for
[RFC-6](../../.design/rfcs/0006-full-words.md), driven by the front end rather
than by pattern matching.

## Why it lives on `main`

**It needs the UNPATCHED parser.** The rewriter reads Thermite 2 source, so it
needs a front end that still has `req`/`ens`/`fx` as keywords. On
`anchor/implementation` those keywords are gone, so the tool cannot build there
against the tree it is migrating.

So: build here, run against a worktree of the work branch.

```
git worktree add ../t3-work anchor/implementation
cargo build --release --manifest-path dev/thermite3-migrate/Cargo.toml
uv run --python 3.11 dev/thermite3-migrate/coverage.py ../t3-work
```

## Why it is not a regex

The last clause's text runs to end of line and therefore contains the function
body, so moving the effect row to the front drags `{ x }` with it. Knowing where
a contract ends and a body begins is parsing. Two regex attempts were backed out.

`parse` gives item boundaries and decides whether text is Thermite at all;
`tokenize` gives every offset; the rewrite splices at spans, so comments, blank
lines and alignment survive byte for byte. Two grammar facts make it exact:

- a clause keyword is a **reserved token**, so `TokKind::Req` is a clause and an
  identifier spelled `req` cannot be one;
- `parse_effect_row` is a **closed grammar with no brace in it**, so the row ends
  at the first token that cannot continue it and the body's `{` is whatever
  follows.

Overlapping edits abort rather than being skipped: a dropped edit is silent text
loss.

## Modes

```
thermite3-migrate gate   < src   # exit 0 if it parses clean as Thermite, 3 if not
thermite3-migrate to-v3  < src   # the migrated source
thermite3-migrate edits  < src   # one line per edit, for inspection
thermite3-migrate count  < src   # clause sites, from the lexer rather than a regex
```

`coverage.py` runs it over a tree and reports what it reached and what it
declined, enumerating the declines rather than counting them.

## Measured at `84d276e7`

| | |
|---|---|
| `.th` corpus | 66 of 67 migrate, no clause keyword surviving |
| the one decline | `conformance/parse/recover_per_item.th`, whose purpose is to not parse |
| fragments in Rust literals | 450 migrate, carrying 1,527 clause sites across 111 files |
| declined, no clause keyword | 340 — declining costs nothing |
| declined, clause-bearing | 43 — `format!` templates, assertion prose, fixtures invalid on purpose |

## Known gaps

**A file with any parse error is declined whole.** `parse` recovers per item, so
migrating the items that parsed would shrink the 43.

**The Rust-literal write path is measured, not built.** `unescape.py` carries the
offset map that maps an edit computed on a literal's *value* back to its
*source*, so escaping outside the edit is preserved. Feeding a literal to the
parser without unescaping leaves stray backslashes from `\`-continuations and
turns 153 clean migrations into declines that look like the corpus's fault.
