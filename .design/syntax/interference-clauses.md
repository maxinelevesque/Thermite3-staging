# Feature: RFC-12 interference clauses

<!--
status: approved
-->

## Summary

RFC-12 adds function-local `interleaves { asks ... promises ... }` contracts
and checks them as rely-guarantee relations at every declared concurrency or
preemption composition site. Version 1 is deliberately limited to monotone
lock-free state whose evidence maps to persistent set, bool, or count facts.
It does not treat an epoch that is merely stable during one protocol round as a
persistent fact; that state belongs to RFC-13.

The work serves `telos/a-clause-is-checked`,
`telos/the-corpus-still-certifies`, `telos/surface-serves-agents`, and
`telos/residual-trust-is-named`. A parsed rely is never a free assumption:
until its preorder, stability, composition, lowering, and evidence consumers
are present, the next unsupported boundary fails closed.

## Requirements

- REQ-1: The lexer shall reserve `interleaves`, `asks`, and `promises`. The
  parser shall accept one optional function-local block after `ensures` and
  before `measures` or the body, with exactly one `asks` clause followed by
  exactly one `promises` clause. Missing, repeated, misplaced, or misordered
  clauses shall produce dedicated structured diagnostics.
- REQ-2: The AST shall represent the block as an optional typed
  `InterferenceContract` on an executable function. Its two relations shall
  retain parsed expressions, verbatim text, spans, and semantic child roles;
  they shall not be stored as arbitrary calls or display-only strings.
- REQ-3: `asks` and `promises` outside `interleaves` shall always be syntax
  errors. Existing programs without the block shall retain byte-for-byte
  equivalent AST meaning and item-level certification outcomes.
- REQ-4: Each relation is a two-state predicate over the function's visible
  shared state. `final(place)` shall be legal in these clauses for shared or
  mutable places, denote the relation's post-state, and retain its existing
  postcondition meaning elsewhere. Invalid, owned-only, or unresolvable places
  shall fail before lowering.
- REQ-5: RFC-12 v1 shall accept only relations classified as monotone persistent
  set inclusion/addition, persistent bool transition, or persistent count
  growth/lower-bound weakening. Arbitrary mutation, subtraction, reset,
  exclusive ownership, and a value merely constant for one protocol round are
  outside the fragment and shall produce an unsupported-language result rather
  than a proof failure or silent weakening.
- REQ-6: Both relations shall denote preorders on the admitted state: reflexive
  and transitive. The checker shall discharge these as explicit obligations and
  reject a relation whose no-op or multi-step closure is not established.
- REQ-7: Every ordinary postcondition shall be stable under `asks`: for
  postcondition `Q` and rely `R`, `Q(s) && R(s,s')` must imply `Q(s')`. Stability
  shall be checked independently of body correctness and peer compatibility.
- REQ-8: Every top-level `concurrent NAME { f, ... }` declaration shall create
  both ordered obligations for each unordered participant pair:
  `promises(f) => asks(g)` and `promises(g) => asks(f)`. Unknown participants,
  duplicate participants, or a participant without an interference contract
  shall fail closed when overlapping shared effects require RFC-12 reasoning.
- REQ-9: `handlers { f at N, ... }` shall preserve each declared priority in
  the AST and checked metadata. Normal context has priority zero. For every
  permitted preemption `high > low`, the checker shall generate only
  `promises(high) => asks(low)`; it shall not invent the reverse obligation.
  Duplicate functions or priorities that do not determine the declared order
  shall receive structured diagnostics.
- REQ-10: Pairwise implication shall be proved over the same canonical
  pre-state/post-state and resolved shared-place identities used by the
  relations. Textual similarity, effect-row overlap alone, or an assurance
  display string shall not discharge compatibility.
- REQ-11: RFC-9 effect commutation remains the first concurrency gate. RFC-12
  may admit a non-commuting overlapping shared access only when its complete
  relational obligations are present and discharged; it shall not weaken
  disjoint-state or lock-protected behavior.
- REQ-12: Parsed RFC-12 programs shall fail explicitly at every downstream
  boundary without an RFC-12 consumer. Syntax, relational validation,
  composition, lowering, formal replay, and release activation may land in
  separate slices only with a negative test proving the next boundary is shut.
- REQ-13: L1 lowering shall preserve executable behavior while materializing
  runtime-checkable boundary portions only; it shall not pretend to observe all
  environment steps. An unsupported relational construct shall fail closed.
- REQ-14: L3 lowering shall emit deterministic obligations for relation shape,
  preorder, postcondition stability, pairwise compatibility, and handler
  directionality, bound to canonical program, function, clause, composition,
  and shared-place identities.
- REQ-15: The Verus path shall map accepted v1 state to persistent set, bool, or
  count mechanisms and prove the relevant monotonicity/weakening facts. The
  independent Lean replay shall check the transcribed relational graph and
  obligation outcomes without claiming to reconstruct Verus semantics.
- REQ-16: Certificates and audit reports shall enumerate each interference
  clause, composition edge, discharged obligation, unsupported fragment, and
  residual trust in parsing, state classification, solver encoding, backend
  correspondence, persistent-token implementation, and platform preemption.
  An undischarged `asks` shall never receive the full boundary coordinate.
- REQ-17: Mutation and negative evidence shall target structure and semantics:
  deleted or swapped clauses, weakened/strengthened relations, reversed handler
  edges, omitted peers, broken stability, reset/non-monotone transitions, and
  forged evidence must be rejected. Body-mutation scoring shall remain explicitly
  unavailable until effect-trace observables exist; it shall not be reported as
  evidence that `asks` or `promises` is strong.
- REQ-18: RFC-12 shall update the requirement registry, language inventory,
  completeness evolution, conformance corpus, route coverage, status views,
  and documentation pins. Its two umbrella requirements become shipped only
  when their direct executable/formal evidence and release-negative fixture are
  present.

## Acceptance Criteria

- [ ] AC-1: (REQ-1, REQ-2, REQ-3) Parser, round-trip, address, and semantic traversal
  tests accept the exact ordered block and reject missing, repeated, misplaced,
  or swapped clauses with stable diagnostics; the old corpus is unchanged.
- [ ] AC-2: (REQ-4, REQ-5) Relational validation accepts persistent set/bool/count
  examples and rejects invalid `final` places, reset/subtractive transitions,
  arbitrary lock-free mutation, and epoch-like per-round stability as named
  unsupported language.
- [ ] AC-3: (REQ-6, REQ-7) Independent positive and negative fixtures establish
  reflexivity, transitivity, and `ensures` stability; mutants removing any one
  obligation are caught.
- [ ] AC-4: (REQ-8, REQ-10, REQ-11) Concurrent compositions generate every
  ordered pair exactly once, accept compatible monotone peers, and reject a
  missing, incompatible, duplicated, or identity-mismatched participant without
  bypassing RFC-9's existing checks.
- [ ] AC-5: (REQ-9) Handler priorities survive parsing and canonicalization;
  tests cover normal-to-handler and nested-handler edges and prove that the
  impossible reverse edge is not generated.
- [ ] AC-6: (REQ-12) Every incremental slice contains a downstream refusal test,
  and no syntax-only or partially checked RFC-12 program receives an older
  certificate.
- [ ] AC-7: (REQ-13, REQ-14) L1 behavior remains equivalent for accepted
  programs, while L3 obligations are deterministic, source-bound, complete,
  and reject clause, participant, edge, or shared-place tampering.
- [ ] AC-8: (REQ-15) Verus evidence covers persistent set, bool, and count
  monotonicity; Lean replay accepts the canonical obligation graph and rejects
  preorder, stability, pairwise, and directionality mutations within the
  repository's allowed axiom set.
- [ ] AC-9: (REQ-16) Generated certificates and human audit output expose every
  conditional rely and its discharge state, with residual trust named and no
  authority derived from prose.
- [ ] AC-10: (REQ-17) Structural and semantic mutation suites catch all listed
  mutants, and reports explicitly distinguish unimplemented effect-trace body
  scoring from a passing score.
- [ ] AC-11: (REQ-18) Registry, inventory, completeness, conformance, routes,
  status, doc-drift, formatting, lint, workspace, and frozen-corpus gates pass.
- [ ] AC-12: (REQ-12, REQ-18) A production release-negative fixture proves that
  an RFC-12 program cannot silently fall back to a pre-RFC-12 fragment or reach
  full assurance with an undischarged rely.

## Architecture

### Surface and identity

`Contract` in `thermite-syntax/src/ast.rs` gains an optional typed interference
block, parsed by `thermite-syntax/src/parser.rs`. The new clause words are
reserved because accepting one as an identifier outside the block risks proving
a caller boundary when the author meant an environment relation. Semantic
traversal and addressing give `asks` and `promises` dedicated roles and stable
ordinals. `handlers` stops collapsing to an undirected `ConcurrentItem`: its
priorities become canonical metadata used by both RFC-10 masking checks and
RFC-12 implication generation.

### Relational checker

A specification pre-pass in `thermite-spec/src/validator.rs` resolves every relation to canonical shared-place
identities and classifies its transition into the closed v1 persistent fragment.
It emits separate preorder and postcondition-stability obligations. Composition
then joins these checked function contracts with RFC-9 concurrency metadata:
ordinary concurrent groups generate both directions; handler groups generate
only feasible priority edges. Missing evidence is an error, never an empty
relation or a full-assurance conditional proof.

### Lowering and evidence

`thermite-lower/src/l1.rs` keeps relational claims out of executable authority except where an existing
boundary predicate is directly checkable. L3 serializes the checked relation and
obligation graph from `thermite-lower/src/lower.rs` with source and semantic identities. Verus supplies the
persistent-state proof mechanism; Lean independently replays graph completeness
and the transcribed verdicts. Certificates name the gap between those layers as
residual trust rather than calling the replay a source-semantics proof.

## Sequence

1. Reconcile RFC status and land syntax, AST, diagnostics, semantic addressing,
   and fail-closed downstream recognition.
2. Add shared-place resolution, the closed persistent-fragment classifier,
   preorder obligations, and postcondition stability.
3. Add ordinary pairwise composition and priority-preserving handler direction.
4. Add L1/L3 lowering, Verus evidence, Lean replay, certificate/audit disclosure,
   claim closure, inventories, and release activation.
5. Run full local qualification, a cold exact-head adversarial review, and two
   green CI observations around any final receipt/pin regeneration before merge.

## Residual trust

- The parser, relation classifier, solver translation, Verus-to-Thermite mapping,
  Rust witness extraction, and target implementation of atomic operations and
  interrupt priorities remain in the named trusted base.
- Effect-trace observables for body-mutation scoring are deferred; RFC-12 uses
  structural, relational, composition, and evidence mutations meanwhile.
- A value constant only during one protocol round, including the motivating
  shootdown epoch, is deferred to RFC-13 rather than misrepresented as
  persistent state.
- Arbitrary lock-free shared mutation and full concurrent separation logic are
  outside Thermite 3's RFC-12 fragment.

## Open Questions

- None. The maintainer approved the v1 persistent-state boundary, mandatory
  postcondition stability, pairwise composition, and RFC-13 deferral on
  2026-09-04.
