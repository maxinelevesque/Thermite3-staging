# Working in this repo

A **staging fork** of [dollspace-gay/Thermite](https://github.com/dollspace-gay/Thermite),
for building out Thermite 3 while [RFC-6](https://github.com/dollspace-gay/Thermite/pull/128)
and [RFC-7](https://github.com/dollspace-gay/Thermite/pull/129) are under review.
Work lands upstream by pull request. It is not a divergence, and if it ever
becomes one that gets decided and recorded rather than discovered.

## Branch discipline

| branch | base | purpose |
|---|---|---|
| `main` | upstream `main` **plus the process layer** | fork-local tooling only |
| `anchor/implementation` | `upstream/rfc/full-words` | RFC-6's front end and migration |
| `rfcs/thermite-3-set` | `upstream/rfc/thermite-3` | RFC-8 … RFC-14, staged |

**Cut upstream-bound branches from an `upstream/*` ref, never from this fork's
`main`** — `main` carries `.claims/`, `.claude/` and this file, and none of that
belongs in a pull request to Thermite.

## The process layer

`day` and `kan` are wired through a SessionStart hook. Five teloi, five recorded
tensions, and nine atoms that mirror the pipeline Thermite's own stage documents
already run:

```
language-probe → spike → design-pass → requirement-register → implement
              → conformance-evidence → stage-gate → residual-trust → upstream-file
```

The shape is taken from the tree rather than invented. `.design/stage1-*.md` and
`stage2-*.md` carry Summary / Requirements / Acceptance Criteria / Architecture /
Open Questions / Out of Scope; stage 3 and 4 add a limits section, which is why
`residual-trust` is an atom and a telos here.

Three guardrails worth restating, each of which Thermite already enforces:

- **An AC cites the REQ it discharges**, and names the artifact that must exist
  plus what it must contain, so someone who did not write it can check it. AC-8
  in `thermite2-program.md` is the model: a gate encoded as a checklist, with the
  certificate, axiom gate, mutation score and burn receipt fields each named.
- **Status is derived, never declared.** REQs go in `.design/reqs/registry.toml`
  and the status view is generated. A stale generated table is an error.
- **Declare a spike's failure signal in advance.** SPIKE-1 named ">40 lemmas";
  SPIKE-2 committed to reporting its hit rate "whatever it is". A spike whose
  failure condition is decided afterwards is a demo.

## Gates

Run them and read each exit code **directly** — `cmd | tail; echo $?` reports
`tail`'s status, and `PIPESTATUS` is bash-only and expands to empty under zsh.

```
uv run --python 3.11 tooling/rfc-check.py
uv run --python 3.11 tooling/req-registry.py --check     # --write regenerates
uv run --python 3.11 tooling/doc-drift.py
uv run --python 3.11 tooling/req-status.py
uv run --python 3.11 tooling/control-plane-check.py
```

Python 3.9 lacks `tomllib`, so the registry gates report *inconclusive* and exit
3 rather than failing. Pin 3.11.

Adding requirements makes the generated status view stale **and** drifts
`.design/tooling/req-registry.md`, which governs the registry. Both need fixing
in the same pass: regenerate, then re-pin `audited-content-sha256` with a dated
note that keeps the prior entry.
