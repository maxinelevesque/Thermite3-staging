# The Effect Algebra — what a row denotes
<!--
tier: 3-component
status: draft
audited-content-sha256: 263663e1584a4f2b5e2089107b636a25a2335a35bc8d201dcf57dc7c2ac8b3d1 (initial pin, 2026-08-11: both governed files are unbuilt, so the digest covers an empty governed set. The routes carry `unbuilt = true` and the pin is re-derived when `effect_basis.rs` lands.)
governs: thermite-syntax/src/effect_basis.rs
governs: thermite-spec/src/effect_commutation.rs
thesis-refs:
  - thermite-design.md §4.1
  - thermite-design.md §5.3
  - thermite-design.md §11
-->

## Summary

`thermite-design.md` §4.1 makes the effect row mandatory and gives it a
vocabulary: `pure`, or a set drawn from the atoms `enum Effect` models. What the
row lacks is a denotation. A label is an opaque token that
`thermite-lower::effects` compares by identity, so every rule about labels is
stipulated one rule at a time: the subsumption order is written down, and
[RFC-9](../rfcs/0009-verified-effect-rows.md)'s conflict table would be written
down beside it.

This component supplies the denotation. An effect label denotes an **algebraic
theory** — a signature of operations with the equations they satisfy — and a row
entry is a theory instance together with the operations that entry uses. Each
label then owes three things that are computed from its denotation: a **frame
condition**, a **composition law**, and a **commutation fact**. The third is
what turns RFC-9's data-race-freedom table into a theorem about the basis.

Proposed by [RFC-8](../rfcs/0008-effect-algebra.md), which introduces the three
requirements this document contracts: `REQ-SYNTAX-EFFECT-BASIS`,
`REQ-SYNTAX-EFFECT-DECLARED` and `REQ-SPEC-EFFECT-COMMUTATION`.

This component is **NOT-STARTED**. Both governed files are unbuilt and every REQ
below carries its open prerequisite in the REQ-status table.

**The boundary this document does not cross.** Subsumption is unchanged.
[`.design/lower/effect-subsumption.md`](../lower/effect-subsumption.md) governs
the check, keeps its 9-atom `u16` mask, and keeps its delegation to the
Verus-proved `thermite_verified::subsumes_masks`. Region-granular checking,
`shared` declarations and the conflict rule are RFC-9's three requirements
(`REQ-SYNTAX-SHARED-DECL`, `REQ-SPEC-EFFECT-ROW-CHECKED`,
`REQ-SPEC-EFFECT-CONFLICT`), recorded here as REQ-10 so the critic does not
expect them.

## Requirements

- **REQ-1 (the admissibility criterion):** A primitive effect is admissible when
  it generates a frame condition expressible in the prover's logic. §11 names
  frame conditions among the contracts L3 targets, so the criterion is a filter
  on basis membership rather than a restatement of one. A candidate primitive
  that generates no such frame condition is either a given atom (REQ-3) or is
  outside the basis. Derived from RFC-8 §The admissibility criterion.

- **REQ-2 (the basis: five theories):** The basis is five theories, each with the
  frame condition it generates:

  | primitive | frame condition |
  |---|---|
  | `state(r)` | may modify `r`, nothing else |
  | `accrues(M)` | adds to a ghost accumulator over a monoid `M` |
  | `exception` | may not return normally |
  | `partiality` | may not return at all |
  | `io(σ)` | a free signature `σ`, no equations; the value is determined by no region |

  The set is closed. A new primitive is a change to this document, and
  `REQ-SYNTAX-EFFECT-DECLARED` exists so that naming a new effect does not
  require one. Derived from RFC-8 §The basis.

- **REQ-3 (two given atoms, and the two axes that separate them from the basis):**
  Basis membership and equational content are independent properties, and a label
  is placed by both:

  | | generates a frame condition | equations |
  |---|---|---|
  | `state(r)` | yes | get-put, put-get, put-put |
  | `io(σ)` | yes — nothing changes, and the value is unconstrained | none (free) |
  | `random` | no | commutativity |
  | `blocks` | no | none stated |

  `random` and `blocks` generate no frame condition, so REQ-1's criterion places
  them outside the basis as given atoms. `random` still carries an equation.
  Independent samples commute, which RFC-8 and RFC-7 both record as
  `random ∥ random accept`, and that fact is the commutativity equation rather
  than a stipulation: a commutative theory is the standard algebraic reading of
  probability. So `random` and `io(σ)` are distinct theories, and the property
  that separates them is checkable — `random` commutes with itself and `io(σ)`,
  having no equations, does not.

  `random` reads as "I claim nothing about this value": the prover treats the
  result as unconstrained, which supports "this holds for every value it could
  take" and supports neither a distribution claim nor an unpredictability claim.
  It is reserved to take a parameter later, `random(D)`, for the distributional
  claims RFC-7 §12 puts on the reachable surface with the discharge open, so the
  atom is a placeholder for proofs about sampling rather than about a PRNG. The
  pseudorandom case is `state(entropy)` and is a basis instance (REQ-5).

  `blocks` records that a function may wait
  ([protocol types](../rfcs/0013-protocol-types.md): "what remains is that the
  function can wait, which is a control effect"), so it is an admission in the
  shape of `diverge` rather than a claim. What separates the two is the
  **discharge route**, and this is the reason `blocks` is recorded and unproved:

  | atom | introduced by | discharged by |
  |---|---|---|
  | `panic` | a panicking operation | nothing; abort propagates |
  | `diverge` | unbounded recursion | `measures`, a well-founded ordering |
  | `blocks` | a receive on an endpoint | no route exists |

  Termination is a liveness property, and the language affords it by reducing it
  to well-founded descent — a finitary witness inside the function. The reduction
  does not transfer to progress. A measure establishing that a block resolves
  would have to decrease on a peer's steps, and the peer is not in this
  computation, so the witness is non-local.

  Stated by quantifier shape: liveness is `∀ finite prefix, ∃ extension`, which
  is dense rather than closed, so it has no finite counterexample. For `blocks`
  the existential ranges over a peer's behaviour. That fixes its refutation
  coordinate ahead of any mechanism — a liveness obligation never carries
  `empirical`, because no finite set of observed runs refutes it — and it makes
  `blocks` the first surface instance of the class
  [the relational-contracts record](../research/relational-contracts.md) §7
  legislates for, arriving before `matches` does.

  Its discharge route is settled in the literature and unbuilt here: binary
  session duality carries deadlock freedom
  ([protocol types](../rfcs/0013-protocol-types.md) §Metatheory), so duality is
  the witness for the existential, the way `measures` is the witness for
  termination. `blocks` therefore has no equations here and contributes no
  commutation fact.

  Neither given atom has a surface label today; both are reserved so that gaining
  one is an addition rather than a break. Derived from RFC-8 §The basis, RFC-7 §5
  and RFC-7 §12, with the given atoms' trajectories from
  [the relational-contracts record](../research/relational-contracts.md) §2, §6.2
  and §8.1.

- **REQ-4 (a row entry is a theory instance plus its operations):** A row entry
  names a theory instance and the subset of that theory's operations the entry
  uses. For the state theory with operations `{get, put}`:

  ```
  read(r)   =  state(r) / {get}
  write(r)  =  state(r) / {get, put}
  ```

  The operation set is what a commutation fact is computed from (REQ-8), so it is
  part of the entry rather than a comment on it. Derived from RFC-8 §A row entry
  is a theory instance plus the operations used.

- **REQ-5 (every surface label has a basis entry):** The map from
  `thermite_syntax::ast::Effect` to basis entries is **total** over the nine
  variants:

  | label | entry |
  |---|---|
  | `read(r)` | `state(r) / {get}` |
  | `write(r)` | `state(r) / {get, put}` |
  | `net(d)` | `state(d) + io(σ_d)` — a combination, REQ-6 |
  | `alloc` | `state(heap) / {get, put}` |
  | `time` | `state(clock) / {get}` |
  | `rand` | `state(entropy) / {get, put}` |
  | `panic` | `exception` |
  | `diverge` | `partiality` |
  | `term` | `state(termios) / {get, put}` |

  Totality is the obligation: a label with no entry is a label whose frame
  condition, composition law and commutation fact are undefined, which is the
  state REQ-1's criterion exists to detect. The region names in this table
  (`heap`, `clock`, `entropy`, `termios`) are the ambient names RFC-9 §Migration
  identifies as prelude-declarable; this document fixes which region each label
  denotes and leaves declaring them to `REQ-SYNTAX-SHARED-DECL`. Derived from
  RFC-8 §The basis and RFC-7 §5.

- **REQ-6 (combination, and `net` as its worked example):** An entry may be a sum
  of basis entries, written `+`, whose frame condition is the conjunction of its
  summands' and whose commutation fact is the conservative meet of theirs. RFC-8
  gives the form as `effect journal(d) = state(d) + exception`. `net(d)` is a
  combination rather than a primitive:

  ```
  effect net(d) = state(d) + io(σ_d)
  ```

  The decomposition is read off the label's existing surface. Its syscall grant
  in [`.design/forge/runtime-sandbox.md`](../forge/runtime-sandbox.md) splits
  along the theory boundary: `setsockopt`/`getsockopt` are `get`/`put` on socket
  state, `sendto` is a `put` toward the far end, and `recvfrom` yields a value
  determined by no region this program frames over.
  [`.design/basis/03-effect-stdlib.md`](../basis/03-effect-stdlib.md) already
  models the label as three primitives (`net_connect`, `net_send`, `net_recv`)
  and puts `recv`'s short/EOF case in the return type as an `Option`/`Result`,
  which is RFC-8's rule for an obligation the caller decides at the call site.
  Keeping `net` out of the basis holds the basis at five theories, which RFC-7 §5
  requires: the label set is designed to shrink. Derived from RFC-8 §User-declared
  effects.

- **REQ-7 (the declaration surface):** `effect name(param) = combination` declares
  a label as a combination of basis entries. The declaration is an AST item, is
  parsed by `thermite-syntax`, and is resolved to basis primitives before
  lowering, so the prover sees only primitives it can encode. A declaration whose
  right-hand side names something outside the basis is a resolution error naming
  the basis. This is `REQ-SYNTAX-EFFECT-DECLARED`. Derived from RFC-8
  §User-declared effects.

- **REQ-8 (commutation is computed per operation pair):** For two entries over
  the same theory instance, the commutation fact is computed from the theory's
  equations over the entries' operation sets. For `state`:

  | | | because |
  |---|---|---|
  | `read(r)` ∥ `read(r)` | accept | `get` commutes with `get` |
  | `read(r)` ∥ `write(r)` | reject | `get` does not commute with `put` |
  | `write(r)` ∥ `write(r)` | reject | `put` does not commute with `put` |
  | `write(a)` ∥ `write(b)` | accept | independent instances are independent theories |

  For `io(σ)` the signature is free, so no operation pair is licensed to commute
  and same-instance pairs reject, while independent instances accept by the same
  rule as `state`. The consumer of these facts is RFC-9's conflict rule, so this
  requirement lands computable and unused. This is `REQ-SPEC-EFFECT-COMMUTATION`.
  Derived from RFC-8 §A row entry is a theory instance plus the operations used.

- **REQ-9 (the diagnostic names the theory — the production consumer):**
  `LowerError::EffectNotSubsumed` names the atoms a callee has that the caller
  lacks (`.design/lower/effect-subsumption.md` REQ-4). It also names each missing
  atom's basis entry and frame condition, so a rejection reports what the caller
  would be permitting. This is the non-test production consumer R-DEFER-1
  requires for the basis API, and it changes no accept/reject behaviour: the
  message text widens, the `missing` set does not. Amends
  `.design/lower/effect-subsumption.md` REQ-4, which owns the diagnostic.

- **REQ-10 (recorded boundary — checking belongs to RFC-9):** Region-granular
  subsumption, `shared` declarations, checking a row against its body, and the
  conflict rule are RFC-9's requirements and are not built here. The commutation
  facts of REQ-8 are the input RFC-9's conflict rule consumes. Building the
  conflict rule here to make REQ-8 observable would invert RFC-7 §14's sequence
  and would pull the Verus-proved subsumption core into a step that does not need
  it. This REQ records the boundary so the critic does not expect the rule here.

- **REQ-11 (recorded boundary — the verified core does not move):** The `Effect`
  representation in `thermite-syntax/src/ast.rs`, the 9-atom `u16` mask in
  `thermite-lower/src/effects.rs`, and `thermite_verified::subsumes_masks` with
  its `bit_vector` proofs are unchanged by every REQ above. The basis is built
  beside the existing representation rather than replacing it. Widening the mask
  for the ninth atom required re-deriving those proofs
  (`.design/verified/self-verification.md` REQ-5); that cost is not paid here.

- **REQ-12 (the footprint projection):** From a row, the basis computes a
  **footprint**: the set of region instances the row may read, and the set it may
  write.

  ```
  footprint(row) -> (reads: Set<Instance>, writes: Set<Instance>)
  ```

  An entry contributes its instance to `reads` when its operation set contains
  `get`, and to `writes` when it contains `put`, so `read(r)` contributes `r` to
  `reads` alone and `write(r)` contributes `r` to both. A combination contributes
  the union of its summands'. An entry over a theory with no region — `exception`,
  `partiality`, a given atom — contributes to neither, and an `io(σ)` summand
  contributes to neither while marking the row as carrying a value no region
  determines (REQ-6).

  This is the projection
  [the relational-contracts record](../research/relational-contracts.md)
  consumes. Its §5.1 relational frame lemma quantifies over two runs agreeing on
  the footprint of a row and reaching states agreeing outside its
  write-footprint, and §5.2 generates the security clauses' coupling invariants
  from the same frames — "the frames are the coupling invariants". That record's
  §10 puts the lemma second in its sequence, ahead of every clause that
  instantiates it.

  The 9-atom `u16` mask in `thermite-lower/src/effects.rs` projects the row's
  **labels**, discarding the region argument (`EffectKind::of` drops it and says
  so). The footprint projects the row's **regions and operations**. Neither is a
  coarsening of the other: a row of `write(r)` alone puts `r` in `reads`, because
  `write(r)` is `state(r)/{get, put}` and a writer may read what it writes, while
  the mask's `Read` bit stays clear because no `read` label appears.

  The two projections are related by which labels carry `put` and which carry
  `get`, and that relation is a law over the REQ-5 map rather than a restatement
  of it (AC-9). The mask stays as it is (REQ-11); the footprint is computed
  beside it.

- **REQ-13 (the discharge route, and what an unbuilt one obliges):** Each atom
  carries a **route**: what discharges its characteristic obligation.

  ```
  route(atom) -> NoObligation | Implemented(form) | Deferred(form) | Open(question)
  ```

  **The boundary this classification respects.** A route says what is left to
  prove *given an honest row*. Whether a body respects its row is a separate and
  global obligation belonging to `REQ-SPEC-EFFECT-ROW-CHECKED`, tracked as an RFC
  rather than per atom, so a state effect is `NoObligation` here: given an honest
  row, its frame condition is its meaning and nothing further is owed. Reading
  this REQ as covering row honesty would put a marker on every effectful function
  in the tree.

  | atom | route |
  |---|---|
  | `read(r)`, `write(r)`, `net(d)`, `alloc`, `time`, `rand`, `term` | `NoObligation` |
  | `panic` | `NoObligation` — abort is described by its propagation |
  | `diverge` | `Implemented(measures)` — a well-founded ordering |
  | `blocks` | `Deferred(session_duality)` |

  `diverge` is the case that shows the shape. Termination is a liveness property,
  and the language reaches it by reducing it to well-founded descent, a finitary
  witness inside the function. `blocks` owes progress, whose witness is a peer's
  behaviour, so the reduction does not transfer. Its route is settled literature
  and unbuilt here: binary session duality carries deadlock freedom
  ([protocol types](../rfcs/0013-protocol-types.md) §Metatheory).

  **`Deferred` and `Open` are separated by whether a sound route is known**, and
  the same construct may sit on both sides. RFC-13 proposes binary sessions and
  records that multiparty session types are substantially harder, so progress is
  `Deferred(session_duality)` for two parties and `Open(multiparty_duality)`
  beyond. `Deferred` claims a route exists and is unbuilt; `Open` claims none is
  known.

  An atom whose route is `Deferred` or `Open` is **residual trust**, and an item
  carrying one is obliged to say so. The obligation is derived rather than
  written: the route table supplies the form, so a bare row carrying such an atom
  is a structured rejection naming what to add — `#[future(session_duality)]` for
  a `Deferred` route, `#[research(<question>)]` for an `Open` one. The
  requirement retires on its own when a route moves to `Implemented`, with
  nothing to un-write, and the route name is the retirement query: what to grep
  for when the route lands.

  This REQ supplies the classification. The attribute surface itself is a change
  to the item grammar and belongs with the RFC that adds it, not here. Two facts
  about today's grammar bear on that: `parse_attribute` accepts at most one
  attribute per item, and attribute names carry no `::` path.

- **REQ-14 (recorded boundary — R-DEFER-1, and why the basis lands without a
  production consumer):** `goal.md` R-DEFER-1 requires a commit adding a new pub
  API to add a non-test production consumer in the same commit. The basis lands
  without one. This REQ records the override, its grounds, and the condition that
  retires it, so the omission is a decision on the record rather than an oversight.

  **The grounds are structural.** Every use of a denotation is a check, and
  RFC-8's step 4 hands checking to RFC-9. A step whose output the next step
  consumes has no in-scope consumer by construction.

  **The candidates, and what each fails.** The diagnostic in
  `thermite-lower/src/lower.rs` is a message: a `Display` implementation that
  prints a lookup does not exercise the lookup, so an incorrect entry in REQ-5's
  map would produce an incorrect message and no failure. It also costs 7,506
  lines across the 13 design docs routed to that file, covering recursion
  schemes, strings, collections and maps, because `lower.rs` is shared by many
  components rather than because the effect diagnostic relates to them. A
  validator check that every label resolves to a basis home is total over the
  nine closed labels and cannot fire. REQ-13's attribute check cannot fire
  either, since `blocks` carries no surface label today. Row minimality — reading
  `read(r)` as implied by `write(r)` on one instance — rejects the row the
  current subsumption rule forces, because a `write(x)` caller calling a
  `read(x)` callee is rejected today and its author must declare both, so
  minimality is unsafe without the subsumption change REQ-11 excludes.

  **What discharges the rule's purpose instead.** R-DEFER-1 exists so new code is
  exercised and validated. AC-9 does that: it derives the footprint and the mask
  by separate routes from REQ-5's map and asserts they agree, so a wrong entry
  fails a test rather than printing a wrong message. The mask side is anchored to
  the Verus-proved `thermite_verified::subsumes_masks`, so the comparison is
  against established code rather than against this component's own output
  (R-CHAR-3). AC-10 does the same for the route table. Both fail without any
  program being written.

  **Retirement.** This boundary closes when RFC-9's `REQ-SPEC-EFFECT-CONFLICT`
  consumes REQ-8's commutation facts, which is the consumer RFC-8 names for them.
  At that point the basis has a production consumer and the override is spent.

## Acceptance criteria

- **AC-1 (basis totality):** A table-driven test asserts that every one of the
  nine `ast::Effect` variants resolves to a basis entry, with the expected entry
  hand-derived from REQ-5's table (R-CHAR-3). A tenth variant added to `Effect`
  without an entry fails this test. (REQ-5)

- **AC-2 (operation sets are the ones REQ-4 states):** `read(r)` resolves to the
  operation set `{get}` and `write(r)` to `{get, put}`, over the same instance
  when `r` is equal. Expected values hand-derived from REQ-4. (REQ-4)

- **AC-3 (`net` resolves to a combination):** `net(d)` resolves to a two-summand
  combination whose state summand is instanced at `d` and whose `io` summand is
  free, and its frame condition is the conjunction of the two. Hand-derived from
  REQ-6. (REQ-6)

- **AC-4 (declaration resolves to primitives):** `effect platform(d) = state(d)`
  parses to a declaration item and resolves to the basis entry `state(d)`. A
  declaration naming a non-basis right-hand side returns a structured resolution
  error listing the basis, never a panic (R-APG-1). (REQ-7)

- **AC-5 (the commutation table is computed, and matches):** The four `state`
  rows and the two `io` rows of REQ-8 are produced by the computation over
  operation sets, with the expected accept/reject values hand-derived from the
  theory equations rather than read back from the implementation (R-CHAR-3).
  (REQ-8)

- **AC-6 (the diagnostic carries the frame condition):** A crafted `! pure`
  caller over a `! net(d)` callee rejects with `EffectNotSubsumed` whose message
  names `net(d)`'s basis entry and frame condition, and whose `missing` set is
  the same atom set the pre-amendment check produced. The second half is the
  no-behaviour-change assertion. (REQ-9)

- **AC-8 (the footprint over the nine labels):** `footprint` returns the
  hand-derived read and write sets for each of REQ-5's nine entries: `read(r)`
  gives `({r}, {})`, `write(r)` gives `({r}, {r})`, `time` gives `({clock}, {})`,
  `alloc` gives `({heap}, {heap})`, `panic` and `diverge` give `({}, {})`, and
  `net(d)` gives `({d}, {d})` with the row marked as carrying an `io` summand.
  Expected values hand-derived from REQ-12 and REQ-5 (R-CHAR-3). (REQ-12)

- **AC-9 (the two projections agree where the map says they must):** Over rows
  built from the nine labels, the footprint and the existing mask satisfy:

  ```
  footprint(row).writes ≠ ∅  ⟺  mask(row) ∩ {Write, Alloc, Rand, Term} ≠ ∅
  footprint(row).reads  ≠ ∅  ⟺  mask(row) ∩ {Read, Write, Alloc, Time, Rand, Term} ≠ ∅
  ```

  Both sides are derived from REQ-5's map — the left by which entries carry `get`
  or `put`, the right by which labels the map sends to a state instance — so a
  disagreement means an entry in REQ-5's table is wrong. This is the check that
  can fail if the basis is mis-specified, and it fails without any program having
  to be written. Expected membership hand-derived from REQ-5 (R-CHAR-3).
  (REQ-5, REQ-12)

- **AC-10 (the route table over the nine labels, and the two that are not
  `NoObligation`):** `route` returns `NoObligation` for the seven state and io
  labels and for `panic`, and `Implemented(measures)` for `diverge`, with
  expected values hand-derived from REQ-13's table (R-CHAR-3). A label whose
  route is `Deferred` or `Open` reports the form or question it names, asserted
  over `blocks` as `Deferred(session_duality)` once `blocks` has a surface label.
  A tenth `Effect` variant added without a route entry fails this test, in the
  same way AC-1 catches one added without a basis entry. (REQ-13)

- **AC-7 (subsumption is untouched):** The existing `thermite-lower` suite passes
  unchanged, including `tests/effects_verified.rs`'s exhaustive 512×512
  mask-equivalence test. Mechanically: no diff to `subsumes`, `EffectKind`, or
  `thermite-verified`. (REQ-11)

## Architecture

Two files, in the dependency order `goal.md` R-DEFER-7 fixes.

`thermite-syntax/src/effect_basis.rs` holds the theories, the entries, the label
map (REQ-5), and combination (REQ-6/REQ-7). It depends on `ast::Effect` and on
nothing else, so it sits beside `desugar.rs` as an analysis over the AST.

`thermite-spec/src/effect_commutation.rs` holds the commutation computation
(REQ-8). It reads basis entries and returns accept/reject per pair. It is in
`thermite-spec` because `REQ-SPEC-EFFECT-COMMUTATION` is a spec-scoped
requirement in `.design/reqs/registry.toml` and because its consumer, RFC-9's
conflict rule, is spec-scoped as well.

### Why `net` decomposes and `term` does not

Both labels were unplaced when RFC-8 was written: its basis table says `net(d)`
and `term` have no home and that the design is incomplete until they do. They
resolve differently, and the evidence is in shipped code rather than in taste.

`term` is the terminal-control register. `examples/editor/editor.th` declares
`! term` on `raw_mode_on` and `raw_mode_off`, whose boundary is the `ioctl`
termios pair `tcgetattr`/`tcsetattr` — a `get` and a `put` on one region. The
same file declares `! read(input)` on `read_key_raw`, so reading a keypress is a
different label. RFC-9 §Migration reaches the same place from the other
direction: "`term` | 3 | becomes a read/write on a terminal region". So `term` is
`state(termios) / {get, put}`, a primitive instance.

`net` carries both halves of a conversation under one label, and the halves land
in different theories (REQ-6). Sending is a `put` toward a region; receiving
produces a value that no region this program frames over determines.

### What the `io` summand buys

The distinction between `state(r)` and `io(σ)` is which values a region
determines. Under `state(r)`, fixing the region's history fixes the value.
Under `io(σ)`, it does not.

That is what makes reproducibility expressible, which RFC-8 names as the payoff
and which §5.3 already asks for at the toolchain level: builds and check results
are bit-reproducible given the same toolchain and seeds, and a program's own
reproducibility is the same question one level down. A row whose entries are all
`state` instances is reproducible by fixing those regions. A row carrying an `io`
summand is not, and `net`'s `io` summand is what reports that.

`rand` maps to `state(entropy)` rather than to the `random` given atom, and the
two say different things. `state(entropy)` gives the frame condition "may modify
`entropy`, nothing else", which holds whether the region is a program-seeded PRNG
or the kernel pool; whether a particular entropy region is reproducible is a
property of that region, settled where it is declared. The `random` given atom
(REQ-3) is the separate claim that a value is drawn from a distribution, which is
a modelling assumption a user states rather than a property an implementation
has.

### Order of construction

1. REQ-1 through REQ-6 and REQ-9 in `effect_basis.rs` with the diagnostic
   amendment: the basis, the total label map, combination as data, and the
   consumer that reads them.
2. REQ-7 in `ast.rs` and `parser.rs`: the declaration item and its syntax. This
   step re-pins the eight design docs that govern those two files.
3. REQ-8 in `effect_commutation.rs`.

Step 1 leaves `net`'s combination available as internal data before step 2 makes
the form writable by a user, so REQ-5's totality does not wait on REQ-7.

## Verification

`cargo test -p thermite-syntax` and `cargo test -p thermite-spec`, covering AC-1
through AC-5; `cargo test -p thermite-lower` for AC-6 and AC-7.

Gauntlet (R-DEFER-6), per owning crate: `cargo test -p <crate>`,
`cargo clippy -p <crate> --all-targets -- -D warnings`, `cargo fmt --check`.

There is no golden Verus or corpus-certificate reference for the basis: it is a
denotation consumed by a diagnostic and by RFC-9, and it emits no certificate
field. Expected values in every AC are hand-derived from the theory equations or
from this document's tables (R-CHAR-3). The conformance corpus reaches this
component only through `conformance/provenance_demo.th`, whose three `! net(db)`
sites are the only `net` uses in the tree and which AC-7 holds unchanged.

## REQ status

| REQ | Status | Evidence |
|---|---|---|
| REQ-1 (admissibility criterion) | NOT-STARTED | open prerequisite: `thermite-syntax/src/effect_basis.rs` is unbuilt (route carries `unbuilt = true`). |
| REQ-2 (the basis: five theories) | NOT-STARTED | open prerequisite: as REQ-1. |
| REQ-3 (two given atoms) | NOT-STARTED | open prerequisite: as REQ-1. |
| REQ-4 (entry = instance + operations) | NOT-STARTED | open prerequisite: as REQ-1. |
| REQ-5 (label map is total) | NOT-STARTED | open prerequisite: as REQ-1. |
| REQ-6 (combination; `net`) | NOT-STARTED | open prerequisite: as REQ-1. |
| REQ-7 (declaration surface) | NOT-STARTED | open prerequisite: REQ-5's map, and the `ast.rs`/`parser.rs` item this adds. |
| REQ-8 (commutation computed) | NOT-STARTED | open prerequisite: `thermite-spec/src/effect_commutation.rs` is unbuilt; depends on REQ-4's operation sets. |
| REQ-9 (diagnostic names the theory) | NOT-STARTED | open prerequisite: REQ-5's map; amends `.design/lower/effect-subsumption.md` REQ-4. |
| REQ-10 (RFC-9 owns checking) | NOT-STARTED | boundary recorded; discharged when the three RFC-9 REQs are owned elsewhere. |
| REQ-11 (verified core does not move) | NOT-STARTED | boundary recorded; asserted by AC-7 once the basis lands. |
| REQ-12 (the footprint projection) | NOT-STARTED | open prerequisite: as REQ-1; depends on REQ-4's operation sets and REQ-5's map. The projection the relational frame lemma reads; asserted by AC-8 and AC-9. |
| REQ-13 (the discharge route) | NOT-STARTED | open prerequisite: as REQ-1. Supplies the classification that makes the `#[future(..)]` / `#[research(..)]` obligation derived; the attribute surface belongs to the RFC that adds it. Asserted by AC-10. |
| REQ-14 (R-DEFER-1 boundary) | NOT-STARTED | override recorded with its grounds and its retirement condition; discharged when RFC-9's `REQ-SPEC-EFFECT-CONFLICT` consumes REQ-8's commutation facts. |

## Open questions (for the orchestrator before the builder runs)

- **OQ-1 (`accrues(M)` has no surface label — resolved):** The basis carries
  `accrues(M)` and the nine current labels use none of it, so an earlier draft of
  this document recorded it as an unused theory carried against a later break.
  It has a named client.
  [The relational-contracts record](../research/relational-contracts.md) §5.3
  derives constant-time as `hides key_store in cost`, a two-run claim aimed at
  the accumulator, and it works because cost is already an effect over a monoid
  rather than a special case. So `accrues(M)` is load-bearing for a scheduled
  item and REQ-2 keeps it on that ground. What remains open is the monoid's
  surface, which arrives with `cost(E)`. Not a blocker.

- **OQ-2 (the contents of `σ`):** REQ-2 gives `io(σ)` a free signature and the
  entries that use it (`net`'s `io` summand, and terminal reads) do not yet name
  the operations in `σ`. Commutation over a free signature does not need them
  (REQ-8 rejects same-instance pairs regardless), so naming them is deferred
  until something reads them. Not a blocker.

- **OQ-3 (`owns(r)` commutes dynamically):** RFC-8 places `owns(r)` as
  `state(r)` whose commutation is established by a lock at run time, so
  `owns(r) ∥ owns(r)` accepts while `owns(r) ∥ write(r)` rejects. REQ-8 computes
  from static equations only, so a dynamically-established fact needs a second
  input. `owns` has no surface label today and arrives with
  [RFC-10](../rfcs/0010-shared-state-invariants.md). Not a blocker for the three
  REQs here.

- **OQ-4 (combination's commutation meet):** REQ-6 defines a combination's
  commutation fact as the conservative meet of its summands', so `net(d)` rejects
  against itself because both summands do. A combination whose summands disagree
  has no case in the tree yet. Recorded so the rule is stated before one exists.
  Not a blocker.
