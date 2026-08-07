---
rfc: 9
title: Verified effect rows — checking what a row claims
status: draft
supersedes: []
introduces:
  - REQ-SYNTAX-SHARED-DECL
  - REQ-SPEC-EFFECT-ROW-CHECKED
  - REQ-SPEC-EFFECT-CONFLICT
---

# RFC-9: Verified effect rows — checking what a row claims

| | |
|---|---|
| **Status** | Draft, **staged and not filed**. Waiting on the direction check in [RFC-7](0007-thermite-3.md) |
| **Baseline** | `dollspace-gay/Thermite @ 84d276e7` |
| **Position** | step 5 of the sequence in [RFC-7](0007-thermite-3.md#14-the-sequence) |
| **Depends on** | [RFC-6](0006-full-words.md), RFC-8 |

> **Not proposed yet.** This document is written so the work is not blocked on a
> reply, and it stays unfiled until [RFC-6](0006-full-words.md) lands and the
> direction in [RFC-7](0007-thermite-3.md) is answered. Filing six capability
> proposals against a surface nobody has adopted is the failure RFC-7's own
> sequencing rule exists to prevent.

Kind: extension to an existing mandatory clause.

## The framing

The proposal is not "add concurrency to Thermite". It is:

> **Make effect rows verified rather than asserted.**

Race-freedom falls out as a consequence, and with it multi-core.

## The problem it starts from

Today `write(db)` is an unchecked claim. Nothing declares `db`; nothing verifies
the function touches only it. The effect row is documentation that happens to be
syntax.

Reproduced at the pin — a body that touches nothing, claiming two resources that
are declared nowhere:

```thermite
fn f(n: u64) -> u64
  req n < 10
  ens result == n
  fx  write(a_resource_that_does_not_exist), read(nor_this_one)
{ n }
```
```
item: f
level: L3
effects: [write(a_resource_that_does_not_exist), read(nor_this_one)]
```

Unchecked in both directions: an undeclared name passes, and a declared effect
the body never performs passes. Effect-row names are not reserved words either
(`lexer.rs`: "Effect-row names and slag field names are not reserved"), so there
is no spelling that would fail.

That is a weakness independent of concurrency. A function may quietly touch
authority it never declared, and the row will not notice.

## Proposal

Declare shared state, so the row can be checked against it:

```thermite
shared scheduler: SchedState
shared shootdown: Shoot
shared clock:     TimeState
```

`shared` rather than `region`, per
[the surface conventions](0007-thermite-3.md): "region" is effect-systems
jargon that names the mechanism, and `shared` names the safety-relevant property.
For a kernel this fits naturally, because shared state is static.

Effect atoms keep their existing `verb(resource)` shape, generalised to a new
resource kind:

```thermite
fn advance() -> u64
  ! write(scheduler), read(clock)
  requires  nothing
  ensures   result < MAX_SLOTS
```

The checker now verifies the row is honest rather than trusting it.

[The surface conventions](0007-thermite-3.md#5-the-effect-row)
carry the rest of the row's structure — the two label families, the composition
law each carries, and the rule that a `shared` thing is named in the row while a
`resource` thing is not. This document is about making the state family
checkable.

## The concurrency consequence

Given honest rows, concurrent composition gets a conflict rule:

| | |
|---|---|
| `write(r)` ∥ `write(r)` | reject |
| `write(r)` ∥ `read(r)` | reject |
| `read(r)` ∥ `read(r)` | accept |

That is ordinary reader-writer exclusion, and it is the condition for
data-race-freedom. By the DRF-SC theorem (Adve & Hill, 1990), a data-race-free
program cannot distinguish its execution from a sequentially consistent one.

The table is **derived rather than stipulated**. Each row is the commutation
condition of the state theory over a region — `get` commutes with `get`, `put`
with neither — so the rule is a theorem about the tensor of independent region
theories. [The effect algebra](0008-effect-algebra.md#a-row-entry-is-a-theory-instance-plus-the-operations-used)
carries the derivation, including the case it predicts that this document would
not have guessed: `random` ∥ `random` accepts, because independent samples
commute.

**So every sequential proof already written stays valid, unchanged, on multiple
CPUs.** That is the whole return on this RFC.

Exclusivity is a property of the *composition rule*, not of the atom.
`write(log)` means the same thing it means today; the rule only fires where two
functions are composed concurrently. Applied uniformly it also catches two
threads writing one file, which is correct and currently unnoticed.

## Where the check happens

Concurrency in a kernel is not spawned dynamically — CPUs run handlers — so the
composition site can be declarative:

```thermite
interleaves shootdown { ack, complete }
```

## Migration

**Making the row checkable is a breaking change for every existing `.th` that
names a resource, and there is currently nowhere to declare one.** That covers
the whole corpus, so the size is worth measuring rather than asserting.

Across the 67 `.th` files in the pinned tree, 149 effect atoms:

| | count | what migration asks of it |
|---|---|---|
| `pure` | 93 | nothing |
| `diverge` | 6 | nothing — a control effect, not state |
| `alloc` | 22 | becomes `write(heap)` |
| `time` | 4 | becomes `read(clock)` |
| `term` | 3 | becomes a read/write on a terminal region |
| named-resource atoms | 24 | the name must resolve to a declaration |

The 24 named atoms carry 7 distinct names: `clock`, `db`, `input`, `log`,
`memory`, `output`, `stdin`. Every one is undeclared today, because declaring one
is what this RFC adds.

**A standard prelude absorbs most of it.** The ambient names — `stdin`, `log`,
`clock`, `heap`, `entropy`, a terminal — are the same in every program and can be
declared once in a prelude the compiler injects. What remains for a program to
declare is the names it invented: `db`, `input` and `output` in the corpus. The
`platform(...)` domains are a separate namespace and should be settled alongside
rather than folded in.

So the break is wide and shallow: 50 of 149 atoms change, and a prelude plus one
declaration line per program covers all of them. Staging it as a warning first
would be worse than useless, because the point of the change is that an
undeclared name is an error.

**Unlike a rename, this one cannot be automated.** Deciding which region an
`alloc` belongs to is a judgement about what the program shares, not a
transformation of its text. Contrast
[the surface conventions](0006-full-words.md#migration), whose much larger
break — all 547 clause sites — is a deterministic rewrite precisely because
nothing about it depends on what the program means.

**And unlike a rename, this one invalidates certificates.** `effects` is in the
`.cert.json` oracle subset, alongside `item`, `level`, `tautology`,
`vacuous_precondition` and `slag`, so changing an atom changes the oracle it is
compared against. Of the 12 oracle items in the corpus carrying an effects field,
one is affected — it carries `alloc` — and the other 11 are `pure`. Small, and it
has to be part of the change rather than discovered by a red CI run.

## The second break: the kernel target refuses `write`

`forge build --target kernel` refuses the central example of this RFC.
Reproduced at the pin:

```thermite
fn tick(n: u64) -> u64
  req n < 100
  ens result == n + 1
  fx  write(shootdown)
{ n + 1 }
```
```
forge: usage error: `forge build --target kernel` refuses `tick`: its transitive
effect row carries the ambient-syscall effect `write(shootdown)` (a `write`
userspace syscall), which kernel code has no ambient surface for. The admitted
kernel effects are pure/alloc/panic/diverge
```

The same file with `alloc` in place of the write builds and reports `fx=[alloc]`.

`KERNEL_REJECTED_FX` is `["read", "write", "net", "term", "time", "rand"]`,
"matched by the leading verb of each `effects_of` token (`read(stdin)` → `read`)"
(`forge/src/build.rs`). A kernel item may not carry a `write` at all, whatever it
names. And under the row systematization, `alloc` becoming `write(heap)` would
turn the one allocation effect the kernel target admits into a rejected one.

Both facts have one cause, and it is the cause this RFC removes: **the row cannot
distinguish a syscall-backed ambient effect from a state effect on declared
state, so the profile has to reject by leading verb.** There is nothing else to
reject by.

Declared shared state gives the profile a better predicate. A kernel build should
refuse effects on ambient, syscall-backed regions and admit effects on regions
the kernel itself declares — rejecting by region kind rather than by verb. That
is a sharper check than the current one, and it is a required companion change
rather than a follow-up: without it, this step produces kernels that cannot be
built.

## What this does not do

**It does not prevent deadlock.** Deadlock is about lock *ordering*, not
aliasing. Two functions each acquiring two regions in opposite orders both pass.
That needs a region partial order, and it is a real gap in this proposal.

**It does not help with lock-free sharing.** That is
[interference clauses](0012-interference-clauses.md).

**Interrupts are concurrency too**, and this is the kernel-specific case with no
userspace analogue. A handler preempts normal context on the *same* CPU, so
`write(scheduler)` in a handler races `write(scheduler)` in normal context even
single-threaded. The row needs to distinguish:

```thermite
fn timer_isr() -> ()
  ! irq, write(scheduler)
  requires  nothing
  ensures   nothing
```

with `irq` functions conflicting with non-`irq` functions over the same region
unless the latter masks interrupts — itself an effect, `masked`.

## Granularity: the containment order

One region per subsystem serialises things that need not be. Sub-regions fix it,
and the conflict rule then needs a **containment order**, because
`write(scheduler)` must conflict with `read(scheduler.runqueue)`.

**Containment follows the type, so it needs no new declaration.** If
`SchedState` has a field `runqueue`, then `scheduler.runqueue` is a sub-region of
`scheduler`. The region tree is the field tree, and conflict is ancestry:

| | |
|---|---|
| `write(scheduler)` ∥ `read(scheduler.runqueue)` | reject — one contains the other |
| `write(scheduler.runqueue)` ∥ `write(scheduler.timers)` | **accept** — siblings are disjoint |
| `write(scheduler.runqueue)` ∥ `read(scheduler.runqueue)` | reject — same region |

So the rule is: two atoms conflict when their regions are equal or one is an
ancestor of the other, and the existing read/write table applies unchanged at
each pair.

The heap makes this concrete. Once `alloc` is `write(heap)`, every allocating
function conflicts with every other, so allocation serialises — correct for a
single global allocator, and currently invisible. Per-CPU heaps as sibling
sub-regions is the fix, and it is the containment order doing the work.

**Blocked on [G4](https://github.com/bulla-systems/bulla/blob/main/docs/language-gaps.md#g4-struct-fields-of-user-declared-types)**,
since a declared type cannot be a field of another declared type today, so a
region cannot have structured sub-regions to name.

**This is not the order the shared-state document wants.** That one needs an
*acquisition* order — which lock may be taken while holding another — and the two
are different relations on the same set. Containment is a tree derived from
types and used by the conflict rule; acquisition is a declared rank used by the
deadlock check. Both documents previously deferred to each other about "the
region partial order" as though it were one thing.

## Metatheory

None new. DRF-SC is from 1990. The conflict check is syntactic: no solver, no
proof obligations. This is the only proposal in the set with that property, which
is why it is the best value per unit of work.
