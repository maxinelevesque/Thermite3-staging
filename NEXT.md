# Thermite 3 — where the work is

Read [PROCESS.md](PROCESS.md) first for branch discipline and the gates. This
document is what to do next and what will bite.

## Where it stands

```
main                    process layer, PROCESS.md, tooling/thermite3-migrate
anchor/implementation   RFC-6's front end — committed, both crates build
rfcs/thermite-3-set     RFC-8 … RFC-14 staged, 19 requirements, gates green
issues #1–#9            the upstream bugs, mirrored
```

Filed upstream and awaiting review:
[RFC-6 / PR #128](https://github.com/dollspace-gay/Thermite/pull/128) (CI green)
and [RFC-7 / PR #129](https://github.com/dollspace-gay/Thermite/pull/129).

### Proved about the anchor

- **The front-end change is 63 insertions / 62 deletions across 5 files**, on
  `anchor/implementation`. `thermite-syntax` and `forge` both build.
- **The migrated corpus certifies identically** — 18 items at L3 across six
  conformance files, same levels *and the same exit status on each*, against
  Verus `0.2026.05.24.ecee80a`.
- **The rewriter reaches the corpus**: 66 of 67 `.th` files with no clause
  keyword surviving, plus 450 embedded fragments carrying 1,527 clause sites
  across 111 Rust files.

## What is next, in order

**1. Migrate the `.th` corpus on `anchor/implementation`.** The tool is built and
proved. It must run from `main` — see the trap below.

**2. Certify, and compare to baseline item for item.** This is the check the
round-trip cannot make: it proves meaning preserved rather than text preserved.
Compare *including the failures* — `map_kv.th` exits 1 before and after, for an
`ens true` that §7.1(a) rejects as `EnsIsTrivial`. Reproducing the rejection
matters as much as reproducing the passes.

**3. Migrate the embedded corpus.** 1,527 clause sites in 450 literals across 111
Rust files — three quarters of the whole migration. **The write path does not
exist yet.** `unescape.py` carries the offset map it needs: compute the edit on
the literal's *value*, map the offsets back to its *source*, and splice, so
escaping outside the edit is preserved.

**4. Regenerate the goldens** rather than editing them.

**5. Run the test suite.** This has never been run post-migration. It is the
actual landability gate and the one genuine unknown here; expect a mix of real
failures and migration artefacts, and separate them before fixing anything.

**6. Handle the 43 clause-bearing declines by hand.** `coverage.py` enumerates
them. They are `format!` templates, assertion prose, and fixtures that are
invalid on purpose — several should stay exactly as they are.

Then: `residual-trust`, `stage-gate`, `upstream-file`.

### Separately, and not on the critical path

**Three productions are in RFC-6 and not in the spike**: conjunct blocks
(`requires { a; b; }`), `requires nothing`, and one-or-more `requires`. They add
productions rather than renaming tokens, so they are the genuinely new part. The
corpus does not use them, so steps 1–2 do not wait on them.

## What will bite

**The rewriter needs the unpatched parser.** It reads Thermite 2 source, so it
needs a front end that still has `req`/`ens`/`fx` as keywords. On
`anchor/implementation` those are gone. Build on `main`, run against a worktree:

```
git worktree add ../t3-work anchor/implementation
cargo build --release --manifest-path tooling/thermite3-migrate/Cargo.toml
uv run --python 3.11 tooling/thermite3-migrate/coverage.py ../t3-work
```

**`verus` is not on PATH** and is required for every `forge check`. About a
minute:

```
curl -sL -o verus.zip https://github.com/verus-lang/verus/releases/download/\
release%2F0.2026.05.24.ecee80a/verus-0.2026.05.24.ecee80a-arm64-macos.zip
```

`macos_allow_gatekeeper.sh` **exits 1 when there is no quarantine attribute**,
which is normal for a `curl`ed archive. Check `verus --version` exits 0 rather
than trusting the script's status. `cadical` is absent, so `EprSolverUnavailable`
is a missing binary rather than a language failure.

**Python 3.9 lacks `tomllib`**, so the registry gates report *inconclusive* and
exit 3 rather than failing. Use `uv run --python 3.11`.

**Read every exit code directly.** `cmd | tail; echo $?` reports `tail`'s status
— that error reached a PR description and needed a public correction. And
`PIPESTATUS` is bash-only: under zsh it expands to empty and reports nothing.

**Cut upstream-bound branches from an `upstream/*` ref**, never from this fork's
`main`, which carries the process layer.

**Look before overwriting.** `.claude/settings.json` here carries Thermite's own
hook wiring, and `control-plane-check.py` exists to catch its loss. It was
clobbered once already by a `cat >` that did not read the file first.

## Discipline

**Probe before specifying.** A three-line file and a `forge check` verdict, not a
reading of the reference. It contradicted the documentation in both directions
repeatedly.

**A round-trip scores what a tool did, never what it skipped.** It reported
382/382 restoring byte for byte while missing every one-line contract and 17
`@bv`-tagged clauses. Pair reversibility with a completeness check.

**Measure a pinned upstream on a `git archive` export of the pin**, never through
a working clone. Counting through one put two foreign probe files into every
published corpus figure.

**When a gate goes red, establish whose fault it is before fixing it.** Run it at
the branch point. Editing `registry.toml` drifts the design doc that governs it,
and the fix is a re-pin rather than a workaround.

**SHIPPED with cited evidence, or NOT STARTED with a named blocker.**

## The framing

The motivation is Thermite's own stated purpose: it is designed to be written
principally by agents, and the surface does not yet serve that. A keyword's real
cost is the prior it activates rather than the tokens it spends. This fork is
staging, not a divergence — work lands upstream by pull request, and if that ever
stops being true it gets decided and recorded rather than discovered.
