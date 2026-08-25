---
rfc: 14
title: The crash clause — what holds if execution stops mid-step
status: draft
supersedes: []
introduces:
  - REQ-SYNTAX-SURVIVES-CLAUSE
  - REQ-SPEC-CRASH-MODEL
---

# RFC-14: The crash clause — what holds if execution stops mid-step

| | |
|---|---|
| **Status** | Draft, **staged and not filed**. Waiting on the direction check in [RFC-7](0007-thermite-3.md) |
| **Fork implementation** | **Not started; independent and unscheduled.** It is not on the RFC-10 → RFC-11 → RFC-12 → RFC-13 critical path. |
| **Baseline** | `dollspace-gay/Thermite @ 84d276e7` |
| **Position** | independent, unscheduled of the sequence in [RFC-7](0007-thermite-3.md#14-the-sequence) |
| **Depends on** | [RFC-6](0006-full-words.md) |

> **Not proposed yet.** This document is written so the work is not blocked on a
> reply, and it stays unfiled until [RFC-6](0006-full-words.md) lands and the
> direction in [RFC-7](0007-thermite-3.md) is answered. Filing six capability
> proposals against a surface nobody has adopted is the failure RFC-7's own
> sequencing rule exists to prevent.

**Unscheduled.** Kind: one new clause, shaped like `ensures`.

## Why it is here

Crash consistency was initially filed as outside the project's reach. That was
wrong, and this document records the correction.

Every obligation in Thermite has the shape "every transition preserves I". A
crash is not a transition of the program: it is an external event that can land
*mid-step*, mapping volatile state to whatever reached durable storage —
nondeterministically, since caching and reordering decide what survived.

So the obligation gains a case:

```
normal:  I(s) ⟹ I(δ(s, e))
crash:   I(s) ∧ crash(s, d) ∧ recover(d) = s′ ⟹ I(s′)
```

Structurally that is *a relation you do not control interfering with your step* —
the same shape as [interference clauses](0012-interference-clauses.md)'s `asks`, with the
environment being physics rather than another CPU.

## Proposal

```thermite
fn commit(j: &mut Journal, b: Block) -> ()
  ! write(disk)
  requires {
    j.open;
    b.len <= MAX_BLOCK;
  }
  ensures   final(j).committed == j.committed + 1
  survives {
    final(j).recoverable;
    final(j).committed == j.committed
      || final(j).committed == j.committed + 1;
  }
```

`survives` states what holds if execution stops at *any* point inside the
function. The classic journalling obligation — the commit either happened or did
not, never half — is that disjunction.

It is the crash-time analogue of `ensures`, which is why it conjugates the same
way and sits beside it: `ensures` is the postcondition of a completed execution,
`survives` the postcondition of an interrupted one.

## Metatheory

Crash Hoare Logic (Chen et al., FSCQ, SOSP 2015), mechanised in Coq. Settled.

**The work is the crash model**, not the logic. Which writes survive, and in what
order, depends on the device and the barriers issued — so the model is a
per-device assumption, and it has to be stated as plainly as the machine model
and the toolchain are.

### A first crash model, so the assumption is concrete

The weakest useful one, and the one FSCQ started from:

> **Synchronous, sector-atomic.** A write is atomic at sector granularity and
> durable when it returns. A crash maps the state to: every returned write
> applied, and the one in flight either applied whole or not at all.

That is enough for the journalling obligation above, and it is **not true of a
real disk**. Devices buffer and reorder, and durability needs a flush. So the
model holds only for a device driven with a flush after every write, and saying
so is the point: it is a trusted input with a name, not a fact about hardware.

The asynchronous model — where a location maps to a *set* of possible values
until a barrier collapses it — is where a real device lives, and it is the next
model rather than a different mechanism. The clause does not change.

### It belongs in the boundary coordinate

Bulla's [assurance model](https://github.com/bulla-systems/bulla/blob/main/docs/assurance-model.md) reduces a claim to per-clause
tuples with a named boundary, and a `survives` clause closes at the crash model
rather than at the program. A tuple carrying `survives` needs a coordinate
naming *which* model was assumed, the way `to_platform(p)` names a platform. A
crash claim without that coordinate is the kind of overstatement this project
exists to avoid.

## Why Bulla has not scheduled it

Nothing in Bulla has durable state. Filesystems are outside the verified core at
every rung of [the ladder](https://github.com/bulla-systems/bulla/blob/main/docs/the-ladder.md), so this becomes relevant only if a
journal or a persistent capability store moves inside.

**That reason is ours and does not transfer.** Thermite's own
`examples/editor/editor.th` is a verified editor carrying
`read(input), write(output), alloc, diverge, term` — a program that writes files,
in the examples directory, today. An editor that dies mid-save guarantees nothing
about the file, and the language has no way to say what it would guarantee. The
corpus also carries `write(db)`, `write(log)` and `net(db)`.

So the upstream case is stronger than a Bulla-scheduling note implies: one clause
shaped like `ensures`, settled metatheory, and existing programs that need it.
What gates it is the crash model above, which is a thing to write rather than a
thing to discover.
