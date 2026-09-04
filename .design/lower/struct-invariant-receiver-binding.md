# Struct-invariant receiver binding

<!--
tier: 3-component
status: shipped
audited-content-sha256: 13b0b66a08f758c88f725d9f0cf2d5b0ba56175681efa9334e007c647648cef3 (re-pinned 2026-08-25 for issue #6 final shared Map lowering changes. prior: 3329cdd885f625f684da2d61487b3598e25e68b6272b806729acf8057351301c)
decision: qualify invariant field paths in the canonical invariant lowerer
issue: github:dollspace-gay/Thermite#110
governs:
  - thermite-lower/src/lower.rs
  - thermite-lower/src/l1.rs
  - thermite-lower/tests/adt_lower_conformance.rs
  - forge/tests/struct_invariant_receiver.rs
  - conformance/nested_adt.th
thesis-refs:
  - thermite-design.md §3
  - thermite-design.md §6
  - thermite-design.md §7
-->

## Summary

Every bare path naming a declared struct field in a type invariant lowers as a
field of the invariant predicate receiver. This rule applies through unary
expressions and variant tests as well as binary expressions, calls, casts,
field access, and method calls. For example,
`keeps !panic_latched || !reschedule_pending` becomes
`!self.panic_latched || !self.reschedule_pending` inside
`well_formed(&self)`, while `keeps privilege is User` becomes
`self.privilege is User` at L3 and a receiver-bound, enum-qualified `matches!`
at L1.

The solver-vacuity harness continues to consume the canonical lowered Verus
declaration. It does not maintain a second field-qualification pass. This keeps
the normal L3 artifact, `forge check`, and `forge battery` on one invariant
representation.

## Root cause

`lower_inv_expr` already rewrote a direct field path to `self.<field>` and
recursed through several compound expression nodes. It originally had no
`Expr::Unary` arm, and issue #5 showed that it also had no `Expr::Is` arm. Either
expression therefore reached a generic lowerer with no struct receiver context.
The unary defect emitted the first bare name below; the variant-test defect
emitted the second:

```text
Expr::Unary(Not, Path("panic_latched"))
    -> generic spec lowering
    -> !panic_latched

Expr::Is(Path("privilege"), "User")
    -> generic spec lowering
    -> privilege is User
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

Its `Expr::Is` arm recursively binds the scrutinee and retains Verus's
type-directed bare variant. The L1 mirror threads the program variant map into
`lower_struct_l1`, recursively binds the scrutinee, and emits the qualified Rust
pattern. The two assurance rungs therefore agree on the receiver and enum while
using their native discriminant syntax.

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
- `conformance/nested_adt.th` combines issue #5's struct-typed and enum-typed
  fields with `keeps privilege is User`; lowerer conformance verifies the L3
  artifact and compiles/runs its L1 mirror, while the standalone Forge regression
  requires every declaration to certify at L3.

## Requirements

<!-- generated:reqs view=struct-invariant-receiver-binding-status -->
Source: `.design/reqs/registry.toml`

| ID | Status | Owner | Title | Follow-up |
|---|---|---|---|---|
| REQ-INVBIND-1 | shipped | `.design/lower/struct-invariant-receiver-binding.md` | Canonical receiver binding under unary operators |  |
| REQ-INVBIND-2 | shipped | `.design/lower/struct-invariant-receiver-binding.md` | Solver-vacuity harness reuses bound invariants |  |
| REQ-INVBIND-3 | shipped | `.design/lower/struct-invariant-receiver-binding.md` | Check and battery regression coverage |  |
| REQ-INVBIND-4 | shipped | `.design/lower/struct-invariant-receiver-binding.md` | Receiver binding through variant tests |  |
<!-- /generated:reqs -->
