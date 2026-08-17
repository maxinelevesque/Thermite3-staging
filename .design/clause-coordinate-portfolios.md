# Feature: Clause-Coordinate Certification Portfolios

<!--
tier: 3-component
status: designed
governs: forge/src/manifest.rs, forge/src/check.rs, forge/src/engine.rs, forge/src/audit.rs, forge/src/result_arbiter.rs, thermite-syntax/src/ast.rs, thermite-syntax/src/parser.rs
-->

## Summary

Make heterogeneous per-clause certification evidence authoritative without
inventing one item-level route. Every expected contract clause receives a
typed, source-bound identity, classification, attempted formal position, and
terminal state; a validated `ClausePortfolio` preserves incomparable results
and permits an item-level coordinate only when every clause is discharged under
the same full route. This supplies the missing substrate between RFC-3
coordinates and the later proved-display and legacy-`Level` retirement work.

This design serves `telos/a-clause-is-checked`,
`telos/residual-trust-is-named`, and `telos/the-corpus-still-certifies`. It
implements the clause-level prerequisite named in
`.design/versioned-language-completeness.md` and consumes, rather than weakens,
the partial-order discipline established by
`.design/rfc3-certification-metatheory.md`.

## Requirements

- REQ-1: `forge/src/manifest.rs` shall define a closed, serializable
  `ClauseAddress` consisting of item identity, clause family, and zero-based
  source ordinal. The initial closed family contains `Ensures`; its canonical
  display shall be `item::ens#k`. A checked conversion taking the proof target,
  `thermite_syntax::ClauseSelector`, and expected certificate item shall accept
  only an indexed `ensures` selector for that item and reject unknown,
  unindexed, overflowing, out-of-range, or item-mismatched identities.
  `thermite-syntax/src/parser.rs` shall use checked ordinal conversion rather
  than truncating the lexer's `u128` value into `ClauseSelector.index`.
  Free-form `ObligationResult.name` text shall never establish clause identity
  or authority.
- REQ-2: Each migrated clause obligation shall atomically carry an additive
  `ClauseCertification` containing its `ClauseAddress`, a versioned full
  artifact fingerprint, a per-clause theorem/query fingerprint, the portfolio's
  expected clause count, a `ClassificationCertificate`, a closed
  `ClauseProcedure` plus versioned semantic frame, the attempted formal
  `CertificationPosition` when one coherently exists, a closed route-specific
  evidence payload, and a closed terminal state. The artifact fingerprint shall
  bind item identity and signature/types, effects and boundary inputs,
  `requires`, effective body/result grounding, the ordered ensures expressions
  and semantic tags, reachable semantic definitions or their meaning digest,
  and classifier/procedure/frame versions. The query fingerprint shall bind the
  exact grounded or lowered theorem and backend input for that clause. Route
  evidence shall bind BV shadow/query/countermodel or reconstruction evidence,
  EPR reconstruction/countermodel evidence, NLSAT solver input and result, or
  author-Lean proof identity, axiom/reconstruction evidence, and clause-local
  `BurnReceipt` as applicable. The terminal states shall distinguish
  discharged, refuted, undecided, and `NotAttempted`; the last carries a closed
  stop cause that is either an earlier terminal clause or a typed pre-clause
  item gate. Historical obligations that omit this optional block shall
  deserialize and reserialize unchanged.
- REQ-3: `ClausePortfolio` shall be a validated first-class view over the
  clause-certified obligations of one certificate. Construction shall require
  one item and artifact fingerprint, one expected count, an exact bijection
  over `Ensures(0..count)`, no duplicate or foreign addresses, coherent
  classification/position pairs, and agreement between each terminal state and
  the surrounding obligation status, engine, trust, verdict, reconstruction,
  fixed-width shadow, certificate-level compatibility burn, and route evidence.
  Discharged maps only to
  `ObligationStatus::Discharged`; refuted, undecided, and `NotAttempted` map to
  `ObligationStatus::Failed`. Every `NotAttempted` chain shall root in an actual
  earlier non-`NotAttempted` terminal or typed item-gate result. The validator
  shall also enforce the bidirectional certificate invariant: a finally
  accepted homogeneous portfolio has singular classification, certification,
  and engine attribution exactly equal to its mechanical derivation, while a
  heterogeneous, incomplete, or finally policy-rejected portfolio has all
  three singular authority fields absent. Any contradiction shall fail closed
  as a typed portfolio error rather than fall back to legacy fields.
- REQ-4: Portfolio aggregation shall preserve heterogeneity. A portfolio is
  discharge-complete only when every expected clause is authoritatively
  discharged; discharge completeness is distinct from final item acceptance.
  It may derive an item-level classification, position, and route attribution
  only when every discharged clause carries the same full classification,
  formal position, typed procedure/frame identity, and residual-trust
  attribution and the final item disposition is accepted. Otherwise it returns
  a heterogeneous or non-accepted diagnostic portfolio with its distinct
  coordinates intact and singular item authority absent; it shall not take a
  scalar minimum, choose a representative clause, or invent a meet for
  incomparable RFC-3 positions.
- REQ-5: The mixed producers in `forge/src/check.rs` shall construct total
  portfolios: `bv_fn_cert` for BV/EPR/NLSAT routing and the G1 Forge gate for
  NLSAT/author-Lean routing. They shall inventory the function's complete
  ensures list before executing engines and append one atomic result for every
  address in source order. Pre-clause covenant, meaning-tower, vacuity, body,
  and prerequisite failures shall produce all-`NotAttempted` portfolios rooted
  in a typed item-gate cause. When a clause terminal prevents later attempts,
  the remaining entries shall cite its typed address and terminal category; the
  implementation shall not run additional solvers merely to populate those
  entries. One sealed portfolio-aware assembler shall own every accepting and
  rejecting exit, preserve the completed prefix and remaining inventory, and
  prevent legacy rejection constructors from discarding portfolio evidence.
- REQ-6: `forge/src/audit.rs`, certificate validation, base-cache separation, and
  deterministic oracle projection shall preserve and validate clause
  portfolios. `forge/src/result_arbiter.rs` shall reject any present malformed
  portfolio before accepting live certificate structure. Mixed, incomplete,
  and policy-rejected portfolios shall be exposed as such in audit output and
  shall not populate singular authoritative item coordinates. The historical
  `Level` and its current minimum remain a compatibility rendering during this
  increment, but no new admission, routing, floor, or display decision may
  consume that scalar instead of the portfolio. Mixed results shall not be
  cached in this increment: the existing cache remains a Verus-base cache under
  its current schema and key, and both portfolio producers rebuild live after a
  base hit. Deserialized portfolio fields are non-authoritative data until a
  live run re-observes the route; no persisted portfolio authority issuer or
  synthetic mixed-result cache path shall be added.
- REQ-7: The G1 re-elaboration mutation policy shall consume the complete
  addressed set of mutation-applicable author-Lean clauses, never a first, last,
  or otherwise representative clause. Existing G1 NLSAT clauses prove the
  stronger body-independent `requires -> ensures` theorem with universally
  quantified `result`; they shall be recorded as mutation-inapplicable and shall
  neither kill a body mutant nor enter the mutation denominator. Mutation replay
  shall use a closed outcome distinct from certification `Verdict`:
  `Discharged`, `ProofRejected`, `Counterexample`, `Unavailable`, `Undecided`,
  or `Inapplicable`. `ProofRejected` requires a successfully invoked,
  correctly source/query-bound Lean checker that rejects the addressed author
  proof against the mutated theorem; exporter refusal, missing Lean, timeout,
  axiom-report failure, and other infrastructure or classification failures are
  unavailable or undecided, never inferred from diagnostic text. For each
  frozen body mutant, any `ProofRejected` or concrete `Counterexample` kills it,
  all applicable clauses `Discharged` makes it survive, and otherwise it is
  unscored. Scoring shall be invariant under clause order, and the item-level
  policy result shall cite every addressed replay outcome from which it was
  derived.

## Acceptance Criteria

- [ ] AC-1: (REQ-1) Tests convert `ensures#0` and `ensures#1` selectors to the
  exact `item::ens#0` and `item::ens#1` certificate addresses and reject an
  unindexed selector, every other keyword, an out-of-range ordinal, a selector
  for a different item, and source ordinals greater than `u32::MAX` without
  truncation or wraparound. Mutating only `ObligationResult.name` does not
  change the typed identity or portfolio result.
- [ ] AC-2: (REQ-2, REQ-3) Serialization tests show that all historical golden
  certificates omit `clause_certification` and round-trip byte-identically,
  while a migrated two-clause certificate round-trips its typed blocks. Deleting,
  duplicating, reordering by a forged ordinal, changing the expected count,
  changing one item or fingerprint, or splicing a clause block from another
  certificate causes typed portfolio validation to fail. Separate mutations of
  signature/types, effects/boundary, `requires`, body/result grounding, each
  ensures expression or tag, a reachable semantic definition, procedure/frame
  version, grounded theorem, and backend input change the appropriate artifact
  or query fingerprint and prevent stale evidence reuse.
- [ ] AC-3: (REQ-2, REQ-3) A mutation matrix exercises every terminal state and
  rejects contradictions: discharged without admitted classification or
  authoritative coordinates, refuted without a refutation witness, undecided
  presented as discharged, `NotAttempted` without a valid rooted cause, route
  attribution inconsistent with position/trust, reconstructed evidence on the
  wrong route, BV shadow evidence on an untagged route, and a proof/burn/query
  payload moved between clause addresses. Mutations giving a heterogeneous,
  incomplete, or policy-rejected certificate singular authority, or giving an
  accepted homogeneous certificate absent or unequal singular authority, fail
  before audit or arbitration.
- [ ] AC-4: (REQ-4) Exhaustive aggregation tests cover empty, incomplete,
  homogeneous-complete, heterogeneous-complete, and invalid portfolios. Exact
  homogeneous BV and exact homogeneous Lean fixtures derive item coordinates;
  BV/EPR, BV/NLSAT, and NLSAT/Lean fixtures remain heterogeneous. A fixture
  containing the intentionally incomparable solver and empirical-Lean RFC-3
  positions succeeds as a heterogeneous portfolio and cannot produce a scalar
  item position. The grammar-impossible empty portfolio is rejected. A
  homogeneous discharge-complete portfolio followed by item-policy rejection
  retains its diagnostic clauses but cannot produce singular item authority.
- [ ] AC-5: (REQ-5) A mixed BV/EPR/NLSAT function produces one source-ordered
  clause certification per ensures clause with the expected route,
  classification, position, and evidence. Checked proof for every clause yields
  a complete heterogeneous portfolio; a counterexample, timeout, unsupported
  clause, unavailable tool, or failed reconstruction records its terminal
  clause plus explicit `NotAttempted` entries for every later clause. A vacuous
  precondition or other pre-clause prerequisite failure produces a total
  all-`NotAttempted` portfolio rooted in the typed item gate. Every rejecting
  exit retains the already-attempted prefix rather than replacing it with a
  fresh one-obligation certificate.
- [ ] AC-6: (REQ-5) A mixed G1 NLSAT/author-Lean function produces a complete
  heterogeneous portfolio with each author proof, query fingerprint, and burn
  receipt bound only to its addressed clause. A fixture containing two
  author-Lean clauses independently mutates each proof and burn payload. Missing
  proof, Lean failure, NLSAT failure, or mutation policy rejection cannot
  relabel another clause and leaves every unattempted later clause explicitly
  represented. Covenant refusal/refutation and meaning-tower rejection produce
  total all-`NotAttempted` portfolios rooted in their typed item gates.
- [ ] AC-7: (REQ-3, REQ-5) A production non-test sibling compile probe cannot
  construct an authoritative portfolio, mark a clause authoritatively
  discharged, or derive homogeneous item coordinates from caller-authored
  public fields. Authority issuance remains confined to the actual
  engine-observation seams in `forge/src/check.rs`. Serialized portfolio data,
  a forged live stamp, or a valid-looking structural portfolio cannot enter the
  arbiter as authority, and no persisted-portfolio authority issuer exists.
- [ ] AC-8: (REQ-4, REQ-6) Audit JSON for a heterogeneous fixture exposes every
  clause's typed coordinates and omits singular authoritative item
  `certification`, `classification`, and engine attribution. Audit JSON for an
  exact homogeneous fixture contains both the complete portfolio and the
  mechanically derived singular coordinates. Policy-rejected homogeneous and
  heterogeneous fixtures retain diagnostic portfolios but omit singular
  authority. Structural tests show no mixed producer or audit path chooses
  formal authority with `Level::min`.
- [ ] AC-9: (REQ-6) Existing corpus certification results and historical JSON
  fixtures remain unchanged; focused Forge, cache, audit, route, requirement,
  path, document-drift, formatting, lint, and workspace test suites pass. Cache
  tests prove that only the Verus base is stored/loaded, a base hit still runs
  the mixed portfolio producer live, and engine selection or final mixed
  results never collide in the base cache. `CHECK_SCHEMA_VERSION` remains
  unchanged unless an independently demonstrated base-cache format change
  requires a bump. The RFC-3 inventory advances clause coordinates but leaves
  engineer-label projection and legacy-`Level` retirement explicitly
  incomplete.
- [ ] AC-10: (REQ-7) A hybrid mutation fixture includes two author-Lean clauses
  and one NLSAT clause over the same frozen mutant set. One mutant is killed
  only by Lean clause zero and another only by Lean clause one; the NLSAT clause
  is explicitly `Inapplicable` and cannot change the numerator or denominator.
  Permuting clause order leaves the score, unscored cases, and survivors
  unchanged. Separate fixtures distinguish a correctly bound Lean
  `ProofRejected` result from exporter refusal, missing Lean, timeout,
  axiom-report failure, and genuine undecided replay; only the first counts as a
  kill. Dropping either applicable clause's replay contribution, reverting to
  the singular `l3_clause` input, treating an inapplicable/undecided/unavailable
  result as a kill, or accepting a mutant that any addressed proof rejects fails
  the mutation-policy tests.

## Architecture

### Typed clause evidence

Add the clause-coordinate types beside `CertificationPosition` and
`ClassificationCertificate` in `forge/src/manifest.rs`. `ClauseFamily` is a
closed enum, initially `Ensures`; `ClauseAddress` owns the item name and `u32`
ordinal. It has a canonical formatter but no parser from the existing diagnostic
strings. The proof surface in `thermite-syntax/src/ast.rs` remains the syntax
owner of `ClauseSelector`; a checked conversion maps its zero-based indexed
`ensures` form plus its proof target into the expected certificate item. The
parser performs checked `u128`-to-`u32` conversion before constructing the
selector. The unrelated one-based loop and invariant namespaces in
`thermite-syntax/src/address.rs` are not widened.

`ClauseCertification` is an optional additive field on `ObligationResult`. It
owns the address and every fact needed to interpret that result, so a parallel
result-coordinate array cannot be reordered or substituted independently. Its
terminal state is closed and typed. `Discharged` carries producer authority;
`Refuted` carries a concrete checked witness identity; `Undecided` carries the
existing closed stage/outcome category and detail; `NotAttempted` carries a
`PortfolioStopCause`. That cause is either `ClauseTerminal`, naming an earlier
address and non-`NotAttempted` terminal, or `ItemGate`, naming a closed gate and
outcome such as covenant refusal, meaning-tower rejection, or vacuous
precondition. Human diagnostics remain descriptive only.

Each block also carries the expected ensures count, a versioned artifact
fingerprint, and a per-clause query fingerprint. The artifact fingerprint binds
the full proof context: signature/types, effects and boundary inputs,
precondition, effective body/result grounding, ordered clauses and semantic
tags, reachable-definition meaning, and route/frame versions. The query
fingerprint binds the exact grounded theorem, lowering, and backend input.
Repeating the count and artifact identity in every atomic block avoids a
separately aligned authority array. Portfolio validation requires them to agree,
then requires the addresses to form the exact non-empty zero-based range the
grammar guarantees. A certificate may retain unrelated non-clause obligations,
but once any obligation carries a clause block, every address in that declared
range must appear exactly once.

`ClauseProcedure` is a closed, versioned enum for the initial BV, EPR, NLSAT,
and author-Lean procedures. Its route-specific evidence enum carries the exact
query, countermodel, reconstruction, or proof data that justifies the terminal
state. Author-Lean evidence owns the clause's proof identity and `BurnReceipt`;
the legacy certificate-level burn is a compatibility copy only when exactly one
author-Lean clause exists and is never authority. Multiple Lean clauses retain
independent receipts only in their atomic clause evidence.

### Portfolio validation and authority

`ClausePortfolio` is constructed only through a validating accessor over a
certificate's obligations. It is not a permissive bag and has no public
unchecked constructor. The accessor returns historical/no-portfolio,
validated-portfolio, or a typed soundness error. Certificate and audit readers
must use that accessor; the presence of one malformed block prevents fallback
to item `level`, `certification`, or `classification`.

The existing opaque-authority pattern in `forge/src/result_arbiter.rs` applies
at clause granularity. `forge/src/check.rs` may issue a clause authority token
only after observing the relevant backend, checked reconstruction, concrete
countermodel, or terminal engine result. A private builder consumes those
tokens and the precomputed inventory to create every accepted or rejected final
certificate without losing the attempted prefix. `forge/src/result_arbiter.rs`
validates any present portfolio before accepting live certificate structure.
Public serde fields and matching strings are defense in depth, not authority;
there is no persisted-portfolio authority issuer in this increment.

### Aggregation without invented meets

Portfolio completion and homogeneity are separate operations. Completion is a
universal check over the exact inventory. Homogeneity additionally requires
exact equality of classification, `CertificationPosition`, procedure/frame
identity, engine attribution, and residual trust for every clause. Only that
case, combined with a finally accepted item disposition, may mechanically copy
the shared values into the certificate's singular item fields. Clause discharge
completeness alone does not certify an item whose later mutation, vacuity, or
other policy gate rejects it.

A complete portfolio containing different procedures or coordinates remains
heterogeneous even when the current `Level` rendering happens to be equal. A
portfolio containing incomparable positions is also valid; incomparability is
information, not an error. The implementation does not call
`CertificationPosition::partial_cmp_assurance` to select a representative and
does not introduce a lattice operation that
`.design/rfc3-certification-metatheory.md` has not proved.

During migration, `Certificate.level` continues to render the historical
minimum so existing consumers do not break. The sealed assembler compensates
for `Certificate::new` and legacy rejection constructors that currently insert
singular positions: heterogeneous, incomplete, and policy-rejected portfolio
certificates must clear those fields, while finally accepted homogeneous
certificates must set them to the exact derivation. A typed coordinate reader
distinguishes historical, accepted homogeneous-item, heterogeneous,
incomplete, and policy-rejected portfolio surfaces so downstream code cannot
mistake absence for an old certificate. Any mismatch is a soundness error, not
an audit-time omission. The later proved-display increment will project this
full surface; the later retirement increment will remove the scalar
compatibility path.

### Producer migration

Before any pre-clause gate in `forge/src/check.rs::bv_fn_cert`, build and
preclassify the complete ordered ensures inventory and its fingerprints. Each
BV, EPR, or NLSAT branch emits an
authoritative clause result instead of only updating `item_level` and the first
`item_attr`. On a terminal branch, the builder records that clause and fills the
remaining inventory with causally linked `NotAttempted` results before rendering
the existing rejected certificate shape. A precondition or prerequisite exit
fills the entire inventory from an item-gate cause. It does not invoke later
engines.

Apply the same preinventory and builder before the G1 Forge gate's covenant and
meaning-tower checks and through its NLSAT/author-Lean loop. Lean proof, burn,
query, and reconstruction evidence bind to the exact clause address, including
when more than one clause takes the Lean route. The later
contract-quality mutation gate remains item policy: if it rejects after all
clauses were discharged, the complete portfolio remains diagnostic evidence but
cannot turn the policy-rejected item into an accepted result or retain singular
item authority. Every legacy rejecting exit is routed through the sealed
assembler so it cannot discard a completed prefix.

Replace the singular `l3_clause` mutation input with an addressed collection
mechanically derived from the portfolio. Existing NLSAT clauses remain on their
stronger body-independent universal-result route and are recorded
`Inapplicable` for body mutation rather than being reinterpreted through a new
query. For every frozen body mutant, replay every author-Lean proof against the
addressed, body-grounded mutated theorem and return a mutation-specific outcome.
`ProofRejected` is available only when Lean was successfully invoked on the
correctly bound query and rejected that proof; generic certification `Unknown`
is not decoded into a kill. Fold the per-clause outcomes as follows: any
`ProofRejected` or concrete counterexample kills, all applicable clauses
discharged means survive, and no kill plus any unavailable or undecided result
means unscored. `Inapplicable` routes do not enter the numerator or denominator.
The fold is over addresses only for deterministic reporting; its result is
permutation-invariant. The item mutation score retains the addressed replay
vector so no clause contribution can disappear behind the aggregate.

The initial migration is deliberately limited to these two heterogeneous
producers. Existing homogeneous Verus, Kani, runtime-L1, and Lean producers keep
their current singular RFC-3 representation until a later mechanical migration;
they are not reinterpreted as missing portfolios.

### Audit, persistence, and governance

`forge/src/audit.rs` projects the validated portfolio verbatim into a function
row. Its singular fields come only from an accepted homogeneous derivation. The
project-level `Level` headline remains compatibility output for this increment;
the audit also exposes that a row is historical, accepted homogeneous,
heterogeneous, incomplete, or policy-rejected so display work can migrate
without string or absence heuristics.

The existing cache remains solely a Verus-base cache. Mixed BV and G1 results
are rebuilt live after any base hit, so engine selection, backend versions, and
portfolio authority never enter or collide in the current key domain. No mixed
result is stored, no persisted portfolio is admitted, and
`forge/src/cache.rs::CHECK_SCHEMA_VERSION` does not change unless implementation
work independently changes that base format. Deterministic portfolio fields
join the final certificate oracle, while existing fixtures without clause
blocks retain their exact serialized form.

The implementation updates the RFC-3 coordinate increment and requirement
evidence in `gates/language-completeness-inventory.toml` and
`.design/reqs/registry.toml`, but it does not mark the display or `Level`
retirement requirements complete.

## Resolved Questions

- Clause identity is a closed certificate address interoperating with the
  existing zero-based `ClauseSelector`; parsing rejects overflowing ordinals,
  and arbitrary obligation strings and the one-based block-address namespace do
  not become authority.
- `ClauseCertification` is attached directly to `ObligationResult`, preserving
  atomic result/evidence binding and additive historical serialization. Full
  artifact and exact per-query fingerprints bind every input to the discharged
  theorem, while closed route evidence holds clause-local Lean burn/proof data.
- Heterogeneous or incomparable coordinates remain a first-class
  `ClausePortfolio`; only exact full-route homogeneity combined with final item
  acceptance derives singular item coordinates.
- Every expected clause is represented. Early termination emits typed
  `NotAttempted` entries rooted in either a real earlier clause terminal or a
  typed pre-clause item gate without executing later engines, and both current
  heterogeneous producers migrate in this increment.
- Mixed final results are rebuilt live and are not cached in this increment;
  the existing cache remains scoped to the Verus base.
- G1 mutation scoring replays the complete addressed author-Lean clause set;
  body-independent NLSAT clauses are explicitly mutation-inapplicable, and no
  representative clause controls the item policy.

## Open Questions

## Residual trust

The Rust portfolio validator is trusted to enforce exact inventory,
cross-field coherence, and authority-token use until the Rust/Lean replay layer
is extended to clause portfolios. Individual backend assumptions remain those
named by each clause: Z3 and its lowering for unreconstructed BV/NLSAT routes,
the EPR checker and Lean kernel for reconstructed routes, and the author-proof
Lean premises for G1 clauses. An artifact fingerprint binds evidence to fresh
source but is not a cryptographic authenticity claim. Serialized portfolios are
non-authoritative data and must be reproduced by a live mixed-route run; this
increment deliberately offers no persisted-portfolio admission path. The
historical project `Level` headline also remains a lossy compatibility display
until its separately tracked retirement.

## Out of Scope

- Defining or proving engineer-label projection; AC-13 through AC-15 remain in
  `.design/versioned-language-completeness.md`.
- Retiring `Certificate.level` or the project-level legacy headline.
- Inventing joins, meets, or representative coordinates for heterogeneous
  portfolios.
- Migrating already homogeneous Verus, Kani, runtime-L1, or Lean producers to
  clause portfolios.
- Expanding certificate clause families beyond `Ensures` in this increment.
- Persisting or caching mixed-route final portfolios.
