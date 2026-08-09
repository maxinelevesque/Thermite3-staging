# Struct-invariant receiver binding

<!--
tier: 3-component
status: shipped
audited-content-sha256: 738ba8b2d59683c1c90319a63289327fd427dc9cca00cfb4649a944cefe091f7 (re-pinned 2026-08-08 for RFC-17: the clause vocabulary moved into the AST and the token kinds - Contract/LemmaItem{req,ens,fx} and FnItem/SpecFnItem/PropFnItem/LoopNode.dec and StructItem.inv to the full words the surface already uses, plus TokKind::{Req,Ens,Fx,Inv,Dec}. Type-directed: cargo check --workspace --all-targets exiting 0 is the completeness proof. prior: 20bdb8fc15a2188e673e0decb2cc0b8d5c3c3b71980aa23750c8cd12d104098b, previously (re-pinned 2026-08-08 for RFC-17: the AST field names and TokKind variants moved to the full words the surface already uses - Contract{req,ens,fx} to {requires,ensures,effects}, TokKind::{Req,Ens,Fx,Inv,Dec} to {Requires,Ensures,Effects,Keeps,Measures}. A type-directed rename with no semantic content: cargo check --workspace --all-targets exiting 0 IS the completeness proof, since an unrenamed site does not compile. prior: e2910830dad12dd9dea025ee1bd16766ea03aeae27d830c52c7a27eae2209def))
decision: qualify invariant field paths in the canonical invariant lowerer
issue: github:dollspace-gay/Thermite#110
governs:
  - thermite-lower/src/lower.rs
  - thermite-lower/tests/adt_lower_conformance.rs
  - forge/tests/struct_invariant_receiver.rs
thesis-refs:
  - thermite-design.md §3
  - thermite-design.md §6
  - thermite-design.md §7
-->

## Summary

Every bare path naming a declared struct field in a type invariant lowers as a
field of the invariant predicate receiver. This rule applies through unary
expressions as well as binary expressions, calls, casts, field access, and
method calls. For example,
`keeps !panic_latched || !reschedule_pending` becomes
`!self.panic_latched || !self.reschedule_pending` inside
`well_formed(&self)`.

The solver-vacuity harness continues to consume the canonical lowered Verus
declaration. It does not maintain a second field-qualification pass. This keeps
the normal L3 artifact, `forge check`, and `forge battery` on one invariant
representation.

## Root cause

`lower_inv_expr` already rewrote a direct field path to `self.<field>` and
recursed through several compound expression nodes. It had no `Expr::Unary`
arm. A unary expression therefore reached the generic spec-expression lowerer,
which has no struct receiver context. Both paths below emitted bare names:

```text
Expr::Unary(Not, Path("panic_latched"))
    -> generic spec lowering
    -> !panic_latched
```

The solver-vacuity harness places the full lowered struct declaration in its
preamble. Verus elaborates that preamble before checking the synthetic
obligation, so the unresolved names stopped both vacuity queries before a
verdict. The same malformed predicate was present in the ordinary L3 lowering;
the vacuity gate exposed it first.

## Decision

`lower_inv_expr` owns struct-field receiver binding. Its unary arm:

1. recursively lowers the operand with the same field-name and string-field
   context;
2. emits Verus's type-directed `!` operator;
3. preserves grouping for a binary operand.

This is the smallest shared correction. Qualifying text while building a
vacuity harness would create a harness-only interpretation and leave the normal
artifact malformed. Rewriting source identifiers by string replacement would
also lose AST boundaries and could alter unrelated names.

## Verification

The hand-authored issue reproduction lives at
`conformance/struct-invariant-receiver/repro.th`. Its postcondition leaves
`panic_latched` unconstrained, so the current mutation floor correctly returns a
structured `WeakContract` verdict after the vacuity harnesses elaborate. The
companion `accept.th` adds the missing preservation postcondition while retaining
the same unary struct invariant.

- Lowerer conformance checks that the emitted `well_formed` predicate contains
  both `self`-qualified fields and passes the real Verus binary.
- Forge conformance runs `forge check <repro> --level l3` and requires the
  post-vacuity `WeakContract` certificate, with no unresolved-name diagnostic.
- Forge conformance runs `forge check <accept> --level l3` and requires a clean
  L3 certificate for `clear`.
- Forge conformance runs `forge battery <accept> clear` and requires a successful
  battery rendering after all solver-vacuity queries elaborate.
- Existing ADT lowering, solver-vacuity, formatting, lint, requirement, and
  documentation gates remain green.

## Requirements

<!-- generated:reqs view=struct-invariant-receiver-binding-status -->
Source: `.design/reqs/registry.toml`

| ID | Status | Owner | Title | Follow-up |
|---|---|---|---|---|
| REQ-INVBIND-1 | shipped | `.design/lower/struct-invariant-receiver-binding.md` | Canonical receiver binding under unary operators |  |
| REQ-INVBIND-2 | shipped | `.design/lower/struct-invariant-receiver-binding.md` | Solver-vacuity harness reuses bound invariants |  |
| REQ-INVBIND-3 | shipped | `.design/lower/struct-invariant-receiver-binding.md` | Check and battery regression coverage |  |
<!-- /generated:reqs -->
