# Feature: RFC-10 shared-state invariants

## Summary

RFC-10 adds declared locks, lexical `holding` blocks, transitive `owns` effects,
lock ordering, and interrupt-lock discipline on top of RFC-9's checked shared
regions. It also implements the settled Tier A relational frame lemma so effect
rows yield compositional non-modification and determinism facts, while concrete
lock and interrupt operations remain explicit target-provider responsibilities.

The implementation serves `telos/a-clause-is-checked`,
`telos/the-corpus-still-certifies`, `telos/surface-serves-agents`, and
`telos/residual-trust-is-named`: lock claims become checked in both directions,
the existing corpus retains its assurance outcomes, the surface uses full words,
and the runtime provider boundary is named rather than silently trusted.

Revision 2 is governed by `.design/rfc10-checked-traversal.md`. Its canonical
child relation, checked-IR boundary, uniform executable-block semantics,
finite-tree metatheory, proof-producing analysis, and verified witness replay
supersede the pass-local traversal assumptions in the implementation checkpoints
below. Checked boxes in this document describe the pre-amendment implementation
evidence. Revision-2's implementation pass is complete, but its governing
acceptance criteria remain authoritative: independent verified production
replay and the full generated payload/phase cross-product are still partial and
are recorded as such in the requirement registry.

## Requirements

- REQ-1: The syntax crate shall parse `lock NAME guards REGION` declarations,
  where `REGION` is an RFC-9 `RegionPath` rooted in a `shared` declaration, and
  represent the declaration explicitly in `thermite-syntax/src/ast.rs`.
- REQ-2: The syntax crate shall parse `holding LOCK { ... }` as a lexical
  statement and shall migrate struct invariant spelling from `inv` to `keeps`
  consistently across parser, diagnostics, semantic addresses, examples, and
  generated documentation.
- REQ-3: An `owns(LOCK)` effect shall be a checked public acquisition footprint:
  every direct `holding LOCK` requires it, and every declared `owns(LOCK)` must
  be justified by a direct holding block or by a transitive callee footprint.
- REQ-4: The specification layer shall resolve each guarded region through
  `thermite-spec/src/regions.rs`, reject unknown or multiply guarded roots where
  exclusivity would be ambiguous, and reject `read` or `write` access to a
  guarded overlapping region unless the access occurs under its declared lock.
- REQ-5: A `holding LOCK` block shall assume the guarded struct's `keeps`
  predicate at entry, permit it to be temporarily false only inside the lexical
  block, and require it to be restored on every normal exit and control-flow
  edge leaving the block.
- REQ-6: The checker shall reject reentrant acquisition through the transitive
  call graph and shall permit at most one simultaneously held lock unless a
  declaration-level `after` relation authorizes the nesting; the `after` graph
  must be acyclic and the lexical nesting must follow it.
- REQ-7: If a handler-visible function owns a lock, normal-context functions
  that own the same lock shall also declare `owns(interrupts)`; `interrupts` is
  statically reserved in RFC-10, while executable masking remains provider-owned.
- REQ-8: L1 and L3 lowering shall emit real acquire/body/release operations for
  `holding`; a build containing `holding` shall fail closed when no explicit
  target lock provider is supplied, and repository tests shall use an explicit
  non-production provider.
- REQ-9: The target-provider contract shall state that acquisition establishes
  exclusive access and exposes the guarded invariant, release requires the
  invariant restored and relinquishes access, and interrupt masking and memory
  ordering are target policy rather than Thermite-owned implementations.
- REQ-10: The formal layer shall prove the Tier A relational frame lemma over
  the canonical RFC-9 effect footprint and write footprint, then expose derived
  non-modification and determinism facts to contract checking without new
  surface clauses.
- REQ-11: Calls inside a `holding` block shall use the relational frame result
  compositionally: a callee may change only its declared write footprint, must
  preserve unrelated regions, and must return with every guarded invariant it
  was permitted to open restored.
- REQ-12: RFC-10 shall preserve item-level certification outcomes for the
  existing corpus and shall keep RFC-12 rely/guarantee clauses, Tier B/C
  relational research, and production Bulla primitives outside this change.
- REQ-13: A declared `shared NAME: TYPE` shall introduce an ordinary place root
  in executable expressions. Local bindings and parameters shall shadow that
  root; otherwise `NAME` and its field projections resolve to provider-backed
  shared storage rather than to an effect-row-only symbol.
- REQ-14: Access to a guarded shared place shall be authorized only within the
  lexical `holding LOCK` scope for the lock whose guard overlaps the place.
  `owns(LOCK)` remains the transitive public footprint and shall not by itself
  authorize an access outside that lexical scope.
- REQ-15: RFC-10 shared-place operations shall permit Copy reads and in-place
  assignment but shall reject moving a non-Copy value out of shared storage or
  allowing a reference derived from shared storage to escape its `holding`
  scope. Explicit cloning may produce an owned value when the type supports it.
- REQ-16: Lowering shall normalize every control-flow edge leaving `holding`
  through one compiler-generated close operation whose proof precondition is
  the guarded struct's `well_formed()` predicate and whose executable effect is
  provider release. Fallthrough, `return`, `break`, `continue`, error
  propagation, and unwind behavior shall not carry separate restoration rules.

## Acceptance Criteria

- [x] AC-1: (REQ-1) Parser tests accept locks guarding a shared root and nested
  field region, retain exact spans, and reject a type name or unknown region as
  a guard target with a structured diagnostic.
- [x] AC-2: (REQ-2) Parser and address tests accept `keeps` on structs and
  `holding LOCK { ... }`; a repository scan and migration inventory show no
  live Thermite source, generated surface, or diagnostic that still presents
  `inv` as the struct-invariant spelling.
- [x] AC-3: (REQ-3) Focused checker tests cover direct ownership, conditional
  holding, transitive callee ownership, missing row entries, and excess
  `owns` entries, with stable structured diagnostics for both mismatch
  directions.
- [x] AC-4: (REQ-4) Region-discipline tests reject unlocked reads and writes at
  the guarded root and its overlapping descendants, accept disjoint-region
  access, and accept access under the correct lock.
- [x] AC-5: (REQ-5) Verification fixtures include a critical section that
  temporarily violates and then restores `keeps`, plus failing fixtures for a
  normal exit, `return`, `break`, and error path that leave it false.
- [x] AC-6: (REQ-6) Call-graph tests reject direct and transitive reentrancy;
  ordering tests reject unapproved nesting, reversed `after` nesting, and an
  `after` cycle while accepting a declared acyclic nesting.
- [x] AC-7: (REQ-7) Handler-composition tests reject an unmasked normal-context
  owner of a handler-visible lock and accept the same program with
  `owns(interrupts)`; no test claims that Thermite itself masks interrupts.
- [x] AC-8: (REQ-8, REQ-9) `forge check` succeeds without a runtime provider,
  `forge build` fails with a named missing-provider diagnostic, and builds with
  the test provider execute exactly one acquire and one release on every normal
  critical-section path.
- [x] AC-9: (REQ-9) Provider conformance tests reject a provider contract that
  omits exclusivity, restoration-before-release, or interrupt-policy evidence,
  and the emitted manifest or certificate names the selected provider.
- [x] AC-10: (REQ-10) Lean proves the relational frame theorem from the same
  canonical basis footprints consumed by `thermite-spec/src/effect_commutation.rs`;
  an axiom probe confirms it introduces no axioms beyond the project's allowed
  set.
- [x] AC-11: (REQ-10, REQ-11) Differential fixtures prove equal results for a
  pure function, equality outside a callee's write footprint, and preservation
  of an unrelated guarded region across a call inside `holding`; mutations that
  write an undeclared or unrelated region are rejected.
- [x] AC-12: (REQ-12) The focused RFC-10 suites, formatting, clippy, routing,
  requirement registry, RFC lint, and scoped document drift pass, and the
  pre-existing corpus certificate comparison preserves every item-level outcome.
- [x] AC-13: (REQ-13) Resolution tests show an unshadowed shared root and nested
  fields become shared places, while a same-named parameter or local remains an
  ordinary local place; unknown shared fields retain structured region errors.
- [x] AC-14: (REQ-14) Checker tests accept guarded reads and writes inside the
  matching `holding`, reject the same accesses before and after the block, reject
  access under the wrong lock, and show that a bare `owns(LOCK)` declaration does
  not authorize body access.

- [x] AC-15: (REQ-15) Type/escape tests accept scalar reads, assignments, and an
  explicit clone; reject a move of a non-Copy field, returning or storing a
  shared-derived reference, and using such a reference after the holding scope.
- [x] AC-16: (REQ-16) L3 fixtures prove restoration on fallthrough and reject
  unrestored `return`, `break`, `continue`, and error-propagation exits. L1
  execution fixtures observe one release on each corresponding executable exit
  and on panic unwinding, with no double release.

## Architecture

### Phase-two implementation checkpoint 2 — shared-place resolution

Implemented and verified on 2026-08-12: executable path/field trees rooted at an
unshadowed `shared` declaration resolve to a distinct `RegionPath` during effect
analysis. Reads and assignment targets infer exact `read` and `write` effects;
parameters and sequential local bindings shadow shared roots. A guarded place is
accepted only while the matching lock is on the lexical held-lock stack, so a
declared `owns` without `holding` and a wrong-lock holding both reject. Ten
focused RFC-10 lowering tests pass, including the new resolution, inference,
shadowing, outside-scope, and wrong-lock cases.

### Phase-two implementation checkpoint 3 — affine shared-place operations

Implemented and verified on 2026-08-12: shared scalar and recursively Copy
places may be read by value; non-Copy places require an explicit clone and may
still be replaced by in-place assignment. Direct moves from provider storage and
references whose provenance is a shared place reject before lowering. Focused
tests cover Copy reads, non-Copy clones and assignments, rejected moves, and
rejected escaping references.

### Phase-two implementation checkpoint 4 — normalized close edges

Implemented and verified on 2026-08-12: L3 preprocessing materializes target
acquire and close calls on normal fallthrough, after evaluation of a holding
tail, before `return`, and before `break`/`continue` when the loop edge actually
leaves the holding scope. Nested close calls run inner-to-outer. The L1 drop
guard remains the unwind backstop and calls the invariant check before provider
release. Executable fixtures observe one acquire/release pair on fallthrough,
return, break, continue, and a panic with a restored invariant; a deliberately
broken invariant fails its close check and never calls release. The language has
no separate `?` statement node: an explicit propagated `Err` is an ordinary
return and follows the same normalized edge.

### Phase-two implementation checkpoint 5 — provider-backed L1/L3 storage

Implemented and verified on 2026-08-12: a provider-backed L3 acquisition yields
one lexical mutable capability for the guarded shared root. Every place access
inside the holding scope is rewritten through that same capability, and close
consumes a mutable borrow of it under the provider-declared `well_formed`
precondition. Strict Verus fixtures accept restored fallthrough, return, break,
and continue edges and reject a mutation that leaves each corresponding edge
unrestored. Provider-free artifact lowering and providers without L3 declarations
fail closed. L1 retains target storage accessors plus close-before-release RAII;
the repository test provider now constructs supported shared values explicitly
inside `UnsafeCell` storage instead of assuming arbitrary types are valid when
zeroed.

### Phase-two implementation checkpoint 6 — integration and trust closure

Completed on 2026-08-12: all 16 RFC requirements and acceptance criteria are
implemented. Formatting, workspace compilation, all-target clippy, the focused
syntax/spec/lowering suites, strict Verus positive and mutation-negative shared
state fixtures, RFC lint, route coverage, path existence, RFC-9 effect inventory,
requirement lint/registry, scoped document drift, the Lean build/axiom probe, and
the existing golden certificate conformance suite pass. The full workspace test
run reached only the previously recorded unrelated local failures in the REQ-8
arithmetic trust expectation and live Lean/EPR replay environment before it was
stopped during the long replay tail; no RFC-10 test failed. The residual trust
boundary is explicit: a production target owns the lock/capability implementation,
interrupt and memory-order policy, and the truth of its provider evidence;
Thermite checks the declared seam and proves all language-side close obligations
without claiming to implement those target primitives.

### Implementation checkpoint 1 — syntax surface

Implemented and verified on 2026-08-12: `LockDeclItem`, `Item::LockDecl`,
`Effect::Owns`, and lexical `Stmt::Holding` are explicit closed-enum variants;
the parser accepts region guards, optional `after`, ownership rows, and holding
blocks. The canonical fixed-token diagnostics now render `keeps` and `measures`,
legacy struct `inv` is rejected, the full `thermite-syntax` suite passes, and
the workspace compiles with every exhaustive consumer assigned an explicit
RFC-10 disposition. Semantic lock checking and executable provider lowering are
deliberately left to later checkpoints.

### Implementation checkpoint 2 — resolution and ownership honesty

Implemented and verified on 2026-08-12: `RegionIndex` resolves lock guards,
records `after` edges, and rejects duplicate locks, unknown guards, unknown
predecessors, and order cycles. Direct lexical holdings join the inferred effect
footprint and propagate through the existing deterministic call-graph fixed
point; missing and excess `owns` entries use the same error/warning contract as
RFC-9 state effects. Five region tests and two ownership-analysis tests pass,
including direct, transitive, missing, and excess cases. Guarded access,
reentrancy, runtime lowering, and formal frame evidence remain later checkpoints.

### Implementation checkpoint 3 — lock discipline and masking

Implemented and verified on 2026-08-12: accesses whose declared or inferred
region overlaps a guard require the corresponding transitive `owns`; disjoint
access remains available. Lexical nesting follows the declared `after` edge,
direct repeated holding rejects, and a call lexically beneath `holding` rejects
when the callee's fixed-point footprint reacquires the held lock. The contextual
`handlers { f at N }` surface reuses the named-composition metadata and makes
handler-visible locks require `owns(interrupts)` in normal context. Six focused
analysis tests cover bypass, disjointness, both nesting directions, direct and
transitive reentrancy, and masked/unmasked handler sharing. Executable masking
remains target-provider evidence rather than a static-check claim.

### Implementation checkpoint 4 — invariant and provider boundary

Implemented and verified on 2026-08-12: lock guards now require an enclosing
invariant-bearing struct and overlapping guards reject as ambiguous. Executable
L1 lowering fails closed without a provider, validates named evidence for
exclusive acquisition, restoration-before-release, and interrupt policy, and
uses a lexical drop guard so release occurs on normal and early exits. Forge
accepts the explicit non-production `--lock-provider test` integration, records
its identity in the build manifest, and leaves production mappings target-owned.
The compiled conformance fixture observes exactly one acquire and one release.

### Implementation checkpoint 5 — Tier A relational frame

Implemented and verified on 2026-08-12: `Thermite.EffectRows.relational_frame`
proves result congruence on the canonical footprint and non-modification outside
the canonical write footprint, with `pure_deterministic` and
`outside_write_equal` as direct consequences. The production Rust consumer
queries frameability from the same resolved write/overlap relation, and explicit
ownership now conflicts with concurrent guarded access. The Lean axiom probe is
green for all three new theorems with no expansion of the allowed axiom set.

### Implementation checkpoint 6 — integration hardening and open boundary

Implemented and verified on 2026-08-12: formatting, workspace compilation,
all-target clippy, focused RFC-10 suites, RFC lint, route coverage, path checks,
effect inventory, requirement validation, the Lean axiom probe, and routed
document drift are green. L3 artifact lowering now rejects `holding` rather
than erasing synchronization, and the CLI rejects an L1-only test-provider flag
on an L3 build. Two acceptance obligations remain deliberately open: the
current source language has no expression that denotes a declared `shared`
cell, so it cannot yet construct AC-5's temporarily-broken shared invariant
fixtures; and no target-owned L3 provider mapping exists in this repository, so
provider-backed L3 artifacts cannot yet satisfy AC-8/AC-9 beyond fail-closed
behavior. These are recorded as boundary gaps, not represented as passes.

### Surface and AST

`thermite-syntax/src/ast.rs` gains a lock declaration carrying a name, guarded
`RegionPath`, optional `after` predecessor, and span. `holding` is a dedicated
statement node rather than a call or an inferred whole-function scope. The
effect AST gains `owns` as a region-like lock acquisition atom and reserves the
special target capability `interrupts`.

`thermite-syntax/src/parser.rs` parses both forms contextually. The existing
`StructItem.keeps` field becomes truthful at the source boundary: the parser
accepts `keeps`, not `inv`, and the RFC-10 migration updates all governed
examples and generated surfaces. Semantic addressing in
`thermite-syntax/src/address.rs` continues to call the invariant family
`keeps`, so the migration removes an existing parser/address vocabulary split.

### Resolution and checked ownership

`thermite-spec/src/regions.rs::RegionIndex` remains the sole authority for
shared roots, field-derived containment, overlap, and resolved guarded types.
A sibling lock index maps lock names to resolved region paths and validates the
`after` DAG. Guards name regions rather than types because the effect checker
reasons about region identity; two shared instances of one type must not become
implicitly guarded by the same declaration.

The effect analysis extends RFC-9's direct and transitive footprint machinery.
`holding LOCK` contributes a direct acquisition. Calls contribute their public
transitive `owns` footprint. The declared row is checked as the upper bound in
both directions, preserving RFC-9's rule that conditional execution may make a
declared effect absent on one run while an unreachable acquisition is not a
license for arbitrary excess declarations.

A lexical held-lock stack checks unlocked guarded access, direct and transitive
reentrancy, and nesting order. The default maximum is one held lock. An `after`
edge permits only the named direction and the declaration graph must remain
acyclic.

### Invariant obligation

The guarded region resolves to a struct whose `keeps` predicate is the resource
invariant. Entry to `holding` exposes that predicate as an assumption under the
provider's exclusivity guarantee. Every edge leaving the block owes the
predicate. The implementation must enumerate normal fallthrough and early
control-flow edges rather than checking only the final expression.

Calls within the block are modular. Their ordinary contracts and checked effect
footprints constrain what they may change; the relational frame theorem proves
that state outside the callee's write footprint agrees across the before/after
comparison used by the caller. A callee that opens any guarded invariant must
restore it before returning, so the caller never inherits an unclosed invariant.

### Shared places and lexical authority

`shared NAME: TYPE` participates in executable name resolution as an ordinary
place root after locals and parameters. Consequently `state.counter` keeps the
language's existing field syntax and requires no `shared::` namespace, while a
parameter or local named `state` shadows the declaration exactly as an ordinary
lexical binding would. The resolved node must retain its shared-root identity;
it cannot be reduced to a plain `Expr::Path`, because lowering and escape
analysis need to distinguish provider storage from a local value.

Authorization is lexical. The checker carries a stack of held locks and accepts
a shared-place read or assignment only when the innermost applicable holding
scope owns the lock whose guard overlaps the resolved place. A transitive
`owns(lock)` row is necessary public honesty but is never sufficient body
authority. This keeps the distinction already established in
`thermite-lower/src/effects.rs`: the row says what a caller must expect, while
the statement says where exclusivity and the invariant are available.

The first shared-place type discipline is deliberately affine. Copy fields may
be read by value and any writable place may be assigned in place. A non-Copy
value cannot be moved from provider storage, and a borrow whose provenance is a
shared place cannot outlive the holding scope or be stored into an outer
binding. An explicit clone yields an ordinary owned value when the existing type
and method rules support cloning. `thermite-spec/src/validator.rs` owns these
move/escape diagnostics; the lowerers must not repair an invalid escape by
silently cloning.

### One close operation for every exit

The verified control-flow representation introduces a compiler-generated
`close_holding(lock, place)` edge operation rather than independently weaving a
predicate assertion into each statement kind. Its proof precondition is the
guarded value's generated `well_formed()` predicate; after it, shared-derived
borrows are dead and the invariant is closed. All edges leaving the lexical
scope—normal fallthrough, `return`, `break`, `continue`, and error
propagation—normalize through this operation before their original transfer.
This makes a future control-flow construct inherit the rule by participating in
edge normalization rather than by adding another ad hoc invariant case.

In L3, the close operation is a proof boundary followed by the target provider's
release primitive. In L1, the existing `__ThermiteLockGuard` remains the unwind
backstop, while explicit normal-edge close consumes/disarms the guard so Drop
cannot release twice. Tests must cover both normal early exits and panic
unwinding. Provider acquisition maps the symbolic shared root to storage and
returns the exclusive place capability used by shared-place lowering; raw
global access is never emitted independently of that capability.

### Tier A relational frame theorem

`thermite-syntax/src/effect_basis.rs` already defines the canonical footprints,
and `thermite-spec/src/effect_commutation.rs` consumes region overlap for
concurrent composition. RFC-10 adds a Lean theorem over that same basis:

> Two executions of an item starting equal on its effect footprint produce
> equal results and remain equal outside its write footprint.

The production checker consumes theorem instances for frame/non-modification
obligations and pure determinism. The theorem creates no new clause syntax.
RFC-10 may ship immediate Tier A consequences that fall directly from the same
machinery, but it does not introduce `hides`, `varies`, `couples`, or `matches`.
Those surfaces and Tier B/C research remain governed by
`.design/research/relational-contracts.md`.

### Runtime provider boundary

Static checking and executable synchronization are separate and both required.
`forge check` needs no runtime provider. L1/L3 build lowering in
`thermite-lower/src/l1.rs` and `thermite-lower/src/lower.rs` emits target-provider
acquire and release operations around the lowered lexical body. A build cannot
erase these operations: if a program contains `holding` and no provider is
selected, Forge fails closed.

The provider seam follows the policy-injection shape already used by
`forge/src/build.rs::validate_freestanding_effects_with`. Thermite owns the
contract and validation; Bulla owns production mappings, lock storage, atomic
memory order, spin/block behavior, interrupt primitives, and platform policy.
Repository conformance uses an explicit test provider whose role is visible in
the build receipt. This is not the L1 runtime-contract fallback: synchronization
is executable semantics at every assurance level.

### Interrupt discipline

Handler metadata and transitive ownership footprints identify locks visible to
interrupt context. If a handler owns a lock, every normal-context owner must
also own the reserved `interrupts` capability. RFC-10 statically proves the
declaration is present and transitive; the target provider supplies and attests
the actual mask/unmask behavior. Certificates must distinguish these facts so a
static row check is never presented as proof that hardware interrupts were
masked.

### Evidence and migration

The implementation adds focused parser, resolution, effect, ordering, lowering,
provider, Forge, and Lean tests. The `inv` to `keeps` rewrite is mechanical but
must be inventoried because it changes source, fixtures, diagnostics, generated
documentation, and content pins. Existing certificate outcomes are compared
item by item, including existing failures, under
`telos/the-corpus-still-certifies`.

## Residual trust

- The target lock provider is trusted to implement exclusive acquisition and
  release with the memory-order behavior its attestation declares.
- Bulla is trusted to map symbolic locks and `interrupts` to the correct kernel
  objects and platform primitives; Thermite checks the provider contract and
  call placement but does not prove the hardware implementation.
- The relational frame theorem is only as sound as the canonical effect-basis
  footprints, the lowering correspondence, the Lean kernel, and the allowed
  axiom set reported by the theorem probe.
- Handler reachability and normal-versus-handler classification remain trusted
  inputs wherever they enter through target integration rather than declared
  Thermite source.

## Resolved Questions

- Struct invariants use `keeps`; RFC-10 includes the migration from the current
  parser spelling `inv`.
- A lock guards an RFC-9 shared `RegionPath`, not a type.
- `holding` is direct and lexical, while public `owns` footprints propagate
  transitively through calls.
- `holding` lowers to real operations supplied by an explicit target provider;
  Bulla owns the production implementation and a missing provider fails closed.
- `owns(interrupts)` is reserved and checked now, while executable masking is
  Bulla-owned and separately evidenced.
- RFC-10 includes the Tier A relational frame lemma and its directly useful
  consequences; RFC-12 retains rely/guarantee interference and Tier B/C
  relational work remains outside RFC-10.
- Shared declarations introduce ordinary place roots, with lexical bindings
  shadowing them; no `shared::` qualifier is added.
- Guarded shared-place access requires the matching lexical `holding` scope;
  `owns` alone is not body authority.
- Copy reads and in-place mutation are admitted first; non-Copy moves and
  escaping shared-derived references reject, while explicit cloning is allowed.
- Every leaving edge normalizes through one close operation requiring
  `well_formed()`; executable Drop is the unwind backstop, not a second semantic
  restoration rule.

## Out of Scope

- RFC-12 `interleaves`, `asks`, and `promises` semantics for lock-free sharing.
- Tier B and Tier C relational-contract research, including operation-graded
  noninterference, probabilistic coupling, sensitivity, and forall-exists
  contracts.
- A production spinlock, mutex, scheduler, memory-order policy, or interrupt
  implementation inside Thermite.
- Flow-sensitive acquire and release statements, implicit whole-function
  critical sections, lock inference, or dynamically selected lock identities.
- Reopening RFC-9's shared-region migration or changing unrelated pre-existing
  working-tree paths.
