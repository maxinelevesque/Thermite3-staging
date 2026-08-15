# Feature: Versioned Language-Wide Soundness and Completeness

audited-content-sha256: 2621b8e421b52ae7f018890d9f070dd12f95bbb944b6d1af91b5dc2d56a6b920 (issue-48 inventory and formal slices: reviewed the fail-closed ledger, stage-indexed vocabulary, RFC-10 refinement, same-lineage expansion, explicit narrowing witness, negative pin, and axiom-probe coverage together)

## Summary

Define a language-wide, versioned framework that states which Thermite programs
are supported at each semantic stage, what each stage proves, and how every
input is classified without conflating unsupported language, policy refusal,
resource exhaustion, missing tools, failed verification, or trusted discharge.
The first version inventories the whole currently claimed language, makes strong
theorems only over explicitly named fragments, and turns every discovered gap
into either work in this project or a tracked item on a durable completeness-review
track.

This design serves `telos/a-clause-is-checked`,
`telos/the-corpus-still-certifies`, and `telos/residual-trust-is-named`. It
generalizes `.design/rfc10-evidence-completeness.md` without treating RFC-10's
checker vocabulary as the language-wide ontology.

## Requirements

- REQ-1: The framework shall enumerate every current parsed language construct
  and every claimed semantic or certification behavior, then classify its
  support at the parser, validator, canonical-semantics, checked-IR, lowering,
  proof-route, policy, and certification stages. An absent or unclassified
  construct shall fail the inventory gate.
- REQ-2: The inventory shall compare documented claims in `thermite-design.md`,
  `docs/language.md`, `docs/verification.md`, `docs/v2/semantics.md`, the RFCs,
  and the requirement registry against executable and formal evidence. Each gap
  shall name the overclaim or missing proof and be dispositioned as in-scope
  implementation, an explicitly bounded fragment exclusion, or a linked issue
  on the completeness-review track.
- REQ-3: A neutral Lean layer, independent of RFC-10 naming, shall define a
  canonical language-program representation, semantic validity, versioned
  supported-fragment predicates, and stage-indexed outcome predicates. RFC-10's
  `SupportedRFC10` and `verify_iff_supported` in
  `lean/Thermite/CheckedTraversal.lean` shall become an instance or proved
  refinement of this layer rather than its foundation.
- REQ-4: Every ordinary fragment revision shall be monotone and carry a
  kernel-checked old-to-new inclusion theorem. A narrowing shall create an
  explicitly versioned compatibility break with a recorded rationale and a
  counterexample to the former inclusion, rather than silently revising an
  existing predicate.
- REQ-5: The framework shall distinguish evidence completeness, producer
  completeness, lowering completeness, proof-route progress, finite-policy
  completeness, and certification completeness. No theorem at one stage shall
  imply a stronger downstream theorem without an explicit composition theorem.
- REQ-6: The logical Lean producer shall be total on each fragment for which
  producer completeness is claimed. The bounded Rust producer in
  `thermite-lower/src/witness.rs` shall refine the logical producer under an
  explicit sufficient-budget premise, including canonical serialization and
  decoding preservation; this absorbs the general producer-refinement portion
  of issue #49.
- REQ-7: Every input admitted by the parser shall reach exactly one structured
  classification at each attempted stage. The outcome vocabulary shall keep
  semantic unsupportedness, invalid source, policy unsupportedness, resource
  exhaustion, unavailable or incompatible tools, counterexample, proof failure,
  internal soundness alarm, and successful certification distinct.
- REQ-8: The certification surface shall use RFC-3's authoritative
  `scope/refutation/trust@boundary` coordinate system and product order. The
  framework shall not introduce a parallel assurance model or preserve
  `manifest::Level` as a live competing projection. Every successful result
  shall identify its proof route, frame, assumptions, checked artifacts,
  discharged obligations, and residual trusted correspondence.
- REQ-9: External solver completeness shall not be assumed. Solver-backed
  routes shall instead prove total classification or progress under explicit
  environment and resource premises, with `Unknown`, timeout, and missing-tool
  outcomes preserved rather than converted to semantic unsupportedness or a
  lower assurance claim without an explicit route transition.
- REQ-10: Mutation and contract-quality policy shall define a deliberately
  finite policy fragment with a total decision theorem, or return a named
  unsupported-policy outcome outside that fragment. Policy incompleteness shall
  not change semantic validity or evidence soundness.
- REQ-11: Subsequent language RFCs shall state whether they preserve, expand,
  or narrow every affected fragment; provide new classifier cases and inclusion
  proofs; update the generated support matrix; and add negative-space witnesses
  demonstrating that omitting each new classifier condition is observable.
- REQ-12: The implementation shall establish a completeness-review issue track
  whose enumerated backlog is generated or checked against the gap inventory.
  Closing an item shall require executable or formal evidence and shall update
  the corresponding fragment status rather than merely removing prose from the
  backlog.
- REQ-13: The non-specialist display projection specified by RFC-3 shall be a
  total, versioned function from formal certification positions and their frames
  to a closed engineer-label vocabulary. Lean shall prove the semantic
  entailment advertised by every label, characterize exactly which formal
  distinctions each label forgets, and reject any collapse not declared by the
  active collapse policy.
- REQ-14: Display labels shall be informational projections only: no admission,
  routing, floor, aggregation, or certification decision may consume an
  engineer label. Stored formal coordinates and frames remain authoritative;
  stored engineer labels shall be checked against the versioned projection so
  they cannot drift independently.

## Acceptance Criteria

- [ ] AC-1: (REQ-1, REQ-2) A checked, generated support matrix covers every
  current AST construct and every documented language claim across all named
  stages; deleting a construct disposition or adding an undispositioned AST
  variant makes its gate fail.
- [ ] AC-2: (REQ-2, REQ-12) The initial gap report names every mismatch found
  between the authoritative documents, requirement registry, implementation,
  tests, and Lean theorems, and every unresolved gap has exactly one checked
  disposition: current-project slice, bounded exclusion, or open
  completeness-review issue.
- [x] AC-3: (REQ-3, REQ-5) A neutral Lean module defines the language-wide
  program, validity, fragment, and stage predicates; RFC-10 has a kernel-checked
  refinement into that vocabulary, and the axiom probe covers the theorem
  family.
- [x] AC-4: (REQ-4) At least two fragment versions exist with a kernel-checked
  inclusion theorem and a negative mutation that makes the inclusion proof
  fail; the compatibility-break path is separately tested with a concrete
  narrowing witness.
- [ ] AC-5: (REQ-5) Lean type signatures and composition theorems prevent an
  evidence-completeness result from being used as lowering, proof-route, policy,
  or certification completeness without the intervening premises.
- [ ] AC-6: (REQ-6) The logical producer is total on the initial claimed
  producer fragment; Rust output refines it under the exact sufficient-budget
  premise; serialization mutations, truncation, version skew, and same-shape
  payload changes fail preservation checks.
- [ ] AC-7: (REQ-7, REQ-9) A generated outcome matrix drives representative
  programs through every available stage and distinguishes all named outcome
  classes. No resource or environment failure is reported as unsupported
  language, proof refutation, successful certification, or silent downgrade.
- [ ] AC-8: (REQ-8) Certificates and audit output expose assurance result and
  trust profile as independently inspectable data, and tests show that two
  routes with the same assurance level but different trusted bases remain
  distinguishable.
- [ ] AC-9: (REQ-9) Solver-route theorems state progress/classification rather
  than solver completeness; timeout, `Unknown`, unavailable, incompatible, and
  successful cases are each exercised without changing semantic-fragment
  membership.
- [ ] AC-10: (REQ-10) The finite policy fragment has a total decision theorem
  and mutation pins for every boundary condition, while an outside-fragment
  fixture yields the named unsupported-policy outcome and remains eligible for
  non-policy semantic and proof analysis.
- [ ] AC-11: (REQ-11) A fixture RFC expansion cannot pass CI without updating
  the fragment classifier, old-to-new theorem, support matrix, and a
  negative-space witness; a semantic narrowing cannot pass as an ordinary
  monotone revision.
- [ ] AC-12: (REQ-12) The completeness-review backlog and gap inventory agree
  in both directions, and closing a fixture backlog item without adding its
  cited executable or formal evidence fails the track's consistency gate.
- [ ] AC-13: (REQ-13) Lean defines the closed engineer-label vocabulary and
  projection relation, proves projection totality on every coherent formal
  position, and proves one meaning theorem per display label. Each theorem
  states the scope, refutation guarantee, residual-trust bound, boundary
  qualification, and fragment/frame premises that the label entails.
- [ ] AC-14: (REQ-13) For every pair of distinct formal positions mapped to one
  engineer label, a kernel-checked collapse theorem states their common
  decision-relevant semantics and a generated disclosure table names every
  distinction lost by the projection. An undeclared many-to-one mapping fails
  validation.
- [ ] AC-15: (REQ-13, REQ-14) The stored formal label and stored engineer label
  round-trip through the versioned collapse policy; mutations of the formal
  tuple, frame, policy version, or display label either recompute consistently
  or reject. Repository search and structural tests demonstrate that gates and
  routing inspect formal data rather than display labels.

## Architecture

### Language-wide formal vocabulary

A new neutral Lean module under `lean/Thermite/` owns the versioned fragment and
stage vocabulary. `lean/Thermite/CheckedTraversal.lean` supplies the first
feature-specific instance. The neutral representation must be rich enough to
bind source constructs, semantic facts, and stage outcomes, but it need not
pretend that every accepted parser node already has proved downstream support.
Instead, the generated inventory makes partial support explicit.

Fragment versions are immutable names. Ordinary expansion proves inclusion;
semantic strengthening that preserves membership proves a preservation theorem;
narrowing creates a new compatibility lineage. The framework records the
distinction so later RFCs cannot use a changed predicate under an old theorem
name.

### Gap inventory and completeness-review track

The initial build derives a source inventory from `thermite-syntax/src/ast.rs`
and compares it with validator handling, `thermite-syntax/src/semantic.rs`,
checked construction, lowerers, Forge routes, policy gates, formal models,
conformance fixtures, and requirement evidence. Documentation claims are
separate inputs because an implemented path can still contradict the assurance
promised by `thermite-design.md` or `docs/verification.md`.

Each gap has a stable identifier, affected stages, smallest counterexample,
current observed outcome, claimed outcome, trust consequence, and disposition.
The project implements gaps required to make the initial formal framework
honest and executable. Independent language-feature gaps become issues on the
completeness-review track; their presence remains visible in generated support
views until closed with evidence.

### Stage-indexed results

The model separates language membership and validity from operational progress.
A program can be in a semantic fragment while a proof route returns unavailable
or resource-exhausted; conversely, a parser-accepted program can be outside a
downstream fragment without being semantically invalid. Stage composition is
therefore a chain of typed results rather than one boolean `supported` flag.

The total-classification target applies to the tool's own control flow: every
attempt ends in a named result. It does not claim that external solvers decide
all obligations or that all valid programs certify. Strong certification
completeness is stated only for fragments whose complete route and assumptions
are proved.

### Certification coordinates and RFC-3

`.design/rfcs/0003-certification-surface.md` is authoritative for certification
structure. Its four coordinates are scope, refutation, trust, and boundary; its
coherent positions form a product order with genuine incomparable elements.
Issue #48 supplies the fragment and classification metatheory that RFC-3 R2-8
requires. Classification fixes the relevant fragment and refutation fiber;
discharge fills the proof-route and trust position; boundary records how far
the claim closes.

The initial gap inventory measures RFC-3 increment by increment. Existing
`forge/src/engine.rs::TrustProfile`, per-obligation attribution in
`forge/src/manifest.rs`, and `AssuranceScope` are evidence of partial adoption,
not grounds to retain `Level` as a second authority. The migration follows
RFC-3 and `.design/rfcs/0004-versioning.md`: formal coordinates and certificate
schema versioning land before removal of Lx, and historical L3 artifacts remain
explicitly ambiguous rather than receiving an invented translation.

### Proved display projection

RFC-3's dual-label surface is part of the formal deliverable. The formal label
is the coordinate tuple plus its frame: fragment version, procedure, axioms,
and residuals. The engineer label is a deliberately lossy statement of what a
non-specialist can act on, such as “proven for all inputs; a false clause gives
you a concrete failing input.” Both are stored, while the versioned collapse
policy declares their relationship.

The neutral Lean layer defines the engineer-label vocabulary and the projection
relation. For each label it proves a meaning theorem from the formal position:
what population the claim covers, what happens when it is false, what boundary
qualifies it, and what remains trusted. A many-to-one projection also requires a
collapse theorem proving the common advertised semantics of all source
positions. The theorem does not claim the forgotten distinctions are equal;
the generated disclosure view lists them so `--explain` can recover the frame.

The projection is total only over coherent, versioned formal positions. It is
not an acceptance order and no gate consumes it. Certificate validation checks
that the stored display label is licensed by the stored formal position and
collapse-policy version. This makes the approachable display useful without
allowing it to become the second source of truth RFC-3 rejects.

### CI and review

`gates/lean-axiom-probe.sh` covers all soundness, completeness, inclusion,
classification, and refinement theorems. Generated negative-space cases remove
one classifier premise or corrupt one boundary artifact and must fail at the
named stage. Corpus comparisons remain regression evidence, not proof of
fragment completeness. Adversarial review focuses on omitted inventory cases,
unproved stage composition, trust-profile laundering, and compatibility
narrowing.

## Open Questions

- None.

## Resolved Questions

- The first inventory covers the entire currently claimed language. Strong
  theorems apply only to explicitly named stage fragments; gaps are never hidden
  by shrinking the inventory.
- The formal framework is language-wide and neutral. RFC-10 refines into it.
- #48 absorbs #49's general producer-refinement and serialization work; #49
  retains only RFC-10-specific semantic and conformance expansion.
- Ordinary fragment evolution is monotone. Narrowing requires an explicitly
  versioned compatibility break.
- Every discovered claim/evidence gap is dispositioned piece-wise into this
  implementation, a bounded exclusion, or the completeness-review issue track.
- RFC-3's certification coordinates, product order, dual labels, and removal of
  Lx are authoritative. Issue #48 supplies their language-fragment,
  classification, and projection metatheory rather than designing another trust
  surface.
- The non-specialist display is a proved, versioned projection. Every label has
  a meaning theorem, every collapse has a common-semantics theorem, and display
  labels never drive certification decisions.

## Residual trust

The framework can prove that a formal certification position entails its
displayed meaning only relative to the formal semantics, fragment classifier,
and route assumptions it names. It does not prove that those semantics match
the author's intent, that an external solver is sound, that the Rust
implementation corresponds to the Lean model before the refinement theorem is
closed, or that rustc, LLVM, the operating system, and platform boundaries
behave as modeled. Each surviving assumption remains an explicit component of
the formal frame and generated trust report rather than being hidden by the
engineer label.

The initial gap inventory is itself derived by repository tooling and can omit
a claim source or misclassify evidence until its source-population completeness
is established. Its fail-closed inventory checks reduce that risk but do not
turn corpus coverage or document scanning into a theorem about the language.
Human review remains responsible for deciding whether a discovered gap belongs
in this implementation, a bounded exclusion, or the completeness-review track.

## Out of Scope

- Proving completeness of Verus, Z3, Kani, CBMC, nlsat, rustc, LLVM, or any
  other external solver or compiler.
- Requiring every valid Thermite program to reach L3 or L4.
- Completing every language-feature gap discovered by the initial inventory
  inside #48.
- RFC-10-specific cross-product expansion that remains after extracting the
  general producer-refinement framework from issue #49.
- Implementing RFC-11 or RFC-12 language features.
- Treating corpus coverage, generated matrices, or adversarial review as a
  substitute for formal completeness claims.
