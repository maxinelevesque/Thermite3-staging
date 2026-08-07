# Startup prompt — next session

Paste the block below to start. Everything above it is context for you, not for the prompt.

---

Thermite 3 — RFC-15, then walk the anchor to upstream.

Read the process log first: day hook session-start injects the teloi, the recorded
tensions and the atoms. Then `day status` for where the work sits, and
`day bridge check land-the-anchor` for the path from here. The reasoning behind every
decision is in kan, published to `.claims/`; read the practice claims before touching
the environment — they now carry this session's environment notes and three specific
traps that cost real time. Then read `atom/conformance-evidence` and
`atom/upstream-file`, which hold the evidence for what already shipped.

Then read `.design/rfcs/0006-full-words.md` (on `anchor/implementation`) and
`0007-thermite-3.md` (on `rfcs/thermite-3-set`).

## Where things stand

The anchor is **implemented and evidenced**. `anchor/implementation` has two commits:
`39045f22` (front end) and `0dc65eb3` (corpus, tests, fixtures, evidence — 254 files).
Certification matches the pin item for item including the failures; the AST comparison
RFC-6 asks for passes; `cargo test --workspace` has an **empty set difference of
failing test names** against `84d276e7`. What remains is not implementation.

Two PRs are open in the fork and unmerged: **#11** (kernel removal +
`--target freestanding`, zero regressions, three gates green) and **#12** (the
session's claims). `main` now has branch protection — PRs required, 0 approvals, no
force-push, no deletion, admin bypass available.

## The work, in order

1. **RFC-15 — harness-agnostic tooling.** A clone should be bare: no `.claude/`,
   `.crosslink/`, `.mcp.json`, `.claims/` active at root. A justfile materializes a
   contributor's own choice (`just stack claude`, `just stack day`). Removes 39 tracked
   files from upstream plus this fork's layer. Staged unfiled like RFC-8..14 until the
   anchor lands. Also propose the telos for it.

   Write it around **two populations, not one**. Config (`.claude/`, `.crosslink/`,
   `.mcp.json`) is inert files a stack copies into place. Process *vocabulary* (day's
   atoms, teloi, tensions) currently travels through `.claims/`, which conflates
   "reasoning worth sharing" with "vocabulary the tool needs to run" — `day doctor`
   reports 18 atoms and nine are witness subjects that exist only so the graph composes.
   A `day pack` install separates them, and `.claims/` goes back to being only the
   former. The nine witness atoms are published **temporarily** for exactly this reason;
   their removal is a step of RFC-15, and the reasoning is in `practice`.

2. **Then the anchor sequence**: `atom/stage-gate` → `atom/residual-trust` →
   `atom/upstream-file`.

   Before filing, **squash `39045f22` and `0dc65eb3` into one commit** — RFC-6 says the
   migration and the parser change land as one PR, and this fork's practice is one commit
   per RFC so the revision derives from git. They were left separate because the
   implementation PR is not filed yet (PR #128 upstream is the RFC *document* on
   `rfc/full-words`, one commit `d042c17c`).

   Two things a reviewer must be told rather than left to find in a 254-file diff:
   - **The scope is larger than RFC-6 states** — 2,910 clause sites, not 2,074. Four
     measurement gaps, each recorded with its cause. The RFC's own numbers need correcting
     before it is re-filed, or the first reviewer to count will find the discrepancy.
   - **A frozen trust anchor moved.** `thermite-kernel`'s contract digest is sha256 over
     a `Debug` rendering that carries clause spans, so migration moves it. Re-pinned with
     a dated note. (If #11 lands first, this evaporates — the crate goes.)
   - **90 `.md` design docs still describe the v2 surface.** Outside RFC-6's stated scope
     but `doc-drift` is a gate, so settle it before `upstream-file`.

## Discipline

Probe before specifying — a three-line file and a `forge check` verdict, not a reading of
the reference; it contradicted the documentation in both directions repeatedly, and this
session found two front-end defects that way. A round-trip scores what a tool did and
never what it skipped. SHIPPED with cited evidence, or NOT STARTED with a named blocker.

Three traps that bit this session, all now in `practice`:

- **Verify exit codes directly, never through a pipe.** `cmd | tail; echo $?` reports
  `tail`'s status. It happened again here on `doc-drift` — read exit 0 when it was 1.
- **Read the gate's own vocabulary.** `doc-drift` exit **3 is inconclusive, not clean**;
  a `git archive` export has no `.git` and always returns it. Treating 3 as 0 would have
  charged 29 pre-existing drifts to this change.
- **Scope a test run by what it actually reached.** `cargo test -p <crate>` uses default
  features; `--workspace` unifies them on. A clean package run hid seven `#[cfg(feature =
  "bv")]` failures. Post-migration evidence has to come from a workspace run.

And: score the suite against a **control**, never against zero. The pin fails ~49 tests in
any environment without Lean/Mathlib built and cadical present. Compare failing test
*names*, not counts.

Record durable findings in kan as you go, citing the claims they build on. When work
trades one telos against another, record the tension rather than resolving it silently.

---

## Environment (verified this session, also in `practice`)

- `verus` is not on PATH and every `forge check` needs it. Fetch
  `release/0.2026.05.24.ecee80a`, asset `verus-0.2026.05.24.ecee80a-arm64-macos.zip`
  (**`arm64`**, not `aarch64` — the obvious guess 404s). `macos_allow_gatekeeper.sh`
  exits 1 when there is no quarantine attribute, which is normal; check
  `verus --version` exits 0 instead.
- `cadical` is absent → `EprSolverUnavailable` is a missing binary, not a language failure.
- `lean/.lake/packages/mathlib` is an unresolvable checkout → every Mathlib-backed
  reconstruction fails identically before and after any change.
- Python 3.9 lacks `tomllib`; use `uv run --python 3.11` for the registry gates.
- The migration rewriter needs the **unpatched** parser, so it lives on `main` and runs
  against a worktree of the work branch. Its `bv` feature is on the path dependency, not
  a feature of the tool crate — `--features bv` is an error.

## Open issues worth triaging

`#1`–`#4` are mirrors of upstream `#115`–`#118` about the layer PR #11 removes. `#2`, `#3`,
`#4` are mooted by it; `#1` ("disambiguate kernel") is substantially answered. They should
not be closed in the fork — the upstream reports stay open until the change reaches
upstream. Note in `#10`.
