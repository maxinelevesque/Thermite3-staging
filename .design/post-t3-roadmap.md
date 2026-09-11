# Feature: Post-Thermite-3 roadmap

## Summary

Turn the completed Thermite 3 core into an engineer-visible assurance product
before committing to the larger research portfolio. The roadmap has one fixed
product spine—process cleanup, issues #55, #56, and #57, then alpha.11—and an
explicit reassessment gate before any post-T3 research issue becomes an
automatic implementation commitment.

The post-alpha.11 checkpoint has now selected a second bounded product spine:
finish checked assurance transport in #58, close the RFC-12 anti-vacuity debt
in #146, cut alpha.12, then extend the product through #59, #60, and #63. A
single design-only relational-assurance research slot follows alpha.12 without
entering the product critical path.

GitHub issues remain the executable work units and
`.github/workflows/roadmap.yml` remains the issue-derived visualization. This
document is the durable sequencing and decision record that explains why the
issues are ordered, what must be true at each boundary, and which research
lanes remain provisional.

## Requirements

- REQ-1: A short process-cleanup slice shall precede assurance implementation.
  It shall reconcile imported tracker references `#169`, `#173`, `#175`, and
  `#273` without rewriting their historical provenance; propose durable local
  issue homes for the RFC-14 crash clause and the composition experiment; make
  workspace-version claim-receipt refresh an explicit release obligation; and
  define a low-churn monitoring protocol.
- REQ-2: Local claim-receipt refresh shall gain an official deterministic
  parallel mode around `gates/claim-closure-author.py`. It shall reuse the same
  eight-shard ownership semantics as `.github/workflows/ci.yml`, merge only a
  complete pairwise-disjoint population, produce byte-identical authoritative
  outputs to the serial `--materialize` path, and retain a serial fallback.
- REQ-3: Monitoring shall be event-driven when the execution environment
  exposes completion events. Otherwise it shall use bounded exponential
  backoff, emit no user update for unchanged state, and notify only on a
  meaningful milestone, a failure requiring diagnosis, a material decision,
  an external-sharing approval boundary, or completion.
- REQ-4: Issue #55 shall be the first product implementation. It shall deliver
  the exact `ProjectPopulation`, `PortfolioLift`, `ItemClaimSet`,
  `common_claim_frontier`, `evidence_frontier`, and `ProjectLift` semantics
  specified by `.design/engineer-assurance-report-and-level-retirement.md`,
  including complete-population and hostile heterogeneous-portfolio coverage.
- REQ-5: Issue #56 shall follow a green merge of #55. It shall introduce the
  fail-closed `CurrentAssurance` authority seam, preserve older `Level` records
  only for loud inspection, bind current authority independently from
  presentation, and reduce production `Level` decisions to zero in one atomic
  migration.
- REQ-6: Issue #57 shall follow a green merge of #56. It shall build all four
  disclosure layers from one validated assurance report object, add exact
  base/head comparison and repository-local formal floors, and preserve the
  least-privilege publication boundary in
  `.design/engineer-assurance-report-and-level-retirement.md`.
- REQ-7: Each product-spine issue shall use the tiered workflow: focused tests
  and structural checks while iterating; tree stabilization; one final affected
  claim-receipt materialization; full qualification; fresh explicit approval
  before any private repository or diff content is sent externally; exact-head
  adversarial review; then CI and merge only while the reviewed head is
  unchanged.
- REQ-8: Alpha.11 shall be cut only after #55, #56, and #57 are merged and the
  assurance-minimum milestone is demonstrably complete. The alpha change shall
  follow the repository's established version-only practice except for the
  mechanically required content-bound receipt refresh and its generated pins;
  it shall not imply a tag or GitHub Release.
- REQ-9: Alpha.11 shall create a roadmap reassessment checkpoint. Issues #58
  through #63 are a provisional dependency order—not an automatic queue—with
  #146 considered after the report semantics from #57 exist. The checkpoint
  shall compare product value, research uncertainty, prerequisite maturity,
  and validation cost before selecting the next implementation.
- REQ-10: Research incubation shall continue without entering the product
  critical path. The maintained lanes are relational contracts in
  `.design/research/relational-contracts.md`, formal-methods and trust-boundary
  work in `.design/research/formal-methods-sota.md`, the preregistered study in
  `.design/research/composition-experiment.md`, RFC-14 in
  `.design/rfcs/0014-crash-clause.md`, and the fully verified Iroh exploration
  tracked by issue #45.
- REQ-11: Issue #58 shall be completed in its existing implementation pull
  request rather than restarted. Completion requires every new test to have
  reviewed partition ownership, all affected claim receipts and generated
  status to be current, the controlling design to state present landing truth,
  full qualification to pass, and the exact admitted head to receive
  adversarial review and green required CI before merge and issue closure.
- REQ-12: Issue #146 shall follow a green merge of #58. It shall define the
  RFC-12 trace/evidence boundary, exercise mutations that weaken, delete, or
  redirect promised effects, and prove that the Rust and Lean checking path
  rejects or honestly classifies each mutation without granting authority to
  an unsupported observation.
- REQ-13: Alpha.12 shall be cut after #58 and #146 are merged. It shall use the
  established version-only release practice, including mechanically required
  content-bound receipt and audit-pin refresh, and shall not imply a tag or
  GitHub Release.
- REQ-14: After alpha.12, issue #59 policy-version migration and report
  comparability shall precede #60 workspace/build-matrix populations. Issue
  #63 organization-level floors shall follow both so that organization policy
  consumes proved transport, migration, and population semantics rather than
  defining a parallel authority system.
- REQ-15: One bounded design-only research slot shall follow alpha.12. Its
  default subject is the relational-contract Tier-A frame lemma. The slot may
  refine a design and name decisive evidence, but shall not silently authorize
  implementation or displace the #59 -> #60 -> #63 product sequence. RFC-14,
  the composition experiment, later relational tiers, formal-methods survey
  work, and the verified-Iroh pressure test remain incubated.
- REQ-16: Execution of the second product spine shall preserve the low-churn
  operating protocol. Prefer completion events; otherwise use bounded backoff.
  Emit no update for unchanged state, make an external-sharing approval request
  at most once per unresolved scope, and notify only on a meaningful milestone,
  actionable failure, material decision, approval boundary, or completion.

## Acceptance Criteria

- [x] AC-1: (REQ-1) Every imported reference `#169`, `#173`, `#175`, and `#273`
  has an explicit provenance-preserving disposition, and RFC-14 plus the
  composition experiment each has a current local issue or an explicit
  decision not to file one.
- [x] AC-2: (REQ-1, REQ-3) A checked-in execution note states the version-bump
  receipt-refresh contract and the event/backoff/no-unchanged-update monitoring
  rules; a review can point to one durable source instead of reconstructing
  them from prior sessions.
- [x] AC-3: (REQ-2) Tests run serial and eight-way materialization over the same
  fixture population and assert byte-for-byte equality of the generated
  registry and closure ledger.
- [x] AC-4: (REQ-2) Missing, duplicate, overlapping, stale, or failed shard
  output prevents publication, while `gates/claim-closure-author.py
  --materialize` remains a working serial fallback.
- [x] AC-5: (REQ-4) Issue #55's hostile tests cover mixed incomparable
  portfolios, absent clauses, absent population members, representative
  selection, and per-axis scalar minimum; all fail closed as specified by the
  issue.
- [x] AC-6: (REQ-5) Issue #56's completion gate reports zero production
  `Level` decisions, rejects current-looking legacy rows and portfolio/cache
  splices, and leaves inspect-only historical evidence non-authoritative.
- [x] AC-7: (REQ-6) Issue #57's deterministic snapshots and hostile
  stale/tampered/injection/fork cases pass, and every disclosure layer is
  derived from the same validated report object.
- [x] AC-8: (REQ-7) Every product-spine pull request records its exact reviewed
  head, qualification result, required CI result, and merge commit; any
  post-review head change invalidates the prior review receipt.
- [x] AC-9: (REQ-8) Main contains merged #55, #56, and #57 before the alpha.11
  version commit; locked metadata reports one workspace version; all refreshed
  receipts are content-current; required CI is green; and no alpha.11 tag or
  GitHub Release is created.
- [x] AC-10: (REQ-9, REQ-10) The post-alpha.11 checkpoint records a deliberate
  next selection or pause, with #58–#63, #146, RFC-14, the composition
  experiment, relational contracts, formal-methods work, and issue #45 all
  classified rather than silently treated as queued implementation.
- [ ] AC-11: (REQ-11) #58 is merged and closed only after the existing pull
  request has reviewed test-partition ownership, current claim evidence and
  design status, full local qualification, an exact-head adversarial-review
  receipt, and all required CI green on that unchanged head.
- [ ] AC-12: (REQ-12) #146 demonstrates at least weakening, deletion, and
  redirection mutations across the declared Rust/Lean observation boundary,
  with every mutant either killed or returned as an explicit unsupported
  result rather than accepted as valid evidence.
- [ ] AC-13: (REQ-13) Main contains merged #58 and #146 before the alpha.12
  version commit; locked metadata reports one workspace version; required
  receipts and pins are current; CI is green; and no alpha.12 tag or GitHub
  Release exists.
- [ ] AC-14: (REQ-14) #59 lands before #60, and #63 consumes their checked
  transport, migration, comparison, and population carriers without adding a
  second current-authority path.
- [ ] AC-15: (REQ-15) The post-alpha.12 research slot produces a checked design
  or an explicit evidence-backed pause for the Tier-A frame lemma, while every
  other incubated lane retains a named disposition and no implicit
  implementation commitment.
- [ ] AC-16: (REQ-16) The execution record contains no repeated unchanged-state
  user notifications or duplicate outstanding approval requests; failures and
  milestones remain observable through their durable CI, review, and merge
  receipts.

## Architecture

### Authority and sequencing

The roadmap has three distinct layers. GitHub issues are the mutable execution
queue and milestone state. `.github/workflows/roadmap.yml` and
`dev/render-roadmap.py` continue to render that issue tree. This file is the
stable rationale and sequencing overlay. It does not duplicate each issue's
full design: issues #55–#57 continue to derive their semantics and hostile
acceptance boundaries from
`.design/engineer-assurance-report-and-level-retirement.md`.

The fixed near-term dependency chain is:

```text
process cleanup -> #55 composition -> #56 authority migration
                -> #57 report/CI -> alpha.11 -> roadmap reassessment
```

#55 comes first because `CurrentAssurance` needs the complete portfolio and
project-lift semantics it will authorize. #56 comes before #57 because the
report must consume one fail-closed current authority seam, not preserve a
second live `Level` decision source for convenience. Alpha.11 follows the whole
milestone so the released version has one coherent assurance contract.

### Process cleanup

Reference reconciliation is additive. Historical documents may continue to say
that imported work was called `#169`, `#173`, `#175`, or `#273`; cleanup adds
the current repository disposition instead of making old prose falsely claim
those were local issues. RFC-14 and the composition experiment receive explicit
local tracking decisions before either is scheduled.

`gates/claim-closure-author.py` already owns serial coordinated
materialization, while `.github/workflows/ci.yml` owns the reviewed eight-shard
execution partition. The local optimization shall expose bounded parallelism
through the existing author rather than introduce a second receipt format or a
second source of shard ownership. Shards compute isolated in-memory results; a
coordinator checks exact per-shard ownership, completeness, and disjointness
before the existing authoritative render step writes the registry and closure
ledger. Tests belong beside the existing coverage in
`gates/tests/test_claim_closure_author.py` and the workflow contract checks in
`gates/tests/test_ci_workflow_contract.py`.

The monitoring protocol is operational, not authority-bearing. Completion
events are preferred. Polling, when unavoidable, starts with a short interval,
backs off to a bounded long interval, resets only on state change, and does not
generate a conversational update for an unchanged check. CI, exact-head review,
and merge receipts remain the evidence; poll frequency does not.

### Product spine

Issue #55 implements the formal composition foundation. Its output is the
validated project authority consumed by #56. Issue #56 then provides the sole
current authority API and isolates historical Level-shaped data. Issue #57
renders and compares that authority without making presentation authoritative.
The three issue bodies and
`.design/engineer-assurance-report-and-level-retirement.md` remain controlling
where this sequencing document is less specific.

Each issue is a separate reviewed pull request unless a later material finding
shows the boundary is unsound. Focused iteration must not continually rewrite
all content-bound receipts. Once the tree is stable, affected receipts are
materialized once, full qualification runs, and the exact committed head is the
unit reviewed and admitted by CI.

### Alpha.11 and research checkpoint

The alpha.11 pull request follows the alpha.8–alpha.10 version practice: the
workspace version changes in `Cargo.toml` and `Cargo.lock`, while no tag or
GitHub Release is implied. Unlike a literally two-file diff, the version change
also invalidates claims bound to Cargo manifests. The planned receipt refresh
and generated audit-pin updates are therefore part of the release contract,
not an unexpected CI repair.

After alpha.11, the provisional post-T3 assurance order is #58 cross-version
and cross-procedure transport, #59 policy migration, #60 workspace/build-matrix
populations, #61 historical discovery and recertification, #62 signed
pre-merge attestations, and #63 organization floors. Issue #146 becomes
actionable after #57 provides the report and comparison surface its mutation
observables can inform. This order is a hypothesis to assess at the checkpoint,
not permission to launch every item automatically.

Research incubation remains parallel intellectual work, not concurrent
implementation by default. The checkpoint may promote one of the relational,
crash-consistency, experimental, Iroh, or proof-boundary lanes only after naming
the question, expected evidence, and opportunity cost against the assurance
sequence.

### Post-alpha.11 reassessment (2026-09-09)

The fixed product spine is complete. The table below is the durable boundary
receipt; GitHub reported every named required check successful on the recorded
head before squash merge.

| Boundary | Exact admitted/reviewed head | Qualification and review | Final CI | Merge on `main` |
| --- | --- | --- | --- | --- |
| #55 / PR #157 | `7f6b2c4d581b32f46644aac098e3efdfb0ecf704` | 581 deterministic claim closures, complete project-composition qualification, and cold Opus 4.8 `APPROVE WITH FOLLOW-UPS` with no blocker | run `34287641676`, 35/35 successful or intentionally skipped | `c5ec03c253bb688c02dc3af950f79def41e0ef47` |
| #56 / PR #158 | `b139f77a2d7e6beb20261dd40ab8ec4b3a246958` | 582 deterministic claim closures, full authority-migration qualification, zero production `Level` decisions, and cold Opus 4.8 `APPROVE WITH FOLLOW-UPS` with no blocker | run `34343131320`, 35/35 successful or intentionally skipped | `745ed990e033ca2042f8f9c924c81081f401715d` |
| #57 / PR #159 | `01eec97e540bd84ed748fd139c5eca090a4bf482` | 583 deterministic claim closures, full report/CI qualification, retained first-run publication evidence, and cold Opus 4.8 `APPROVE WITH FOLLOW-UPS` with no blocker | run `34411473052`, 36/36 successful or intentionally skipped | `1479d4b3785982093e8273525901344d01dd0c0b` |
| alpha.11 / PR #160 | `921a4468975a7ef189cb0c1651db4b6f477c30c1` | locked metadata, format, workspace check, 583 refreshed closures, full completeness replay, and structural qualification | run `34426428509`, 36/36 successful or intentionally skipped | `ffacc03a9f68d2d0dc5f851bc26b06af775012b9` |

For #57 specifically, `LiveAssuranceReport` remains the non-serializable
validated capability from which all four disclosure layers are rendered. Unit
and live-process evidence covers byte determinism, complete-population and
presentation tampering, missing/stale exact-base comparison, schema and policy
skew, HTML injection, bounded/deep/unknown JSON, untrusted publication, and
live-capability-only floor evaluation. The first PR artifact receipt is recorded
in `.design/engineer-assurance-report-and-level-retirement.md`.

Alpha.11 followed the intended release boundary: #55, #56, and #57 were already
merged and their issues closed; `Cargo.toml` and every workspace package entry
in `Cargo.lock` report `3.0.0-alpha.11`; all 583 content-bound receipts were
rematerialized and the governed audit pin refreshed. The repository has neither
an alpha.11 tag nor a GitHub Release.

The checkpoint compares four factors qualitatively: immediate product value,
research uncertainty, prerequisite maturity, and validation cost. `Promote`
means one next candidate, not permission to start every dependent item.

| Lane | Product value | Research uncertainty | Prerequisite maturity | Validation cost | Checkpoint disposition |
| --- | --- | --- | --- | --- | --- |
| #58 cross-version/procedure transport | High: makes assurance comparable across compiler, model, and procedure evolution and forms the basis for #59 and #63 | Medium: simulation and incompatibility witnesses need a precise authority boundary | Ready: #48 and #54 are closed and the current authority/report carriers exist | High: hostile skew cases, transport proofs, receipts, and cross-version fixtures | **Promote as the next implementation candidate, beginning with a design checkpoint.** No implementation starts in this checkpoint. |
| #59 policy migration/report comparability | High once policies evolve | Medium | Partly ready; #54 and #57 are closed, but #58 should establish the transport vocabulary first | High | Defer until #58 lands and exposes the exact migration obligations. |
| #60 workspace/build-matrix populations | High for multi-package, feature, target, and platform use | Medium | Ready at the project-report layer, but should consume stable transport semantics | High because the population matrix grows quickly | Defer behind #58; reassess against concrete multi-workspace demand before #59/#60 ordering is fixed. |
| #61 historical discovery/re-certification | Medium; valuable for legacy estates but not current authority | Low to medium | Ready: #56 and #57 are closed | Medium | Demand-driven defer; do not revive legacy `Lx` records as modern authority. |
| #62 signed pre-merge attestations | High integrity value for distributed publication | High threat-model and operational uncertainty | The report carrier exists, but protected regeneration is currently authoritative and PR artifacts are intentionally untrusted | High security and key-lifecycle cost | Research/design incubation; require a threat model before implementation. |
| #63 organization-level floors | Medium to high only with real multi-repository governance demand | High policy, inheritance, exception, and delegation complexity | All original dependencies are closed, but #58/#59 should precede it | High | Defer until transport and policy migration are proved and an organizational consumer exists. |
| #146 RFC-12 effect-trace mutation observables | High bounded correctness value for detecting weakened/deleted/redirected promises | Medium | Ready after #57 supplied the comparison surface | Medium | Promote as the bounded correctness follow-up after #58, unless a small design probe first shows an invalid-proof risk requiring earlier interruption. |
| RFC-14 / #154 crash clause | Potentially high for persistence protocols | High: sector atomicity, flush, recovery relation, and trust boundary remain unsettled | Not ready for code | Very high | Continue design incubation; no implementation commitment. |
| Composition experiment / #155 | High epistemic value for whether the method improves independent contract composition | Medium experimental risk | Preregistered, but frozen task packs, stripped comparison library, runner, and scorer are not built | Very high: 36 model sessions plus harness and scoring | Preserve the preregistration and schedule only with an explicit experiment budget; keep off the product critical path. |
| Relational contracts | Tier A frame and `hides` work has credible near-term value; probabilistic, dynamic-authority, and certificate-algebra work is longer horizon | Low to very high by tier | Tier A is conceptually mature; Tier B/C are not | Medium for Tier A, very high beyond it | After #58, consider a separate Tier A frame-lemma design. Keep Tier B/C as research. |
| Formal-methods / trust-boundary survey | High as design discipline, low as a standalone feature | Medium because the survey's historical `L0`–`L3` vocabulary no longer matches current certificates | Source material exists but needs translation | Medium | Refresh terminology into current certificate coordinates when a consuming design needs it; do not implement directly from the archival taxonomy. |
| Fully verified Iroh / #45 | High as a pressure test of the architecture, low as a near-term deliverable | Very high across async failure, cancellation, temporal/network semantics, QUIC/TLS, runtime, and OS trust | Not ready | Multi-year | Retain as a long-horizon pressure test; extract questions for protocol designs, but do not schedule full implementation. |

The deliberate next selection is therefore **#58**, with design refinement
before code. #146 is the next bounded correctness follow-up. All other rows
remain deferred or incubated exactly as classified above; this checkpoint does
not queue them and grants no external-sharing or merge authority.

### Second product spine activation (2026-09-10)

The #58 design and implementation have been authored, but authoring is not the
landing boundary. PR #162 remains the active implementation vehicle at head
`5206908bd5a0c236b1cce6ef79f8918889ae15d5`. Its first CI run exposed three
unreviewed catch-all Rust tests in the test-partition gate and failures in the
claim-closure fanout. Those are closure work for the existing pull request,
not grounds to discard the checked transport implementation or to begin #146
in parallel.

The activated dependency chain is:

```text
#58 landing repair and closure -> #146 RFC-12 mutation observables
                               -> alpha.12
                               -> bounded Tier-A relational design slot
                               -> #59 policy migration/comparability
                               -> #60 workspace/build-matrix populations
                               -> #63 organization floors
```

The research slot is intentionally design-only and may overlap planning, but
not authority-bearing implementation, with the product sequence. #59 precedes
#60 because #58 supplies its transport relation directly and policy/report
comparability should be stable before population matrices multiply the number
of compared variants. #63 remains last because an organization floor is a
consumer of all three semantics, not a shortcut around them.

Alpha.12 is the release boundary for checked transport plus the bounded
RFC-12 anti-vacuity repair. It deliberately does not wait for #59 or #60; doing
so would produce a much larger admission unit and obscure whether transport,
mutation observability, migration, or population semantics caused a failure.

The operating protocol applies to the whole chain. A dispatched long-running
job receives one initial observation, then event-driven completion or bounded
backoff. An unchanged poll produces no conversational update. An approval
request is surfaced once and remains pending without reminders. Exact-head
qualification, review, CI, and merge receipts—not polling frequency—are the
durable evidence of completion.

## Residual trust

The roadmap trusts GitHub issue and milestone state as the scheduling view, but
issue metadata does not prove implementation completion. Completion still
depends on repository qualification, exact-head review, required CI, merge
state, and the formal evidence named by each issue's controlling design.

The proposed local parallel materializer can reduce wall-clock time but cannot
increase claim authority. Its correctness continues to depend on the existing
claim author, execution-identity partition, deterministic renderer, Python
runtime, filesystem isolation, and the tests that compare it with serial
materialization.

The post-alpha.11 ordering is based on present dependencies and product value.
New research evidence, upstream changes, maintenance constraints, or user
priorities may change that ordering at the explicit reassessment checkpoint.
No roadmap entry carries forward permission to disclose private repository
content to an external reviewer.

## Resolved Questions

- Q-1: The sequencing source of truth is `.design/post-t3-roadmap.md`; GitHub issues
  remain the execution units and the existing workflow remains the rendered
  issue view.
- Q-2: Imported issue numbers are preserved as provenance and mapped to current
  dispositions rather than silently renumbered.
- Q-3: Local materialization gains a deterministic eight-shard coordinator around
  the existing author; redesigning content-bound evidence is deferred.
- Q-4: Monitoring prefers events, otherwise bounded exponential backoff, and stays
  silent when state is unchanged.
- Q-5: Issues #58–#63 are reconsidered after alpha.11 and are not an automatic queue.
- Q-6: The second product spine finishes #58 in its existing pull request, then
  runs #146 before cutting alpha.12.
- Q-7: After alpha.12 the product order is #59, #60, then #63; #61 and #62
  remain demand-driven or research-bound rather than implicit prerequisites.
- Q-8: The first post-alpha.12 research allocation is one design-only Tier-A
  relational frame-lemma slot and does not authorize implementation.
- Q-9: Monitoring and approval notifications are state-change-driven and
  one-shot; unchanged state is silent.

## Open Questions

- None.

## Out of Scope

- Implementing #55, #56, #57, or any research item in this design pass.
- Treating the post-alpha.11 research order as pre-approval to create branches,
  share private content externally, or merge changes.
- Redesigning the claim-receipt evidence model during the local parallelism
  cleanup.
- Rewriting historical imported issue numbers as if they had existed in this
  repository.
- Creating a tag, GitHub Release, or stable-version commitment for alpha.11.
