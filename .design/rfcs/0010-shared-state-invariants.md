---
rfc: 10
title: Shared-state invariants — locks, ordering, and masking
status: draft
supersedes: []
introduces:
  - REQ-SYNTAX-LOCK-DECL
  - REQ-SYNTAX-HOLDING-BLOCK
  - REQ-SPEC-LOCK-DISCIPLINE
---

# RFC-10: Shared-state invariants — locks, ordering, and masking

| | |
|---|---|
| **Status** | Draft, **staged and not filed**. Waiting on the direction check in [RFC-7](0007-thermite-3.md) |
| **Baseline** | `dollspace-gay/Thermite @ 84d276e7` |
| **Position** | step 6 of the sequence in [RFC-7](0007-thermite-3.md#14-the-sequence) |
| **Depends on** | [RFC-6](0006-full-words.md), RFC-9 |

> **Not proposed yet.** This document is written so the work is not blocked on a
> reply, and it stays unfiled until [RFC-6](0006-full-words.md) lands and the
> direction in [RFC-7](0007-thermite-3.md) is answered. Filing six capability
> proposals against a surface nobody has adopted is the failure RFC-7's own
> sequencing rule exists to prevent.

Kind: extension reusing the struct invariant.

## Summary

[Verified effect rows](0009-verified-effect-rows.md) permits only disjoint concurrent
access. This permits *shared* access, serialised, with the invariant holding at
every boundary.

The observation that makes it small: **a resource invariant is a struct
invariant.** `keeps` already exists and already means "always true of this type".
The only new thing is a scope in which it may be temporarily false.

## Proposal

```thermite
struct Counter { n: u64 }
  keeps n <= MAX

lock counters guards Counter;

fn bump(c: &mut Counter) -> ()
  ! owns(counters)
  requires  c.n < MAX
  ensures   final(c).n == c.n + 1
{ holding counters { c.n = c.n + 1; } }
```

`lock NAME guards TYPE;` is a new item form. It keeps the locking concern out of
the type declaration — the struct stays a plain struct with a plain invariant.

`owns(r)` joins `read(r)` and `write(r)` as a third mode: exclusive, established
dynamically.

## The scope of a lock

The rule that the invariant may be broken inside the critical section is only
meaningful once the critical section has edges. Two constructs give them, and
they carry different facts.

> **The row declares which locks a function takes. A block declares where.**

```thermite
fn bump(c: &mut Counter) -> ()
  ! owns(counters)
  requires  c.n < MAX
  ensures   final(c).n == c.n + 1
{
  … work outside the section …
  holding counters { c.n = c.n + 1; }
  … more work outside …
}
```

`owns(r)` in the row is what a caller sees and what the conflict rule composes.
`holding r { … }` is where the lock is actually held. Neither is optional, and
neither is inferred from the other.

### The three rules

1. **`owns(r)` requires a `holding r` block**, always. There is no whole-body
   default. Scoping the whole body is written `holding r { … }` around it.
2. **A `holding r` block requires `owns(r)` in the row.** Otherwise it is a parse
   error, by the same mechanism that puts `asks` inside `interleaves`: the row
   must be honest, and a caller cannot see a body.
3. **The row is an upper bound.** A `holding` block under an `if` is fine — the
   row says the lock may be taken, not that it is.

Rules 1 and 2 are one rule in two directions: the row and the body must agree
about every lock.

**Why no default.** An implicit whole-body scope makes an over-declared lock
invisible — `owns(r)` with no block is indistinguishable from "I meant the whole
body", so a lock that is declared and never taken reads as correct. That is a
declared-but-not-performed effect, which is what
[verified effect rows](0009-verified-effect-rows.md) exists to stop, appearing one
level down:

```
error: `bump` declares `owns(sched_lock)` and no `holding` block takes it
```

It also makes the one-lock rule below checkable at all: without lexical blocks
there is nothing to count, and in the case where nesting is permitted the order
is structural rather than unstated.

### The obligation

For a `lock r guards T`, the guarded type's invariant is **assumed on entry to a
`holding r` block and must be proved on exit**, and is not assumed between. That
is the standard concurrent-separation-logic reading: you need to break the
invariant to do the update.

It needs no new clause syntax. The obligation derives from the
`lock … guards …` declaration rather than from anything written at the block, and
loops already show the language annotating a block, so this is not a new
syntactic category either. Thermite has no `assert` and no proof blocks
(verified: zero occurrences), and this needs neither.

### Why a block rather than a function

An earlier draft made the function body the critical section, on the grounds that
it reduces to clauses that already exist. Three things are wrong with that.

**It hides an over-declared lock.** Covered above, and it is the decisive one.

**Factoring is expensive here.** Narrowing a section by extracting a function
costs a full contract — `requires`, `ensures` and the row are all mandatory — so
scoping-only functions carry real signature noise. What is cheap in Rust is not
cheap in Thermite.

**Function scope cannot express acquisition order.** `! owns(a), owns(b)` does
not say which is taken first, so a deadlock check has nothing to work with, and
counting held locks needs a scope to count over. Nested blocks give both.

Because a block is lexical, the extent stays a syntactic property: reentrancy is
containment — no call lexically inside a `holding r` block may reach a function
whose row carries `owns(r)` — rather than a dataflow question. That would only
become flow-sensitive with separate acquire and release statements, which this
does not propose.

## The conflict rule gains three rows

The effect-rows RFC's table covers `read` and `write`. `owns` needs its own, and
the omission matters, because the interesting rows are the ones that reject:

| | |
|---|---|
| `owns(r)` ∥ `owns(r)` | **accept** — the lock serialises them, which is the point |
| `owns(r)` ∥ `write(r)` | **reject** — the writer bypasses the lock |
| `owns(r)` ∥ `read(r)` | **reject** — the reader observes a broken invariant |

So a guarded region may be touched only under its lock. Without those two
rejections, `lock r guards T` would be advisory: nothing would stop a second
function from writing `r` directly while the first holds it, and the invariant
that the lock exists to maintain would be maintained by convention.

This is also what makes `owns` different from "exclusion is achieved at runtime
so it conflicts with nothing statically". Runtime exclusion holds *between
holders of the lock*. It says nothing about a function that never takes it, and
that function is the one the static rule has to stop.

## Metatheory

Concurrent separation logic (O'Hearn and Brookes, 2004). Mechanised repeatedly,
and the pinned Verus ships `vstd/invariant.rs` and `vstd/rwlock.rs`, so the
substrate is present.

## Ordering: one lock by default

`owns(a), owns(b)` in one function and `owns(b), owns(a)` in another deadlocks,
and today both typecheck.

**The default rule is that a function holds at most one lock at a time.** No
order is declared, because there is no nesting to order, and the entire
deadlock-by-ordering class is gone by construction. The check is counting active
`holding` blocks:

```
error: `migrate_task` holds `sched_lock` and takes `frame_lock` — a function may
       hold at most one lock; see `after` if the nesting is required
```

A verified core is the right place to be restrictive, and most kernel critical
sections take one lock. We also have no verified subsystem yet, so we have zero
examples of a genuine two-lock critical section — designing an ordering system
for a case nobody has produced would be specifying ahead of the evidence.

**When nesting is genuinely needed, the order is relative:**

```thermite
lock sched_lock guards SchedState;
lock frame_lock guards FrameTable after sched_lock;
```

`after` names the relationship rather than a coordinate. It yields a DAG, which
is the partial order actually wanted; cycles are detected at declaration rather
than at a use site; and adding a lock is one line saying what it comes after,
with nothing renumbered.

An earlier draft proposed a numeric rank, `at 1` / `at 2`. It is rejected:
the number carries no reason, inserting a lock between two existing ones forces
renumbering, and independent additions collide. Inferring the order from every
nesting in the program was also considered and rejected — the order would become
whatever was written first, a library could not state what its callers must do,
and a violation would surface as a cycle spanning modules rather than as a local
mistake.

The restriction ships first and `after` is what lifts it, which keeps the
mechanism tied to a case that exists.

## A lock a handler takes needs masking, not ordering

This is the defect no ordering rule catches, because both functions involved take
only one lock. If a handler carries `owns(r)`, then normal-context code holding
`r` deadlocks against it on its own CPU — the handler fires, waits for the lock,
and the code that would release it has been preempted and cannot resume until the
handler returns. One core, no race, permanent hang.

> if any function in `handlers { }` carries `owns(r)`, every normal-context
> function carrying `owns(r)` must also carry `owns(interrupts)`

Checkable from the rows and the handler declaration, and among the most common
concurrency defects in real kernels.

## This is not the effect-rows document's order

That one needs a **containment** order — a tree derived from the type structure,
so `write(scheduler)` conflicts with `read(scheduler.runqueue)` — used by the
conflict rule. This document needs an **acquisition** order, used by the deadlock
check, and under the rule above it usually needs none at all. Both documents
previously deferred to each other as though there were a single order to specify.

## Open question

- **Reentrancy.** Does holding `owns(r)` permit calling something that also wants
  `owns(r)`? The answer is no, and because a `holding` block is lexical this is
  checkable by containment rather than by dataflow: no call inside a `holding r`
  block may reach a function whose row carries `owns(r)`.
