# Feature: Generalize checked cross-version and cross-procedure assurance transport

audited-content-sha256: a57469efcaefba6f63dc760cdd21e8108cb163ed51726a6c0a5b7e0d7e0bbc0f (re-pinned 2026-09-11 after binding the canonical transport matrix to a generated Lean replay theorem. prior: 865c462eb7b60ce85a3ca487f2fe3368c2d58c999b2d64d1d4aef9e7250a9437)

## Summary

Issue #58 defines the typed transport and simulation framework needed when a
semantic version, implementation-model version, or admitted assurance
procedure relates to a predecessor. A transport preserves exact authority
coordinates while explicitly translating programs, claims, evidence,
observations, contexts, and boundaries into a target fiber. Every declared
predecessor relation is checked as either transported or intentionally
incompatible, with deterministic Rust replay and CI comparison behavior.

This design is the assurance-extension layer after the Assurance V2
foundation. The product-facing `CurrentAssurance` and historical `Level`
migrations landed earlier through issues #56–57; this work consumes that
authority boundary without reopening it.

## Requirements

- REQ-1: The formal model shall define typed semantic-version, implementation-model-version, and procedure-simulation witnesses that translate the relevant program/member identities, claims, evidence, observations, residual contexts, and boundary qualifications without erasing exact source or target coordinates.
- REQ-2: A supported transport shall preserve the indexed certification meaning: fragment membership, claim truth, accepted-evidence validity, refutation soundness/completeness, context entailment, and boundary qualification must each be represented by a checked obligation rather than inferred from matching strings.
- REQ-3: Evidence fibers shall remain exact for semantic version, implementation-model version, fragment/classifier lineage, procedure, environment, tool/resource identity, residual context, boundary identity, and reconstruction identity. Transport may produce a target-fiber record only through an explicit witness and shall not normalize incompatible fibers by convention.
- REQ-4: Procedure simulation shall generalize the current equality-only `ProcedureRefines` relation. Equality shall remain the identity simulation; any non-identity simulation shall translate procedure evidence and observations and state the environment, tool, resource, and failure-preservation premises it relies on.
- REQ-5: Each version or admitted procedure shall declare a finite direct-predecessor comparison domain. The checker shall derive and classify every predecessor reachable in that declared domain as either a checked transport/simulation or a checked incompatibility witness; an unclassified predecessor shall prevent the new version or procedure from issuing current authority for that domain.
- REQ-6: A checked incompatibility witness shall be typed, replayable, and semantically distinct from missing classification. It shall explain why the relationship remains incomparable or policy-skewed while preserving each independently valid certificate and forbidding stronger/weaker inference across the relation.
- REQ-7: Transport composition and identity shall be proved for supported paths. Composing two checked witnesses shall yield a witness with the composed translations and premises, and the identity witness shall preserve all transported coordinates and evidence exactly.
- REQ-8: Rust replay shall implement the formal identity, composition, transport, and incompatibility decisions over canonical serialized coordinates. Deterministic CI comparison shall classify strengthened, weakened, incomparable/policy-skewed, changed-fiber, and missing-classification outcomes without consulting archival `Level`/`L0…L4` values.
- REQ-9: The replacement for archival `Lx` terminology shall be the typed coordinate tuple already emerging from `forge/src/manifest.rs` and the Assurance V2 model: `CertificationScope`, `RefutationChannel`, `ResidualTrust`, `CertificationBoundary`, `ClassificationCertificate`, `ClauseProcedure`/frame, clause identity and portfolio composition, semantic/model versions, fragment lineage, procedure/environment/tool/resource identity, residual context, claim/evidence identity, and composition witnesses. No single scalar or engineer display label shall serve as the transport authority.

## Acceptance Criteria

- [x] AC-1: (REQ-1) Lean contains distinct semantic-version, model-version, and procedure-simulation witness types with explicit translation fields for programs or members, claims, evidence, observations, contexts, and boundaries; a fixture using a mismatched model or procedure identity is rejected.
- [x] AC-2: (REQ-2) A supported transport fixture proves preservation of fragment membership, claim truth, accepted evidence, refutation behavior, residual-context entailment, and boundary qualification; deleting any one preservation premise makes the fixture fail.
- [x] AC-3: (REQ-3) The evidence-fiber fixture keeps different semantic/model versions, procedures, environments, tools/resources, contexts, boundaries, and reconstruction identities separate until a checked transport produces a target-fiber record; string-equal coordinates without a witness do not compare.
- [x] AC-4: (REQ-4) Procedure replay covers the equality identity case and at least one non-identity simulation; mutating the procedure version, environment, tool/resource premise, evidence translation, or observation translation causes replay failure.
- [x] AC-5: (REQ-5) A predecessor-domain matrix fixture passes when every reachable predecessor has exactly one transport or incompatibility classification, and fails when any reachable predecessor is omitted, duplicated, or classified by an unproved relation; an unclassified predecessor blocks current-authority issuance for the new domain.
- [x] AC-6: (REQ-6) An incompatibility fixture produces a deterministic incomparable/policy-skew result, preserves both input certificates as independently valid, and rejects any attempted stronger/weaker or aggregate comparison across the incompatible relation.
- [x] AC-7: (REQ-7) Lean proves transport identity and composition, including composed program/member and evidence translations and preservation of the composed observation/context/boundary obligations; Rust replay agrees with the generated or replayed Lean decisions.
- [x] AC-8: (REQ-8) Deterministic CI comparison emits stable results for strengthened, weakened, incomparable/policy-skewed, changed-fiber, and missing-classification cases across repeated runs and process boundaries.
- [x] AC-9: (REQ-9) The implementation inventory and negative tests show that no transport, comparison, authority, routing, aggregation, cache-authority, audit-authority, or CI decision reads archival `Level`/`L0…L4` as its transport coordinate; archival values remain inspect-only compatibility data.

## Architecture

### Formal transport layer

Extend the neutral indexed model in
`lean/Thermite/CertificationMetatheory.lean`. The existing `FrameRefines`
relation correctly keeps semantic and implementation-model versions exact and
allows only an explicit boundary weakening. Issue #58 adds typed semantic
version witnesses around that relation rather than weakening its equality
fields. A semantic witness owns program/member reindexing, claim translation,
fragment compatibility, evidence translation, observation preservation,
residual-context entailment, and boundary qualification preservation.

`lean/Thermite/ImplementationModel.lean` is the model-family seam. Its existing
`ModelRefinement`, `ModelExpansion`, and `ModelCompatibilityBreak` provide the
shape for implementation-model transport: source and target families retain
their own input and behavior types, and denotation correspondence is checked
on the named fragment. A model expansion is a supported predecessor relation;
an explicit compatibility break is an incomparable relation unless a separate
typed witness proves another path.

Procedure transport is a separate relation, not an alias for model transport.
The current `ProcedureRefines` equality in
`lean/Thermite/CertificationMetatheory.lean` becomes the identity case of a
typed simulation carrying procedure evidence and observation translations,
plus exact environment, tool-version, resource-budget, and terminal-failure
premises. A procedure that cannot supply those premises remains a separate
fiber and cannot enter a comparison merely because its display name matches.

Every witness has identity and composition operations. Composition composes
program/member and evidence translations and derives the combined preservation
obligations. The formal relation is directional: a source certificate can be
transported to a target only when the witness proves the target claim from the
source meaning. No reverse strengthening is inferred from a one-way witness.

### Exact fibers and comparison domain

`lean/Thermite/AssurancePolicyV2.lean` remains the finite-family policy
abstraction, not the source of transport truth. Its `EvidenceFiberKey` and
`ClaimFiberKeyV2` distinctions remain exact for semantic/model versions,
fragment lineage, procedure, environment, tool/resource identity, residual
context, and boundary. A transport produces an explicitly identified target
fiber and records the witness used; it does not merge source and target fibers
by normalization.

Each new semantic/model/procedure identity declares direct predecessors. The
checker computes the reachable predecessor closure within that declared domain
and requires exactly one checked classification for every reachable relation.
The matrix is finite because the domain is declared, not because the system
pretends that all possible versions or bounds are enumerable. Missing
classification is a gate failure and blocks current authority for the new
domain. A checked incompatibility is complete classification and yields
policy skew/incomparability without invalidating either input certificate.

The comparison result is about the relation between two formal authorities,
not a replacement certificate coordinate. It must therefore expose the exact
source and target coordinate tuples, the relation kind, the witness or
incompatibility receipt, and any changed boundary/context/fiber distinctions.
`AssurancePolicyV2` may compare transported target positions only after the
transport witness has established that they inhabit a compatible target fiber.

### Rust representation and replay

`forge/src/manifest.rs` is the Rust authority seam. The current typed fields
`CertificationScope`, `RefutationChannel`, `ResidualTrust`,
`CertificationBoundary`, `ClassificationCertificate`, `ClauseProcedure`,
clause addresses/portfolios, and the `CurrentAssurance`/legacy inspection
split are the coordinate vocabulary that replaces archival `Level` for this
work. The transport envelope shall carry these formal coordinates plus the
semantic/model/procedure identities and witness receipt; it shall not use
`Level` or an engineer display label as an ordering input.

The replay path shall be generated or pinned from the Lean witness vocabulary,
with exhaustive constructor coverage. It shall canonicalize typed bounds and
identities before hashing or comparing, retain source-ordered predecessor
matrix entries, and fail closed on contradictory transport facts, missing
predecessors, duplicate classifications, or altered witness receipts.

The deterministic CI comparison gate shall run the same matrix in repeated
processes and compare canonical bytes. It shall distinguish transport-derived
strengthening or weakening from incomparable/policy-skewed relations, changed
fibers, and missing classification. CI publication is evidence that the
comparison implementation agrees with the checked replay; it is not itself
certification authority.

### Authority and migration boundary

Issue #58 supplies the transport contract consumed by later policy migration,
workspace-population, and organization-floor work in issues #59, #60, and
#63. It does not remigrate decision consumers, redesign the report layers, or
remove legacy JSON parsing. Historical `L0…L4` values remain inspect-only
compatibility records. Current-authority consumers must accept the typed formal
coordinate/witness envelope and reject a legacy level as a substitute.

The design preserves Thermite's append-only assurance invariant: transport
adds a new checked relation or a checked incompatibility claim; it does not
rewrite or destroy the predecessor subject. Matrix and replay artifacts are
content-addressed to the exact source/target identities and witness bytes.

## Residual trust

The transport framework does not make the underlying semantic, model, solver,
compiler, platform, or checker assumptions disappear. Each transported
certificate retains its residual context, named boundary, procedure/tool/resource
premises, accepted evidence, and reconstruction identity. A transport witness
adds only the checked translation and preservation theorem it proves; it does
not claim whole-tool or whole-platform correctness outside the named fragment.
Rust replay and deterministic CI comparison remain implementation witnesses
whose authority depends on agreement with the Lean-defined relation. Missing,
contradictory, or incompatible facts fail closed as comparison outcomes rather
than being converted into stronger assurance.

## Open Questions

- Q-1: Scope is limited to typed transport/simulation, predecessor comparison,
  Rust replay, and deterministic CI; the already-landed `CurrentAssurance`,
  report UX, and final `Level` migration from issues #56–57 are not reopened.
- Q-2: Exact evidence fibers are preserved; transport creates a target-fiber record only through an explicit witness and never normalizes incompatible fibers by convention.
- Q-3: Procedure simulation generalizes equality with typed evidence/observation/environment/tool/resource preservation; unsupported procedures remain separate fibers.
- Q-4: The comparison domain consists of declared direct predecessors plus their reachable closure; every reachable predecessor needs exactly one checked transport or incompatibility classification.
- Q-5: Checked incompatibility is a complete policy-skew/incomparable result that preserves independent authority, while missing classification blocks new authority for the declared domain.

## Out of Scope

- Reopening the authority, routing, floor, aggregation, cache, audit, and UI
  migration from `Level`; that work was completed by issues #56–57.
- Defining a shared normalized fiber that erases semantic/model/procedure or context distinctions.
- Treating report prose, CI success, engineer labels, or scalar `L0…L4` values as transport evidence.
- Inventing a total order across incompatible or unsupported procedure/model relations.
- Implementing whole-Rust compiler semantics beyond the named Thermite-emitted fragments.
- Expanding the predecessor domain to every historical artifact without an explicit declared comparison domain.
