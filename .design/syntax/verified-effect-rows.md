# Feature: Verified Effect Rows
<!--
tier: 3-component
status: implemented
audited-content-sha256: 28216e5190c5f36590cd6b25b0668731cc9954e496e33b60b98fec2fc01c4a00 (re-pinned 2026-08-27 after adding the effect-free recursion/tuple claim corpus; the RFC-9 inventory now tracks 68 `.th` files and effect semantics are unchanged. prior: 9daaee5d825e5b8a223ed6e9e360ccbb002766bf292e24ab281d66eebd85d180)
-->

## Summary

RFC-9 turns a function's mandatory effect row from an asserted upper bound into
a checked, region-sensitive account of what its body and callees may do. It adds
declared shared regions, field-derived containment, and named concurrent
compositions whose transitive footprints are checked using the commutation facts
computed by RFC-8.

The component supplies new Thermite metatheory for effect framing, propagation,
and concurrent composition. It establishes the footprint structure required by
the future relational frame lemma in
`.design/research/relational-contracts.md`, but does not implement that later
hyperproperty theorem, coupling logic, or relational-contract surface.

## Requirements

- REQ-1: **Shared declarations.** The surface accepts a top-level
  `shared NAME: TYPE` declaration. `NAME` is unique in its compilation unit,
  `TYPE` resolves through the existing declared-type environment, and every
  region path in `read(...)` or `write(...)` resolves from a shared root.
  Unknown roots and unknown fields are structured errors.

- REQ-2: **Region paths and containment.** A region is a nonempty path rooted
  at a `shared` declaration. If the root type has fields, each field extends the
  path and inherits its declared type. Containment is reflexive ancestry:
  `scheduler` contains `scheduler.runqueue`; sibling paths such as
  `scheduler.runqueue` and `scheduler.timers` are disjoint. Cyclic declared
  types do not make path resolution recurse without bound.

- REQ-3: **Containment spike.** Before production implementation, a focused
  parser, validation, and lowering spike demonstrates a declared type nested as
  another declared type's field and resolves a two-segment region path through
  it. The existing `StructItem.fields`, `FieldDef.ty`, and `Type::Named`
  representation makes the path representable; the spike must establish that
  the complete current pipeline preserves it. A failure becomes an explicit
  prerequisite in this design rather than a reason to omit containment.

- REQ-4: **Exact inferred footprint.** Each executable function has an inferred
  footprint containing region-sensitive operations from its body, its resolved
  callees, and effectful intrinsic operations. Inference preserves full region
  paths and distinguishes reads from writes. It reaches a deterministic fixed
  point over recursive call components.

- REQ-5: **Intrinsic footprints.** Every effectful intrinsic recognized by
  `thermite-lower` has one canonical footprint mapping derived from
  `thermite-syntax/src/effect_basis.rs`. Allocation writes `heap`, time reads
  `clock`, pseudorandom generation writes `entropy`, and terminal control writes
  `termios`. A newly added effectful intrinsic without a footprint mapping is a
  closed-enum or validation failure, never silently pure.

- REQ-6: **Declared/inferred comparison.** After propagation, the declared row
  is compared with the inferred footprint at full operation-and-region
  granularity. An inferred operation absent from the declaration is an error.
  A declared operation absent from the inference is a loud, span-bearing
  warning. Equality is clean. Diagnostics report normalized missing and excess
  sets in deterministic order.

- REQ-7: **Caller propagation.** The current atom-kind projection in
  `thermite-lower/src/effects.rs` is replaced for RFC-9 checking by
  region-sensitive propagation. A caller declaring `write(db)` does not cover a
  callee that writes `log`. Recursive calls, method calls, calls below control
  flow, and calls nested in expressions all participate in the same inference.

- REQ-8: **Effect algebra is authoritative.** Footprints, composition, and
  concurrent compatibility consume the algebraic entries from
  `thermite-syntax/src/effect_basis.rs` and the computed facts from
  `thermite-spec/src/effect_commutation.rs`. RFC-9 must not introduce a second
  hard-coded read/write table. An unknown or unresolved commutation result
  rejects; it is never promoted to acceptance.

- REQ-9: **Concurrent composition declaration.** The surface accepts a
  top-level declaration of the form
  `concurrent NAME { ROOT, ... }`, where `NAME` uniquely names the composition
  and every `ROOT` resolves to an executable function. This form declares that
  the roots may execute concurrently. It is distinct from RFC-12's
  function-local `interleaves { asks ... promises ... }`, which remains reserved
  for rely-guarantee contracts.

- REQ-10: **Transitive conflict check.** For each `concurrent` declaration, the
  checker compares the inferred transitive footprints of every unordered pair
  of roots. Operations on disjoint regions accept. Operations whose regions are
  equal or related by containment are decided by the RFC-8 commutation fact.
  Every rejection names the composition, both roots, both operations, and the
  overlapping region ancestry.

- REQ-11: **Kernel profile companion.** `forge build --target kernel` classifies
  state effects using resolved region identity or region classification rather
  than rejecting every `read` or `write` by leading verb. The design does not
  choose Thermite's eventual ambient-region prelude or Bulla's kernel-owned
  registry; until that policy is supplied, the kernel target must fail closed
  with a diagnostic naming the unresolved classification. It must not preserve
  the current verb-based rejection as the final RFC-9 behavior.

- REQ-12: **Migration and compatibility.** Migration covers all `.th` files,
  Thermite fragments in Rust literals, JSON conformance programs, certificate
  oracles whose `effects` field changes, and kernel-build fixtures. Existing
  `alloc`, `time`, `rand`, and `term` spellings may remain accepted during this
  increment only through their canonical basis mappings; they participate in
  honesty and conflict checking exactly like explicit state operations.

- REQ-13: **Formal boundary.** The component states and tests these Thermite
  effect principles: declared rows denote frame upper bounds; inferred
  footprints are closed under calls; containment induces overlap by ancestry;
  and concurrent acceptance is pairwise commutation over overlapping theory
  instances. Any Lean artifact needed to justify those principles belongs in
  this increment. The k-fold self-composition theorem, `hides`, `varies`,
  couplings, hyper-arity, and the certificate algebra remain governed by
  `.design/research/relational-contracts.md` and are outside RFC-9.

- REQ-14: **Determinism and failure discipline.** Region normalization,
  footprint inference, warning order, and conflict order are deterministic.
  Parse, resolution, inference, and commutation failures are structured results;
  no unresolved name, unsupported operation, recursion cycle, or unknown fact is
  treated as success.

## Acceptance Criteria

- [x] AC-1: **Shared parsing and resolution.** Parser tests accept shared roots
  over primitive and declared types, reject duplicate roots, reject unknown
  declared types, and produce structured errors for unknown region roots and
  fields. Covers REQ-1 and REQ-14.

- [x] AC-2: **Containment spike.** A fixture with `SchedState` containing a
  declared `RunQueue` field parses, validates, and lowers, and region resolution
  returns the paths `scheduler`, `scheduler.runqueue`, and a nested child with
  the expected types. A recursive type fixture terminates deterministically.
  Covers REQ-2 and REQ-3.

- [x] AC-3: **Intraprocedural honesty.** Fixtures demonstrate a clean exact
  row, a missing direct intrinsic effect as an error, and an unused declared
  effect as a warning containing the function span and normalized excess set.
  Covers REQ-4, REQ-5, REQ-6, and REQ-14.

- [x] AC-4: **Region-sensitive calls.** A caller with `write(db)` calling a
  `write(log)` callee fails with `write(log)` missing; adding `write(log)` and
  retaining unused `write(db)` produces the excess warning; exact rows pass.
  Nested-expression, method-call, branch, loop, and mutually recursive fixtures
  reach the same fixed point. Covers REQ-4, REQ-6, and REQ-7.

- [x] AC-5: **Closed intrinsic map.** A table-driven test covers every current
  `Effect` and every recognized effectful intrinsic, asserting the hand-derived
  canonical footprint. Exhaustive Rust matches make adding an unmapped variant a
  compile failure. Covers REQ-5 and REQ-8.

- [x] AC-6: **Concurrent surface.** Parser and resolver tests accept
  `concurrent shootdown { ack, complete }`, reject duplicate composition names,
  duplicate roots, unknown roots, and non-executable roots, and continue to
  reserve function-local `interleaves` for RFC-12. Covers REQ-9 and REQ-14.

- [x] AC-7: **Commutation and containment.** Hand-derived fixtures accept
  read/read on the same path and write/write on sibling paths; reject
  write/write and write/read on the same path; and reject
  `write(scheduler)` against `read(scheduler.runqueue)`. Diagnostics identify
  both roots and the ancestry overlap. Covers REQ-2, REQ-8, and REQ-10.

- [x] AC-8: **Computed facts only.** A test exercises the production conflict
  consumer through `thermite_spec::effect_commutation`; changing a basis fact in
  the test fixture changes the consumer verdict without editing a second table.
  Unknown facts reject. Covers REQ-8 and REQ-10.

- [x] AC-9: **Kernel companion.** Kernel fixtures accept a write to a region
  classified as kernel-owned, reject an ambient syscall-backed region, and fail
  closed when classification is unavailable. No test decides solely from the
  `read` or `write` verb. Covers REQ-11.

- [x] AC-10: **Migration inventory.** A checked migration report accounts for
  every effect atom across tracked `.th` files, Rust string programs, and JSON
  programs; every changed oracle is enumerated; and the post-migration scan has
  no unclassified state effect. Covers REQ-12 and REQ-14.

- [x] AC-11: **Corpus certification.** The full pre-migration and
  post-migration corpus comparison matches item-by-item assurance outcomes,
  except for hand-enumerated certificate `effects` changes and newly expected
  honesty failures or warnings. Existing failures remain failures and are
  compared by identity, not discarded. Covers REQ-6 and REQ-12.

- [x] AC-12: **Formal checks.** The effect-algebra and RFC-9 formal artifacts
  establish frame upper-bound soundness, call-footprint closure, ancestry
  overlap, and the link from computed commutation to concurrent acceptance. The
  artifact and tests make no claim about self-composition, noninterference,
  coupling, or hyperproperty completeness. Covers REQ-8 and REQ-13.

## Architecture

`thermite-syntax/src/ast.rs` gains AST nodes for shared declarations, structured
region paths, and concurrent compositions. `thermite-syntax/src/parser.rs`
parses the two new top-level forms. Shared and concurrent remain contextual at
item dispatch unless the spike demonstrates that reserving either word is
needed for unambiguous recovery. The parser records structure; name and type
resolution remain downstream checks.

Region resolution builds a declared-type table from `StructItem` and `EnumItem`
definitions and a shared-root table from the new declarations. A path walk uses
`FieldDef.ty` and `Type::Named` to derive child regions. The walk carries a
visited-type set and a finite source path, so recursive types are representable
without eagerly constructing an infinite region tree. Containment is computed
from path prefixes when two concrete operations are compared.

`thermite-lower/src/effects.rs` evolves from its current `EffectKind` bitmask
comparison into a program analysis with two products per function: a direct
footprint and a transitive footprint. Direct footprints come from body
intrinsics and explicit operations. Transitive footprints are the least fixed
point of union across the resolved call graph. Strongly connected components or
an equivalent monotone worklist make recursion deterministic. The existing
verified mask check may remain as a compatibility or sanity projection, but it
cannot authorize a region-sensitive result.

The declared row is normalized through
`thermite-syntax/src/effect_basis.rs` into the same operation-and-region domain
as inference. Set difference then yields missing effects (errors) and excess
effects (warnings). Warnings must travel through the public `forge` result and
CLI surfaces; printing to stderr inside the lowerer would make them unavailable
to structured consumers.

`thermite-spec/src/effect_commutation.rs` remains the sole owner of operation
commutation. The concurrent-composition consumer resolves each root, reads its
transitive footprint, selects pairs whose region paths overlap by equality or
ancestry, and asks the commutation module for the verdict. This design preserves
RFC-8's provenance rule: the conflict table is derived from theory entries.

The kernel target in `forge/src/build.rs` must consume resolved region
classification. The language-level representation should permit a later Bulla
kernel registry or prelude without baking Bulla's ambient names into Thermite.
The temporary safe state is an explicit unclassified-region error.

Formalization is split at the mathematical boundary. RFC-9 owns the effect
algebra laws it operationalizes, exact footprint propagation, containment, and
the commutation consumer. The relational lifting from footprints to paired runs
is deliberately left to `.design/research/relational-contracts.md`; RFC-9 must
preserve enough structure that the later lemma consumes checked footprints
rather than reconstructing them from syntax.

Implementation follows the repository dependency order: syntax and resolution,
spec commutation consumer, lowerer inference and diagnostics, then forge kernel
and certificate integration. Tests and migration inventory land with their
owning layer.

## Open Questions

- Q-1: Concurrent roots use the distinct top-level syntax
  `concurrent NAME { ROOT, ... }`. RFC-12 retains function-local `interleaves`
  for rely-guarantee clauses.

- Q-2: Rows are checked against region-sensitive call propagation and
  intrinsic footprints. Missing declarations are errors; excess declarations
  are loud warnings.

- Q-3: Field-derived containment ships in this increment. A focused spike
  verifies the current nested-declared-type pipeline before production work.

- Q-4: Ambient prelude and kernel-owned-region policy are deferred to the
  Bulla integration boundary. Thermite exposes classification and fails closed
  while policy is absent.

- Q-5: “No new metatheory” means no new frontier research is required.
  RFC-9 still owns the effect-algebra, footprint, containment, and commutation
  metatheory new to Thermite; later relational-contract metatheory remains out of
  scope.

## Residual trust

- Body footprint inference is only as complete as the intrinsic and call
  resolution inventory. Exhaustive mappings and corpus tests reduce this risk;
  they do not prove that every future operation is classified unless its API is
  closed over the mapping.
- Pairwise commutation proves freedom from the conflicts represented by the
  effect algebra. It does not prove deadlock freedom, liveness, fairness,
  interrupt safety, or arbitrary lock-free interference safety.
- Field ancestry treats sibling fields as disjoint according to the declared
  type structure. Foreign aliasing, interior sharing hidden behind a boundary,
  and platform state require an explicit boundary classification; the language
  cannot infer those hardware or FFI facts.
- Loud excess-row warnings remain warnings. Until a release policy promotes them
  to errors, a tree may certify while retaining over-conservative rows that
  reduce concurrency and obscure the exact footprint.
- The kernel-owned versus ambient classification is intentionally unresolved in
  Thermite. Failing closed prevents an unsafe build, but useful kernel builds
  still depend on Bulla or another platform supplying that policy.
- RFC-9 establishes the footprint structure needed by the relational frame
  lemma but does not prove the later self-composition theorem. Claims of
  determinism, noninterference, constant-time behavior, or hyperproperty
  composition remain unavailable from this component alone.

## Out of Scope

- Lock declarations, `holding` blocks, lock acquisition ordering, masking, and
  the `owns` conflict extensions from RFC-10.
- Function-local `interleaves`, `asks`, `promises`, stability, and monotone
  rely-guarantee lowering from RFC-12.
- Resource linearity, protocols, crash clauses, and proof-layer restructuring.
- `hides`, `varies`, `distributes`, `couples`, `matches`, hyper-arity, the
  relational frame lemma, and the certificate algebra from the relational
  contracts research program.
- Dynamic spawning, arbitrary thread topology inference, deadlock freedom, and
  liveness.
- Choosing Bulla's ambient-region prelude or kernel-owned-region registry inside
  this language component.
