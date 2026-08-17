# Feature: Typed Forge Result Arbiter

<!--
tier: 3-component
status: shipped
audited-content-sha256: 66587f20b425d64e3a4a407a460977f38d573004b470ea8a85783936cb50a7fd (re-pinned 2026-08-16 after cold-review repairs for live typed stamps, candidate-shape and identity checks, exhaustive transitions, and context-safe policy rendering.)
pin-extract: forge/src/result_arbiter.rs=code-normalized
governs: forge/src/result_arbiter.rs
-->

## Summary

Replace late certificate rewriting with one typed, total result-combination
layer for the current Verus, Lean fallback, EPR reconstruction, solver-vacuity,
and mutation-policy producers. Producers report typed evidence or policy
decisions carrying candidate certificate payloads; the arbiter decides whether
to preserve, select, upgrade, reject, or raise a soundness alarm and is the only
layer allowed to choose the emitted public `Certificate`.

This design serves `telos/a-clause-is-checked`,
`telos/the-corpus-still-certifies`, and `telos/residual-trust-is-named`: checked
evidence must remain attributable, existing corpus certificates must remain
wire-compatible, and combining engines must not erase residual trust, policy,
or boundary facts.

## Requirements

- REQ-1: `forge/src/result_arbiter.rs` shall define closed typed outcomes for
  base proof disposition, supplemental proof evidence, and policy decisions.
  The combination operation shall be total over the currently produced Verus,
  Lean, EPR, solver-vacuity, and mutation outcomes; `forge/src/check.rs` shall
  not decide replacement eligibility by matching certificate reject strings.
- REQ-2: The arbiter shall implement a single precedence rule: an explicitly
  inconclusive result may be settled by complete checked proof or refutation
  evidence; compatible accepted proof evidence may strengthen an accepted
  result; proof versus refutation in either order is a soundness alarm; a
  settled policy rejection is preserved; partial or unavailable supplemental
  evidence does not change item authority. Timeout-derived L2/L1 degraded
  results count as explicitly inconclusive and remain eligible for upgrade.
- REQ-3: Rendering an arbiter result shall preserve orthogonal item context,
  including `AssuranceScope` and its RFC-3 certification boundary, accepted
  contract-quality/mutation evidence, advisory strengthening evidence,
  covenant evidence, and the meaning audit, while replacing route-specific
  proof obligations, engine attribution, and certification coordinates with
  those of the selected proof evidence.
- REQ-4: The public serialized `Certificate` schema in
  `forge/src/manifest.rs` shall remain unchanged. Cache deserialization shall
  pass through a single fail-closed adapter into the typed outcome model. The
  private cache format shall bind its query key and canonical certificate
  digest, and `forge/src/cache.rs::CHECK_SCHEMA_VERSION` shall be bumped so
  pre-arbiter or unbound verdicts cannot bypass the new combination semantics.

## Acceptance Criteria

- [x] AC-1: (REQ-1) Unit tests enumerate every base disposition—accepted,
  explicit timeout, timeout-derived degraded, witnessed counterexample, weak
  contract, semantic tautology, and vacuous precondition—and every relevant
  supplemental disposition—complete proof, partial proof, counterexample,
  unavailable, and unknown—through the same arbiter API.
- [x] AC-2: (REQ-2) A certificate assembled by
  `check::assemble_certificate` from an actual `VerusOutcome::Counterexample`
  combined with complete EPR proof evidence produces
  `EprVerifierDisagreement`; WeakContract and both solver-vacuity rejections are
  byte-for-byte preserved; clean L3, explicit timeout, and timeout-derived
  degraded inputs upgrade.
- [x] AC-3: (REQ-2) The automatic Lean fallback and EPR reconstruction call the
  same arbiter transition rather than separate `auto_should_try_lean` and
  `finish_epr_reconstruction` eligibility policies. Partial EPR evidence remains
  outside the authoritative item certificate.
- [x] AC-4: (REQ-3) A complete proof upgrade of a `ToBoundary` caller retains
  the exact `via` value in both `Certificate.assurance_scope` and
  `CertificationPosition.boundary`; an accepted mutation score and
  strengthening suggestions, covenant evidence, and meaning audit survive the
  upgrade, while stale Verus route coordinates do not.
- [x] AC-5: (REQ-1, REQ-2) Solver-vacuity and mutation scoring construct typed
  policy decisions consumed by the arbiter; no proof engine is permitted to
  replace a policy rejection. Lean-only and automatic selection retain their
  existing observable certificate outcomes.
- [x] AC-6: (REQ-4) Existing certificate JSON fixtures deserialize unchanged,
  focused Forge tests and conformance tests pass, and a cache entry written
  under the previous schema misses after the schema bump.

## Architecture

### Outcome model

Add `forge/src/result_arbiter.rs` and export it privately from
`forge/src/main.rs`. Its central owned value is an `ItemOutcome` containing a
`Certificate` plus an explicit `BaseDisposition`. The certificate remains the
payload needed by existing pipeline code, but it no longer decides transition
policy itself.

`BaseDisposition` distinguishes `Accepted`, `Inconclusive`, `Refuted`, and
`PolicyRejected`. Inconclusive carries a typed reason that distinguishes raw
timeout, timeout-derived degradation, and an engine's unknown result.
`PolicyRejected` carries a typed policy reason for weak contract, semantic
tautology, or vacuous precondition. Fresh producers construct these variants
directly. When existing pipeline stages temporarily carry the public rendering,
the certificate has a private, non-serialized `LiveResultDisposition` stamp;
`ItemOutcome::from_certificate` reads that stamp instead of rediscovering a
fresh verdict from reject strings. A narrow structural adapter exists only for
deserialized cache and legacy boundaries; contradictory shapes fail closed as
an arbiter soundness alarm.

Supplemental engines return `ProofCandidate`: complete checked proof, partial
proof, refutation, unavailable/incompatible tool, or unknown/proof failure.
Complete proof owns a candidate certificate carrying the selected route's
obligations and attribution. It cannot overwrite the base directly and carries
no caller-controlled policy-preservation switch. `combine`, `select`, and
`apply_policy` reject candidate item/effect identity mismatches before rendering,
so an arbitrary full certificate payload cannot be relabeled as the base item.

### Total combination

`ItemOutcome::combine(candidate)` is the single transition function. Its
decision table is exhaustive:

| Base disposition | Complete proof | Partial/unavailable/unknown | Refutation |
|---|---|---|---|
| Accepted | upgrade | preserve | soundness alarm |
| Inconclusive | upgrade | preserve | reject with candidate witness |
| Refuted | soundness alarm | preserve | preserve the established refutation |
| PolicyRejected | preserve | preserve | preserve the policy rejection |

Explicit engine selection uses the sibling `ItemOutcome::select(candidate)`
operation in the same arbiter. It retains settled policy gates and symmetric
proof/refutation alarms, while allowing an explicitly selected engine's honest
unknown certificate to replace a non-policy base for diagnostics. Automatic
fallback uses `combine`, where unknown preserves the base.

An upgrade retains orthogonal context through a dedicated renderer. The
renderer copies assurance scope, accepted contract-quality/mutation results,
advisory strengthening data, covenant evidence, and meaning audit from the
base, then applies the candidate's proof obligations, engine attribution, and
certification position. The candidate's genuine counterexample obligations are
likewise retained when it refutes. Policy rendering goes through a sibling
positive-allowlist merge, so the mutation decision applied after a Lean proof
cannot erase proof-independent context. This prevents accidental transfer of
stale Verus authority while making every retained field reviewable.

### Producer migration

`forge/src/check.rs::assemble_certificate` returns a typed `ItemOutcome` for
Verus success, timeout, and counterexample. The degrade ladder changes the
disposition to typed inconclusive without discarding its L2/L1 certificate.

Solver-vacuity and mutation scoring return `PolicyDecision` values; the arbiter
renders their existing rejection certificates. This retains the current
pipeline ordering—vacuity before the main proof, mutation after a successful
body proof—while centralizing the fact that a policy decision is settled and
cannot be superseded by a contract-only engine proof.

The automatic Lean route and EPR route convert engine results to
`ProofCandidate` and call `combine`. `auto_should_try_lean` and
`finish_epr_reconstruction` cease to be independent policy authorities. Engine
selection may still avoid invoking an optional engine when the typed base
disposition cannot change, but that optimization queries the arbiter's typed
eligibility method.

### Compatibility and cache boundary

No serialized field is added to `Certificate`; the live disposition stamp is
`serde(skip)`. Public JSON, audit projection, and frozen fixtures therefore
retain their current shape. Cache hits are decoded by one
`ItemOutcome::from_persisted_certificate` adapter after existing artifact
validation and cache-envelope verification. The envelope binds schema, query
key, and a canonical digest of the complete stored certificate. The adapter
then checks the structural shape—level, reject, failed obligations,
lowered-assurance marker, and vacuity/mutation policy quality fields—and returns
a soundness alarm for contradictions rather than guessing from one string.
The digest detects corruption and unsynchronized edits; it is not authentication
against an actor able to rewrite both a local row and its digest, who remains
inside the local cache trust boundary.

Because the fresh result is now a different function of the same cache-key
inputs and the cache format is integrity-bound,
`forge/src/cache.rs::CHECK_SCHEMA_VERSION` advances from 9 to 11 (10 was the
first arbiter format; 11 closes the review-found unbound-policy row).

## Resolved Questions

- The arbiter covers every current producer in this change: Verus, Lean
  fallback, EPR, solver-vacuity, and mutation scoring.
- Typed outcomes are internal. The serialized `Certificate` schema remains
  unchanged.
- Complete checked evidence may upgrade raw timeouts and timeout-derived L2/L1
  degraded outcomes. Counterexamples and policy rejections remain settled.

## Open Questions

## Residual trust

The arbiter closes transition completeness only for the five migrated producer
families. It does not prove that an individual engine's evidence is sound; the
named Verus, Lean kernel, solver, lowering, mutation-family, and policy premises
remain exactly those carried by the selected route. The persisted-certificate
adapter must reconstruct a typed disposition from a schema that predates the
arbiter, so its structural checks are trusted Rust until that adapter is covered
by the RFC-3 replay boundary. Specialized Forge, BV, and NLSAT paths remain
outside this arbiter and retain their current local combination logic.

## Out of Scope

- Converting every specialized Forge/BV/NLSAT certificate constructor outside
  the current Verus/Lean/EPR/policy pipeline.
- Adding an append-only evidence ledger or changing public certificate JSON.
- Retiring legacy `Level` values or completing the remaining RFC-3 display and
  aggregation work tracked by `.design/versioned-language-completeness.md`.
