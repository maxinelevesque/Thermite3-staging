# Feature: RFC-10 checked traversal and proof-carrying interpretation

<!--
audited-content-sha256: f376ba8d0728b41d824b1dd13139c9e5cf4baa51d5933cfd2aea9ec4216d062f (pinned 2026-08-14 after RFC-10 revision 2 landed; covers the canonical semantic inventory, checked IR, lock lowering, and deterministic witness producer.)
-->

## Summary

This amendment makes RFC-10 semantics structural rather than pass-local. A
single canonical child relation covers every executable statement, expression,
pattern, match guard, condition, and block; validation produces a checked IR
that all Forge and lowering paths consume; and a small verified checker validates
proof-producing analysis evidence against the canonical source AST. `holding`
keeps uniform lexical-block semantics throughout the existing expression
language, with statement-only placement reserved solely as a fallback if the
uniform semantics encounters a demonstrated proof obstruction.

The amendment serves `telos/a-clause-is-checked`,
`telos/the-corpus-still-certifies`, `telos/surface-serves-agents`, and
`telos/residual-trust-is-named`. It responds directly to the cold-review rounds
recorded on kan subject `rfc10-impl`, which repeatedly found semantic child
positions covered by one handwritten traversal and omitted by another. The
durable claim history, rather than session-local review scratch files, is the
review provenance for this amendment.

## Requirements

- REQ-1: `thermite-syntax` shall define one canonical, exhaustive semantic child
  relation for executable `Program`, `Item`, `Block`, `Stmt`, `Expr`, match-arm,
  pattern, and loop nodes. It shall include conditions, match guards,
  expression-owned blocks, closure bodies, and every other child that can carry
  effects, bindings, control flow, or `holding`.
- REQ-2: Canonical traversal shall use an explicit worklist over stable node
  identities rather than native-stack recursion. The abstract language shall
  retain semantics over all finite syntax trees; implementation work limits
  shall yield a non-certifying `ResourceLimit` result and shall not make a
  program semantically invalid or permit assurance degradation.
- REQ-3: RFC-10 shall admit `holding` statements in every ordinary executable
  block admitted by the existing grammar, including expression-owned blocks.
  Acquisition occurs at block evaluation, the block yields only after close,
  and evaluation order is the ordinary evaluation order of the enclosing
  expression. A restriction to direct statement-control blocks requires a
  separately recorded impossibility proof under the amendment's stated
  assumptions and an explicit RFC revision; difficulty finding or mechanizing
  a proof is not sufficient evidence for weakening.
- REQ-4: Region resolution, lexical binding resolution, direct effect
  collection, transitive footprint closure, shared-place authority, lock
  ordering, direct and transitive reentrancy, escape discipline, and close-edge
  construction shall produce a `CheckedProgram`-family IR. Every accepted
  holding scope shall carry resolved lock, guarded region, lexical capability,
  held-lock transition, and leaving-edge close evidence.
- REQ-5: L1, L2, L3, provider-free `forge check`, vacuity checks, mutation
  checks, strengthening probes, and artifact builds shall consume the checked
  IR. Public compatibility entry points may accept a parsed `Program`, but must
  validate it and delegate to the checked implementation; no certifying or
  executable path may lower unchecked syntax.
- REQ-6: The Rust analysis shall emit a deterministic witness artifact covering
  the canonical AST inventory, semantic child edges, binding and region
  resolution, direct effects, call edges, transitive footprints, lock-order
  paths, lexical held-lock transitions, and normalized acquire/close edges.
- REQ-7: A small checker with correctness proved in Lean shall validate the
  witness artifact against a canonical serialization of the original AST. An
  RFC-10 L3 certificate shall require successful witness replay, so omission of
  a condition, match guard, holding scope, effect occurrence, call edge, or
  leaving close edge cannot certify.
- REQ-8: The formal development shall prove checked-IR well-formedness,
  footprint completeness, lock-discipline soundness, and exactly-once close
  coverage over finite syntax and finite graphs. The proof may use structural
  induction, finite-worklist termination, finite-lattice fixed points, and
  graph path induction without imposing a language-level depth bound.
- REQ-9: The implementation shall generate a traversal-conformance matrix that
  places direct reentrancy, reverse ordering, transitive reentrancy, shared
  reads/writes, non-Copy moves, escaping borrows, invariant-breaking mutations,
  and deep finite expressions into every compatible semantic child slot and
  compares outcomes across analysis, checked IR, replay, L1/L2/L3, and Forge.
- REQ-10: The RFC and residual-trust statement shall maintain a delta ledger for
  language expressiveness, provability, metatheory, backend completeness,
  compatibility, performance limits, and trusted correspondence. Future
  interpretation-proof strengthening may reduce trust without changing the
  language surface or invalidating source programs.

## Acceptance Criteria

- [x] AC-1: (REQ-1) The canonical child relation and inventory fixtures cover
  every current executable AST variant and explicitly pin `Stmt::If.cond`,
  `LoopKind::While`, `MatchArm.guard`, closure parameters/body, patterns, and
  expression-owned blocks. Adding a new AST variant fails the exhaustive
  semantic match; adding a field to an existing variant still requires review
  of the explicit inventory fixtures and is not claimed to fail compilation.
- [x] AC-2: (REQ-2, REQ-8) Parsed finite fixtures with deep binary chains, deep
  references, deep match guards, and deep expression-owned blocks complete via
  iterative traversal or return `ResourceLimit`; no analysis, holding detection,
  checked-IR construction, replay, or lowering path panics, overflows the native
  stack, or silently skips the input.
- [x] AC-3: (REQ-3) Differential fixtures place `holding` in function bodies,
  branches, initializers, assignments, returns, tails, conditions, match arms,
  match guards where the grammar admits blocks, closures, callees/arguments, and
  loop tests; supported forms have identical authority, order, acquisition, and
  close semantics across L1 and L3, while grammatically impossible forms reject
  during parsing rather than acquiring special semantic exceptions.
- [x] AC-4: (REQ-4) Checked-IR tests show each holding scope carries a resolved
  lock and guarded region, each shared access names its authorizing capability,
  every lock-stack transition satisfies non-reentrancy and `after`, and every
  leaving edge names exactly one inner-to-outer close sequence.
- [x] AC-5: (REQ-5) Structural call-path tests demonstrate that every Forge and
  lowering entry point reaches the same checked-IR constructor; `CheckedProgram`
  fields remain private with validation as their only public construction path,
  and an invalid source receives the same rejection cause through provider-free
  checking and artifact builds. This criterion does not claim a source-mutation
  test for replacing an entry point with raw lowering.
- [x] AC-6: (REQ-6) Witness artifacts are deterministic under repeated runs,
  bind the canonical serialized AST, enumerate every semantic node and edge,
  and change when any binding, effect, call, holding, order, or close fact changes.
- [ ] AC-7 (deferred expansion): (REQ-7, REQ-8) Lean replay accepts witnesses for valid nested and
  expression-position holding programs and rejects independently mutated
  witnesses that omit or forge a condition child, match guard, direct effect,
  call edge, transitive owner, lock-order edge, capability, or close edge; the
  axiom probe remains within the repository allowlist.
- [ ] AC-8 (deferred expansion): (REQ-8) Lean proves termination and soundness of the witness checker
  over finite ASTs, finite region/lock/call graphs, and finite effect lattices,
  and states separately that operational work-budget exhaustion is not a source
  typing or semantic judgment.
- [ ] AC-9 (deferred expansion): (REQ-9) The generated cross-product suite covers every compatible
  payload/position pair and asserts phase agreement; deleting any canonical
  child edge produces at least one demonstrated failure, and the suite preserves
  existing corpus certificate outcomes item by item.
- [x] AC-10: (REQ-10) The amended RFC contains a reviewed delta table and named
  residual-trust section; requirement views distinguish language support from
  backend support and proof from trusted correspondence, and record the future
  interpretation-strengthening target without requiring a source-language
  change.

### Implementation-pass status

Revision-2 slices 1 through 8 and the rung-4 extension are implemented. AC-1
through AC-6 are closed. AC-7 through AC-9 remain deliberately unclaimed at
their original, stronger wording and are scoped as post-RFC-10 expansion work;
they are not premises of the evidence-completeness theorem or of the current L3
route. GitHub issue #49 owns that bounded RFC-10 expansion; issue #48 owns the
language-wide versioned-completeness discipline. Forge now renders separate
canonical-AST and production
witness values into the finite Lean checker and requires the kernel to establish
`verify ast witness = true`; producer-supplied direct effects and calls are
checked against an independently constructed syntax projection and Lean
recomputes their fixed-point closure. Mutation pins cover structural identity,
direct and transitive footprints, added and omitted call edges, holding and
shared-place coverage, capabilities, close evidence, and authority evidence.

AC-7 and AC-8 are deferred at their full wording: the canonical projection does
not yet derive the exact expected lock/region/order/close and shared-place
authority records for Lean to compare field-for-field, and the formal theorem
therefore establishes the executable checker's present predicates rather than
the full checked-IR semantics claimed by REQ-8. The generated matrix crosses
ten expression positions with shared reads/writes, direct reentrancy, reverse
ordering, transitive reentrancy, unauthorised access, and deep finite terms; the
non-Copy move, escaping-borrow, and invariant-restoration payloads remain in
focused tests rather than the generated cross-product, and the matrix does not
yet invoke every L2 and Forge route. The corpus-preservation clause of AC-9 is
complete through the frozen 18-file certification-level sweep, but that result
does not discharge the missing generated cross-product. AC-10 is closed by the
final independent cold-review decision
`bafyreick7fyhphabbxq7bb7eehdmehm4duvjmehkhic3cfwv5kxu57zihm` on kan subject
`rfc10-impl` and the follow-up closure pass described below.

Cold review round 4 found and the subsequent fix round closes three production
completeness regressions. The canonical inventory now retains string allocation,
full call paths, and lexical binding introductions; optimized effect analysis
must agree with that independent projection on every `CheckedProgram` build.
Forge treats shared/lock/concurrent declarations as whole-program metadata
rather than isolated certificate items, and `conformance/shared_state_rfc10.th`
pins a real L3 certificate through the production route. A frozen 18-file root
corpus sweep now checks item identities and certification levels, allowing only
named tool-unavailable downgrades. Lean also binds each holding node to the exact
canonical lock name. Exact guarded-region, order, close normalization, and
shared-place authority binding remain the honest AC-7/8 residual tracked by
GitHub issue #49.

The following orthogonal completeness review found five further
evidence-integrity defects, all closed in the implementation fix round: corpus
expectations now distinguish the solver-equipped L4 result from the stable L3
fallback without downgrading a Verus proof when EPR is unavailable; Lean replay
requires a content-addressed in-tree checker, bounded execution, an explicit
artifact theorem token, and an axiom/no-`sorryAx` probe; shared observations in
pre-acquisition `requires` are rejected rather than rewritten from a later
holding snapshot; an explicit L2 request cannot succeed with an empty
certificate array; and equivalent-mutant exclusions are surfaced in the
certificate manifest. Generated replay additionally proves
`producerRefines ast witness`, comparing structural and payload fields exactly
and transitive footprint effects setwise because their list order is not
semantic. These closures require an independent cold acceptance review before
AC-10 may close; the following paragraph records that review and its closure
pass.

The final independent review returned **APPROVE WITH FOLLOW-UPS**. Its two
behavioral findings are closed: every optional EPR tool absence, version
mismatch, or reconstruction timeout preserves an independently clean L3 base;
and shared-root observations in `requires` or `ensures` now receive a typed
`SharedStateInContract` validation error before any vacuity harness. Its ledger
findings are also closed by the field-specific independence and replay-executable
trust statements below, and by narrowing AC-1/AC-5 to the enforcement evidence
that exists. The requirements gate already returned nonzero on missing
`tomllib`; an oracle test now pins that behavior. Focused RFC-10 suites, the Lean
axiom probe, the frozen corpus, all 527 Forge unit tests, clippy, formatting, and
the 553-requirement/124-view registry pass. The complete workspace run has only
the twelve previously identified issue-#44 failures across four build-attestation
test binaries on macOS arm64, all caused by the Rust component manifest lacking
a standalone `libLLVM`; the independent review established that failure is
downstream of and unrelated to RFC-10.

## Architecture

### Amendment status and precedence

This document amends `.design/rfcs/0010-shared-state-invariants.md` and the
implementation contract in `.design/syntax/shared-state-invariants.md`. Where
the earlier documents describe independent effect, holding-detection, or
lowering traversals, this amendment takes precedence: semantic discovery occurs
once through the canonical child relation, semantic acceptance occurs once in
checked-IR construction, and all later phases consume that result.

### Canonical syntax graph

`thermite-syntax/src/ast.rs` remains the parsed surface representation but gains
stable per-program node identities or an arena projection. A canonical
`semantic_children(node)` relation is the sole inventory of children relevant
to binding, effects, control flow, or lowering. It includes wrapper records such
as match arms and loop kinds rather than relying only on exhaustive matches over
`Expr` and `Stmt`; the three reviews showed that enum exhaustiveness does not
detect an omitted field inside a matched variant.

The traversal engine uses an explicit worklist. Consumers receive enter/leave
events plus lexical context rather than recursively rediscovering children.
Binding-sensitive clients update a scope context; holding-sensitive clients
update a persistent held-lock stack; transformation clients record results by
node identity and construct output after child results are available.

The common traversal may be adopted incrementally, but RFC-10 consumers are the
first mandatory adopters. The infrastructure is repository-wide so later
effect, dependency, address, and clause passes can share the same child
definition instead of creating another feature-local visitor.

### Finite semantics and operational resources

The metatheory ranges over finite ASTs and finite program graphs without a
maximum nesting depth. Read-only traversals terminate because a finite node
inventory is consumed by an explicit worklist. Context-sensitive traversals use
a finite state space derived from syntax nodes, lexical scopes, locks, regions,
and effects. Transitive footprints are a monotone fixed point over a finite
powerset lattice and lock-order validation is finite graph reachability.

The implementation may cap source bytes, AST nodes, worklist states, witness
size, or solver resources. Exhaustion yields a structured, non-certifying
`ResourceLimit` and never falls through to L2/L1 assurance. Such limits are tool
budgets, not language validity judgments, so changing them affects availability
but not the meaning of a program that is checked successfully.

### Uniform lexical-block semantics

`holding` remains a statement, and every ordinary executable block uses the
same statement semantics regardless of whether the block belongs to a function,
statement branch, block-valued `if`, match arm, closure, condition, argument, or
other expression. Acquisition occurs when evaluation reaches the statement.
The holding body evaluates under its resolved lexical capability; all leaving
edges close inner-to-outer; only then may the block yield its value to the
enclosing expression.

This preserves expression-oriented composition and refactoring stability. It
also requires evaluation order to be preserved by checked-IR lowering. L1's
  current inability to lower expression-position holding remains an implementation
  gap to close, not a language restriction. The fallback requires an impossibility
  proof for the uniform semantics under the stated language and backend
  assumptions—not backend inconvenience, proof complexity, a missing lemma, or
  failure to find a mechanization path—and an explicit RFC revision restricting
  holding to direct statement-control blocks.
No position-by-position intermediate subset is permitted because that recreates
the omission class this amendment removes.

### Checked IR and phase agreement

A `CheckedProgram` family sits between parsing and every certifying or executable
consumer. It retains the source node identity and adds resolved facts. A checked
holding node includes its lock, guarded and invariant regions, lexical
capability, incoming and outgoing held-lock states, and normalized close edges.
A checked shared place includes its resolved region, access mode, authorizing
capability, value/borrow discipline, and source identity. Checked calls carry
resolved callees and their direct/transitive footprints.

`thermite-lower/src/effects.rs`, `thermite-lower/src/locks.rs`, and
`thermite-spec/src/regions.rs` currently split those facts across independent
walks. The revised architecture moves fact construction behind one fallible
boundary. `thermite-lower/src/l1.rs`, `thermite-lower/src/l2.rs`,
`thermite-lower/src/lower.rs`, and `forge/src/check.rs` consume checked nodes.
Compatibility functions accepting `&Program` invoke validation internally and
cannot expose a raw-lowering branch.

### Proof-producing analysis and verified replay

The production Rust analysis remains responsible for efficient construction,
diagnostics, and integration, but its output is not trusted merely because its
fields are self-consistent. It emits a deterministic witness bound to a
canonical serialization of the source AST. The witness inventories source nodes
and semantic edges and supplies evidence for resolution, direct effects, call
edges, fixed-point closure, order paths, capability scopes, and close coverage.

A small verifier modeled and proved correct in Lean checks the witness against
the canonical AST. Verification is evidence checking rather than re-running the
optimized Rust implementation: every source node and relevant child must be
accounted for, direct occurrences must have witnesses, closure sets must contain
the required consequences, and every leaving edge must carry the right close
sequence. Forge includes successful replay in the L3 certification premise.

This is stronger than proving the frame theorem over Rust-supplied footprints:
an omitted match guard or condition makes the witness incomplete and replay
fails. It avoids proving line-by-line simulation of Rust maps and ownership
machinery. Parser-to-canonical-AST decoding, the Lean kernel/allowlisted axioms,
the replay invocation, and target provider behavior remain named trust
boundaries.

After this amendment merges, a future proof project may mechanize or replace
more of the source interpretation, including a proved AST-to-checked-IR
function or generated implementation. That can reduce the decoding and Rust
correspondence boundary without changing `lock`, `holding`, `owns`, shared-place
semantics, or any source program.

### Traversal-conformance matrix

Tests are generated from two explicit inventories: semantic payloads and child
slots. Payloads include every RFC-10 obligation class plus a deep finite term.
Slots come from the canonical child relation. Compatibility rules exclude only
grammar/type-impossible combinations and are themselves reviewed data.

Each fixture is observed at parser, analysis, checked-IR, witness replay, L1,
L2, L3, and Forge boundaries. The expected relationship is phase agreement,
not identical success: for example an operational backend may return a named
unsupported result only where the requirements still declare a backend gap, but
it may not accept semantics the checker rejected or erase a construct the
checker observed.

### Slice-checkpoint roadmap

The revision is implemented in dependency order, and each slice ends with a
kan result carrying commands and artifacts the next slice can verify.

1. **Requirement registration and baseline:** register REQ-1 through REQ-10,
   regenerate views, freeze current corpus outcomes, and retain the three cold
   reports as negative-design evidence.
2. **Canonical node inventory:** add stable node identities, wrapper-node
   dispositions, the exhaustive semantic-child relation, and the inventory gate
   in `thermite-syntax`; migrate no semantic consumer until the inventory itself
   is independently tested.
3. **Iterative traversal and resource model:** implement enter/leave worklist
   traversal, lexical contexts, deterministic work accounting, and structured
   non-certifying `ResourceLimit`; demonstrate deep parsed trees cannot overflow
   in inventory or traversal.
4. **Checked IR construction:** define checked regions, bindings, shared places,
   calls, holdings, capabilities, held-lock transitions, and close edges; migrate
   RFC-10 analysis from independent walks and prove phase-equivalent diagnostics
   on the existing focused suite.
5. **Backend convergence:** migrate L1, L2, L3, Forge per-item checking, vacuity,
   mutation, strengthening, and artifact builds to checked IR; implement uniform
   expression-position holding in L1 and assert that no raw-lowering route
   remains.
6. **Witness format and Rust producer:** define canonical AST serialization and
   deterministic evidence for node/edge coverage, resolution, footprints,
   ordering, capabilities, and close normalization; add mutation-negative format
   and binding tests.
7. **Verified witness checker:** implement the small Lean checker and prove
   finite-worklist termination, footprint completeness, lock-discipline
   soundness, and close coverage; connect successful replay to L3 certification
   and retain exact residual-trust accounting.
8. **Generated conformance and closure:** generate the payload-by-position
   matrix, run corpus outcome comparison and repository gates, update the delta
   ledger and requirement statuses, then request a fresh cold adversarial review
   before stage gate.

No slice may weaken uniform block semantics because its backend or proof work is
difficult. A statement-only revision is a separate design event requiring the
impossibility evidence specified by REQ-3.

### Language and assurance delta ledger

| Dimension | Before amendment | After amendment | Tradeoff |
|---|---|---|---|
| Expressiveness | Lexical holding intended broadly but backend coverage varied by position | Every ordinary executable block has uniform holding semantics | L1 must gain expression-aware normalization; no source narrowing |
| Provability | Frame consequences proved over assumed semantic fields | Source-bound witnesses establish completeness and lock/close facts before consequences | More proof artifacts and replay work; materially smaller trusted checker boundary |
| Metatheory | Informal structural traversal and ad hoc implementation depth bound | Finite-tree, finite-graph, finite-lattice semantics with explicit termination arguments | Requires canonical node/child formalization |
| Compatibility | Public lowering APIs accept parsed programs and may rediscover facts | Compatibility wrappers validate; internal lowerers require checked IR | Internal API migration, source language unchanged |
| Backend completeness | L3 covers expression holding; L1 fails closed outside statement positions | L1 and L3 preserve holding in the generated expression-position matrix | L2 and Forge agreement still rely on focused route tests rather than the full generated cross-product |
| Resource behavior | Recursive passes may overflow or use inconsistent depth limits | Iterative traversal plus deterministic operational work limits | Possible resource rejection of very large valid programs, never certification |
| Residual trust | Rust footprint and lowering correspondence largely nominal | Canonical decoding, replay invocation, kernel/axioms, and provider remain named | Future proof strengthening can shrink this without surface change |

## Residual trust

- The parser and canonical AST decoder are trusted to represent the source
  faithfully until a future interpretation-strengthening project proves or
  replaces that boundary.
- The production Rust analyzer constructs the witness, while a separate Rust
  canonical projection constructs the syntax-side node inventory, direct
  footprints, free-function call graph, holding-node inventory, and
  shared-place-node inventory. The direct-footprint and free-call legs are
  genuinely different traversals; holding and shared-place payloads are
  duplicated same-algorithm walks over the same semantic inventory and region
  index. Lean checks their rendered agreement and recomputes transitive
  footprints, but a common algorithmic error in those duplicated payload walks
  remains trusted. GitHub issue #49 owns an independently derived replacement.
- `canonical_ast_sha256` currently hashes a versioned Rust `Debug`
  representation. The version tag prevents silent format reuse, but the
  structural fidelity and stability of that representation remain trusted.
- Forge renders the decoded production witness and canonical projection as Lean
  values and asks Lean's kernel to elaborate a proof that the executable
  `Thermite.CheckedTraversal.verify` returns true. Forge binds the runtime
  checker file to the copy embedded at compilation, imposes a 60-second replay
  deadline, and accepts only an exit-zero run carrying the versioned positive
  token plus both artifact theorem axiom reports with no `sorryAx`. The bridge,
  token/report parser, and Lean string/record renderer remain trusted transport
  code; missing or changed checker content, failure, timeout, malformed output,
  a forbidden axiom, or an unsolved goal is non-certifying. The executable named
  by `THERMITE_LEAN_LAKE` (or `lake` resolved from `PATH`) is trusted with the
  replay invocation: control of either can forge the accepted output protocol.
- Guarded-region identity, held-set/lock-transition contents, lock-order paths,
  close-edge normalization, shared-place path/mode, and authorizing-lock identity
  are required to equal the canonical-projection payload exactly. This rejects
  unilateral transport/serialization forgery; it does not reject a coordinated
  same-algorithm error in the two Rust payload walks.
- The Lean kernel, the repository's explicit axiom allowlist, and the witness
  checker's formal statement are trusted in the same sense as the existing
  proof spine.
- Rust, Verus, and backend code generation are trusted to execute the checked
  IR emitted for them; the phase-agreement and exact-source gates test this
  correspondence but do not constitute a proof of the compilers.
- The target provider remains trusted for exclusive acquisition, release,
  memory ordering, interrupt masking, storage identity, and the truth of its
  platform attestation. The verified witness checker proves language-side lock
  and close discipline, not hardware behavior.
- Operational resource policy is trusted only for availability. Exhaustion is
  required to be non-certifying, so it cannot strengthen an assurance claim.

## Resolved Questions

- The canonical child relation is repository-wide infrastructure, with RFC-10
  consumers required to adopt it first.
- Lowering consumes checked IR; parsed-program entry points are compatibility
  wrappers that validate rather than alternate unchecked implementations.
- `holding` has uniform semantics in every ordinary executable block. A
  statement-only restriction is acceptable only after an impossibility proof
  under the stated assumptions and a new explicit RFC decision; absent such a
  result, proof work continues until a mechanization path is found.
- Thermite's abstract semantics covers all finite syntax trees. Native-stack
  depth is not a language property; operational resource exhaustion is a
  structured non-certifying outcome.
- RFC-10 targets proof-producing Rust analysis plus a small verified witness
  checker, rather than a line-by-line proof of the optimized Rust implementation.
- A future project may strengthen the interpretation proof, including a proved
  AST-to-checked-IR transformation, without changing the language surface.

## Out of Scope

- Changing the `lock`, `holding`, `owns`, `shared`, or `keeps` source spelling.
- Restricting holding to a hand-maintained subset of expression positions for
  backend convenience.
- Proving the source parser, UTF-8 lexer, Rust compiler, Verus implementation,
  Lean kernel, or target provider correct in this amendment.
- Proving optimized Rust data structures line-by-line equivalent to Lean data
  structures; the source-bound witness checker is the correspondence boundary.
- RFC-12 rely/guarantee clauses, lock-free interference, dynamically selected
  locks, flow-sensitive acquire/release statements, or production Bulla lock
  primitives.
- Treating operational work-budget exhaustion as a source-language type error
  or permitting it to degrade into a lower-assurance certificate.
