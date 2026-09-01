# Feature: RFC-11 resource types

<!--
status: approved
audited-content-sha256: 4a95bcbe315b8b47fc78d1c64b79f453c5ddb659aa0d0d40adb580f719ddd176
-->

## Summary

RFC-11 upgrades selected owned values from affine to linear: a resource value
may not be copied and must be consumed on every path that returns. Resource
provenance is explicit, contagion propagates through every owning type
constructor, and deliberate abandonment is the checked `forget(value)`
operation whose complete region footprint must appear in the function's effect
row.

The work serves `telos/a-clause-is-checked`,
`telos/the-corpus-still-certifies`, `telos/surface-serves-agents`, and
`telos/residual-trust-is-named`. In particular, the surface must never parse a
resource claim that a downstream stage silently treats as an ordinary droppable
value. Each implementation slice therefore fails closed until its next semantic
consumer is present.

## Requirements

- REQ-1: `thermite-syntax/src/parser.rs` shall parse `resource(REGION, ...)
  struct` and `resource(REGION, ...) enum` for a directly resource-owning type,
  plus bare `resource struct` and `resource enum` for a contagious container
  whose provenance is derived from its fields or variants. `resource` remains a
  contextual full-word modifier and shall not reserve the identifier elsewhere.
- REQ-2: `thermite-syntax/src/ast.rs` shall represent the resource marker and
  its declared provenance separately from `StructItem.sealed`, retain exact
  spans, and reject duplicate provenance paths and an empty
  `resource()` declaration with structured diagnostics.
- REQ-3: `thermite-syntax/src/parser.rs` shall parse `forget(value);` as an
  explicit executable statement over one owned place or value. It shall not be
  represented as an ordinary name-based call that a downstream consumer could
  overlook.
- REQ-4: `thermite-syntax/src/ast.rs` and `thermite-syntax/src/parser.rs` shall
  add `forgets(REGION)` as a first-class effect atom using the same canonical
  `RegionPath` representation as `read`, `write`, and `net`; duplicate atoms
  shall normalize without changing the semantic footprint.
- REQ-5: `thermite-spec/src/validator.rs` shall build an order-independent
  resource-type environment and compute a finite provenance set for every type.
  A direct resource declaration contributes its explicit regions; a declared
  struct or enum contributes the union of its resource-bearing fields and
  variant payloads.
- REQ-6: Resource contagion shall be uniform through owning constructors:
  `Box`, `Vec`, `Option`, either side of `Result`, either side of `Map`, tuples,
  named declared types, and generic applications are resource-bearing whenever
  any owned type argument or component is resource-bearing. References and
  slices borrow obligations rather than owning them and therefore do not make
  the reference value itself a resource.
- REQ-7: A declared type with a resource-bearing field or variant shall carry
  the bare or explicit `resource` modifier, and its explicit provenance, when
  present, shall equal the computed union. Missing, excess, or contradictory
  provenance shall fail with a diagnostic naming the declaration and the
  responsible field or variant path.
- REQ-8: The specification layer shall perform path-sensitive ownership
  analysis over executable function bodies. Every resource-bearing parameter,
  local, destructured component, and temporary must be in exactly one of the
  states live, transferred, returned, or forgotten, and every returning edge
  must have no unconsumed live resource.
- REQ-9: Moving a resource into a by-value call transfers its obligation to the
  callee; returning it transfers the obligation to the caller; destructuring
  consumes the container and creates obligations for its resource-bearing
  components. Borrowing does not consume, copying is rejected, and overwriting
  a live resource place is an implicit drop and is rejected.
- REQ-10: Branch joins, loop back-edges, `break`, `continue`, early `return`, and
  nested blocks shall use one deterministic flow rule. A resource consumed on
  only some returning paths, consumed twice, used after transfer, or carried
  inconsistently around a loop shall fail before lowering.
- REQ-11: Panic and declared divergence shall follow the existing partial-
  correctness boundary: a path that does not return has no resource
  post-obligation. This exception shall not apply to ordinary teardown, error
  returns, `break`, or `continue`, all of which remain checked returning or
  continuing edges.
- REQ-12: `forget(value)` shall accept exactly one live owned resource-bearing
  value, consume its obligation, and infer the complete deduplicated provenance
  set of that value. The caller's declared row must contain `forgets(region)`
  for every inferred region, and an unpriced, non-resource, borrowed, or already
  consumed forget shall fail.
- REQ-13: `thermite-lower/src/effects.rs` shall integrate `forgets` into the
  existing direct-body inference, transitive call-graph fixed point, missing-
  effect rejection, excess-effect warning, and row-subsumption rules without
  weakening the checks for any existing effect atom.
- REQ-14: Parsed programs containing an RFC-11 construct shall fail explicitly
  at every downstream boundary that has not yet acquired its RFC-11 consumer.
  Syntax-only, provenance, flow, lowering, formal replay, and release activation
  may land separately, but no intermediate slice may validate, build, or certify
  an unchecked resource claim.
- REQ-15: L1 lowering shall preserve ownership transfer and lower deliberate
  abandonment explicitly; L3 lowering shall emit a deterministic checked
  resource-flow witness bound to the canonical program and replayed by an
  independent formal checker before resource-bearing code can receive a
  kernel-grounded assurance result.
- REQ-16: The formal model shall prove that an accepted flow witness assigns
  each resource obligation exactly one terminal disposition on every returning
  path, that branch and loop joins preserve the live-obligation set, and that a
  recorded forget footprint equals the value's computed provenance set. The
  axiom probe shall remain within the repository's allowed axiom set.
- REQ-17: The certificate and audit surfaces shall disclose resource-flow
  checking, every `forgets` footprint, the formal replay result, and the
  remaining trust in parsing, type/provenance resolution, and executable target
  behavior. Verus token machinery or trusted macros shall not be presented as a
  proof unless their exact trust contribution is named.
- REQ-18: RFC-11 shall update the requirement registry, versioned language
  inventory, completeness evolution artifacts, conformance corpus, mutation
  battery, route coverage, and documentation pins while preserving every
  pre-existing item-level certification outcome.

## Acceptance Criteria

- [ ] AC-1: (REQ-1, REQ-2) Parser and round-trip tests accept direct resource
  structs and enums with one and multiple regions, accept bare contagious
  declarations, preserve spans, and show that an unrelated identifier named
  `resource` remains legal.
- [ ] AC-2: (REQ-2) Negative parser tests reject `resource()`, repeated region
  paths, the modifier on a non-struct/non-enum item, and malformed modifier
  ordering with stable structured diagnostics.
- [ ] AC-3: (REQ-3, REQ-4) AST exhaustiveness and parser tests prove that
  `forget(value);` and `forgets(region)` have dedicated variants, exact spans,
  canonical paths, and no string-named fallback route.
- [ ] AC-4: (REQ-5, REQ-7) Declaration tests accept direct provenance and exact
  contagious unions, including a multi-region container, and reject missing,
  excess, and conflicting declarations while naming the field or variant that
  introduced each region.
- [ ] AC-5: (REQ-5, REQ-6) Type-propagation tests cover `Box`, `Vec`, `Option`,
  both `Result` positions, both `Map` positions, tuples, named ADTs, generic
  applications, nested combinations, and borrowed references; a mutation that
  omits any owning constructor must fail.
- [ ] AC-6: (REQ-8, REQ-9) Positive flow tests cover parameter-to-callee
  transfer, return transfer, destructuring, reconstruction, and complete
  consumption; negative tests cover copying, implicit drop, overwrite, double
  consumption, and use after transfer.
- [ ] AC-7: (REQ-8, REQ-10, REQ-11) Control-flow tests cover both sides of an
  `if`, every `match` arm, fallthrough, early return, error return, nested loops,
  `break`, `continue`, and a declared diverging path. Branch-only consumption
  and inconsistent loop-carried ownership must fail deterministically.
- [ ] AC-8: (REQ-12, REQ-13) Effect tests accept a correctly priced single- and
  multi-region forget, infer the same footprint through transitive calls, and
  reject missing atoms, non-resource values, borrows, and already consumed
  values while retaining the existing excess-row warning policy.
- [ ] AC-9: (REQ-14) Each incremental implementation pull request includes a
  negative downstream test showing that its newest syntax cannot pass the next
  unimplemented validation, lowering, build, or certification boundary.
- [ ] AC-10: (REQ-15) L1 execution fixtures observe ordinary transfer and an
  explicit abandonment operation without an implicit clone or drop path; L3
  fixtures bind the complete flow witness to the source and checked-program
  digests and reject tampering with either.
- [ ] AC-11: (REQ-15, REQ-16) The independent formal replay accepts canonical
  positive programs and rejects mutations that delete a terminal disposition,
  duplicate a disposition, alter a join set, or remove a forget region; the
  repository axiom probe remains green.
- [ ] AC-12: (REQ-17) A generated certificate and human audit report enumerate
  the resource-flow verdict, all abandonment regions, formal-replay identity,
  and residual trust without deriving authority from display strings.
- [ ] AC-13: (REQ-18) The requirement and language inventories report all four
  RFC-11 requirements as shipped only when their direct executable or formal
  evidence is present, and the completeness-evolution gate rejects an RFC-11
  syntax addition without matching fragment updates.
- [ ] AC-14: (REQ-18) The focused syntax, specification, effect, L1, L3, formal,
  conformance, mutation, registry, route, document-drift, format, lint, and
  workspace suites pass; the frozen existing corpus retains every prior item-
  level outcome.
- [ ] AC-15: (REQ-14, REQ-18) A release-negative fixture proves that a resource
  program cannot receive a non-resource certificate or silently fall back to an
  older language fragment at any supported stage.

## Architecture

### Surface and checked boundary

The parser extends the existing contextual-item dispatch in
`thermite-syntax/src/parser.rs` rather than reserving `resource` globally.
`StructItem` and `EnumItem` in `thermite-syntax/src/ast.rs` carry a resource
declaration containing zero or more explicit `RegionPath` values: non-resource
is absence, a bare contagious declaration is an empty declared set, and a
direct declaration is a non-empty set. `forget` is a statement because its
purpose is disposition rather than value production. `forgets` is an `Effect`
variant, so effect equality, ordering, serialization, and audit code cannot
mistake it for an arbitrary call.

The first implementation slice is deliberately syntax-only and fail-closed.
Every exhaustive consumer must either understand the new variants or return a
named RFC-11-unsupported diagnostic. Removal of that diagnostic is staged with
the consumer that replaces it.

### Provenance and contagion

The specification pre-pass in `thermite-spec/src/validator.rs` already collects
all declared structs and enums before walking bodies. RFC-11 extends that
order-independent environment with a monotone fixed point from named types to
finite sets of resource regions. Direct declarations seed the sets; fields and
variant payloads propagate unions until stable. Cycles with no direct resource
seed remain non-resource, while a recursive type reachable from a seed carries
that seed after convergence.

Owning type constructors recurse into all arguments. References and slices
retain the provenance of what they borrow for access checking but do not create
an owned disposition obligation. A bare `resource` declaration is valid only
when its computed field/variant union is non-empty. If an explicit region list
is present, it must equal that union plus any direct regions introduced by the
declaration; diagnostics report both the declared and computed sets.

This set-valued model is load-bearing. A container can own a heap grant and a
device token simultaneously, and forgetting it must infer both
`forgets(heap)` and `forgets(device)` exactly once.

### Flow analysis

The executable-body walk uses a finite state keyed by semantic place identity,
not identifier spelling alone. Each owned resource obligation is live until a
move, return, destructure, or forget gives it a unique successor disposition.
Branch joins require equal live-obligation sets on every continuing edge.
Loops are solved to a stable header set; `continue` must reproduce that set,
`break` contributes to the enclosing exit join, and returning edges must be
empty after accounting for returned values.

Moving an aggregate transfers its whole obligation. Destructuring replaces it
with obligations for its resource-bearing components. Assignment consumes the
right-hand side into the destination only when the destination has no live
resource; replacement is otherwise an implicit drop. Calls use declared
parameter and return types, so by-value arguments transfer to a separately
checked callee and returned resources become caller obligations. Borrowed
arguments leave the source live.

Panic and declared divergence use the existing partial-correctness boundary:
they produce no returning edge. An error value that is returned, and control
flow through `break` or `continue`, remains an ordinary checked edge.

### Effects, lowering, and proof

`thermite-lower/src/effects.rs` adds inferred forget footprints to the same
fixed point and subset checks used by existing effects. A value with provenance
`{heap, device}` requires both atoms in its enclosing row, independent of
surface order or duplication.

L1 lowering makes abandonment explicit and never synthesizes it for ordinary
scope exit. L3 does not claim that Rust moves or a trusted Verus token macro are
the proof. Instead, the checked resource-flow graph and its terminal
dispositions become a deterministic witness bound to the canonical source and
checked-program identities. The independent formal replay proves the finite
graph property used for certification: every obligation has exactly one
terminal disposition on every returning path, joins preserve the live set, and
forget footprints match computed provenance.

The certificate names what remains trusted: parsing, type resolution, witness
extraction, the target behavior behind a resource, and any substrate mechanism
not proved by the replay. The versioned completeness artifacts in
`.design/versioned-language-completeness.md` gain the RFC-11 syntax and semantic
fragment rather than treating the new forms as belonging to an older complete
fragment.

## Residual trust

After RFC-11 ships, Thermite trusts the parser to preserve the program supplied
to the checked boundary, the type/provenance resolver to assign the intended
declared types and region paths, and the witness extractor to transcribe the
checked control-flow graph presented to formal replay. Digest binding and
hostile witness mutations detect accidental or adversarial substitution but do
not prove those producers correct.

The formal replay proves returning-path disposition only for the graph and
provenance facts it receives. It does not prove that releasing or abandoning a
resource has the intended physical effect, that a target allocator or device
provider is correct, or that state survives panic, reset, or power loss. L1
execution remains ordinary compiled Rust beneath Thermite's checked source
discipline. Any Verus tracked value, tokenized-state-machine macro, external
solver, or target primitive used by lowering remains named according to its
actual trust category and does not inherit authority from the resource-flow
verdict.

### Implementation sequence

1. Activate the design and requirement records without changing executable
   language behavior.
2. Add syntax and AST variants with explicit downstream rejection.
3. Add the provenance environment and declaration/contagion checks.
4. Add path-sensitive flow and `forget` effect inference.
5. Add L1/L3 lowering and the checked witness format.
6. Add independent formal replay, certificate disclosure, hostile mutations,
   inventories, and final release activation.
7. Run a cold independent adversarial review against this document before the
   implementation pull request is allowed to merge.

## Resolved Questions

- Keep `.design/rfcs/0011-resource-types.md` as the proposal and rationale;
  this document is the current implementation contract, following RFC-10's
  proposal/implementation-document split.
- Land syntax first only with explicit downstream rejection. No parsed resource
  construct may silently validate, lower, build, or certify before its owning
  consumer exists.
- Make resource provenance explicit at direct declarations and compute a finite
  set through contagious containers. `forget` charges the complete set rather
  than guessing from constructor or function names.
- Apply contagion uniformly through every owning constructed type, not through
  a built-in allowlist. References borrow and therefore do not create a second
  owned obligation.
- Represent multi-region ownership as a deduplicated set. A contagious
  declaration may use bare `resource`; a declaration that introduces resource
  ownership directly uses `resource(REGION, ...)`.
- Prove the checked flow witness independently for kernel-grounded assurance;
  do not treat Rust move errors or trusted Verus token macros as the proof of
  returning-path linearity.

## Out of Scope

- Unconditional cleanup on panic, process abort, power loss, or other
  non-returning executions; those remain governed by the existing partial-
  correctness and crash models.
- Destructor syntax, implicit finalizers, garbage collection, or automatic
  insertion of `forget`.
- Borrow checking beyond the ownership-transfer facts required to distinguish
  owned values from references in the current Thermite type surface.
- Region polymorphism or user-declared multi-parameter generic type definitions;
  the design propagates resource sets through the constructed types the current
  AST can represent.
- RFC-12 interference clauses, RFC-13 protocol progress, and production target
  implementations of allocators, devices, or protocol endpoints.
