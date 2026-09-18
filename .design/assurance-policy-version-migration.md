# Feature: Checked assurance policy-version migration

audited-content-sha256: 7d0df445db5341005b5f04f2f5c6cc5e7f260e63be306d754881fc011904429b (identity-on-kind semantic hardening after organization-policy adversarial review)

## Summary

Assurance reports currently compare only when their collapse-policy versions
are equal.  This feature permits comparison across an explicitly declared
policy-version edge only when Lean proves a total, exact-fiber translation of
the formal policy domain and Rust replays a receipt bound to the same edge and
translation.  A migration never rewrites certificate authority.  It translates
the report's policy points and common/evidence frontiers solely for diagnostic
comparison; source authority digests remain the evidence identities shown to
the reviewer.

Absent, reverse, endpoint-mismatched, non-total, non-monotone, or
information-losing migrations render policy skew.  In particular, a lossy
migration cannot produce a strengthened or weakened code verdict.

## Requirements

- REQ-1: Lean shall define a directional `PolicyVersionMigration` whose source
  and target versions are distinct, whose translation is total over all six
  `AssurancePolicyV2` constructor families, and whose result preserves the
  exact claim fiber and population admissibility.
- REQ-2: A comparison-capable migration shall prove order preservation and
  order reflection.  These obligations make the translation an embedding of
  the supported source policy domain rather than an unchecked relabeling.
- REQ-3: Lean shall lift a migration to `AntichainNF` and prove that migrated
  membership is exactly the image of source membership, that the output remains
  a normalized downset, and that migration commutes with finite intersection.
- REQ-4: Identity and composition shall be typed.  Composition requires exact
  middle-version agreement and preserves exact fiber identity.
- REQ-5: The migration disposition shall distinguish checked-compatible from
  information-loss and checked-incompatible edges.  Only checked-compatible
  edges may enter report comparison.
- REQ-6: Rust shall expose a closed migration receipt containing exact source
  and target policy versions, all six source-to-target family rows, the named
  Lean witness, and a canonical digest over those fields.  Validation shall
  reject reverse/equal endpoints, duplicates, omissions, unknown kinds,
  non-monotone or non-reflecting maps, semantics-changing family relabelings,
  empty witnesses, and digest tampering. Until Rust directly replays Lean's
  population-admissibility law, checked-compatible receipts shall be
  identity-on-kind version renames.
- REQ-7: Report comparison shall continue to require the exact base revision,
  schema equality, structural report validation, and exact source/target policy
  versions.  It shall compare only after a validated migration translates the
  base policy points/frontiers to the head policy domain.
- REQ-8: Authority digests shall never be rewritten by policy migration.
  Comparison output shall separately record the migration receipt and both
  original authority digests.  Presentation digests remain version-bound.
- REQ-9: A missing, lossy, reverse, misbound, forged, or incompatible migration
  shall return explicit policy skew and an empty movement list.  It shall never
  return strengthened, weakened, equal, or incomparable code movement.
- REQ-10: One generated Rust/Lean replay matrix shall cover compatible,
  missing, lossy, reverse, incomplete, non-monotone, non-reflecting, misbound,
  and forged cases with deterministic canonical bytes.
- REQ-11: CI, requirement registry, routes, typed claim closure, axiom probing,
  and documentation drift shall fail closed when the migration surface or its
  evidence changes without coordinated updates.

## Acceptance Criteria

- [x] AC-1: Lean constructs an exact identity-shaped policy migration between
  two distinct policy versions and proves point, antichain, intersection,
  identity, and composition laws without new axioms.
- [x] AC-2: Deleting one family row, changing one target family, changing an
  endpoint, or reversing the edge makes the Rust receipt invalid and the Lean
  replay row reject. Even a bijective order automorphism is rejected when it
  changes kind semantics.
- [x] AC-3: A report pair with different policy versions remains `policy_skew`
  under the existing comparison API and becomes comparable only through the
  new API with an exact validated witness.
- [x] AC-4: Compatible migration yields deterministic item and common-frontier
  comparison while preserving the reports' original authority digests.
- [x] AC-5: Lossy, non-monotone, non-reflecting, missing, reverse, misbound, and
  forged migrations produce policy skew with no item movements.
- [x] AC-6: The CLI accepts an optional migration receipt alongside
  `--compare`; supplying one without comparison inputs, or supplying a report
  mismatch, is a usage/validation error rather than silent fallback.
- [ ] AC-7: Generated replay, focused Rust tests, Lean build and axiom probe,
  typed claim closure, full local qualification, cold exact-head adversarial
  review, and required CI all pass before merge.

## Architecture

The Lean object is proof authority.  Its family translation is deliberately
closed over the current six constructor families while its fiber parameter is
arbitrary.  Report comparison uses only the finite family projection because
every report item already carries its exact fiber and formal position; a
migration may not alter either.  Order reflection is required in addition to
monotonicity so two distinct source claims cannot collapse into one target
claim and manufacture an apparent code improvement or regression.

The Rust receipt is replay evidence, not a substitute for the Lean theorem.
It validates the same finite obligations and binds the witness name and
endpoints into a canonical digest.  The generated matrix checks semantic
agreement between the two implementations.  The receipt is consumed only by
diagnostic report comparison and never by live admission, floors, caching,
aggregation, or certificate issuance.

## Residual Trust

Rust trusts the reviewed witness name to identify the corresponding Lean
theorem; the generated replay and axiom probe keep that association visible but
do not turn a string into proof authority.  The migration compares reports
whose certificate authority was established independently.  It does not prove
the reports were produced by a trusted CI runner, and it cannot recover meaning
lost by an older report schema.

## Out of Scope

- Migrating report schemas that omit required formal authority data.
- Re-certifying historical `Level` records or inventing missing fibers.
- Cross-semantic, cross-model, cross-procedure, or boundary transport; those
  remain governed by checked assurance transport.
- Workspace/build-matrix population composition (#60).
- Organization policy inheritance and exceptions (#63).
