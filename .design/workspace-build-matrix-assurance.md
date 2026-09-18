# Feature: Complete assurance populations across workspaces and build matrices

audited-content-sha256: d26b5bd0780b38dfbb7cbe01a9feaf0f512755c7017256a0795bcb05d69f8516 (re-pinned 2026-09-18 after adding the live exact-floor consumer and its checked-normalization fixture for issue #63. prior: 0ea88e48b7cccb100d23aabb72eaea046e6158dbf305c4388b8c7beab7de0122)

## Summary

Project assurance currently binds one source file and one crate/target build
identity.  That is insufficient for a workspace in which packages, targets,
feature selections, platforms, or generated inputs expose different claim
populations.  This feature adds an independently declared, canonical matrix
plan and composes live project reports against it.  The plan is the denominator:
a missing report becomes an explicit non-claim and cannot be omitted from the
workspace headline.

The workspace result is a conjunction of already checked `ProjectLift`
judgments.  It does not collapse target-specific claim fibers or boundaries.
Each embedded project portrait retains its exact formal frontiers, artifact,
fiber, boundary, and evidence identities.  Serialized workspace JSON is
diagnostic; only live in-process project capabilities can construct a live
workspace report.

## Requirements

- REQ-1: A canonical workspace plan shall enumerate every intended matrix cell
  by package, target, sorted duplicate-free features, platform, sorted
  duplicate-free generated sources, exact source path, and source-ordered item
  inventory.  Matrix input order shall not affect its identity; duplicate
  coordinates shall fail closed.
- REQ-2: Composition shall match live project reports to plan coordinates
  rather than caller order.  Extra and duplicate reports shall fail; missing
  reports shall remain explicit unavailable matrix cells.
- REQ-3: The workspace population shall qualify every item identity by its
  exact matrix coordinate and preserve the concatenated canonical matrix order
  and each project's source order.  Its member partition shall equal that
  independent inventory exactly with no duplicate or omitted row.
- REQ-4: A present report shall validate structurally and match the exact
  revision, trust class, package, target, features, platform, generated-source
  list, source path, and intended item inventory of its plan cell.
- REQ-5: Target-, feature-, platform-, and generated-source-specific claim
  fibers and boundaries shall remain in their embedded project portraits.  No
  workspace algorithm shall compare or merge unlike project fibers by string
  convention.
- REQ-6: Missing matrix reports shall partition all their planned items as
  typed non-claims.  The workspace summary shall retain their exact identities,
  numerator, denominator, and unavailable coordinates.
- REQ-7: `WorkspaceLift` shall exist only for a nonempty, entirely accepted
  workspace in which every matrix has a live whole-project claim and every
  project frontier carries a checked `ProjectLift`.  It shall bind the exact
  workspace population and the source-ordered project-lift digests.
- REQ-8: Lean shall prove exact/nodup population membership, canonical matrix
  identity under permutation and duplicate presentation, conjunction
  invariance under project order and duplicate presentation, and sound
  non-vacuous workspace conjunction.
- REQ-9: Workspace comparison shall validate both reports and the exact base
  revision, detect added/removed or changed matrix cells and intended
  populations, distinguish unavailable/restored reports, and report formal
  project movements without treating artifact churn alone as matrix drift.
- REQ-10: The CLI shall construct a workspace report only by rechecking every
  planned source under its exact matrix coordinate.  It shall emit stable JSON,
  a bounded headline, and optional exact-base comparison; persisted JSON shall
  not become live floor authority.
- REQ-11: A generated Rust/Lean replay shall exercise package, target, feature,
  platform, generated-source, item, duplicate-coordinate, empty-population,
  and missing-project-lift hostility.  CI, routes, registry, axiom probing,
  typed claim closure, and documentation drift shall bind the feature.
- REQ-12: Workspace composition and comparison shall not read archival
  `Level`/`L0`–`L4` values.  It consumes only current typed project authority,
  exact matrix coordinates, dispositions, frontiers, and lift identities.

## Acceptance Criteria

- [x] AC-1: Reordering plan input or live project-report input yields identical
  canonical workspace output; duplicate matrix coordinates or duplicate live
  reports are rejected.
- [x] AC-2: Omitting a package or target report preserves its planned items in
  the denominator, yields explicit unavailable identities, and suppresses the
  whole-workspace claim.
- [x] AC-3: Changing package, target, feature selection, platform, generated
  sources, source path, or planned item inventory changes the plan/population
  identity and is visible to comparison.
- [x] AC-4: A project report misbound to any matrix coordinate or with a
  different item inventory is rejected before composition.
- [x] AC-5: A nonempty all-current two-package fixture acquires a workspace
  lift that binds every project lift and renders an exact whole-workspace
  conjunction headline.
- [x] AC-6: A non-claim, missing report, empty workspace, or missing project
  lift produces only a subset/no-items portrait with exact counts.
- [x] AC-7: Lean proves population exactness, matrix and conjunction
  permutation/duplicate invariance, non-vacuity, and workspace conjunction
  soundness without new axioms.
- [x] AC-8: `forge assurance --workspace-plan` checks each planned source and
  emits one deterministic workspace report; invalid flag combinations and
  parsed-source inventory drift fail explicitly.
- [x] AC-9: Exact-base comparison reports matrix drift separately from formal
  project movement and refuses invalid, schema-skewed, policy-skewed, or
  wrong-base inputs.
- [x] AC-10: The ten-row generated replay and focused Rust/CLI/Lean hostility
  suite cover every denominator coordinate and all whole/subset/rejected
  outcomes.
- [ ] AC-11: Typed claims are materialized once after stabilization; full local
  qualification, cold exact-head adversarial review, and required CI are green
  before merge.

## Architecture

`WorkspacePlanV1` is the source/build-graph declaration.  Its constructor
validates each coordinate, sorts matrix cells, rejects duplicates, and retains
each matrix's independent source-ordered item inventory.  The plan digest does
not depend on caller insertion order.

`build_live_report_for_coordinate` extends the existing project reporter with
an exact build coordinate while continuing to derive the artifact digest from
source, parsed program, toolchain, and that coordinate.  This prevents two
feature or platform builds of the same source text from sharing an artifact
identity by accident.

`build_live_workspace_report` accepts only non-serializable
`LiveAssuranceReport` references.  It indexes them by coordinate, walks the
canonical plan, and creates missing matrix portraits where no live report is
available.  `WorkspacePopulationV1` flattens matrix-qualified identities and
dispositions without erasing their fiber.  `WorkspaceLiftV1` then binds every
checked project lift when and only when the whole denominator is current and
nonempty.

The report embeds complete project reports because those are the stable formal
portraits containing target-specific frontiers, boundaries, evidence, TCB, and
artifact identity.  Workspace comparison uses the coordinate (without artifact
digest) as the matrix key, so ordinary artifact evolution is a project formal
movement while actual package/target/feature/platform/generated-source changes
are build-matrix changes.

## Trust boundary

The independent workspace plan, source parser, build graph supplying that plan,
Rust canonical serializer and SHA-256 implementation, live project-report
validator, named Lean theorem, and checked project-lift identities remain in
the TCB.  The Lean theorem proves the abstract conjunction law and canonical
set laws; Rust separately validates concrete strings, ordering, JSON bounds,
digests, and exact report binding.  Neither layer claims that a user-provided
plan is the repository's true build graph; protected CI must generate or audit
that plan from the intended workspace configuration before treating its output
as organizational evidence.

## Out of Scope

- Cross-fiber normalization not already licensed by checked assurance
  transports.
- Organization inheritance, exceptions, expiry, ownership, or cross-repository
  floor policy; issue #63 consumes this workspace layer.
- Treating engineer labels, percentages, badges, or archival levels as
  workspace authority.
- Making uploaded workspace JSON capable of satisfying a live floor.
