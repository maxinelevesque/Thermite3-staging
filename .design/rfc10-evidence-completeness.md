# Feature: RFC-10 Evidence Completeness

## Summary

Define the complete current RFC-10 evidence-replay fragment explicitly and prove that every structurally supported canonical AST/witness pair is accepted by the Lean replay checker. Keep this theorem separate from Verus discharge, mutation policy, and final L3 certification, while establishing the pattern later RFCs can extend monotonically.

## Requirements

- REQ-1: `lean/Thermite/CheckedTraversal.lean` defines a declarative `SupportedRFC10` proposition covering every field and semantic obligation consumed by the current `verify` checker.
- REQ-2: `lean/Thermite/CheckedTraversal.lean` proves both completeness (`SupportedRFC10 ast witness -> verify ast witness = true`) and the existing soundness direction, yielding an acceptance characterization rather than a one-way audit claim.
- REQ-3: `SupportedRFC10` covers the complete current semantic-inventory surface represented by `thermite-syntax/src/semantic.rs`, including nested control flow, closures, matches, quantifiers, early exits, holdings, shared places, calls, and direct/transitive footprints.
- REQ-4: logical completeness is independent of the finite Rust traversal budget; `thermite-syntax/src/semantic.rs` and tests state and check the sufficient-budget relation for the bounded inventory and walk implementations.
- REQ-5: `thermite-lower/tests/traversal_witness.rs` demonstrates that production witnesses for representative nodes from the complete current RFC-10 inventory elaborate against the universal completeness theorem, not only a generated one-off `by simp` goal.
- REQ-6: `gates/lean-axiom-probe.sh` builds and axiom-checks the completeness theorem under the repository's existing Lean trust policy.
- REQ-7: the scope excludes Verus proof discharge, mutation scoring, contract-quality policy, and final L3 certification; those stages remain named follow-up work rather than implicit premises.

## Acceptance Criteria

- [x] AC-1: REQ-1 and REQ-2 — `#print axioms Thermite.CheckedTraversal.verify_complete` succeeds and its axioms are within the repository allowlist.
- [x] AC-2: REQ-2 — Lean proves `verify ast witness = true <-> SupportedRFC10 ast witness` for arbitrary inputs without `sorry`.
- [x] AC-3: REQ-3 and REQ-5 — a Rust conformance matrix includes bindings, nested control flow, early exits, holdings, shared reads/writes, free calls, methods, constructors, and literals, and every produced witness passes Lean replay.
- [x] AC-4: REQ-4 — tests show `semantic_inventory` succeeds at its exact node-count budget and `walk_semantic` succeeds at twice that count, while one-less budgets return `ResourceLimit`.
- [x] AC-5: REQ-6 — `bash gates/lean-axiom-probe.sh` includes `Thermite.CheckedTraversal.verify_complete` and passes.
- [x] AC-6: REQ-7 — the design and follow-up GitHub issue explicitly distinguish evidence completeness, producer refinement, and end-to-end certification completeness.

## Architecture

`lean/Thermite/CheckedTraversal.lean` remains the formal boundary. `SupportedRFC10` is declarative: it names version agreement, structural identity, well-formed edges, exact canonical direct footprints and calls, call closure, holding coverage and well-formedness, and shared-place coverage and well-formedness. `verify_complete` proves that these obligations are sufficient for the executable Boolean checker to accept; `verify_sound` retains the reverse direction. The combined theorem makes the supported fragment inspectable and versionable.

The theorem is deliberately about the replay evidence layer. Lean's `produce` assembles a canonical witness from the rendered canonical projection and computes footprint closure. The Rust producer in `thermite-lower/src/witness.rs` remains executable, while every generated replay proves `producerRefines ast witness`: all structural and payload fields agree exactly, and each transitive-footprint row agrees as an effect set (Rust `BTreeSet` order is not semantic). Direct footprints and free calls are produced by genuinely different Rust traversals; holding and shared-place payloads are duplicated same-algorithm walks, so agreement proves transport integrity but still trusts their shared algorithm. `thermite-lower/tests/traversal_witness.rs` exercises the refinement across the cross-language conformance matrix.

`thermite-syntax/src/semantic.rs` keeps resource exhaustion operational and non-certifying. Completeness quantifies over the logical finite inventory; the implementation lemma is expressed by exact-budget tests because the inventory requires one unit per node and the event walk requires two units per node. Resource availability is therefore separated from language support.

Future completeness fragments should be named versions with an inclusion theorem from the preceding fragment. A subsequent RFC that adds syntax or semantics must either prove the old predicate implies the new predicate and add the new induction cases, or leave the construct outside the new supported classifier. This repository-wide discipline is proposed in the follow-up GitHub issue.

## Resolved Questions

- The fragment covers the complete current RFC-10 semantic inventory rather than only the production anchor subset.
- This demonstration proves Rust-produced witness acceptance against Lean's checker; moving the producer into Lean and proving Rust refinement is deferred.
- The logical producer is unbounded, while bounded execution is related by sufficient-budget obligations.
- Evidence replay completeness excludes Verus, mutation scoring, and final L3 certification completeness.

## Residual trust

- Rust constructs the canonical projection and production witness in separate traversals; production replay proves exact logical-producer refinement modulo non-semantic footprint effect ordering. Independence is field-specific: direct footprints and free calls are dual-derived, while holding/shared-place payloads use duplicated same-algorithm walks. The remaining trust includes the canonical projection's interpretation of the parsed AST and common-mode errors in those duplicated payload walks.
- Canonical shared-place and holding payloads are exact at witness version 3; future language semantics still require versioned fragment-extension proofs.
- The Rust compiler, serialization renderer, Lean parser/elaborator, and kernel remain trusted transport and checking infrastructure. The `THERMITE_LEAN_LAKE` override and ordinary `PATH` resolution select a fully trusted replay executable; control of either can forge the output protocol.
- Sufficient-budget tests characterize the current implementation arithmetic but are not yet a mechanically verified refinement theorem about Rust execution.

## Open Questions

- None.

## Out of Scope

- A Lean implementation of parsing and canonical semantic-inventory construction.
- A source-level theorem relating the Rust parser directly to the Lean canonical AST, beyond the per-artifact kernel-checked producer refinement.
- Completeness of Verus, external solvers, mutation-equivalence policy, or final certification levels.
