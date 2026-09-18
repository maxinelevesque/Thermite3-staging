# Feature: Organization-level formal assurance policies

audited-content-sha256: d5a388c91bbf37c221225c71689569501b52a3e47c2f8c5a476c942fafe1bcbd (issue #63 implementation plus identity-on-kind migration hardening from adversarial review)

## Summary

Thermite now has repository-local formal floors, checked policy migration, and
complete workspace/build-matrix composition. This feature composes those
surfaces into reusable organization policy without creating a new authority
model. An organization plan independently names every repository, exact
revision, and workspace-plan digest. A versioned policy package then declares
floors only at exact `(repository, workspace plan, claim fiber)` coordinates.

Evaluation accepts only protected exact-SHA live workspace capabilities. A
persisted report cannot be converted back into admission authority. Every
project common-claim frontier must dominate the floor and carry a checked
`ProjectLift`; the repository must also carry a checked `WorkspaceLift`.
Engineer labels, percentages, badges, report prose, evidence-frontier
membership, and representative scalar levels are structurally absent from the
policy schema.

Policy packages form one digest-bound version chain. A child may retain or
strengthen a comparable inherited floor, but cannot weaken it. Incomparable
floors are an explicit unresolved conflict. Exact-scope delegations authorize a
different owner to strengthen, except, or redelegate only that coordinate.
Exceptions expire against an explicit evaluation epoch and never rewrite the
fact that the formal floor is unsatisfied.

## Requirements

- REQ-1: An independent organization plan shall name a nonempty,
  duplicate-free canonical repository population. Each repository row shall
  bind its exact revision and workspace-plan digest before results are known.
- REQ-2: A policy floor shall contain only an exact repository/workspace/fiber
  key, a closed `AssuranceKindV2` minimum, and its owner. Presentation labels,
  percentages, badges, legacy Levels, and evidence frontiers shall be
  ineligible for admission by construction.
- REQ-3: Organization admission shall consume only in-process live workspace
  capabilities rebuilt at protected exact SHAs. Serialized project or
  workspace portraits remain diagnostic-only data.
- REQ-4: A floor is formally satisfied only when every exact build-matrix cell
  carries a checked `ProjectLift`, the repository carries a checked
  `WorkspaceLift`, and every live common-claim frontier in the named exact
  fiber dominates the required kind.
- REQ-5: Policy packages shall bind schema, family identity, strictly
  increasing version, owner, organization-plan digest, collapse-policy
  version, activation/expiry epoch, exact parent digest, declarations,
  delegations, exceptions, and a canonical package digest.
- REQ-6: Inheritance shall be a single exact parent chain. A child may retain
  or strengthen a comparable floor. A weakening, incomparable update, missing
  parent, mismatched family, non-increasing version, or forged digest shall
  fail closed.
- REQ-7: A non-root owner may act only under a prior exact-key delegation.
  Delegation does not expand to sibling repositories, workspace plans, or
  claim fibers, and an undelegated owner cannot redelegate.
- REQ-8: A scoped exception shall bind the exact floor key, delegated owner,
  explicit expiry, human reason, audit provenance, and source package digest.
  An active exception may permit the gate while recording
  `formal_satisfied=false`; an expired exception has no effect.
- REQ-9: Equal collapse-policy versions need no normalization. Version skew
  shall be rejected unless an exact forward `PolicyMigrationReceiptV1`
  validates as an identity-on-kind checked version rename from the package
  version to the live version. This is a total order isomorphism that preserves
  both formal-kind semantics and the exact fiber.
- REQ-10: Missing or duplicate repositories, plan/revision mismatch,
  unprotected trust, schema skew, mixed live policy versions, incomplete
  populations, missing lifts, absent fibers, orphan exceptions, and empty
  effective floors shall fail closed with deterministic reasons.
- REQ-11: Evaluation shall be deterministic under canonical plan/package
  ordering and explicit evaluation epoch. Its receipt shall bind the plan,
  package digests, migration receipts, exact floor results, exception
  provenance, and a canonical evaluation digest.
- REQ-12: Lean shall define exact floor keys, comparable inheritance,
  lift-gated frontier dominance, exception honesty, and checked normalization
  preservation. A generated Rust/Lean matrix shall cover success, exception,
  failure, and rejection hostility cases.
- REQ-13: The CLI shall support organization evaluation from an organization
  plan, exact repository-to-workspace-plan bindings, ordered policy packages,
  explicit epoch, and optional checked migration. It shall rebuild all
  workspace reports live rather than reading persisted authority.
- REQ-14: CI, route/path/registry gates, typed claim closure, axiom probing,
  test partitioning, and documentation drift shall fail closed when this
  authority surface changes without coordinated evidence.

## Acceptance Criteria

- [x] AC-1: A complete protected live workspace whose every exact project
  common frontier dominates the floor passes organization evaluation.
- [x] AC-2: A missing repository, different workspace plan or revision,
  unprotected trust class, missing `ProjectLift`, missing `WorkspaceLift`, empty
  population, absent exact fiber, or insufficient common frontier cannot
  satisfy a floor.
- [x] AC-3: Versioned packages validate their canonical digest, active window,
  exact parent edge, family identity, and strictly increasing version.
- [x] AC-4: Comparable strengthening is deterministic; weakening and
  incomparable inheritance are rejected rather than reduced to a scalar.
- [x] AC-5: Exact delegation permits the named owner to act at that key and no
  other; undelegated changes and redelegation fail closed.
- [x] AC-6: An active exception permits a failed gate only with exact scope,
  owner, expiry, reason, and audit provenance. The receipt still says the
  formal floor is unsatisfied, and expiry removes the waiver.
- [x] AC-7: Checked forward policy normalization preserves floor order;
  missing, reverse, lossy, forged, or endpoint-mismatched migration remains
  policy skew.
- [x] AC-8: Stable normalized output binds plan/package/migration identities,
  every effective floor result, applied exception provenance, explicit epoch,
  and the evaluation digest.
- [x] AC-9: The CLI can rebuild multiple exact repository workspace plans and
  returns verification failure when the resulting organization gate fails.
- [x] AC-10: Rust hostility tests, Lean theorems, and the twelve-row generated
  replay cover pass, excepted, failed, and rejected outcomes.
- [ ] AC-11: Typed claim materialization, full local qualification, exact-head
  cold adversarial review, required CI, and merge all succeed.

## Architecture

The policy chain is governance over existing formal authority, not a seventh
assurance family. Each live workspace capability originates in the same
admitted certificates and exact plan used by issue #60. The organization layer
asks the live capability one question: for this exact fiber and formal minimum,
does every checked project conjunction dominate it under a checked workspace
conjunction? It never reads display layers.

The organization plan is the independent cross-repository denominator. Its
repository identities, protected revisions, and workspace-plan digests prevent
an unavailable or inconvenient repository from disappearing. CLI path bindings
are execution locators only and are checked against the plan digest before any
verification begins.

Inheritance is deliberately a linear digest-bound chain for version one. This
makes authority and conflict resolution auditable: the root owner controls all
keys; a child owner controls only exact keys delegated earlier in the chain.
Within one key, equality is stable, comparable strengthening replaces the
prior minimum, weakening fails, and incomparability fails. There is no
last-writer-wins rule over incomparable claims.

Exceptions are policy decisions, not evidence. The evaluation receipt keeps
three booleans separate: `formal_satisfied`, `excepted`, and `gate_passed`.
Thus downstream audit can distinguish a theorem-backed pass from a temporary
governance waiver. Time is an explicit epoch input rather than ambient wall
clock, so replay is deterministic.

## Residual Trust

The organization evaluator trusts each repository's protected workflow to
check out the declared exact revision and supply the intended workspace-plan
path. Thermite then rebuilds certificates and workspace lifts in-process; the
path string itself is not policy authority. The explicit epoch is supplied by
the invoking governance workflow and is bound into the receipt. Audit
provenance strings identify an external record but do not prove its contents.

Rust validates the named Lean migration receipt through the closed six-family
order matrix and rejects any non-identity family relabeling until it can replay
Lean's population-admissibility law directly. As elsewhere, the reviewed
witness name and replay connect Rust to Lean but do not turn a string into a
proof. Organization policy does not
claim completeness for GitHub, source-control identity, organizational
authentication, or the external audit system.

## Out of Scope

- Turning report prose, engineer labels, percentages, badges, evidence
  frontiers, or historical Levels into authority.
- A distributed policy service, ambient identity provider, or remote
  attestation protocol.
- Wildcard repository/fiber delegation, policy DAGs, majority voting, or
  last-writer-wins conflict resolution. Version one uses an exact linear chain.
- Reverse or lossy policy migration, schema invention, or synthesizing a
  missing `ProjectLift`/`WorkspaceLift`.
- Treating an exception as formal satisfaction or allowing an unbounded,
  ownerless, reasonless, or provenance-free waiver.
