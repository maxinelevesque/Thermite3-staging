# Feature: Post-Thermite-3 roadmap

## Summary

Turn the completed Thermite 3 core into an engineer-visible assurance product
before committing to the larger research portfolio. The roadmap has one fixed
product spine—process cleanup, issues #55, #56, and #57, then alpha.11—and an
explicit reassessment gate before any post-T3 research issue becomes an
automatic implementation commitment.

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
- [ ] AC-7: (REQ-6) Issue #57's deterministic snapshots and hostile
  stale/tampered/injection/fork cases pass, and every disclosure layer is
  derived from the same validated report object.
- [ ] AC-8: (REQ-7) Every product-spine pull request records its exact reviewed
  head, qualification result, required CI result, and merge commit; any
  post-review head change invalidates the prior review receipt.
- [ ] AC-9: (REQ-8) Main contains merged #55, #56, and #57 before the alpha.11
  version commit; locked metadata reports one workspace version; all refreshed
  receipts are content-current; required CI is green; and no alpha.11 tag or
  GitHub Release is created.
- [ ] AC-10: (REQ-9, REQ-10) The post-alpha.11 checkpoint records a deliberate
  next selection or pause, with #58–#63, #146, RFC-14, the composition
  experiment, relational contracts, formal-methods work, and issue #45 all
  classified rather than silently treated as queued implementation.

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
