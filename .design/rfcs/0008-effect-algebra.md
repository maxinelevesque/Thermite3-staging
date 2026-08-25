---
rfc: 8
title: The effect algebra — what an effect row is
status: draft
supersedes: []
introduces:
  - REQ-SYNTAX-EFFECT-BASIS
  - REQ-SYNTAX-EFFECT-DECLARED
  - REQ-SPEC-EFFECT-COMMUTATION
---

# RFC-8: The effect algebra — what an effect row is

| | |
|---|---|
| **Status** | Draft, **staged and not filed**. Waiting on the direction check in [RFC-7](0007-thermite-3.md) |
| **Fork implementation** | **Shipped on `staging`.** Proposal status remains draft because upstream filing is a separate, deferred reconciliation. |
| **Baseline** | `dollspace-gay/Thermite @ 84d276e7` |
| **Position** | step 4 of the sequence in [RFC-7](0007-thermite-3.md#14-the-sequence) |
| **Depends on** | [RFC-6](0006-full-words.md) |

> **Not proposed yet.** This document is written so the work is not blocked on a
> reply, and it stays unfiled until [RFC-6](0006-full-words.md) lands and the
> direction in [RFC-7](0007-thermite-3.md) is answered. Filing six capability
> proposals against a surface nobody has adopted is the failure RFC-7's own
> sequencing rule exists to prevent.

The status above is proposal status, not implementation status. The staging
fork implements the fixed basis, declared effects, computed commutation, and
their checked registry evidence. Keeping `status: draft` records that this RFC
has not been filed or accepted upstream; it does not mean the fork is waiting
to build RFC-8.

**Cross-cutting**, like [the surface conventions](0007-thermite-3.md), and
underneath [verified effect rows](0009-verified-effect-rows.md): that document makes
the row checkable, this one says what a row *is*.

## Why there is a row at all

A return type and an effect row sit on different axes, and the difference is what
makes the row worth having in a language that proves things.

> **A return type says what the call produces. An effect row says what the call
> leaves alone.**

The row is a *negative* statement, and a return type cannot carry one — it
describes the value produced and says nothing about anything else. So if `P(s)`
held before a call, a return type is silent about whether it still holds after.
`! pure` answers for everything. `! write(r)` answers for everything except `r`.

That is the frame rule, and framing is the single thing that makes modular
verification scale. Without it every call invalidates everything known, and
proofs stop composing.

There is a second difference, in *when* the obligation is discharged. A return
type is discharged at the call site by construction — you get a `Result` and deal
with it there, and your own caller sees nothing. An effect is discharged nowhere
unless something discharges it, so it accumulates and reappears in your own
signature. A local obligation against a non-local one.

Which gives the rule for deciding where a thing belongs:

> Put it in the **type** when the caller must decide at the call site.
> Put it in the **row** when the caller inherits it.

**The language already prices the interaction.** `-> A ! diverge` certifies at L1
where `-> A ! pure` reaches L3, because the row changed what the return type
promises: not an `A`, but an `A` if one arrives. The certificate records the
weakening rather than letting it pass.

## What an effect is

An effect label denotes an **algebraic theory** — a signature of operations plus
the equations they satisfy — and a row is a set of such theories. That is the
Plotkin–Power reading, and taking it seriously is what makes the language's rules
derivable instead of stipulated.

Each label owes three things:

| | what it decides |
|---|---|
| **theory** | which equations the prover may use when reasoning through the effect |
| **composition** | what a caller inherits — union, or, sum |
| **commutation** | whether two of them may run concurrently |

The third is the one that pays off, because it means
[the conflict table](0009-verified-effect-rows.md#the-concurrency-consequence) is a
theorem rather than an axiom. See below.

## The admissibility criterion

Effects and contracts meet at the **frame condition** — "this call may change `r`
and nothing else" — which is first-order and SMT-friendly. So:

> **A primitive effect is admissible when it generates a frame condition
> expressible in the prover's logic.**

That is a real filter rather than a restatement. It admits and excludes things we
would otherwise argue about by taste, and it did both.

## The basis

Five theories that generate a frame condition:

| primitive | frame condition | covers |
|---|---|---|
| `state(r)` | may modify `r`, nothing else | `read(r)`, `write(r)`, `owns(r)`, `forgets(r)`; `alloc` → `state(heap)`; `time` → `state(clock)`; a PRNG → `state(entropy)`; `platform(d)`; terminal writes |
| `accrues(M)` | adds to a ghost accumulator over a monoid | `cost(E)` over (ℕ, +, 0) |
| `exception` | may not return normally | `panic` |
| `partiality` | may not return at all | `diverge` |
| `io(σ)` | a free signature, no equations | terminal *reads*, where the value does not exist until it does |

The basis owes a home to every label the surface carries, and the two the table
above leaves out are an instance and a combination rather than new primitives.

`term` is `state(termios) / {get, put}`. The #106 terminal-control atom is the
termios control register, read by `tcgetattr` and written by `tcsetattr`, which
are the state theory's two operations over one region. Reading a keypress is a
different label, `read(input)`.

`net(d)` is `state(d) + io(σ_d)`, a combination in the declaration form below.
Its transfer splits along the theory boundary: `setsockopt` and `getsockopt` are
`get` and `put` on socket state, `sendto` is a `put` toward the far end, and
`recvfrom` yields a value that no region this program frames over determines.
The sum is a conservative combination rather than a model of a socket. It
asserts no equations between its summands, so it does not express that a
`setsockopt` timeout changes what a later `recvfrom` does; tensor is ruled out
for the same instance because that interaction exists, and the sum's silence
licenses no commutation the interaction would refute. Expressing the
interaction needs directed equations or a distributive law, which this document
does not propose.
Holding `net` out of the basis keeps the basis at five theories, which is the
direction [the surface conventions](0007-thermite-3.md#5-the-effect-row) set when
they made `alloc`, `time` and `rand` into declared state: the label set shrinks.
The `io` summand is also what keeps the reproducibility reading below honest,
since a row carrying one is not reproducible, which is the right answer for a
receive.

And two given atoms, which generate no frame condition. Generating no frame
condition and carrying no equations are separate properties, and the two atoms
differ on the second:

| atom | frame condition | equations |
|---|---|---|
| `random` | none — no first-order denotation for a distribution, see below | undetermined |
| `blocks` | none — a liveness claim, and this project proves no liveness | none stated |

The frame-condition column is what separates `random` from `io(σ)`, and it is
enough on its own: `io(σ)` generates one, and `random` generates none. So they
are distinct rather than two spellings of one free signature.

The equations column is open for `random`, and the commutation table below
inherits its `random ∥ random` accept rather than computing it. Fubini is a
theorem about product measures and needs a distribution, which the
unparameterized atom does not carry, so attributing the row to Fubini claims
more than the atom supplies. Read as nondeterministic choice the atom commutes,
because the finite powerset monad is commutative; read as a free operation it
does not commute; read as sampling it commutes by Fubini once a distribution
exists. `random(D)` settles this by putting the distribution in the denotation,
which is another reason the parameter position is reserved.

`blocks` is recorded and unproved because its discharge route is unbuilt, rather
than because progress cannot be stated. `diverge` shows the shape. Termination is
a liveness property, and the language reaches it by reducing it to well-founded
descent through `measures`, a finitary witness inside the function. Progress does
not reduce that way, because the witness is a peer's behaviour. Binary session
duality carries deadlock freedom
([protocol types](0013-protocol-types.md)), so the route is settled in the
literature and arrives with that document.

`accrues` rather than `writer`: the category-theory name describes the mechanism,
and what the primitive does is monotone accumulation, which is what "accrues"
means. It also cannot be `write(M)`, which would collide with `write(r)` — two
different theories under one word, adjacent in the same row.

## A row entry is a theory instance plus the operations used

This is the structural bit that makes the conflict table fall out.

```
read(r)   =  state(r) / {get}
write(r)  =  state(r) / {get, put}
```

Commutation is then computed per operation pair from the state equations:

| | | because |
|---|---|---|
| `read(r)` ∥ `read(r)` | accept | `get` commutes with `get` |
| `read(r)` ∥ `write(r)` | reject | `get` does not commute with `put` |
| `write(r)` ∥ `write(r)` | reject | `put` does not commute with `put` |
| `write(a)` ∥ `write(b)` | accept | independent instances are independent theories |

So the data-race-freedom rule is the commutation condition for the tensor of the
region theories. The effect-rows document says "Metatheory: none new", which is
true of DRF-SC and understates where its own table comes from.

It also predicts something worth having:

```
random ∥ random    accept     independent samples commute (Fubini)
```

Concurrent randomness is safe in a way concurrent writing is not, and that falls
out rather than needing an argument.

`owns(r)` is the same theory with commutation established **dynamically**: the
lock serialises the puts, so `owns(r)` ∥ `owns(r)` accepts while `owns(r)` ∥
`write(r)` rejects. That is
[the three-row table](0010-shared-state-invariants.md#the-conflict-rule-gains-three-rows),
now with a reason.

## User-declared effects

The tractable ambition is that effects are **written in the language**, as
combinations of the basis, with the language computing the rest:

```thermite
effect platform(d) = state(d)
effect journal(d)  = state(d) + exception
effect net(d)      = state(d) + io(σ_d)
```

`+` is the **sum** of theories in the Hyland–Plotkin–Power sense: a free
combination, with no equations relating the summands, so nothing in one commutes
with anything in the other. It is a different operation from the composition law
each family carries in
[the surface conventions](0007-thermite-3.md#5-the-effect-row) table, which says
how a caller inherits a callee's effects. Distinct region instances combine by
**tensor** instead — every operation of one commutes with every operation of the
other — and that is the combination the conflict rule reads, which is why
`write(a) ∥ write(b)` accepts. Naming the two is worth the sentence, because
"sum" appears in both senses across these documents.

The prover only ever sees primitives it knows how to encode; users get to name
and structure their own vocabulary; and the conflict rule, composition law and
frame conditions are generated rather than hardcoded — including for effects the
language never anticipated. The label set is fixed today, so an effect the
language does not already name gets no conflict checking and cannot be given
any without a change to the language itself. This lifts that ceiling.

The discipline is the one already used a level down: you do not get to define
type constructors with arbitrary kinding rules, you build from a fixed set.

## What the criterion removed: `exception`

An exception carrying accumulating context is attractive — each frame appends, so
a failure arrives with a trace. Under the rule above it belongs in the **type**,
not the row, and the deciding fact is checkable.

Probed at the pin: no `catch`, `try`, `throw`, `raise` or unwinding anywhere in
the surface. `panic` is abort — `.design/basis/03-effect-stdlib.md` calls it
"exit 101, never a wrong value" — and the shipped effect wrappers are explicitly
*total*, handling their error arms as return values with "no `unwrap`-panic".

**Thermite has no unwinding.** A recoverable failure is therefore decided at the
call site, which puts it in the type: `Result<T, M>` with `M` a monoid, where the
monoid is what makes the context chain. `exception` stays in the basis only as
the short-circuit primitive underneath `panic`.

The four cases stay distinguishable, and gain precision about where each lives:

```thermite
fn might_stop_badly(…)   -> T              ! panic       // abort, inherited
fn might_not_stop(…)     -> T              ! diverge     // no promise to return
fn might_fail_handled(…) -> Result<T, M>   ! pure        // decided here
fn goofy(…)              -> Result<T, M>   ! panic, diverge
```

## What the criterion kept, differently: `random`

Sampling has no frame condition, because there is no first-order denotation for a
distribution in the theories an SMT solver has. That is the formal version of
"probability is a different monad".

But **a PRNG is not probability.** `next(seed) -> (value, seed')` is
deterministic, satisfies the state equations, and generates a perfectly good
frame condition. So the pseudorandom case is `state(entropy)`, and what is *not*
state is the modelling assumption that a value is drawn from a distribution —
which is a claim a user makes, not a property an implementation has.

Forcing that stipulation makes an incompatibility visible that is otherwise
invisible: an argument assuming true randomness cannot be discharged against an
implementation declared `state(entropy)`. That gap has a name in the field —
predictable PRNGs — and no language currently notices it.

**The honesty constraint on the atom.** `random` with no equations means the
prover treats the result as an *unconstrained value*. It supports "this holds for
every value it could take", which is sound and useful. It does **not** support
"this value is uniformly distributed" or "this value is unpredictable", which
need measure theory and a hardness notion respectively. So its reading is *"I
claim nothing about this value"*, and a cryptographic argument still is not
discharged — it just gets to state its assumption where a reader can see it.

There is a payoff for this project specifically: the row would say whether a
program is **reproducible**. `state(entropy)` is; `random` is not. Bulla's whole
determinism apparatus rests on that distinction and has no way to read it off the
source today.

## `irq` is not a primitive

An interrupt is *preemption without parallelism*: the handler runs on the same
CPU, inside a gap between two instructions, to completion, and normal code cannot
preempt it. It decomposes into three things that already exist.

**The handler is the environment.** `asks` is what a function tolerates the
environment doing, and from interrupted code's point of view the handler set *is*
the environment. So the obligation is the ordinary rely-guarantee one.

**Masking is a lock.** Disabling interrupts excludes the handler set for a
window, which is what a lock does, so it is `owns(interrupts)` over the hardware
enable bit, and `holding interrupts { … }` is the masked window.

**Asymmetry is one platform assumption.** The pairwise obligation has two halves,
and only one is real: normal code must tolerate the handler's guarantee, while
the handler's own rely is that nothing interferes with it, which is true because
of how the hardware dispatches. That is the strongest possible rely and nothing
in the language justifies it, so it belongs in the trusted base, written where a
reader meets it.

Treating the handler as concurrent allows more interference than actually occurs,
so it is conservative and therefore sound; it costs precision, not correctness.

### `handlers { }`

The composition site, and what it declares is a **preemption order**:

```thermite
handlers {
  ipi_shootdown  at 2,
  timer_isr      at 1,
}
```

Normal context is level 0. For every pair where `level(h) > level(g)`, generate
`promises(h) ⟹ asks(g)` and nothing in the other direction. Nested interrupts
fall out: `ipi_shootdown` must be tolerated by `timer_isr` as well as by normal
code.

Two consequences. **Handlers are roots of the call graph** — hardware invokes
them, nothing calls them — so the rule that effects propagate upward terminates
there rather than escaping into some caller's row. And a lock any handler takes
[needs masking rather than ordering](0010-shared-state-invariants.md#a-lock-a-handler-takes-needs-masking-not-ordering),
which is the defect no acquisition order catches.

## What is deferred, and to where

**User-supplied equations.** Deciding whether two theories commute from their
equations is a word problem, so commutation would have to be declared and
discharged as an obligation rather than computed. A fixed basis with user
combinations is an RFC; arbitrary equations are research.

**Handlers as a programming feature.** Effect handlers in the Koka or Eff sense —
resuming a computation from an operation — bring their own metatheory and are not
proposed.

**Distributional proofs.** The shape is well defined even though the discharge is
not: *stipulate a distribution on a source input, certify that the output has a
stated distribution.* That is what a cryptographic argument needs, and it is why
`random` is designed to accept a parameter later — `random(D)` — so gaining it is
not a breaking change. It belongs on
[the ladder](https://github.com/bulla-systems/bulla/blob/main/docs/the-ladder.md#the-rungs) beside noninterference and cost: surface
reachable now, discharge open.

## Relationship to the other documents

| document | what it takes from here |
|---|---|
| [verified effect rows](0009-verified-effect-rows.md) | its conflict table, derived rather than stipulated; the containment order refines the instance relation |
| [shared-state invariants](0010-shared-state-invariants.md) | `owns(r)` as dynamically-established commutation |
| [resource types](0011-resource-types.md) | `forgets(r)` as an ordinary state effect |
| [protocol types](0013-protocol-types.md) | `blocks` as a given atom, recorded and unproved |
| [the surface conventions](0007-thermite-3.md) | the two-family shape, which is this basis seen from the surface |
