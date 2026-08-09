# Stage 4: checked reconstruction for the stratified cage

<!--
tier: 3-component
status: shipped
audited-content-sha256: 6e82abe29360eb22fb84a348c29e991e53fc345265887654a0c5ebd99e8b4cf9 (re-pinned 2026-08-08 for the RFC-17 wire-format pinning: three serialized structs gained #[serde(rename)] so the audit manifest and certificate bindings keep their v1 keys while the Rust fields carry the full word. Attributes only; no logic changed. prior: 6d3327bb4cde80d7c516c57ea0f554d9a90a292380544118cecb4dd6c22b89f2, previously (re-pinned 2026-08-08 for RFC-17: the clause vocabulary moved into the AST and the token kinds - Contract/LemmaItem{req,ens,fx} and FnItem/SpecFnItem/PropFnItem/LoopNode.dec and StructItem.inv to the full words the surface already uses, plus TokKind::{Req,Ens,Fx,Inv,Dec}. Type-directed: cargo check --workspace --all-targets exiting 0 is the completeness proof. prior: 84f0a1faece7cc3a3254283f62f63298ee82161b0da27cbcce8eae6d3a10c364, previously (re-pinned 2026-08-08 for RFC-17: the AST field names and TokKind variants moved to the full words the surface already uses - Contract{req,ens,fx} to {requires,ensures,effects}, TokKind::{Req,Ens,Fx,Inv,Dec} to {Requires,Ensures,Effects,Keeps,Measures}. A type-directed rename with no semantic content: cargo check --workspace --all-targets exiting 0 IS the completeness proof, since an unrenamed site does not compile. prior: 0f6f176b8e926cf7292c869641b0e53f263d0cc2bb8c0411d459bd4eb3feb3f8, previously (re-pinned 2026-08-08 for rustfmt only: migrating `req`/`ens`/`fx` to `requires`/`ensures`/`!` lengthened call sites past the width, so rustfmt re-wrapped them and added trailing commas. No governed file changed meaning; the wrapped lines are `parse_program(...)`-style test fixtures. prior: 34d29a283b500fd031013fc3ae8b67065e7d3314738af6272df9ccc6365554bd, previously (re-pinned 2026-08-07 for RFC-6: the governed files moved from the v2 clause surface (`req`/`ens`/`fx`/`inv`/`dec`) to full words with the effect row on the arrow (`requires`/`ensures`/`!`/`keeps`/`measures`). Prose in this document was migrated in the same commit, so the pin covers a re-read rather than a bump. prior: 842320f86217266716ff38b2e6d278d6d2afac06016e1fe9aa1ab54cde41caee, previously (re-pinned 2026-08-06: `.github/workflows/ci.yml` gains one step, the RFC front-matter gate (#127). Nothing else this document governs changed, and the pipeline it describes is unaffected) (re-pinned 2026-08-01 after restoring the Rust cache before the pinned Stage 4 tools; Gate G4 identities remain enforced))))))
governs: canonical S₂.0 bridge, typed Lean reconstruction, production routing,
         audit boundary, proof tooling, and Gate G4 (see tooling/spec-routes.toml)
-->

Status: shipped. Gate G4 is `bash scripts/g4-gate.sh`.

Stage 2 proved the shape of the stratified encoder, but left relation and
array-property atoms interpreted by the solver. Stage 3 added checked replay for
QF_LIA and QF_BV. Stage 4 closes the remaining S₂.0 trust gap: every formula
accepted by the current stratified classifier must either produce a genuine
countermodel or a kernel-checked proof of its actual `req → clause` theorem.

The external solver may search for a proof, but it is not trusted. Lean rebuilds
the finite grounding and CNF, checks an LRAT certificate, and derives the clause
theorem. `scripts/install-g4-tools.sh` builds CaDiCaL 2.1.3 at
`f13d74439a5b5c963ac5b02d05ce93a8098018b8` and drat-trim at
`effa1dcce85c878236f8313133dff1a2b766cd7c`; the gate accepts only those
identities.

CI restores the general Rust build cache before the separately keyed Stage 4
tool cache. The installer then validates both executable identities immediately
before Gate G4, so a stale Cargo `target` snapshot cannot replace the pinned
solver pair after validation.

False clauses take the other path. CaDiCaL supplies a Boolean assignment, Forge
realizes its QF_LIA and QF_BV leaves, and Lean checks the concrete source model.
Z3 is used only to find an integer witness; `omega` checks that witness. If a
Boolean assignment is impossible under the typed arithmetic, Forge blocks that
exact mask and asks CaDiCaL for another one. Missing tools, exhausted budgets,
and unrealized models are named failures, never proofs.

## Scope

`S2Recon` is exactly the current S₂.0 admission language:

- formulas: atoms, negation, conjunction, disjunction, implication, and sorted
  universal and existential binders;
- terms: bound variables, named constants, valued literals, sequence reads and
  lengths, width-preserving casts, index offsets, admitted multiplication, and
  unary spec-function applications;
- relations: equality, inequality, and the four ordered comparisons;
- embedded quantifier-free atoms, discharged through the existing QF_LIA and
  QF_BV replay paths.

An admitted formula returning `Unsupported` is a gate failure. Later fragment
versions may add sequence-sort binders, nested sequences, floating point, or
higher-order and recursive propositions; those are not S₂.0.

For postconditions, admission is decided after replacing `result` with the
source body. The substitution descends through calls, method receivers, fields,
indexes, tuples, casts, and quantifiers. If the body introduces an operation
outside S₂.0, such as `to_string`, the grounded obligation stays on the ordinary
backend. Forge never models `result` as an unconstrained reconstruction
constant.

## Requirements

### REQ-1 — freeze the reconstruction fragment

The production bridge and classifier share one explicit S₂.0 constructor
inventory. Every admitted formula, term, relation, and QF leaf has a positive
test; nearby unsupported constructs have named refusal tests. A later widening
must change the versioned canonical wire format.

### REQ-2 — one canonical clause representation

Translate real `thermite_syntax::Expr` clauses into the classifier language.
The translation carries:

- actual literal values and stable names for free constants;
- binder sorts and de Bruijn indices;
- source item and clause addresses;
- declared function signatures;
- array, length, cast, and index operations.

Its wire form is deterministic. Classification, SMT emission, Lean replay,
certificate hashing, diagnostics, and drift checks consume this representation.

### REQ-3 — typed relation and array semantics

Lean defines sort-indexed carriers and interpretations for constants, relations,
unary functions, sequences, reads, and lengths. `evalTm`, `evalAtom`, and
`evalFrm` are total on well-sorted S₂.0 syntax.

`strat_lowering_faithful` is strengthened so relation atoms use this semantics
instead of an unconstrained Boolean oracle.

### REQ-4 — checked normalization and Skolemization

NNF, prenex conversion, substitution, and Skolemization preserve the relevant
meaning or satisfiability statement. The reconstruction polarity is the
negation of the actual validity query: `req ∧ ¬clause`.

### REQ-5 — finite grounding

Lean builds a sort-indexed ground universe from constants and Skolem terms,
closing under admitted functions in sort-graph order. Acyclicity proves
termination. Exhaustive instantiation is equisatisfiable with every admitted
S₂.0 formula.

### REQ-6 — checked ground theory

The ground formula includes justified clauses for equality, relation and
function congruence, reads, lengths, supported array extensionality, casts, and
index operations. Arithmetic leaves use the QF_LIA or QF_BV checker. Every
theory lemma is proved in Lean or has its own checked replay evidence.

### REQ-7 — CNF and LRAT

Lean recomputes a Tseitin CNF from the grounded formula and proves the CNF
correspondence. A pinned proof-producing SAT solver emits LRAT. The existing
kernel LRAT checker derives unsatisfiability and therefore `req → clause`.

Missing Lean, Z3, SAT, or certificate tooling is a failure, never a skip.

### REQ-8 — complete evidence and cache keys

Reconstruction evidence records the source, canonical IR, solver query, ground
universe, CNF, and LRAT hashes; fragment version; instantiation and theory-clause
counts; theorem and checker; axiom report; elapsed time; and budget result.

Every verdict-determining field participates in the proof-cache key.

### REQ-9 — automatic production routing

Normal `forge check` classifies each real clause. Admitted S₂.0 clauses route
through reconstruction by default and certify at L4 only after successful
kernel replay. False clauses return concrete finite models. Timeouts and replay
failures remain named failures and never migrate trust.

The `@bv` parser plumbing is enabled in normal release builds, and a tagged
clause automatically selects the bit-vector route. Explicit engine flags remain
diagnostic overrides.

Automatic routing does not erase a settled result from an earlier gate. A
witnessed body-safety failure, vacuous contract, weak contract, or triage
refusal remains a failure; Lean is only a fallback after a genuine timeout or
timeout-derived degrade. Boundary and `#[slag]` functions also keep their L1
scope. Their bodies are foreign or trusted by fiat, so reconstructing a
postcondition cannot raise the implementation assurance to L4.

### REQ-10 — Gate G4

One fail-fast gate covers the complete S₂.0 constructor inventory, true and
false formulas, malformed or tampered evidence, missing dependencies, generated
Rust/Lean/solver differential tests, the axiom allowlist, and the absence of
`sorry`, `admit`, custom axioms, and `native_decide`. The gate applies a 6 GiB
address-space ceiling, serializes Rust tests and reconstruction runs, and uses
one Lean worker so the same command is safe on low-memory builders.
The sharded CI jobs prebuild `Thermite.Strat.EprReplay` and
`Thermite.Strat.TestModel` before nextest starts, so separate test processes do
not race to compile the shared Lean artifacts.

## Acceptance criteria

- [x] Source clauses translate deterministically to a well-sorted canonical IR,
  and every source construct in S₂.0 has a positive and a refusal test.
- [x] Relation and array atoms have typed Lean semantics and no longer pass
  through a free `relModel`.
- [x] Normalization, Skolemization, and grounding theorems are axiom-clean and
  have negative pins for polarity, capture, dependency omission, and empty
  carriers.
- [x] Every admitted corpus clause reconstructs or returns a checked
  countermodel; none is unsupported.
- [x] Lean rebuilds the CNF and accepts the exact `req → clause` theorem only
  after LRAT verification.
- [x] Equality, congruence, reads, lengths, casts, index offsets, and mixed
  QF_LIA/QF_BV leaves have end-to-end tests.
- [x] Certificate tampering at every recorded hash boundary fails.
- [x] Normal release builds accept `@bv`, and ordinary `forge check` performs
  clause routing without an engine flag.
- [x] Restratified formulas and their side obligations have positive and
  fail-closed tests.
- [x] `bash scripts/g4-gate.sh`, the workspace tests, Lean build and axiom probe,
  audit, drift checks, and requirement-registry checks all pass with no skipped
  dependency.

## Implementation map

- `thermite-spec/src/s2_recon.rs` owns the source-to-canonical bridge and its
  deterministic wire format.
- `lean/Thermite/Strat/Model.lean` through `EprReplay.lean` define the typed
  semantics, normalization, Skolemization, grounding, theory closure, and final
  implication theorem.
- `forge/src/epr_reconstruct.rs` runs the pinned SAT/LRAT tools, validates
  models, replays proofs, records evidence, and maintains the checked cache.
- `forge/src/check.rs` applies the route automatically and changes trust only
  after checked replay.
- `scripts/g4-gate.sh` is the memory-bounded completion gate.

## Residual trust after G4

The Lean kernel, its standard axiom set, the Rust-to-Lean source correspondence,
and the compiler toolchain remain visible trust boundaries. The SAT solver and
SMT solver do not remain in the trust line of a successfully reconstructed
S₂.0 clause.

Formulas rejected by S₂.0 remain forge-routed with their classifier reason.
The finite countermodel search has explicit model-family and QF-mask budgets; an
exhausted budget returns `Timeout` or a named reconstruction failure, not a
fabricated counterexample. Fragment widenings require a new grammar, semantics,
pins, and gate.
