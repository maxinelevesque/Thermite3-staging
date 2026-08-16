# Feature: RFC-3 Certification Metatheory

## Summary

Define the proof-theoretic foundation beneath RFC-3 as versioned, indexed
certification judgments ordered by semantic refinement. Full judgments retain
fragments, implementation models, residual assumptions, boundary contexts,
evidence, and refutation contracts; finite assurance positions and engineer
labels are sound, explicitly lossy abstractions rather than parallel sources of
truth. The same refinement structure gives Thermite a monotone path from named
external trust, through executable correspondence on expanding fragments, to
internal verified replacements such as a future Thermite implementation of the
`rustc` behavior Thermite consumes.

This design is the companion metatheory anticipated by
`.design/rfcs/0003-certification-surface.md`. It completes the formal basis for
`REQ-COMPLETE-RFC3-COORDINATES` in
`.design/versioned-language-completeness.md` without reopening that document's
already-shipped AC-1 through AC-12 work.

## Requirements

- REQ-1: Lean shall define a full `CertificationJudgment` indexed by a
  versioned semantic frame, residual assumption context, named language or
  procedure fragment, certification procedure, semantic claim, evidence type,
  and observable refutation contract. RFC-3 scope, refutation, trust, and
  boundary values shall be proved projections of this judgment rather than
  independent quality scores.
- REQ-2: Lean shall define semantic meaning and refinement for certification
  judgments. Refinement shall preserve claim meaning while explicitly
  translating fragment membership, evidence, residual assumptions, and
  observable failure guarantees. Fragment expansion, stage composition,
  executable-producer refinement, boundary closure, and TCB reduction shall be
  expressible as instances or consequences of this relation.
- REQ-3: The concrete order shall retain indexed distinctions, including
  `bounded(n)` versus `bounded(m)`, fragment version and compatibility lineage,
  procedure/model version, boundary assumption context, and residual
  assumption context. Antisymmetry shall be stated only after quotienting
  semantic equivalence; no Rust or Lean order may identify distinct bounds or
  contexts merely because they share an RFC-3 rendering.
- REQ-4: Lean shall separate the semantic claim domain, realizable
  certification judgments, and finite policy abstraction. The semantic domain
  may use predicate/set denotation to obtain natural lattice operations;
  realizable procedures may form only a sub-poset; no lattice instance shall be
  declared for realizable judgments unless all joins and meets are constructed
  and proved meaningful.
- REQ-5: The current seven-position, N5-shaped RFC-3 order shall be treated as a
  versioned candidate abstract policy domain. A projection/concretization
  theorem, preferably a Galois connection or a documented weaker sound
  abstraction, shall prove every floor, Pareto, aggregation, or display
  decision that consumes it. Its exact seven-point shape is not an axiom: a
  missing abstract point shall be added when required for soundness or required
  precision.
- REQ-6: The metatheory shall define a generic refinement framework with typed
  model families. Each family owns its input, behavior, denotation, fragments,
  and domain-specific correspondence law; the shared framework owns model
  versioning, expansion and compatibility breaks, executable observation,
  refinement composition, residual-assumption replacement, and certification
  integration. The first substantive instance shall model the version-pinned
  `rustc` behavior consumed by Thermite on an explicitly named fragment.
- REQ-7: A denotational implementation model alone may name and narrow a
  residual assumption but shall not remove the modeled implementation from the
  effective TCB. Removal for a certificate requires either a universal
  executable-to-model refinement theorem covering that artifact or replayable
  per-artifact evidence accepted by a checker whose soundness theorem entails
  the modeled behavior.
- REQ-8: Every claimed TCB reduction shall provide a semantic entailment from
  the new residual context to the old obligation and shall name the discharged
  assumption, replacement evidence, and remaining context. Completeness of the
  producer, checker, fragment, and workflow shall be stated separately from
  accepted-artifact soundness and shall affect coverage rather than silently
  reintroducing a discharged producer into that artifact's TCB.
- REQ-9: Rust certificate and audit types in `forge/src/manifest.rs` and
  `forge/src/audit.rs` shall refine the Lean formal vocabulary through a
  versioned serialization/replay boundary. Formal judgments remain
  authoritative; finite policy positions and engineer labels cannot drive
  admission, routing, certification, or TCB discharge without the proved
  abstraction appropriate to that decision.
- REQ-10: The work shall preserve the existing language-wide stage and outcome
  distinctions in `lean/Thermite/LanguageCompleteness.lean`. Classifier
  soundness/completeness, producer totality/refinement, proof soundness,
  refutation soundness/completeness, and stage completeness shall remain
  separately named theorem families.

## Acceptance Criteria

- [x] AC-1: (REQ-1, REQ-10) A new neutral Lean module defines the indexed
  certification judgment, semantic meaning, and separately typed theorem
  families; the axiom probe reports no assumptions beyond the repository's
  allowed set.
- [x] AC-2: (REQ-2) Lean proves refinement reflexivity and transitivity and
  supplies checked examples connecting existing `Expands`,
  `CompositionPremise`, and producer-refinement results from
  `lean/Thermite/LanguageCompleteness.lean` to the generalized refinement
  relation.
- [x] AC-3: (REQ-3) Lean proves monotonicity for at least two unequal bounded
  scopes and for boundary-context weakening; negative pins fail if unequal
  bounds or semantically distinct boundary contexts are identified without an
  equivalence proof.
- [x] AC-4: (REQ-3, REQ-4) A finite executable model enumerates representative
  full judgments and reports whether the realizable sub-poset has each requested
  join and meet. Missing operations are named rather than filled with an
  uninterpreted or policy-only point.
- [x] AC-5: (REQ-4, REQ-5) Lean defines the semantic denotation order and the
  candidate finite policy domain, then proves a sound abstraction theorem for
  every policy consumer enabled in Rust. A mutation that maps a concrete
  position to an abstract point incapable of justifying the same floor decision
  fails.
- [x] AC-6: (REQ-5) The N5 claim is either proved for the selected abstract
  domain, including its joins, meets, non-modularity, and quotient/collapse
  relation, or replaced by a differently shaped checked domain with the
  counterexample that forced the change recorded.
- [x] AC-7: (REQ-6) Lean defines the generic typed model-family/refinement
  vocabulary and a first versioned `rustc` model instance over a named
  Thermite-emitted Rust fragment. Expanding or narrowing that fragment follows
  the existing inclusion/compatibility-break discipline.
- [x] AC-8: (REQ-7, REQ-8) Fixtures distinguish model-only, universal-refinement,
  and checked-per-artifact discharge. Model-only evidence leaves the component
  residual; either checked discharge path removes exactly the named assumption
  while retaining and reporting its replacement premises.
- [x] AC-9: (REQ-8) Lean proves residual-context entailment for every fixture TCB
  reduction. A mutation that deletes an assumption without supplying a
  refinement or checker-soundness path fails, while missing producer/checker
  completeness changes coverage and does not invalidate an already accepted
  artifact's soundness theorem.
- [ ] AC-10: (REQ-9, REQ-10) Generated Rust/Lean replay covers every formal
  coordinate, model/frame version, residual context, boundary context,
  classification result, and policy abstraction used by certificates. Mutating
  any authoritative field or substituting an engineer label for formal data
  causes replay or a structural non-authority gate to fail.

## Architecture

### Full certification judgments

Add a neutral Lean module under `lean/Thermite/`, alongside
`lean/Thermite/LanguageCompleteness.lean`, rather than embedding the metatheory
in a feature-specific proof. Its central judgment has the conceptual shape:

```text
M ; Γ ; F ; P ⊢ evidence : claim ▷ observation
```

`M` is a versioned semantic frame and implementation-model selection; `Γ` is
the residual assumption context; `F` is the immutable fragment version; `P` is
the procedure with explicit environment/tool/resource premises; `claim` is the
semantic proposition over admitted programs; and `observation` is the
refutation/failure contract. Scope and boundary affect the proposition and its
hypotheses. Refutation describes procedure behavior when the proposition is
false. Residual trust records hypotheses needed to connect executable artifacts
to the theorem. These roles remain distinct in Lean types.

The initial theorem vocabulary shall reuse, not duplicate, `Program`,
`FragmentVersion`, `Fragment`, `Expands`, `CompatibilityBreak`, `Stage`,
`Outcome`, `CompleteAt`, `CompositionPremise`, and the producer/solver contracts
already defined in `lean/Thermite/LanguageCompleteness.lean`.

### Semantic refinement and denotation

`Refines strong weak` is witnessed by explicit program/member reindexing,
evidence translation, semantic preservation, residual-context entailment, and
observation preservation. Where two judgments entail one another, Lean may
define semantic equivalence and quotient it to obtain antisymmetry.

Each judgment also denotes the semantic worlds, executions, models, and
evidence states compatible with it. Reverse inclusion supplies a natural
semantic dominance order and a complete predicate/set lattice. Realizable
certification procedures embed into this domain but are not assumed closed
under semantic joins or meets. This separates a meaningful combined claim from
the existence of an implementation that can certify it.

### Finite policy abstraction

The seven positions described by
`.design/rfcs/0003-certification-surface.md` are the initial finite abstract
domain, not the full formal positions. Define a versioned abstraction and
concretization. Prove the strongest available sound relationship—ideally a
Galois connection—before enabling declared floors, Pareto-frontier reporting,
aggregation, or engineer-label projection against that domain.

Bounds, fragment/model versions, assumption contexts, and boundary identities
remain concrete even when policy intentionally forgets them. Every many-to-one
policy collapse belongs to a versioned policy and names the distinctions lost.
AC-13 through AC-15 in `.design/versioned-language-completeness.md` consume this
proved layer rather than formalizing display labels directly over the current
Rust enum.

### Typed implementation-model families

Define a generic model-family interface whose component-specific instances own
input and behavior types, denotation, fragments, and correspondence laws. The
shared framework provides version identities, monotone expansion,
compatibility-break witnesses, executable observation, refinement composition,
and residual-context updates.

The first `rustc` family models only behavior Thermite consumes on
Thermite-emitted Rust: version-pinned acceptance/rejection and the target or
execution behavior required by the certificate route. It does not claim a
whole-Rust compiler semantics. Subsequent fragment versions expand with proved
inclusion; semantic narrowing creates a new compatibility lineage. Verus, Z3,
serialization, platform, and proof-checker families may instantiate the same
framework without sharing an artificial common behavior type.

A future Thermite rewrite of `rustc` targets this denotation. Universal
refinement of the internal implementation, or checked evidence for each emitted
artifact, permits substitution while preserving the semantic claim and reducing
the external residual context.

### Effective TCB and discharge

Residual contexts are semantic assumptions, not string counts. `ContextRefines
new old` means every world satisfying the new context satisfies the old
obligation. A `TcbReduction` names the old context, new context, discharged
assumption, replacement evidence, entailment theorem, and remaining premises.

Two discharge modes are permitted:

1. Universal refinement covers every admitted input in the named model
   fragment.
2. Per-artifact evidence is accepted by a small checker whose soundness theorem
   yields the modeled relation for that artifact.

A model with no discharge path improves specification and audit precision but
does not reduce the effective TCB. Producer, checker, fragment, and workflow
completeness are reported separately. An untrusted producer can leave the TCB
for an accepted artifact even when it is incomplete, as already demonstrated by
checked proof production in the Stage 3 and Stage 4 reconstruction designs.

### Rust integration and staged path

`forge/src/manifest.rs` currently carries a partial Rust coordinate model and
`forge/src/audit.rs` copies it into per-function rows. Treat commit `bd49553b` as
an implementation probe, not the source of the metatheory. The next work is
ordered:

1. Formalize full judgments, semantic meaning, and refinement in Lean.
2. Probe bound and boundary ordering and enumerate missing joins/meets before
   declaring a lattice instance.
3. Define and verify the candidate policy abstraction.
4. Define typed implementation-model families and the first narrow `rustc`
   instance.
5. Add TCB-discharge fixtures and residual-context entailment.
6. Generate/replay the Rust certificate representation from the formal model.
7. Only then wire policy floors/frontiers, complete classification persistence,
   engineer labels, and final Lx removal.

`gates/language-completeness-inventory.toml` continues to report RFC-3
increments as partial until their complete acceptance evidence exists.

### Implemented foundation

`lean/Thermite/CertificationMetatheory.lean` completes AC-1 and AC-2 without
treating the earlier Rust coordinate probe as the metatheory. It defines the
fully indexed judgment, semantic meaning, distinct classifier/producer/proof/
refutation/stage theorem families, and an explicit semantic refinement witness.
Refinement is reflexive and transitive. Checked bridges embed the existing
fragment-expansion, stage-composition, and RFC-10 logical-producer results into
the general relation. `gates/lean-axiom-probe.sh` builds the module and probes
all five bridge/refinement theorems within the repository's allowed axiom set.

AC-3 adds unequal-bound monotonicity, an executable two-to-five bound witness,
and explicit end-to-end-to-platform boundary weakening. Negative pins reject
both the reversed five-to-two bound and an upgrade from platform-qualified to
end-to-end certification. The concrete semantic order, policy abstraction,
model families, TCB discharge, replay, and `Level` retirement remain open.

AC-4 adds `lean/Thermite/CertificationOrder.lean`, a four-position executable
realizable sub-poset whose 16 pairwise join/meet requests are computed from the
order. The solver-complete and Lean-empirical branches remain incomparable:
their meet is the bounded position and their join is reported absent. No
lattice instance or synthetic policy top is introduced, and negative pins
reject either branch being substituted as the missing join.

AC-5 adds the versioned finite abstraction in
`lean/Thermite/CertificationPolicy.lean`. The repository consumer audit found
that the only enabled `CertificationPosition` order decision is the
`dominates(self)` coherence path; other order calls are tests. The general
`floor_allows_sound` theorem nevertheless covers every pair in the candidate
domain: abstract floor acceptance entails the concrete representative order.
A mutation that collapses solver-complete onto Lean-empirical accepts an
unjustified floor and is rejected. No Pareto, aggregation, display consumer, or
Galois-connection claim is enabled by this checkpoint.

AC-6 resolves the candidate-shape question against N5. The selected domain is
a checked four-point fork: runtime below bounded, with incomparable
solver-complete and Lean-empirical branches above it. The concrete counterexample
is the solver/Lean pair: it has bounded as a meet but no realizable upper bound,
so no join exists. `lean/Thermite/CertificationShape.lean` proves both the
missing join and the four-not-five cardinality. The domain is therefore not
declared a lattice or non-modular N5, and no synthetic top is added.

AC-7 adds `lean/Thermite/ImplementationModel.lean` and its negative-space pin
module. The generic family keeps component-specific input and behavior types,
an exact model/version identity, a named `Fragment`, denotation, and executable
observation; typed refinements compose without forcing unrelated components
through a shared behavior type. The first instance is exactly `rustc 1.95.0`
on the named Thermite-emitted v1 Linux-target fragment and proves observation
correspondence there, not for whole Rust. Ordinary v1-to-v2 fragment growth is
an `Expands` witness; narrowing uses `CompatibilityBreak`. Mutations reject an
unchanged behavior payload relabeled as rustc 1.96.0 and a same-lineage v3 that
silently drops the v2-only witness. This checkpoint defines a model and names
trust only: it supplies no universal or per-artifact refinement and makes no
TCB-discharge claim; AC-8 and AC-9 supply those separate fixtures below.

AC-8 and AC-9 add `lean/Thermite/TcbDischarge.lean` and its negative-space
pins. `ModelOnlyTrust` retains the exact modeled component and cannot construct
a reduction. `DischargeEvidence` has only universal and checked-per-artifact
constructors. Both rustc fixtures produce a `TcbReduction` naming rustc 1.95.0,
replacement evidence, and the platform/checker premises that remain; each
record contains the checked `ContextRefines new old` entailment. The artifact
checker binds its evidence to the exact model version, program digest, and
target. Negative pins reject assumption deletion without entailment and reuse
of evidence for another artifact. Coverage is stored in a separate type and
cannot produce correspondence; an accepted artifact stays sound even when
producer, checker, and workflow completeness are false. These fixtures narrow
the modeled residual for their stated certificate only; they do not claim
whole-rustc, producer, checker, fragment, or workflow completeness.

## Resolved Questions

- The metatheory is a separate `.design/rfc3-certification-metatheory.md`
  companion; `.design/versioned-language-completeness.md` remains the broader
  issue-48 integration design.
- Full indexed certification judgments and semantic refinement are
  authoritative. The seven-point N5 structure is a candidate versioned policy
  abstraction whose shape must be justified rather than assumed.
- The implementation-model layer is a generic refinement framework with typed
  model families. Version-pinned `rustc` behavior consumed by Thermite is the
  first substantive instance.
- A model alone names trust. Effective-TCB removal requires universal refinement
  or sound checked per-artifact evidence, plus residual-context entailment;
  completeness remains separately stated coverage.

## Open Questions

- Q-1: Resolved — record this as the separate RFC-3 certification-metatheory
  companion while retaining versioned-language-completeness as the broader
  issue-48 integration design.
- Q-2: Resolved — full indexed judgments and semantic refinement are
  authoritative; N5 is a candidate finite policy abstraction justified by
  abstraction theorems, not the foundational formal lattice.
- Q-3: Resolved — use a generic refinement framework with typed model families,
  with the version-pinned rustc behavior consumed by Thermite as the first
  substantive instance.
- Q-4: Resolved — a model names trust; effective-TCB removal additionally
  requires universal refinement or sound checked per-artifact evidence and a
  residual-context entailment theorem. Completeness remains separate coverage.

## Residual trust

AC-1 through AC-9 establish the indexed judgment and refinement laws, checked
bound/boundary and realizable-order probes, the sound finite floor abstraction,
typed model families, narrow rustc correspondence, and the two permitted
certificate-specific discharge theorem shapes. The rustc model is still a
deliberately small Lean fixture over tagged representative programs; no theorem
connects the production rustc executable or a production artifact checker to
that denotation. The seven-position Rust order in `forge/src/manifest.rs`
remains an implementation probe, no generated Rust/Lean replay covers the
formal fields, and `Level` remains live. Consequently these fixtures prove how
a future reduction must be justified but do not reduce the effective TCB of any
current Thermite certificate.

## Out of Scope

- Implementing the Rust schema migration, production executable-refinement or
  artifact-checker bridge, policy consumers, or certificate TCB reduction in
  this slice.
- Claiming a complete semantics for Rust, `rustc`, Verus, Z3, Lean, operating
  systems, or hardware in the first model version.
- Declaring the realizable certification sub-poset a lattice before joins and
  meets are constructed and checked.
- Treating the seven-point abstraction or engineer labels as authority for
  admission, routing, certification, or TCB discharge.
- Removing `Level` or marking `REQ-COMPLETE-RFC3-COORDINATES` shipped before the
  formal replay and migration acceptance criteria pass.
- Automatically filing or changing GitHub issues, upstream branches, or release
  state.
