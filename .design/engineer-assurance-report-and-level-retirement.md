# Feature: Engineer assurance report and Level retirement

## Summary

Replace the historical scalar `Level` as both Thermite's human assurance
headline and its remaining policy input. The replacement is not another score.
It is one versioned assurance system with four progressively disclosed views:

1. the full formal portrait of realized item judgments and project composition;
2. a project common-claim engineer portrait;
3. a source-ordered crate and per-item drill-down;
4. a deterministic machine report that CI can publish and compare like a
   coverage service.

The formal portrait remains authoritative. Engineer labels are stored,
versioned, theorem-licensed projections that state what population is covered,
what happens when a claim is false, and which boundary qualifies the statement.
They never authorize certification, routing, floors, caching, or aggregation.
Historical `L0..L4` certificates remain readable only as inspect-only records
with a loud re-certification warning; they cannot be promoted into current
formal positions.

Project aggregation uses semantic refinement rather than a scalar minimum. For
each exact semantic/frame/boundary fiber, the report retains both the
non-dominated input positions (`evidence_frontier`) and the maximal common lower
bounds supported by every applicable item (`common_claim_frontier`). The latter
is the finite presentation of the intersection of the items' lower cones. A
genuine meet is a proved special case, never an assumption. Cross-fiber
composition requires an explicit checked rebase/refinement witness. A separate
Lean project-lift theorem is required before a common policy floor becomes an
authoritative project conjunction.

This design first resolves three existing, non-equivalent order models: the
seven-case Rust `AssuranceElement`, the four-point Lean `CertificationPolicy`,
and the four-position `CertificationOrder` missing-join probe. None is reused
silently. Lean defines one new `AssurancePolicyV2` carrier and proves the total
abstraction from admitted realized records. RFC-3's lattice wording is then
retained, qualified, or corrected according to the proved V2 order rather than
according to an older quotient. The semantic downset domain always admits
intersection; the parameterized realizable policy need not be assumed to be a lattice.

## Requirements

- REQ-1: Lean shall define a realized certification carrier that packages the
  full indexed `CertificationJudgment`, exact accepted evidence, proof
  soundness, applicable refutation soundness and completeness, interpreted
  claim population, semantic/frame/fragment identity, residual context, and
  boundary qualification. It shall separately define a closed indexed
  `AdmittedRealizedCertificationV2` sum whose constructors correspond exactly
  to current authority-producing families and carry their realized record and
  family witness. Coordinate enums, replay strings, or arbitrary open judgments
  alone shall not entail an engineer-facing semantic statement.
- REQ-2: Lean shall define a closed, versioned engineer-display vocabulary.
  Its claim labels shall distinguish no current certification, checked for one
  execution, checked through an exact bound, proved for all inputs with no
  complete witness promise, proved for all inputs with a complete concrete
  witness promise, and proved for all inputs with empirical falsification. Its
  coverage qualifier shall visibly distinguish end-to-end, a named boundary,
  and a frozen named platform.
- REQ-3: Every engineer label shall have a meaning theorem covering its exact
  quantified population, accepted-evidence proof soundness, counterfactual
  refutation contract, residual-trust upper bound, boundary qualification,
  fragment and semantic/model versions, procedure, environment, tool version,
  resource premises, and axioms. Complete-witness labels require both
  refutation soundness and completeness under their named operational premises.
- REQ-4: Lean shall define the closed, parameterized `AssurancePolicyV2`
  constructor signature used by
  projection, aggregation, floors, and comparison, including canonical typed
  bounds. It shall enumerate the exact admitted claim families and prove a
  total abstraction from every `AdmittedRealizedCertificationV2` constructor;
  producer issuance sites shall be exhaustive for that closed wrapper.
  `NoClaim` is not a runtime policy point. The V2 order, including whether
  `LeanComplete` is an upper bound, shall be proved by symbolic constructor-pair
  laws over arbitrary canonical parameters and generated or replayed exactly in Rust.
  Engineer projection shall be total by constructor family and parameters,
  preserve exact bounds and boundary identities, characterize every
  many-to-one fiber, and generate the distinctions each label forgets.
- REQ-5: The stored engineer display shall contain typed claim, typed coverage,
  and collapse-policy version. Issuance shall compute it from the formal record;
  validation shall recompute the unique licensed projection. The
  `authority_digest` shall bind the entire canonical formal authority record:
  subject/source/artifact and population identities, frame, claim, procedure,
  context, boundary, accepted evidence, and composition witnesses, excluding
  only display/presentation. A separate
  `presentation_digest = H(authority_digest, report_schema, policy_version,
  display)` shall bind rendering without making policy evolution an authority
  change. A structural gate shall
  keep display fields out of admission, routing, floor, aggregation,
  certification, cache-authority, and audit-authority decisions.
- REQ-6: Lean shall define an exact `EvidenceFiberKey` including semantic and
  implementation-model versions, procedure/environment/tool/resource identity,
  fragment/classification lineage, residual context, and boundary context. It
  shall separately define the V2 claim fiber used for aggregation and a proved
  abstraction that states exactly which evidence-fiber distinctions it may
  forget. Scope and refutation remain one indexed cell; boundary is an
  action/context, not an independently minimized axis.
- REQ-7: A source/build-derived `ProjectPopulation` shall enumerate every
  intended crate item under exact target, feature, platform, generated-source,
  and artifact identities and partition it totally into `Accepted`, `NonClaim`,
  or `LegacyUnversioned`. An accepted heterogeneous clause portfolio shall
  first pass `PortfolioLift`, proving that its complete source-ordered clause
  inventory and evidence certify one item conjunction judgment. Every lifted
  item shall denote an `ItemClaimSet` downset in each compatible V2 claim fiber:
  `down(abstract(x))` for a homogeneous item and the intersection of transported
  addressed-clause claim sets for a heterogeneous portfolio, justified by the
  item's connective-semantics theorem. No representative item position shall be
  invented. Only these lifted item claim sets may enter project aggregation.
  For a candidate V2 claim fiber, every accepted project-population member must
  contribute a transported item claim set; absence contributes the empty set,
  not omission. The formal common policy claims shall be
  `Common(X) = intersection(X)` over the complete population. The finite
  report shall serialize the maximal elements of this downset as
  `common_claim_frontier`. It shall separately serialize the maximal elements
  of the union of item claim sets as `evidence_frontier`, with provenance from
  each maximal policy point to its supporting item and clause evidence. The
  names and schemas shall prevent
  consumers from confusing the audit summary of observed strengths with a
  whole-project claim.
- REQ-8: Lean shall prove common-frontier soundness, coverage of every common
  lower bound, antichain membership, permutation and duplicate invariance,
  singleton and homogeneous laws, and conditional agreement with a genuine
  greatest lower bound. Because V2 is parameterized, Lean shall define a
  canonical finite `AntichainNF` representation and a terminating decidable
  symbolic intersection/normalization algorithm derived from constructor-pair
  lower-bound laws. For every finite input family, its denotation shall equal
  the mathematical downset intersection; output shall be complete, sorted,
  deduplicated, and independent of fold order without enumerating the V2 domain.
  Empty input shall have an explicit `NoItems` policy
  result and shall not silently become top. A report with any `NonClaim` or
  `LegacyUnversioned` population member shall not state a whole-project claim;
  it may report an explicitly subset-qualified accepted portrait with exact
  numerator, denominator, and excluded identities. An empty common frontier
  shall mean no shared formal claim under the active fiber/policy, not that an
  item failed.
- REQ-9: V1 shall keep different semantic versions, implementation-model
  versions, and procedure-simulation identities in separate evidence fibers;
  shipped `FrameRefines` equality shall not be described as cross-version
  normalization. Context or boundary rebasing within compatible versions shall
  require checked `FrameRefines` / `BoundaryRefines` witnesses. Any future
  cross-version or cross-procedure normalization requires new typed semantic,
  model, and procedure-simulation witnesses translating programs, claims,
  evidence, and observations. Raw comparison across unsupported fibers is
  forbidden.
- REQ-10: Lean shall define `PortfolioLift`, its induced heterogeneous
  `ItemClaimSet`, and a project judgment whose claim
  is the conjunction of the complete `ProjectPopulation`'s source-ordered item
  claims, whose evidence is the corresponding
  evidence vector, and whose `ProjectLift` theorem establishes conjunctive
  soundness after every required transport. A false-conjunct observation shall
  soundly refute the project. Refutation completeness shall be claimed only
  with a total finite scheduler and every applicable item completeness premise.
  Without `ProjectLift`, the common frontier is a reportable common policy floor
  but not a newly realized project procedure result.
- REQ-11: Rust shall expose `Certificate::current_assurance() ->
  Result<CurrentAssurance, NotCurrent>` with closed current variants for an
  accepted homogeneous realized position, an accepted and lifted clause
  portfolio, and a typed non-claim/disposition. Portfolio validation shall run
  before singular fallback, and live or freshly cache-admitted capability shall
  be required. A separate `inspect_legacy()` API may return
  `LegacyUnversioned`; historical data shall never inhabit the authority seam.
- REQ-12: All production decisions that currently read `Certificate.level` or
  compare `Level` shall migrate to the validated authority seam. This includes
  certificate certification, project aggregation, result arbitration, proof
  eligibility and composition, verified-build floors, lemma citation, repair
  classification, goal/fill completion, Kani/degrade control, oracle identity,
  and cache identity. `CheckLevel` and `BuildLevel` user mode selections are not
  `manifest::Level` and shall not be removed by this migration.
- REQ-13: Rust shall produce a stable, versioned `AssuranceReport` from admitted
  certificates and the current program. It shall contain source/artifact and
  policy identities, the project common-claim portrait, the per-fiber
  `evidence_frontier` and `common_claim_frontier`, all normalization witnesses,
  source-ordered item portraits, exact clause portfolios, formal frames,
  residual trust, boundaries, TCB, contract-quality evidence, and generated
  engineer-label disclosures. Every authoritative field shall trace to a
  validated formal record or checked composition theorem.
- REQ-14: The assurance report shall support progressive disclosure without
  changing its meaning:
  (a) a one-screen project engineer headline,
  (b) a crate summary grouped by common-claim fiber and exception class,
  (c) a per-item and per-clause drill-down, and
  (d) the full formal machine portrait. The visible headline shall include its
  boundary qualifier. `--explain` shall disclose exact coordinates, frame,
  procedure/tool/resource premises, residual context, axioms, normalization
  witnesses, collapse-policy version, forgotten distinctions, and all source
  positions in the projection fiber.
- REQ-15: CI shall be able to generate the report deterministically, upload the
  complete JSON as a retained artifact, render a standalone human/HTML summary,
  and publish a compact status-check summary and pull-request comparison. The
  comparison shall distinguish changed source population, stronger/weaker/
  incomparable formal movement, changed boundary or residual context, changed
  common-claim frontier, newly historical/unrealized items, and report-schema or
  policy-version skew. Base comparison shall use only a report bound to the
  exact base SHA and digest; absence yields `no comparison`, never a fallback
  to latest. It shall never summarize incomparability as regression or
  improvement without a declared formal floor or checked migration theorem.
- REQ-16: CI publication shall follow the Codecov interaction model without
  outsourcing authority: a compact check links to the full retained portrait;
  changed crates/items are reviewable; an optional formally declared project
  floor may gate; report generation, upload, or presentation alone does not
  certify. Forked or untrusted pull requests shall not receive write-capable
  publication credentials, and a stale report shall not survive artifact/source
  identity mismatch. PR report jobs shall have `contents: read` only; optional
  floors shall execute over live admitted authority before serialization.
  PR artifacts and summaries shall be explicitly labeled untrusted diagnostics
  because PR-controlled checker code could forge their contents. They shall not
  enter a privileged durable publisher. Durable authoritative publication shall
  run only after protected merge/push, generate and validate the report anew
  from that protected exact SHA, and never consume PR-produced JSON or HTML as
  assurance. Actions shall be commit-pinned; generated HTML shall escape
  source-controlled strings and carry a restrictive CSP.
- REQ-17: A schema envelope shall ship before historical classification.
  Certificates predating it shall deserialize as
  `LegacyUnversioned { legacy_level }` for inspection only; no schema version
  shall be invented. Every
  human and machine rendering shall prominently state that the certificate is
  not valid for current assurance decisions, name missing formal evidence, and
  require re-certification. L3 shall additionally name its ambiguity between
  solver-incomplete and Lean-empirical semantics; L2 shall name its missing
  bound. No historical level, including L0/L1/L4, may synthesize a modern frame,
  boundary, position, or engineer label.
- REQ-18: Current authoritative certificate and report schemas shall not emit a
  compatibility `level`. Human legacy inspection may display the stored level
  beside the loud warning. If machine compatibility is unavoidable, it shall
  be a separate opt-in legacy-inspection export with no cache, audit-admission,
  floor, build, or certification input path. Cache/check schema versions shall
  bump when authority identity changes.
- REQ-19: RFC-3, the audit-manifest design, CLI documentation, golden fixtures,
  and `thermite-verified` aggregation proofs shall be revised to state the
  exact relationship among `AssurancePolicyV2`, `AssuranceElement`,
  `CertificationPolicy`, and `CertificationOrder`. The old scalar-min theorem may remain
  only as a theorem about the deprecated compatibility rendering until it is
  deleted; it shall not justify a project assurance claim.
- REQ-20: Migration shall be staged so every intermediate release has one
  source of authority. The schema envelope ships first. In one atomic authority
  migration, `level` becomes private/purely derived and every production
  decision read moves to `current_assurance`; constructors and presentation
  reads are inventoried separately. A report-only stage may display a derived
  deprecated Level, but it is never a decision input. Removal shall occur only
  after persisted identities, displays, fixtures, and documentation have moved.

## Acceptance Criteria

- [ ] AC-1: (REQ-1, REQ-3) Lean fixtures construct admitted realized runtime, bounded,
  solver-incomplete, solver-complete, Lean-empirical, and Lean-complete
  certifications and prove the corresponding engineer meanings. Replacing the
  realized carrier with coordinates, strings, `True` fragment/boundary
  predicates, evidence-free replay, an arbitrary open judgment, or an unlisted
  producer family makes the meaning theorem or issuance-exhaustiveness check fail.
- [ ] AC-2: (REQ-2, REQ-4) Generated projection rows cover every
  `AssurancePolicyV2` constructor family exactly once, and a totality theorem
  covers arbitrary typed bound, version, identity, resource, and context
  parameters. Mutants deleting a family, changing a bound/boundary, mapping
  incomplete or empirical refutation to a concrete witness, injecting no-claim
  into runtime, or adding an undeclared many-to-one mapping fail a named theorem
  or replay check. The executable matrix enumerates constructor pairs rather
  than pretending that arbitrary parameter values form a finite carrier.
- [ ] AC-3: (REQ-4, REQ-5) Solver-complete and Lean-complete fixtures project to
  the same concrete-witness engineer claim while retaining unequal formal
  positions and a generated disclosure of residual trust, procedure, tool,
  axiom, and reconstruction distinctions. Mutating stored formal position fails
  the authority digest; mutating display, coverage, schema, or policy version
  fails only the separate presentation digest and cannot change an authority
  decision.
- [ ] AC-4: (REQ-6, REQ-7, REQ-8, REQ-10) Exhaustive constructor-pair laws and
  finite-report fixtures compute both
  frontiers for singleton, homogeneous, comparable, incomparable, and
  multiple-maximal-lower-bound inputs. They prove order/duplicate invariance and
  conditional-meet agreement. A representative-item selector and independent
  per-axis minimum each fail on recorded counterexamples. A heterogeneous
  portfolio cannot enter the item set without a complete `PortfolioLift`.
  A mixed portfolio with incomparable clause positions yields the proved
  intersection downset and never a representative scalar/item position.
  Property tests compare `AntichainNF` folds across permutations/bracketings;
  a mutant that enumerates a bounded sample of parameter values, drops a
  maximal lower bound, fails to normalize duplicates, or loops fails the
  denotation/termination theorem or generated replay.
- [ ] AC-5: (REQ-4, REQ-6, REQ-19) Lean enumerates the
  `AssurancePolicyV2` constructor signature, proves its
  exact order and abstraction from all admitted realized families, and generates
  or replays the same matrix in Rust. It records how the older seven-case Rust
  and four-point Lean models map—or fail to map—to V2. RFC-3 lattice wording
  follows the V2 theorem; the old missing-join probe remains documented as an
  older model unless V2 proves the same fact.
- [ ] AC-6: (REQ-9) Different semantic/model versions and unsupported procedure
  simulations always remain separate. Compatible same-version boundary/context
  fixtures remain separate by default and aggregate only after exact checked
  rebasing. No shipped equality-based `FrameRefines` test is described as
  cross-version transport.
- [ ] AC-7: (REQ-7, REQ-10) A mixed-route fixture proves `PortfolioLift` from
  every expected addressed clause into an item conjunction; dropping a clause
  makes the lift impossible. A multi-item fixture then proves `ProjectLift` for
  the complete `ProjectPopulation` and transports every item evidence. One
  observed false conjunct
  refutes the project soundly. Completeness cannot be constructed after deleting
  the scheduler, one population member, or one required per-item completeness
  premise. Any non-claim/legacy member suppresses the whole-project headline and
  produces only an exact subset-qualified portrait.
- [ ] AC-8: (REQ-11, REQ-12, REQ-20) A source inventory classifies every
  `Level` occurrence as constructor, presentation, or authority decision; the
  decision set becomes empty in one atomic migration where the field becomes
  private and derived. Hostile tests edit only
  Level, remove/splice a portfolio, deserialize a current-looking historical
  row, and substitute a cache envelope; every current decision either rejects
  or remains identical through `current_assurance`. `inspect_legacy()` can
  render but cannot satisfy the current API type.
- [ ] AC-9: (REQ-7, REQ-13) A mixed-route crate produces a deterministic report
  whose population denominator equals the source/build inventory and whose
  formal portrait contains every function, every expected clause coordinate,
  both project frontiers, exact fiber keys, transports, engineer display and
  disclosure, TCB, boundary, and residual-trust fact. Dropping any item or
  clause, changing source order/address, or changing current artifact identity
  fails report construction or validation.
- [ ] AC-10: (REQ-14) Snapshot tests cover the project headline, crate summary,
  per-item drill-down, `--explain`, and full formal JSON for homogeneous,
  heterogeneous, incomplete, policy-rejected, historical, multi-fiber, and
  multiple-common-frontier, and partial-population projects. Every headline
  prints the covered numerator/denominator; no primary human surface leads with
  `Lx`.
- [ ] AC-11: (REQ-15) Two runs over identical admitted inputs produce
  byte-identical normalized JSON and rendered output. A pull-request comparison
  fixture reports strengthened, weakened, incomparable, boundary-changed,
  residual-changed, population-added/removed, historical, and policy-skew cases
  separately and links every summary row to formal evidence. A missing exact
  base-SHA report yields `no comparison`; cross-policy/schema comparison yields
  skew unless a checked migration theorem is supplied.
- [ ] AC-12: (REQ-15, REQ-16) An untrusted PR job with only `contents: read`
  uploads the JSON and standalone
  report even when an optional formal floor fails, publishes a bounded summary
  rather than flooding logs/comments, and exposes a stable check name. A
  tampered/stale artifact, untrusted-fork publication attempt, missing report,
  HTML injection fixture, oversized/deep/unknown JSON, unpinned action, forged
  PR assurance content, and
  presentation-only success cannot acquire certification authority. Durable
  publication runs only after protected merge/push and regenerates the exact-SHA
  report; it neither consumes nor republishes PR-produced assurance JSON/HTML.
- [ ] AC-13: (REQ-16) An opt-in project policy can declare a formal floor by
  exact fiber or an explicitly normalized project fiber. The check gates only
  through proved dominance/common-frontier semantics; engineer labels and
  evidence-frontier membership cannot satisfy it.
- [ ] AC-14: (REQ-17) Unversioned historical L0-L4 fixtures remain parseable as
  `LegacyUnversioned` and render the
  loud re-certification warning. L2 names the absent bound and L3 names its
  semantic ambiguity. Check, build, citation, cache authority, aggregation, and
  floors all reject the records as current assurance.
- [ ] AC-15: (REQ-17, REQ-18, REQ-20) The schema-envelope release distinguishes
  current envelopes from `LegacyUnversioned` input without field-presence
  guessing. Current output omits Level. An opt-in legacy-inspection export may
  render it but has no current deserializer/admission route. Check/cache identity
  bumps; frozen old JSON remains inspectable but never audit-admitted.
- [ ] AC-16: (REQ-12, REQ-18, REQ-19) `manifest::Level` has no production
  authority consumer and is removed or isolated in a compatibility module.
  Project, verified-build, repair, review, goal/fill, lemma, Kani, degrade,
  audit, metrics, cache, and CLI tests assert the new authority/report paths.
- [ ] AC-17: (REQ-13 through REQ-16) A checked example report is published from
  CI for a representative mixed crate. Reviewers can move from status headline
  to common project claims, changed-item list, exact per-item/clause evidence,
  and the complete machine portrait without rerunning certification locally.

## Architecture

### Realized semantics before labels

`lean/Thermite/CertificationMetatheory.lean` currently defines the right indexed
judgment, while `lean/Thermite/CertificationOrder.lean` probes the realizable
partial order. The finite replay projection is intentionally too weak to prove
a sentence such as “proven for all inputs”: its reconstructed fragment and
boundary predicates are nominal and it carries no accepted evidence. Add a
open `RealizedCertification` carrier around an actual judgment, accepted
evidence, the judgment's `ProofSoundness`, applicable refutation theorems, and
an interpreted population. This open semantic carrier is not itself projectable.
`AdmittedRealizedCertificationV2` is a separate closed indexed sum over the six
currently admitted producer families; each constructor packages an open record
plus the family-specific witness. Every authority-producing issuance site must
construct one of these variants, and a source/replay inventory rejects an
unclassified producer.

The population interpretation is explicit because `Program -> Prop` alone does
not distinguish one execution, bounded executions, and all admitted inputs.
`ClaimPopulation` relates the scope cell to the actual language semantics. Its
bounded constructor carries the exact bound; its per-execution constructor
carries the observed execution identity. Boundary interpretation similarly
relates a named boundary to the assumptions used by the claim rather than
replacing it with `True`.

The engineer vocabulary is behavior-oriented:

```text
NotCertified
CheckedThisExecution
CheckedThroughBound(n)
ProvedAllInputsMayNotProduceWitness
ProvedAllInputsWithConcreteWitness
ProvedAllInputsWithEmpiricalFalsification
```

Each claim is paired with visible coverage:

```text
EndToEnd
ToBoundary(via)
ToPlatform(platform)
```

Example renderings are “Checked through bound 8; an in-bound failure produces a
trace” and “Proven for all inputs; a false claim is guaranteed to produce a
concrete witness — assuming boundary `ffi::clock`.” Exact trust and procedure
facts remain in the formal portrait and disclosure. In particular, the primary
label does not say “kernel checked” merely to recreate a quality ladder; the
generated explanation says which route was kernel checked and what it removed
from residual trust.

Before projection, Lean defines `AssurancePolicyV2`: the sole closed
parameterized claim signature for aggregation, floors, and comparison. It has parameterized
constructors for runtime and bounded claims and distinct admitted families for
solver-incomplete, solver-complete, Lean-empirical, and Lean-complete claims;
`NoClaim` remains outside the carrier. The definition does not inherit either
the current seven-case Rust comparisons or the older four-point Lean quotient.
A total abstraction theorem maps each constructor of
`AdmittedRealizedCertificationV2` to V2, and exact symbolic order laws for each
constructor pair make the treatment of `LeanComplete` explicit. The signature
has finitely many constructor pairs, but its bound, version, identity, resource,
and context parameters are not falsely enumerated. Generated Rust decision
rules and property replay cover those symbolic laws. Typed canonical bounds
replace string bounds before they participate in equality, hashing, or ordering.

The engineer projection defines `ProjectsUnder` and `EngineerMeaning`. Its
constructor-family totality theorem covers arbitrary parameters, and its exact
fiber theorem characterizes all equal-label pairs. A collapse proves refinement
to one common engineer judgment, never equality of source formal judgments.
Certificate issuance stores `EngineerDisplayV1 { policy_version, claim,
coverage }`; validation recomputes it from realized formal authority. Formal
authority and presentation have separate digests so a display-policy change is
visible and tamper-evident without changing what was certified.

### Project aggregation as common semantics

Aggregation begins with a `ProjectPopulation` derived from the exact source and
build graph: crate, target, features, platform, generated sources, and artifact
identity. Every intended item appears exactly once as accepted, non-claim, or
legacy-unversioned. A whole-project statement is available only when every
population member is accepted and lifted. Otherwise the UI may summarize the
accepted subset, but must print its numerator/denominator and excluded item
identities and must not call it the project guarantee.

A homogeneous accepted item already realizes one item judgment. A heterogeneous
clause portfolio does not. `PortfolioLift` consumes the complete source-ordered
clause inventory, each addressed accepted clause judgment/evidence, and the
item's connective semantics to construct an item conjunction judgment. Missing,
duplicate, unattempted, refuted, or policy-rejected clauses prevent the lift.
This is the only bridge from clause portfolios into project aggregation.

The aggregation input is uniformly a claim set, not a representative item
position. In a compatible V2 claim fiber `f`, a homogeneous item's
`ItemClaimSet_f` is `down(abstract(realized))`. A heterogeneous item's set is the
intersection of the downsets of every addressed clause transported into `f`;
its maximal antichain may contain zero, one, or several points. Its full
`PortfolioLift` conjunction remains in the formal portrait even when this
summary intersection is empty. Clauses that cannot be transported into one
claim fiber remain as separate fiber portraits; the implementation never picks
one clause or synthesizes a singular certification/classification/engine field.

For each V2 claim fiber `f`, let `P_f` be the semantic-refinement poset and let
`X_f` contain one source-ordered entry for every accepted project-population
member. If an item's full conjunction cannot be transported into `f`, its entry
is the empty downset; the item is never silently omitted. The project common
claims are:

```text
Common(X_f) = intersection (claims in X_f) claims
```

An intersection of downsets is canonical even when the realizable sub-poset has
no meet, but V2 cannot be enumerated. Lean therefore represents every reportable
downset by a canonical finite maximal antichain `AntichainNF`. A symbolic
`intersectNF` uses the proved constructor-pair lower-bound frontier laws, then
sorts, removes dominated points, and deduplicates. Termination and decidability
are proved from the finite input lists and structural parameter comparisons;
denotational soundness and completeness prove
`denote(intersectNF a b) = denote(a) intersection denote(b)`. Folding this
operator yields byte-stable output independent of order and bracketing. The
report stores that normal form rather than searching the policy domain. If a
unique greatest lower bound exists, Lean proves that the frontier is the
singleton containing it. Future policy domains may have several incomparable
maximal common lower bounds; all listed guarantees then hold, but no one
supported realizable point summarizes them.

`evidence_frontier_f = Max(union X_f)` answers a different audit question:
which non-dominated lifted item strengths occurred? Each maximal policy point
carries the identities of every supporting item and originating clause evidence.
The order here is the V2 policy order over points, never an unstated order over
downsets. This frontier does not describe what the entire project guarantees.
The schema and UI always spell out both names.

Every realized record retains an `EvidenceFiberKey` containing semantic/model,
procedure, environment, tool/resource, fragment lineage, residual context, and
boundary identities. `AssurancePolicyV2` defines a coarser claim fiber only
through an explicit theorem saying which evidence distinctions may be forgotten
without changing the claim. Cross-fiber values are not fed into an order. V1
keeps different semantic/model versions and unsupported procedure simulations
separate; the shipped equality-based `FrameRefines` cannot normalize them.
Within compatible versions, checked `Rebase` witnesses may transport boundary
or context through `FrameRefines`, `BoundaryRefines`, context entailment,
fragment compatibility, and claim/evidence translation. Boundary closure
accumulates reached named contracts before transport. A future cross-version
feature must introduce typed semantic/model/procedure simulations; it cannot
weaken the V1 key by convention.

Finally, common lower bounds summarize each lifted item separately; they do not
replace its conjunction evidence. `ProjectLift`
constructs a source-ordered evidence vector for every member of the complete
`ProjectPopulation` and certifies the conjunction of item claims. The
authoritative project claim cites `PortfolioLift` where needed, this theorem,
and every transport. The report may show a common policy floor before this lift
exists, but labels it as an accepted-subset report floor with exact coverage,
not a realized whole-project certificate.

### One Rust authority seam

Add a borrowed, fail-closed `CurrentAssurance` view in
`forge/src/manifest.rs`:

```text
Accepted { claim: Homogeneous(realized position, classification, item claim set)
                | LiftedClausePortfolio(validated portfolio, lift identity,
                                        item claim sets by fiber) }
NonClaim { typed disposition and progress }
```

`Certificate::current_assurance()` checks live/cache admission, validates any
present portfolio first, requires its lift identity, and only then validates a
homogeneous RFC-3 pair. `inspect_legacy()` is a separate type and call path; a
`LegacyUnversioned` value cannot satisfy a current API parameter. A separate
presentation seam derives the engineer display from accepted authority; callers
never infer a portfolio by noticing absent singular fields.

The authority migration is atomic: first ship the schema envelope and new
types, then make `level` private and purely derived in the same change that
migrates every decision consumer. A checked inventory classifies occurrences as
constructors, presentation reads, or decision reads; the last set must be empty
at the migration commit. Subsequent stages may migrate persistence and displays
without creating a second authority source.

### Assurance report: four disclosure layers

The versioned `AssuranceReport` is the trust deliverable that supersedes the
scalar portions of `AuditManifest` in `forge/src/audit.rs` while retaining its
TCB, boundary, fragment, semantic-fork, and residual-trust content. Its human
views replace the Level-led portions of `forge/src/cli.rs`.

**Layer 1 — project headline.** A short engineer statement names the common
claim, visible boundary, and exact covered/total item count. Multiple
common-frontier points are rendered as
“these incomparable common guarantees all hold; no single supported formal
position summarizes them.” If any intended item is non-current, the headline
says “accepted subset portrait, not a whole-project claim,” names the exception
classes, and links to their identities. No shared claim is rendered explicitly
and is not conflated with a failed item.

**Layer 2 — crate portrait.** Group items by exact/normalized fiber, engineer
claim, boundary, and terminal/exception class. Show counts only alongside the
actual population denominator. List lowered, incomparable, historical,
incomplete, refuted, or policy-rejected items as named exceptions.

**Layer 3 — item and clause portrait.** In source order, show each item's
engineer statement and exact disposition, then each clause address, procedure,
formal position, terminal, route evidence, boundary, residual trust, and
mutation-policy contribution. Multi-author burns stay clause-local.

**Layer 4 — formal portrait.** A schema-enveloped stable normalized JSON carries realized
judgments or their replayable serialization, frames, evidence identities,
classifications, clause portfolios, both frontiers, normalization witnesses,
project lift, collapse policy and disclosures, TCB, source/artifact digests, and
schema versions. This is the comparison and automation contract.

The CLI provides the headline by default, `--items` for the crate/item portrait,
`--explain <item-or-project>` for disclosure and formal framing, and `--json`
for the complete report. Exact flag spelling may follow existing CLI conventions
but all four views consume one validated report object.

### CI publication and comparison

Add an `assurance-report` job to `.github/workflows/ci.yml` after certification.
It writes normalized JSON and a standalone static rendering, uploads both as
retained artifacts, and emits a bounded GitHub step summary. On pull requests it
compares the base and head reports only after validating schema, policy, source
population, and artifact identity.

The comparison vocabulary is formal: strengthened, weakened, equal,
incomparable, changed fiber/boundary/residual context, added/removed population,
new non-claim, newly historical, and schema/policy skew. It reports the changed
project common frontier separately from changed item evidence. A comparison is
made only against an artifact bound to the exact base SHA and report digest;
otherwise the result is `no comparison`. Schema or policy skew is not normalized
without a checked migration theorem. A compact check links to the full artifact
and may later be embedded in the fork's existing GitHub Pages publication; it
does not require a third-party authority service.

An optional repository policy declares exact formal floors and whether a
missing project lift is allowed. That policy can gate through the formal
frontier/dominance theorem only. Report generation itself is informational and
uploads diagnostics even on floor failure. Floor evaluation runs in the live
admitted process, never by trusting uploaded JSON.

Every PR, including a same-repository PR, is treated as untrusted. Its report
job has only `contents: read`, no comment/Pages/checks-write/OIDC permission,
and no publication secret. Third-party actions are commit-pinned.
PR artifacts and their sanitized HTML remain visibly labeled untrusted
diagnostics and expire with ordinary retention. No privileged follow-up consumes
them. After protected merge/push, a trusted workflow checks out that protected
exact SHA, reruns certification and report validation there, and produces new
canonical JSON and escaped/CSP HTML for durable publication. Thus the durable
portrait is regenerated from protected code and authority rather than merely
schema-validating attacker-authored claims. A missing or expired exact-base
protected report yields a visible delivery limitation, not a substituted
“latest” baseline.

### Historical migration and loud warnings

First add a certificate/report schema envelope. Old unversioned JSON remains
parseable as `LegacyUnversioned` because it is evidence people may need to
inspect, but it never silently receives modern meaning. Every historical view
begins:

```text
HISTORICAL CERTIFICATE — NOT VALID FOR CURRENT ASSURANCE DECISIONS.
Formal frame and coordinates are missing; re-certification is required.
```

The warning then names what is unknowable. L2 lacks its bound. L3 is ambiguous
between materially different refutation/trust semantics. All levels lack the
current frame, versions, exact boundary context, residual assumptions, accepted
evidence, and collapse-policy license. There is no `legacy_position` authority
conversion and no invented historical schema version. Current authoritative
JSON contains no compatibility Level. An optional legacy-inspection export is a
terminal presentation format with no current deserializer or admission route.

Migration stages:

1. Ship the schema envelope and classify earlier input as `LegacyUnversioned`.
2. Define `AssurancePolicyV2`; reconcile every older order model; prove realized
   engineer meanings, projection families, `PortfolioLift`, population-complete
   common-frontier laws, supported rebase, and `ProjectLift`.
3. Add `CurrentAssurance`, make Level private/derived, and atomically migrate
   every authority decision read. Bind oracle/cache identity to formal authority
   and bump schemas.
4. Ship the layered human/JSON/CI report, historical warnings, separate
   authority/presentation digests, and least-privilege publication workflow.
5. Replace the scalar project-min proof and delete/isolate the enum after its
   frozen human/legacy-inspection compatibility window.

Each stage has one authority path. Compatibility can coexist as presentation,
but never as a second decision source.

## Residual trust

Engineer meaning theorems remain relative to the named language semantics,
fragment classifier, implementation model, residual context, boundary
contracts, procedure environment, tool versions, resource premises, and axioms.
The label makes these assumptions recoverable; it does not remove them.

The closed parameterized policy abstraction may forget concrete distinctions. Its soundness
and exact-fiber theorems prevent that loss from becoming authority, but choosing
which common semantics are useful to engineers remains a versioned product
policy. A future policy expansion can change report frontiers and labels without
changing underlying judgments; CI must report that skew instead of pretending
it is a code regression.

Project conjunction trusts the checked item inventory and source/artifact
binding to be complete. `ProjectLift` establishes sound composition for the
inventoried items; it does not prove the build system included every intended
crate, target, feature, platform, or generated source. The report therefore
names its population identity and build configuration.

CI trusts GitHub's artifact storage, status rendering, base-commit selection,
and retention policy as a delivery mechanism. Those systems cannot mint
Thermite authority. A missing, expired, or unpublished report impairs review
visibility but does not change an existing certificate's formal meaning.

## Resolved Questions

- Q-1: This is one staged design covering proved engineer labels, project
  aggregation, report UX, authority migration, and Level retirement.
- Q-2: Project aggregation uses a fiber-indexed common-claim downset rendered by
  its maximal antichain, plus a separately named evidence frontier. A meet is a
  conditional theorem consequence.
- Q-3: Historical JSON is inspect-only and every rendering loudly requires
  re-certification; no legacy level synthesizes current authority.
- Q-4: Primary human output uses the engineer-facing layer rather than Level.
  Engineer labels are stored, versioned, proved, visibly boundary-qualified,
  and structurally non-authoritative.
- Q-5: The trust deliverable has multiple disclosure levels and is generated,
  diffed, and published by CI in the interaction style of a coverage-reporting
  service while retaining all authority inside Thermite's checked formal data.

## Out of Scope

- Implementing the design in this design pass.
- Treating engineer labels, report prose, percentages, badges, or CI success as
  certification evidence.
- Inventing a total order, cross-fiber comparison, representative item, or
  per-axis scalar minimum for incomparable assurance positions.
- Claiming completeness for external solvers, Lean, Verus, Kani, rustc, LLVM,
  operating systems, platforms, GitHub, or the report renderer.
- Hosting a multi-tenant external assurance service. The initial publication is
  repository CI artifacts, summaries, and optionally the existing fork-owned
  Pages surface.
- Defining organization-wide floors or product policy beyond the exact,
  repository-declared formal floor mechanism.
- Deleting historical evidence before the explicit compatibility window and
  schema migration have shipped.
