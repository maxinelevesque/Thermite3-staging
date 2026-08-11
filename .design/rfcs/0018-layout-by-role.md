---
rfc: 18
title: A layout by role — what checks, what changes, and what a contributor opted into
status: draft
supersedes: []
introduces: []
---

# RFC-18: A layout by role — what checks, what changes, and what a contributor opted into

| | |
|---|---|
| **Status** | Draft, unfiled |
| **Supersedes** | — |
| **Baseline** | `staging @ 43fabc09` |
| **Scope** | Repository layout, not the language |
| **Relation** | Amends RFC-15 §3.1–§3.2 (naming). Independent of RFC-6..14, RFC-16, RFC-17; touches no `.th` file |

Like RFC-15, this proposes nothing about Thermite the language. It is about
where things sit, and specifically about the fact that the repository currently
files its checks by **what language they are written in** rather than by what
they do.

## 0. What this asks

Two moves, and one deliberate refusal.

1. `gates/` — everything that **checks** and can fail, in any language.
2. `dev/` — everything that **changes or provisions**, which is not the same
   thing and does not belong beside it.
3. It does **not** move the seven workspace crates. §2 is about why.

## 1. The problem is not the name `tooling/`

### 1.1 The gates are split by implementation language

Seven Python gates live in `tooling/`. Three shell gates live in `scripts/`:

| gate | lives in | invoked by |
|---|---|---|
| `doc-drift.py`, `req-status.py`, `rfc-check.py`, `control-plane-check.py`, `spec-discipline.py`, `anti-pattern-gate.py`, `req-registry.py` | `tooling/` | `ci.yml` `checks` job, `Makefile` |
| `g3-gate.sh` | `scripts/` | `ci.yml` `g3` job |
| `g4-gate.sh` | `scripts/` | `ci.yml` `g4` job |
| `lean-axiom-probe.sh` | `scripts/` | `ci.yml` `lean-probe` job |

`doc-drift.py` and `g3-gate.sh` are the same kind of object: a check CI runs,
which fails the build when an invariant breaks. They are filed apart for one
reason, which is that one is Python and one is Bash. Nothing else about them
differs. A contributor looking for "the checks" must know to look in two places
and must know the rule that decides which.

### 1.2 `tooling/` is a residue, not a category

What is actually in it:

* seven gates (§1.1);
* `tests/` — the unittest suite for those gates;
* `reqs` — an executable CLI, invoked as `uv run tooling/reqs check`;
* `spec-routes.toml` — the route table `spec-discipline.py` reads;
* `thermite3-migrate/` — a one-off Thermite 2 → 3 source rewriter, which is not
  a gate, is not run by CI, and is not run by anyone except during a migration.

Four unlike things and a migration tool. The directory has no membership rule
that would tell you whether a new file belongs in it, which is the operational
test for whether a name is a category or a residue.

### 1.3 The name misdescribes, in the direction `telos/surface-serves-agents` cares about

`tooling` reads as *build tooling* — the things that build the project. It
contains none of that. Per the telos, the cost of a misdescribing name is not
brevity but misdirection: an agent reading `tooling/` predicts a build system
and finds a gate suite.

## 2. What this deliberately does not ask

The obvious next move is `crates/` for the seven workspace members, which would
take the repository root from seven crate entries to one. This RFC does not
propose it. Measured, on `staging @ 43fabc09`:

| move | files touched | references to rewrite |
|---|---|---|
| `scripts/` → `gates/`, `dev/` | 31 | 163 |
| `tooling/` → `gates/`, `dev/` | 71 | 349 |
| `conformance/` → | 193 | 1,067 |
| `thermite-syntax` → `crates/` | 115 | 1,211 |
| `forge` → `crates/` | 225 | 3,767 |

Plus **154 `crate_pattern` entries** in the route table and **60 design
documents carrying pins** (59 with `audited-content-sha256`, 46 with a legacy
`audited-sha`, 60 distinct).

The two moves this RFC proposes are ~512 references. The crate move alone is
roughly ten times that, and it buys one line of root tidiness. That is the diff
shape `telos/it-lands-upstream` cannot absorb — the same objection already
recorded as the tension between `it-lands-upstream` and
`surface-serves-agents`, where 547 clause sites across 67 files was judged "not
a diff anyone reviews line by line". A crate move deserves its own RFC and its
own argument, where a maintainer can reject it without also rejecting this.

## 3. Proposal

### 3.1 The distinction

**`gates/` checks. `dev/` changes.** A gate answers *is the tree in a legal
state* and may fail. A dev tool does work: it rewrites sources, or it fetches a
pinned binary. The test for a new file is one question, which is what §1.2 says
`tooling/` lacks.

Note this is deliberately **not** "CI runs it versus a human runs it".
`audit.sh` is a check that only a human runs — `make audit`, the re-derivation a
skeptic performs on their own machine — and it belongs with the checks.
`install-g4-tools.sh` is provisioning that only CI runs, and it does not.

### 3.2 What moves where

```
gates/
  anti-pattern-gate.py   control-plane-check.py   doc-drift.py
  req-registry.py        req-status.py            rfc-check.py
  spec-discipline.py     reqs                     routes.toml
  g3.sh                  g4.sh                    lean-axiom-probe.sh
  audit.sh               tests/

dev/
  thermite3-migrate/
  install-g4-tools.sh    g4-toolchain.env         g4-tools/drat-trim
```

`spec-routes.toml` becomes `gates/routes.toml`: the `spec-` prefix
disambiguated it inside `tooling/`, and inside `gates/` it does not need to.
The `-gate` suffix comes off `g3-gate.sh` and `g4-gate.sh` for the same reason.
Both renames are separable from the moves and can be dropped without affecting
the rest.

### 3.3 What follows the move

`ci.yml` (12 invocation sites), `Makefile` (14), and the route table's own
entries — three `crate_pattern`s point at `tooling/doc-drift.py`,
`tooling/req-registry.py` and `tooling/reqs`, while their `design` fields are
unaffected.

Three couplings are not mechanical and are named here so they are not
discovered late:

* `doc-drift.py` hardcodes `ROUTES_RELPATH = "tooling/spec-routes.toml"`, which
  becomes `gates/routes.toml`.
* `doc-drift.py` decides pin-extract ownership with `"tooling/" in command`
  (§4). That test becomes `"gates/"`.
* `.claude/settings.json` wires three hooks by path into `tooling/`, and
  `control-plane-check.py` asserts exactly those three. Both move together, and
  `.design/gates/control-plane.md` is re-pinned once as a consequence.

A separate question this raises and does not answer: `.design/gates/` would
then hold the design docs governing `gates/`. Renaming it costs 3 documents, 10
`design =` fields, 16 referencing files — and **3 re-pins**, which is worth
recording because the obvious prediction is zero. A pin digests the governed
*source*, and the source does not move; but the governed Python sources cite
their own design doc in their module docstrings, so rewriting the reference
changes the source and moves the digest. Same family as the glob-membership
case in §4: what moved was not the governed file but something the governed
file mentions. Out of scope *here* only to keep this RFC to one argument.

## 4. Residual trust

Per `telos/residual-trust-is-named`, what this does **not** discharge:

* **`doc-drift` is blind to this change everywhere but one document.** Its pins
  are `content-sha256` over file *contents*, and a move leaves contents
  byte-identical, so the gate stays green through a re-layout that invalidates
  every path around it — the same shape as the recorded finding that a
  round-trip proves information preservation and is silent about coverage.
  `spec-discipline` does fail loudly, since an unrouted file blocks under
  R-XLATE-2, so the hole is narrow. But this RFC does not add a check that a
  route's target exists, and it should not be read as claiming the move is
  digest-neutral.

  The exception is `.design/gates/control-plane.md`, and it is instructive.
  That document opts into `pin-extract: .claude/settings.json=claude-hooks`
  (RFC-16 layer 1), and `doc-drift.py` decides which hooks the repository *owns*
  by testing `"tooling/" in command`. The three owned hooks invoke
  `tooling/spec-discipline.py` and `tooling/anti-pattern-gate.py`, so renaming
  the directory changes the extracted region's bytes and moves the digest. The
  move therefore requires a deliberate re-pin of exactly one document, plus a
  one-word change to the ownership test inside `doc-drift.py` itself — a gate
  that must be edited to keep recognising the gates.

  This is a second instance of the recorded rule that editing a governed source
  file drifts every document whose route digests it, arriving from an unusual
  direction: here the *governed file is unchanged* and only its path moved, yet
  a digest still shifts, because the pinned region was defined by a path
  substring rather than by structure.
* **A glob's membership is part of the digest, so a move drifts documents that
  govern globs the moved file happened to match.** `_content_digest` digests,
  per pattern, the *set* of files the pattern matches. `scripts/g4-*` governs
  `.design/stage4-epr-reconstruction.md`, and moving `g4-gate.sh` out of
  `scripts/` removed a member — so that document drifted with no file's content
  changed and no pattern string changed. Worse, the moved file **silently lost
  its route**: the glob no longer reaches it, and nothing scans for unrouted
  files, so the loss surfaces only when someone tries to edit it and R-XLATE-2
  blocks them. This RFC adds no check for either. *(Both observed rather than
  predicted — §4 as first written claimed only one document would drift.)*

* **A green local gate suite is not evidence about the shell gates.** The suite
  runs the seven Python gates and their 67 unit tests. It never *executes*
  `g3.sh` or `g4.sh`, which need Lean, Verus and the pinned CaDiCaL/drat-trim
  pair — so it is structurally incapable of catching a bad path inside them.
  Measured: step 1 shipped with `gates/g3.sh` still invoking
  `scripts/lean-axiom-probe.sh`, a file step 1 had itself moved, and CI failed
  the `g3` job with exit 127 after the local suite had reported green. The
  cheap check that does catch it — assert every repo-relative path referenced
  by the shell gates, the `Makefile` and `ci.yml` exists — is not wired into
  any gate by this RFC.

* **`ci.yml` and the `Makefile` are rewritten by hand.** Nothing pins that they
  agree with the new layout beyond CI going green, and a step that is silently
  skipped rather than failed would not be caught.
* **`dev/` is defined negatively.** "Not a gate" is a weaker membership rule
  than `gates/` has, and it will attract residue the way `tooling/` did unless
  something stops it. This RFC proposes nothing that stops it.

## 5. Sequence

1. `gates/` lands: both halves of the gate suite move, `ci.yml`, `Makefile` and
   the route table follow.
2. `dev/` lands: the migration tool and the g4 provisioning move.
3. `opt-in/` lands, per RFC-15 as amended below.

Each step is independently reviewable and **each is reversible** — unlike
RFC-15 step 3, this RFC contains no step that cannot be un-decided by a revert.

## 6. Amendment to RFC-15

RFC-15 is draft and unfiled, so this revises it in place rather than superseding
it. Four changes, none touching its argument:

* `packs/` becomes **`opt-in/`**, and `just stack <name>` becomes
  **`just use <name>`**. The directory name then states the claim
  `telos/the-clone-is-neutral` makes: everything under it is a choice.
* **Three stacks, not two.** `opt-in/claude/` (`settings.json`, `agents/`),
  `opt-in/crosslink/` (`.crosslink/`, and `.mcp.json` — which declares three
  `crosslink-*` servers and is therefore crosslink's, not Claude Code's), and
  `opt-in/day/`, which stays empty until RFC-15 step 4 since PR #21 already
  removed day's duplicated hooks and its project-scoped MCP entry.
* A **fourth measured cost** for RFC-15 §1: the tracked control plane is not
  self-contained. `.gitignore` marks `.claude/hooks/`, `.claude/commands/` and
  `.claude/mcp/` as generated by `crosslink init`, so they are absent from a
  fresh checkout — while tracked `.claude/settings.json` wires nine hooks, six
  of which point into `.claude/hooks/`, and tracked `.mcp.json` names three
  servers under `.claude/mcp/`. A contributor who has Claude Code but not
  crosslink gets a control plane referencing nine files that do not ship. This
  needs no second harness to observe, which makes it stronger than §1.2 and
  §1.3, both of which are conditional on this fork's own tooling.
* RFC-15 §3.2's "materializes `.claude/` and `.mcp.json` from `packs/claude/`"
  is wrong on the second object and is corrected accordingly.
