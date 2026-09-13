# Tier-A relational frame metatheory

audited-content-sha256: dcdbfab3a92633255c995a5a75585a69c4671d14797c1b98218de36fafcb363e (implementation and executable claim-closure pin, 2026-09-12)

<!--
tier: research-derived implementation
status: implemented; final repository qualification pending
governing-roadmap: .design/post-t3-roadmap.md REQ-15 / AC-15
research-basis:
  - .design/research/relational-contracts.md §3, §5.1, §7, §10
  - .design/syntax/effect-algebra.md REQ-2 through REQ-6 and REQ-12
  - .design/syntax/verified-effect-rows.md residual trust and out-of-scope boundary
  - .design/verified/thermite-semantics.md T1/T2 semantic-preservation architecture
-->

## Summary

Thermite's current `lean/Thermite/EffectRows.lean::relational_frame` packages
`ItemSemantics.resultCongruent` and `ItemSemantics.framesWrites`; it does not
derive either property from Thermite execution semantics or the checked effect
row. This design closes that gap. It defines a compositional two-run semantics,
derives the largest relational projection licensed by each existing effect
theory, checks an exact per-artifact witness, and transports the resulting claim
through the already-proved source-to-lowering faithfulness routes.

The unit of success is not a new surface clause. It is an end-to-end certificate
whose relational facts are consequences of the program, operational semantics,
effect laws, and checked artifact identity rather than fields supplied as free
premises. Effects that need new mathematics remain usable, but they cannot mint
the unsupported projection; the certificate names the missing research gate.

## Requirements

- REQ-1: The Tier-A frame result shall derive result congruence and write-frame
  facts from Thermite operational semantics plus checked effect-row laws. The
  admitted theorem path shall not accept `resultCongruent`, `framesWrites`, or
  an equivalent restatement of either conclusion as an unchecked semantic
  structure field.
- REQ-2: Relational support shall be maximal under the current metatheory rather
  than restricted to a hand-picked convenience fragment. A closed, exhaustive
  classification shall assign every primitive in
  `.design/syntax/effect-algebra.md` and every current
  `thermite_lower::effects::EffectKind` the relational projections it can
  justify: result, write-frame, terminal-outcome, termination, trace, and
  accumulator agreement.
- REQ-3: The canonical semantic carrier shall extend the existing bounded-value
  and body-state denotations in `lean/Thermite/Exec.lean` and
  `lean/Thermite/Exec/Stmt.lean` with declared region state and explicit
  outcomes. It shall distinguish completed partial correctness, exceptional
  outcomes, divergence, external observations, and peer-dependent blocking
  rather than treating each as an ordinary returned value.
- REQ-4: Row composition shall compute the strongest supported relational
  projection from the laws of every row entry. Region agreement shall use the
  canonical read/write footprint and overlap semantics already shared by
  `.design/syntax/effect-algebra.md`, `thermite-lower/src/effects.rs`, and
  `lean/Thermite/EffectRows.lean`; a row containing one unsupported projection
  shall not erase independent projections that remain provable.
- REQ-5: Lean shall prove a universal relational theorem over the canonical
  semantics. The proof shall proceed from primitive-effect laws through row and
  program composition, including self-composed execution where two-run
  reasoning is required, and shall expose no project-specific axiom or `sorry`.
- REQ-6: Each admitted artifact shall carry a canonical input and a derived
  witness binding the exact program identity, body, normalized row, read/write
  footprint, effect-law classification, semantic fragment, and requested
  relational projections. The checker shall follow the
  `Canonical`/`Witness`/`producerRefines` pattern in
  `lean/Thermite/CheckedTraversal.lean` and the emitter/replay pattern in
  `thermite-lower/src/witness.rs`.
- REQ-7: A source-level relational result shall become an end-to-end artifact
  claim only through a proved semantic-preservation or verified-validation
  route. The design shall reuse `body_ref_sound`, the supported loop semantics,
  and `lean/Thermite/Faithfulness.lean` wherever their admitted fragments apply,
  and shall label any result whose executable transport is still open as
  source-only.
- REQ-8: The certificate representation in `forge/src/manifest.rs` shall retain
  relational projections independently. It shall identify whether each fact is
  derived, conditional on a named coupling/input relation, source-only,
  end-to-end transported, or research-gated, together with its exact witness
  and theorem receipt. Presentation code shall not promote a partial projection
  into a stronger aggregate claim.
- REQ-9: The evidence suite shall include hostile mutations of the program,
  row, footprint, region path, effect law, outcome classification, transport
  witness, and artifact identity. Weakening, deletion, redirection, or
  substitution shall either be rejected or remove the affected relational
  projection; it shall never retain authority through stale evidence.
- REQ-10: Research-gated boundaries shall be typed and exhaustive. At minimum,
  the design shall distinguish bare probabilistic `random`, peer-progress for
  `blocks`, unconditional value equality for free `io(σ)`,
  termination-sensitive claims for `partiality`, and later `hides`, cost,
  sensitivity, coupling, and certificate-algebra work. Adding a gate shall not
  be interpreted as discharging it.
- REQ-11: This Tier-A work shall add no relational surface syntax. Existing
  Thermite programs and certificates shall remain readable, while absence of
  the new evidence shall mean that no new relational authority is present.
- REQ-12: The implementation plan shall be vertically sliced so the first
  admitted fixture exercises source semantics, paired execution, row-law
  derivation, per-artifact replay, semantic transport, and certificate output.
  Subsequent increments may widen construct coverage only while preserving that
  same complete chain.

## Acceptance Criteria

- [x] AC-1: (REQ-1, REQ-5) The admitted Lean theorem derives result congruence
  and write framing without taking either fact, or a proposition definitionally
  equivalent to it, as a premise; `#print axioms` reports only the repository's
  enumerated standard Lean axioms and no `sorryAx` or project axiom.
- [x] AC-2: (REQ-2, REQ-10) A checked classification artifact covers every
  primitive effect theory and all twelve `EffectKind` variants in
  `thermite-lower/src/effects.rs`; deleting or duplicating a row fails the gate.
- [x] AC-3: (REQ-2, REQ-3, REQ-4) Focused theorem tests demonstrate the maximal
  supported projections for state, monoid accumulation, I/O, exception, and
  partiality, plus explicit research-gated results for bare `random` and
  `blocks`.
- [x] AC-4: (REQ-3, REQ-5) The paired semantics has non-vacuous examples for
  equal and unequal initial footprints, a permitted write, a forbidden outside
  write, a successful pair, an exceptional outcome, and a partial execution.
- [x] AC-5: (REQ-4) Row-composition tests show that a component lacking result
  congruence can still retain a proved write-frame projection, and that region
  overlap uses the same canonical decision as the production footprint path.
- [x] AC-6: (REQ-6) An emitted canonical/witness pair replays in Lean and binds
  the exact artifact digest, normalized row, footprints, fragment identifier,
  requested projections, and derived classification; independent mutations of
  every bound field fail replay.
- [x] AC-7: (REQ-7) At least one real compiled Thermite function produces an
  end-to-end relational claim by composing the source theorem with an existing
  T1/T2 transport theorem; a fixture outside the transported fragment is
  reported as source-only rather than end-to-end.
- [x] AC-8: (REQ-7, REQ-12) The end-to-end fixture fails when its production
  lowering is replaced by a semantically different lowering even when its
  source-level relational theorem remains true.
- [x] AC-9: (REQ-8, REQ-10, REQ-11) Certificate JSON, `forge review`, and
  `forge audit` render the same typed relational projections and named research
  gates, and a pre-feature certificate deserializes without acquiring relational
  authority.
- [x] AC-10: (REQ-9) Hostile tests kill weakened, deleted, redirected, stale,
  and cross-artifact relational evidence or return an explicit unsupported
  classification; no mutant is accepted with its original authority.
- [x] AC-11: (REQ-11) Parser and lowering golden tests confirm that no new
  source clause or spelling is accepted by this Tier-A increment.
- [x] AC-12: (REQ-12) The implementation sequence names a focused command set
  for each vertical increment and one final qualification boundary, with claim
  materialization deferred until the implementation tree is stable.

## Architecture

### The correction: properties are conclusions, not semantic inputs

`lean/Thermite/EffectRows.lean::ItemSemantics` currently contains a `run`
relation and two fields named `resultCongruent` and `framesWrites`. The theorem
`relational_frame` constructs its conjunction by invoking those fields. That
wrapper is internally valid, but the hard relational claims enter before the
proof begins. The new route retains a run/denotation carrier and replaces the
free relational fields with data that can be checked: canonical program syntax,
the normalized row, the finite footprint, and an effect-law classification.

The universal theorem must reach congruence and framing by induction over the
canonical execution and row derivation. A structure named `RowSound` is useful
only if its evidence is itself derived or replay-checked; moving
`resultCongruent` behind another field name would not satisfy REQ-1.

### Canonical execution and relational observation

The source side begins with the existing semantic family:

- `lean/Thermite/Exec.lean` gives bounded exec-expression values and explicit
  `Option` partiality;
- `lean/Thermite/Exec/Stmt.lean` gives the `S_B` big-step state transformer;
- `lean/Thermite/Exec/Loop.lean` gives fuel-indexed v1-loop execution and its
  partial-correctness rule;
- `lean/Thermite/Faithfulness.lean` composes the proven reference-encoder
  equalities with per-artifact translation-validation premises.

The Tier-A carrier extends this state with a declared region store rather than
maintaining the unrelated `Region → Nat` abbreviation as the entire program
state. Values in regions must use the canonical executable value domain or a
clearly defined abstraction relation to it. An execution outcome distinguishes
normal return, exceptional completion, and absence of a completed derivation.
External observations and ghost accumulators are separate projections so that
I/O nondeterminism cannot be mistaken for region-state nondeterminism.

A relational observation over two executions records:

1. the initial agreement relation, including explicit inputs, declared read
   regions, and any named environmental coupling;
2. the two execution derivations or completed outcomes;
3. equality or a declared relation on results/outcomes;
4. final agreement outside the canonical write footprint;
5. equality or a declared relation on traces and accumulators; and
6. the transport scope: source-only or a named emitted-artifact theorem.

The base theorem is partial-correctness shaped: it says what follows for two
completed derivations. A separate termination projection records whether the
current evidence proves that matching initial states complete together.

### Maximal effect-law classification

The classification is closed and generated from the effect algebra, not a list
maintained only in prose. The initial expected interpretation is:

| Effect-theory or surface entry | Relational consequence available from current metatheory |
| --- | --- |
| `read(r)` | Result dependence may inspect `r`; no region is modified. |
| `write(r)` | Result dependence may inspect `r`; only overlapping `r` may be modified. |
| `alloc` | State law over the ambient `heap` region. |
| `time` | State-read law over the ambient `clock` region; equality requires equal clocks. |
| surface `rand` | State law over the ambient `entropy` region; this is not the bare `random` atom. |
| `term` | State law over the ambient `termios` region. |
| `accrues(M)` | Accumulator relation derived from the declared monoid operation and identity. |
| `io(σ)` | Region non-modification; result/trace equality only under a named relation on environmental observations. |
| `net(d)` | Meet of `state(d)` and `io(σ_d)`: state framing survives even when unconditional result equality does not. |
| `exception` / `panic` | Frame facts for represented terminal outcomes; no normal-return claim. |
| `partiality` / `diverge` | Facts about completed pairs; termination only through a separately checked measure. |
| bare `random` | Research-gated until its nondeterministic or distributional denotation is fixed. |
| `blocks` | Research-gated on a non-local progress witness such as proved session duality. |
| `owns` / `forgets` | Structural authority/resource constraints on witness admissibility, not invented result equations. |

For a sum or row, each projection composes independently. `net(d)` is the
worked example: the free I/O summand prevents unconditional result equality,
but it does not erase the state summand's write frame. This product-of-facts
view replaces the current tendency to describe “the relational frame theorem”
as one indivisible Boolean.

### Universal proof and per-artifact replay

The Lean layer proves primitive rules, row-composition rules, and the universal
program theorem. Deterministic state uses lockstep self-composition, exactly as
`.design/research/relational-contracts.md` specifies. Conditional I/O facts use
an explicit relation on observations; the relation must appear in the claim and
cannot be synthesized from an empty footprint. No distributional or liveness
law is smuggled into Tier A.

The artifact layer follows the established RFC-10 replay shape:

```text
CanonicalRelationalInput
  program/body identity
  normalized effect row
  canonical read/write footprints
  semantic-fragment identity
  requested relational projections

RelationalWitness
  derived effect-law rows
  paired-execution obligations
  optional named coupling/input relation
  source-to-target transport receipt
```

The producer derives the witness from the canonical input. Lean separately
checks `producerRefines`-style equality and the semantic theorem. The artifact
digest and all authority-bearing fields are included in the receipt identity so
that a witness from another body, row, or lowering cannot be replayed.

### Transport to the emitted artifact

The source theorem is necessary but not sufficient for an executable claim.
For the already mechanized frozen subset, the route composes with
`body_ref_sound` and the T2 theorem in `lean/Thermite/Faithfulness.lean`. The
shared-region extension must define the relation between the region store and
the target state explicitly and prove that the relevant reference/production
encoder preserves it.

Coverage is monotone. A construct with a proven source denotation but no target
transport may report a source-only fact. A construct with both may report an
end-to-end fact. Adding a transport theorem upgrades only artifacts whose exact
witness selects that theorem; it does not retroactively upgrade old
certificates by schema inference.

### Certificate and assurance semantics

Relational evidence is a typed claim set integrated with the current authority
path in `forge/src/manifest.rs`, not a parallel Boolean badge. Each projection
names:

- the relation proved;
- its population/arity, including two-run evidence where applicable;
- the discharge route and theorem receipt;
- source-only versus transported scope;
- conditional inputs or coupling witnesses; and
- named research gates and residual trust.

Project aggregation may compare or meet compatible projections, but it may not
turn a state-frame fact into result determinism, a completed-run fact into a
termination theorem, or a source-only theorem into an end-to-end one.

### Implementation increments

1. **Relational semantic kernel:** introduce the region-aware paired carrier,
   explicit outcomes, the exhaustive effect-law classification, and
   non-vacuous Lean examples.
2. **Maximal law derivation:** prove primitive, row-composition, and program
   theorems for every projection licensed by the current algebra.
3. **Artifact witness:** emit and replay canonical relational inputs and derived
   witnesses, including mutation tests and exact identity binding.
4. **Semantic transport:** compose supported cases with T1/T2, extend the
   shared-region correspondence, and land one complete real fixture.
5. **Certificate integration:** add typed projections to the current assurance
   authority path, render them consistently, and run the full hostile suite.

Each increment ends with focused Lean/Rust checks. Content-bound claim receipts
are materialized once after the complete tree stabilizes, followed by the full
qualification and exact-head review protocol used elsewhere in the repository.

## Implementation evidence

The implementation preserves the design's separation between a source theorem
and executable transport. `Thermite.RelationalFrame.Bounded.Program.relational_frame`
derives paired result and frame conclusions from the executable program and
initial agreement. `Thermite.RelationalFrameWitness.produce_complete` admits an
exact canonical input/witness pair. Only
`Thermite.RelationalFrameTransport.bounded_return_pair_end_to_end` transports
the region-free return fragment, and the transport receipt lists the exact
transported projection subset. Consequently the current identity fixture shows
`Result`, `Outcome`, and `Termination` end to end while `WriteFrame` remains
source-only; the shared-state increment fixture remains wholly source-only.

The following checked surfaces discharge the acceptance criteria:

| Evidence | Discharged boundary |
| --- | --- |
| `lean/Thermite/RelationalFrame.lean` and `gates/lean-axiom-probe.sh` | Universal paired execution, explicit outcomes, non-vacuous examples, exhaustive primitive/`EffectKind` lists, independent projection composition, and the allowed-axiom boundary (AC-1 through AC-5). |
| `lean/Thermite/RelationalFrameWitness.lean` and `thermite-lower/src/relational_frame_witness.rs` | Canonical body, row, footprint, effect-law, fragment, projection, scope, and artifact binding; every authority-bearing field is mutation-tested (AC-2, AC-6, AC-10). |
| `lean/Thermite/RelationalFrameTransport.lean` and `thermite-lower/tests/relational_frame_witness.rs` | Exact T2 return transport, changed-body/cross-artifact receipt rejection, and source-only fallback outside that transport (AC-7, AC-8). |
| `forge/src/check.rs`, `forge/src/manifest.rs`, `forge/src/review.rs`, `forge/src/audit.rs`, and `forge/src/cli.rs` | One private audit authority feeds certificate JSON, review, and audit without inferring authority for legacy certificates or promoting source-only projections (AC-9, AC-10). |
| `cargo test -p thermite-syntax` and an empty `git diff origin/main -- thermite-syntax` | The existing language and parser corpus remain unchanged; Tier A adds no surface spelling (AC-11). |

Focused iteration used these commands:

```text
cargo test -p thermite-lower relational_frame --lib
cargo test -p thermite-lower --test relational_frame_witness
cargo test -p forge --bin forge relational_ -- --nocapture
cargo test -p thermite-syntax
cargo clippy -p thermite-lower -p forge --all-targets -- -D warnings
cargo fmt --all --check
bash gates/lean-axiom-probe.sh
```

The final boundary is deliberately singular: after the implementation and this
evidence map are stable, add the typed claim-closure draft, materialize the
content-bound receipts once, regenerate the requirement status view, and then
run the complete registry, route/path, documentation-drift, claim-completeness,
workspace build/test/clippy/fmt, and Lean axiom gates. The committed exact head
is then the unit of adversarial review and CI admission. A change to an
authority-bearing artifact after materialization invalidates that boundary and
requires one new materialization, qualification, and exact-head review cycle.

## Resolved Questions

- Q-1: The target is the missing semantic derivation of the Tier-A relational frame
  result; `hides` syntax and later relational clauses are not part of this slot.
- Q-2: Coverage is the largest fragment supported by existing metatheory. The design
  shall not impose a smaller convenience subset; genuinely new mathematics is
  isolated behind named research gates.
- Q-3: The proof architecture combines one universal Lean metatheorem with exact
  per-artifact checked witnesses.
- Q-4: End-to-end transport is required wherever a proved semantic-preservation
  route exists. Source-only facts remain available but are labeled and cannot
  be presented as executable guarantees.

## Residual trust

- The formal operational semantics remains a model of the intended Thermite
  language. Kernel checking can prove consequences of that model but cannot
  prove that the model captures the language the user meant.
- The source-to-target claim inherits the explicit residuals in
  `.design/verified/thermite-semantics.md`, including the relation between
  Thermite and Verus/Rust target states and any T2 premise still discharged by
  Verus/Z3 rather than reconstructed in Lean.
- The Lean kernel, Mathlib definitions used by the proof, and the exact allowed
  axiom set reported by the theorem probe remain trusted as enumerated inputs.
- Platform declarations that associate ambient regions such as `heap`, `clock`,
  `entropy`, and `termios` with real resources remain target-owned. The theorem
  proves facts about the declared model, not undocumented hardware behavior.
- A conditional I/O result or trace claim remains only as strong as its named
  environmental relation. The relation is part of the certificate and cannot
  be treated as a theorem about the external system.
- The artifact path remains dependent on canonical serialization, digest
  binding, and the Rust-to-Lean input correspondence. Hostile replay tests make
  that boundary observable but do not turn the Rust producer itself into a
  kernel-verified compiler.
- Implementation may expose proof-engineering choices, but any choice that
  changes the semantic claim, effect classification, trust boundary, or
  end-to-end scope requires a design amendment rather than an
  implementation-local decision.

## Out of Scope

- Adding `hides`, `varies`, `distributes`, `couples`, or `matches` syntax.
- Choosing a denotation for bare `random` or proving probabilistic coupling
  soundness.
- Proving peer-dependent progress, global deadlock freedom, or a general
  liveness calculus for `blocks`.
- Termination-sensitive noninterference without an existing checked termination
  witness.
- Constant-time backend leakage models and cost-instrumentation validation.
- Tier-B operation-graded information-flow derivation and Tier-C dynamic
  authority or certificate-algebra research.
- Verifying the entire Rust compiler/lowerer as one monolithic theorem.
