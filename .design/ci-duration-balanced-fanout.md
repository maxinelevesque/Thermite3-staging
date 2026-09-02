# Feature: Duration-balanced CI fan-out

## Summary

Replace test-count partitioning and monolithic G3/G4 jobs with deterministic,
duration-balanced CI jobs. Preserve complete fail-closed coverage and stable
aggregate check names while reducing the green-run critical path to the longest
genuinely indivisible proof-backed test or gate segment plus runner setup.

The measured baseline is GitHub Actions run `31811912559` on 2026-08-14. Its
four nextest phases took 18m51s, 12m16s, 10m40s, and 5m16s; G3 took 7m56s in
its gate step, and G4 took 16m02s. The test logs contain 1,506 passed tests and
149m34s of summed per-test duration. The apparent 583.921s and 345.343s longest
tests are serial case matrices and are divisible. The longest observed
indivisible test after that split is
`forge::bin/forge epr_reconstruct::tests::production_reconstructs_sequence_extensionality`
at 319.485s, followed by `forge::editor_runs
editor_content_pinned_ops_still_certify_l3` at 303.310s.

## Requirements

- REQ-1: The workspace test suite shall run in thirteen deterministic partitions:
  twelve duration-balanced partitions plus one partition that owns every new or
  otherwise unassigned test.
- REQ-2: A checked-in partition manifest shall assign every known nextest test
  exactly once, and a fail-closed coverage gate shall reject duplicates,
  omissions, stale test identifiers, and an absent catch-all partition.
- REQ-3: Partition generation shall use checked-in timing observations and a
  deterministic longest-processing-time allocation, with stable lexical
  tie-breaking and the indivisible-test duration as the target upper bound.
- REQ-4: Serial matrix tests in `forge/tests/verified_build.rs` shall become
  independently schedulable named test cases without weakening their shared
  assertions, fault coverage, or publication-atomicity checks.
- REQ-5: G3 shall fan out into a parser/lowering job and a checked-replay/BV-route
  job; G4 shall fan out into bridge/Lean pins, LRAT/cache replay, release
  defaults/automatic routing, and hygiene/axiom jobs.
- REQ-6: Thin aggregate `g3` and `g4` jobs shall retain the existing required
  check names and fail unless every corresponding child job succeeds.
- REQ-7: The thirteen test jobs shall share the existing prepared Lean artifact,
  pinned Verus distribution, pinned Stage-4 proof tools, and Rust caches without
  weakening tool presence, version, axiom, or placeholder checks.
- REQ-8: CI shall publish machine-readable per-test and per-gate-segment timing
  artifacts so rebalancing is reviewable and does not depend on opaque GitHub
  history.
- REQ-9: A before/after report shall compare critical-path time, aggregate runner
  time, partition maximum/minimum execution time, and coverage cardinality
  against run `31811912559`.
- REQ-10: The optimization shall land in a separate pull request based on fresh
  `staging` after PR #50 merges; RFC-10 commits and assurance claims shall not be
  rewritten by this work.
- REQ-11: Tool-aware claim-closure replay shall run in eight deterministic,
  disjoint shards keyed by execution identity, so requirements sharing an oracle
  or formal-verifier identity retain one execution cache and are never split.
- REQ-12: A stable `claim-closure` aggregate shall fail unless all eight replay
  shards succeed, while `lean-probe` shall own only the Lean spine build and
  axiom boundary named by that job.
- REQ-13: Every claim-closure shard shall publish machine-readable timing even
  on failure, and the optimization shall be evaluated against the serial
  47m54s claim-replay step from run `33686853981` rather than skipped work.

## Acceptance Criteria

- [x] AC-1: (REQ-1, REQ-3) A partition simulation over the checked-in baseline
  produces thirteen buckets, assigns the longest measured reconstruction test
  alone or with
  only work that keeps its predicted bucket at the target bound, and reports no
  predicted bucket above the longest indivisible test except a documented
  timing-noise tolerance.
- [x] AC-2: (REQ-2) The partition coverage gate exits zero only when the union of
  explicit assignments and the catch-all equals `cargo nextest list` exactly and
  explicit assignments are pairwise disjoint; deletion, duplication, and rename
  mutants each make it exit nonzero.
- [x] AC-3: (REQ-4) Each former case in
  `every_tv_phase_and_nonpass_class_blocks_publication` and
  `every_injected_commitment_failure_is_atomic` is independently selectable,
  and a manifest-backed test proves the original 16-case and 12-case sets remain
  complete.
- [x] AC-4: (REQ-5, REQ-6) GitHub Actions displays all six G3/G4 child checks and
  aggregate `g3`/`g4`; forcing any child to fail makes only its aggregate fail
  while unrelated children finish and report their results.
- [x] AC-5: (REQ-7) Every proof-backed test partition retains the live Lean,
  Verus, CaDiCaL, drat-trim, and Z3 environment required by its selected tests;
  removing any required tool fails the owning partition rather than skipping.
- [x] AC-6: (REQ-8) Every test partition uploads nextest JUnit timing data and
  every gate child prints and uploads segment timing data with stable logical
  names.
- [x] AC-7: (REQ-9) Two consecutive green validation runs show no missing tests
  and a test critical path no greater than the longest observed indivisible test
  plus setup and a 15% timing-noise allowance.
- [x] AC-8: (REQ-9) The before/after report accounts separately for execution,
  setup, queue delay, and total runner-minutes; improvement is not claimed from
  queue delay or skipped work.
- [x] AC-9: (REQ-10) The implementation branch merge-base is the post-#50
  `staging` tip, and its pull request contains no RFC-10 implementation rewrite.
- [ ] AC-10: (REQ-11, REQ-12) Eight green shard jobs jointly execute every one
  of the 576 draft entries exactly once by stable execution identity, and the
  aggregate rejects any missing or failed child.
- [ ] AC-11: (REQ-12, REQ-13) A green live run shows the standalone
  `lean-probe` completing independently of claim replay and records the longest
  shard, aggregate runner time, queue delay, and end-to-end critical path.

## Architecture

### Deterministic test partitions

Add `gates/ci-test-partitions.toml` as the reviewed assignment manifest and
`gates/ci-test-partitions.py` as its checker and selector. Test identity uses the
fully qualified nextest identifier (`package::binary test-name`) emitted by
`cargo nextest list`, never source line numbers or discovery order.

The generator consumes a checked-in timing snapshot under `gates/fixtures/` and
sorts tests by descending duration, then by lexical identity. It assigns each
test to the currently lightest bucket, using bucket number as the final stable
tie-breaker. Matrix tests split under REQ-4 participate as ordinary independent
tests. The manifest records the timing-run SHA and run ID.

Twelve buckets contain explicit assignments. Bucket thirteen is the catch-all and runs
the complement of those assignments, so a newly discovered test runs immediately
instead of disappearing. The coverage gate still fails to require a reviewed
rebalance; execution is fail-safe while maintenance is fail-closed. The checker
also verifies that every explicit identifier exists and occurs once.

The test matrix in `.github/workflows/ci.yml` changes from nextest
`count:${shard}/4` partitioning to thirteen selector names. Every job keeps the current
tool and artifact setup. JUnit output is uploaded even on failure. A local
simulation prints predicted bucket totals and the longest member of each bucket.

The count follows from the measured bound rather than a round-number preference.
The initial baseline model selected nine explicit buckets plus catch-all from
47m03s of runner wall and 3.18x effective within-job parallelism. Two green live
runs then measured 2.17x at the slowest bucket under full fan-out and a 367.125s
longest indivisible test. Twelve explicit buckets are the minimum satisfying
`9840.129 / (367.125 * 2.17)`, with bucket thirteen providing the required
catch-all. Setup remains visible and is not folded into the execution-bound
claim.

### Splitting matrix tests

Refactor the two serial loops in `forge/tests/verified_build.rs` into shared
helpers plus macro-generated or explicitly named cases. Each case retains the
same command, exit-code, diagnostic, no-bundle, and no-staging-tree assertions.
A small inventory test freezes the complete phase/verdict and commitment-fault
sets, preventing parallelization from dropping a matrix cell.

Other multi-minute tests remain indivisible unless inspection shows a similarly
independent case matrix. In particular, sequence-extensionality reconstruction
and whole-editor certification define the initial scheduling bound rather than
being weakened merely to improve CI timing.

### Gate fan-out

Refactor `gates/g3.sh` and `gates/g4.sh` to accept a closed, validated segment
name while retaining an `all` mode for local reproduction.

G3 children:

- `g3-parser-lowering`: release parsing with and without `bv`, fixed-width
  lowering, and invariant preservation.
- `g3-checked-replay`: checked validity replay, live BV routing, and the G3-owned
  reconstruction probe.

G4 children:

- `g4-bridge-lean`: classifier differential and Lean normalization,
  Skolemization, grounding, and replay pins.
- `g4-lrat-cache`: production LRAT replay and cache-tamper checks.
- `g4-release-routing`: release build defaults and automatic BV routing.
- `g4-hygiene`: axiom footprint plus the no-placeholder/no-custom-axiom scan.

The child split targets the measured serial concentrations: G3 step 5 took
7m05s; G4 LRAT/cache took 10m22s and release routing took 5m08s. The LRAT test
filter already exposes independent named tests, so its CI child may become a
small matrix if its 319s, 193s, and 107s cases remain the child critical path.

Aggregate `g3` and `g4` jobs use `needs` and `if: always()` to inspect every child
result. They perform no proof work and succeed only when all children succeeded.
This preserves branch-protection names while making failures source-specific.

### Claim-closure fan-out

`gates/claim-closure-author.py --check-draft-shard INDEX/COUNT` validates the
complete draft population before selecting a shard. Selection hashes a canonical
execution identity, not requirement order: executable claims bind verifier plus
oracle, formal claims share their built-in axiom-verifier identity, and exact
populations bind their extractor. This keeps every cache-sharing or
identity-collision domain within one child while assigning every draft entry to
exactly one child.

The workflow runs eight children with `fail-fast: false`, the same pinned Lean,
Rust, Verus, CaDiCaL, and drat-trim environment as the former serial job, and a
per-shard Rust cache. A stable `claim-closure` aggregate inspects the matrix
result with `if: always()` and fails closed. Each child publishes normalized
timing JSON even when its replay fails.

`lean-probe` retains the prepared Lean artifact, Mathlib cache, spine build, and
axiom inspection, but no longer installs Rust, Verus, or Stage-4 tools and no
longer serializes the complete draft replay ahead of its named responsibility.
The claim and Lean jobs start from the same `lean-prepare` dependency and proceed
in parallel.

### Measurement and maintenance

The green workflow uploads nextest JUnit plus a normalized timing JSON artifact.
A rebalancing command updates the timing fixture and manifest mechanically, but
CI verifies the result independently. Timing updates are ordinary reviewed diffs;
CI never silently changes partition membership from historical service data.

The comparison report uses GitHub job step timestamps for setup/execution and
nextest timing output for test work. Queue time is reported separately because
additional fan-out can increase runner contention even while execution improves.

This design serves `telos/the-corpus-still-certifies`: the same complete suite
must run, with stronger static evidence that its partition union is exhaustive.
It also serves `telos/residual-trust-is-named`: historical timing is an explicit
maintenance input, not an invisible scheduler oracle.

## Validation evidence

The baseline is green run `31811912559`: 22m19s end-to-end critical path,
89m31s aggregate runner time, and four test jobs totaling 47m03s. Coverage was
the same 1,533 tests after the two long matrix loops were atomized.

The initial ten-bucket topology was green twice in run `31837152080` attempts 1
and 2. Critical path was 14m42s and 14m41s; aggregate runner time was 120m52s
and 121m57s. JUnit exposed a 4m22s..9m14s test-execution spread, so it improved
the baseline but did not satisfy AC-7's irreducible-path bound.

The selected thirteen-bucket topology was green twice in run `31843256167`
attempts 1 and 2. Critical path was 12m11s and 14m09s (13m10s average, 41.0%
below baseline); aggregate runner time was 124m40s and 137m08s (130m54s
average, 46.2% above baseline and 7.8% above the ten-bucket average). The
slowest nextest suite was 384.433s and 415.912s, both below the 422.194s live
bound. Every run reported 1,533 tests exactly once; catch-all was empty. Fan-out
queue delay was 0..4s. Non-suite per-test-job time (checkout, Lean/tool restore,
inventory, and upload) ranged from roughly 2m06s to 4m04s and is excluded from
the execution-bound claim. G4 LRAT/cache, not a test partition, determined both
selected-run critical paths at 470.618s and 615.460s of gate execution.

The claim-closure follow-up baseline is green run `33686853981`. The step named
`Claim-closure draft slices (tool-aware)` occupied 47m54s of the 51m47s
`lean-probe` job; the actual Lean spine and axiom probe took 2m21s. Two preceding
green runs spent 45m27s and 44m24s in the same serial claim step, establishing
that the bottleneck is the closure replay rather than Lean. Live eight-shard
results will be recorded here before AC-10 and AC-11 close.

Thirteen wins the stated tradeoff: versus ten it reduces average critical path
10.4% for 7.8% more runner time, satisfies the two-run bound, and leaves the
remaining critical-path variance in an independently visible irreducible gate.

## Residual trust

The scheduler trusts the checked-in timing sample to predict future duration;
coverage does not. Tests still execute when timing data is absent or stale, and
the catch-all prevents an unmeasured test from disappearing. Balance can regress
when solver or runner performance changes, so two-run validation and published
timings are evidence of performance, not a permanent guarantee.

GitHub runner availability, CPU count, cache service behavior, and queue policy
remain external. Aggregate checks establish child-job success from GitHub's
reported conclusions; they do not independently replay child artifacts. The
proof-tool and compiler trust boundaries are unchanged from the existing suite.

## Resolved Questions

- Q-1: The change lands after PR #50 as a separate pull request from fresh `staging`.
- Q-2: Partition count is derived from the longest irreducible test, not fixed in
  advance; live full-fan-out measurement selects twelve explicit partitions plus
  catch-all.
- Q-3: Partition membership and timing inputs are checked in, deterministic, and
  protected by complete/disjoint coverage checks.
- Q-4: G3 and G4 fan out internally while preserving their aggregate required-check
  names.

## Out of Scope

- Changing Thermite language semantics, RFC-10 assurance claims, or
  certification levels.
- Weakening, sampling, quarantining, or conditionally skipping proof-backed
  tests to meet a wall-clock target.
- Treating GitHub queue delay as an execution improvement.
- Replacing pinned proof tools or changing their trust boundary.
- Optimizing developer-local incremental builds beyond reusing the segment and
  partition selectors introduced here.
