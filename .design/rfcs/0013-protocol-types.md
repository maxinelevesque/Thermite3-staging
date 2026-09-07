---
rfc: 13
title: Protocol types — sessions whose endpoints cannot be abandoned
status: draft
language-evolution: tracked
supersedes: []
introduces:
  - REQ-SYNTAX-PROTOCOL-DECL
  - REQ-SPEC-PROTOCOL-ENDPOINT
---

audited-content-sha256: ee2d0b66d9d8291cbc198fb456f3a40925ac85b7b7673c46ecab73331c536efa (RFC-13 final review-fix pin; exhaustive branch detection plus statement-ordered, fail-closed payload typing added. prior: 277fdd2b88f6af460a94944dfff2e5e968088426ff1db51d01bc741d48af6eed)

# RFC-13: Protocol types — sessions whose endpoints cannot be abandoned

| | |
|---|---|
| **Status** | **Shipped on `main` via PR #149; issue #77 closed.** |
| **Fork implementation** | **Complete.** The parser, binary projection checker, endpoint completion discipline, L1/L3 carriers, versioned witness, Lean replay, Forge replay gate, and conformance anchors are implemented. |
| **Baseline** | `maxinelevesque/Thermite3-staging @ 69475aaa` (`3.0.0-alpha.8`) |
| **Position** | step 9 of the sequence in [RFC-7](0007-thermite-3.md#14-the-sequence) |
| **Depends on** | [RFC-6](0006-full-words.md), RFC-11 |

> **Shipped 2026-09-06.** The RFC-7 direction and dependency sequence are
> resolved. This document remains the rationale and records the exact v1 surface
> delivered by the reviewed implementation.

Kind: new item form. Orthogonal to the effect-rows RFC through the
interference-clauses RFC.

## Implemented v1 contract

The v1 implementation deliberately closes the binary core and leaves broader
session calculus features explicit:

- `protocol P { Role { field: T, ... }, ..., end }` and the terminal
  `repeat | end` form parse as global declarations. Exactly two sender roles are
  admitted; multiparty projection remains deferred.
- `P::Role` endpoints may enter a function only as owned parameters, may not be
  stored in `shared` state, and require `! blocks`.
- `c.send(...)` and `c.receive()` advance the inferred local projection.
  `let payload: T = c.receive_payload()` exposes a received payload, requiring
  `T` to equal the declared scalar or tuple payload type. Send payloads must be
  literals, typed locals, explicit casts, or tuples of those forms whose type
  the v1 checker can establish; bind more complex expressions to an explicitly
  typed local before sending. An uninferrable send payload fails closed.
- A repeating protocol's first sender chooses `repeat()` or `end()`; its peer
  receives that discriminant with `receive_repeat()` or `receive_end()`.
- Every return and fallthrough must reach `end`. Endpoint aliasing, wrong-role
  actions, skipped turns, payload mismatches, invalid repeat exits, and
  abandonment are structured compile-time errors.
- General control-flow paths containing protocol actions are rejected in v1.
  This prevents a branch-flattened proof witness; path-sensitive session-flow
  witnesses are a future additive extension.

The checked Rust report is content-bound into a versioned witness. Lean
independently reconstructs binary projections and checks complementarity,
transition order, repeat rounds, and endpoint completion before Forge may issue
any certificate for a protocol-bearing file. Each resulting certificate and
audit row carries the checked protocol definitions and endpoint flows, the
content-bound replay digests, the kernel-accepted verdict, and the residual
trust list as typed public evidence. Audit construction requires the matching
live replay authority, so editing the displayed evidence cannot manufacture an
accepted audit row.

The theorem stops at the channel boundary. Generated L1/L3 endpoint carriers
reserve the protocol operations, while the platform supplies transport,
blocking/wakeup behavior, peer identity, failure/cancellation, and received
payload bytes. L3 marks those carrier methods `external_body`; L1's typed
`receive_payload` stub cannot run without a platform replacement. These are
residual platform assumptions, not consequences of Lean's duality/completion
replay.

## Summary

Those RFCs type the **shared state** between concurrent parties. This types the
**conversation** between them. A kernel with a static IPC topology is an
unusually good fit, because the protocol is known at build time.

## Proposal

A protocol is a **sequence of turns**, each labelled with the role whose turn it
is, ending in `end`:

```thermite
protocol PageRequest {
  User     { op: u32, count: u64 },
  Provider { status: u32, base: u64 },
  end
}

fn pager(c: PageRequest::Provider) -> ()
  ! blocks
  requires  nothing
  ensures   nothing

fn app(c: PageRequest::User) -> ()
  ! blocks
  requires  nothing
  ensures   nothing
```

Roles are **names, not keywords**, so a protocol says what its roles are in its
own domain — `Reader`/`Writer`, `Initiator`/`Responder`, `Coordinator`. Two
reserved words could never fit all of those. They are capitalised because they
are types: `PageRequest::Provider` is the type of that endpoint.

An earlier draft used `provides PageRequest` and `uses PageRequest`. The path
form is better on two counts. It is a **noun in a type position**, where the
subject is `c` rather than the function — `c` does not provide anything, it *is*
an endpoint — and the verb form violated the same rule it was chosen to satisfy.
And it **generalises**: `Transfer::Coordinator` is well-formed for any number of
roles, where two verbs cannot name three parties and there is no third word
meaning "the other other one". `dual` was rejected as a mathematician's word for
the mirror of a thing, and `client`/`server` bakes in an assumption about who
connects to whom.

`end` is the last element of the sequence rather than a marker floating after it.
That is what it is: in the theory a session type is `!T . ?U . end`, and `end` is
a type. A header flag saying the protocol repeats was tried and withdrawn,
because real repetition is *conditional* — a server loops until told to close —
so termination is a branch:

```thermite
protocol PageStream {
  User     { op: u32, count: u64 },
  Provider { status: u32, base: u64 },
  repeat | end
}
```

**A branch costs a message.** If the User decides `end` while the Provider is
still waiting, the Provider waits forever, so the choice has to be transmitted.
`repeat | end` implies a discriminant on the wire, sent by whichever party acts
next — here the first role, since repeating restarts with it, which makes the
chooser inferable in this case. General branching needs the chooser named:

```thermite
protocol Request {
  User picks {
    ok:  { op: u32 }    then Provider { value: u64 },
    err: { }            then end,
  }
}
```

**Termination is load-bearing rather than cosmetic.** An endpoint is a
`resource`, so it must be consumed exactly once, and what consumes it is running
the protocol to completion. A protocol that never ends has an endpoint that can
never be released, and the linearity obligation becomes undischargeable. So
protocols need `end` to be compatible with resource types.

## Binary is a degenerate global type

Worth fixing now, because the alternative framing does not survive contact with a
third party.

The syntax above is a **global type** — it describes the session from outside,
naming who sends at each step, rather than describing one endpoint's view. That
matters because duality does not generalise. Duality is a binary operation, and
with three roles there is no "the dual"; worse, pairwise duality does not
compose. Three pairwise-dual channels can deadlock in a cycle, with every
pairwise check passing.

The multiparty answer (Honda, Yoshida, Carbone, 2008) replaces duality with
**projection**: write one global type, project it onto each role to get that
role's local type, and check each implementation against its own projection.
Consistency holds by construction because all the local types come from one
source.

So `PageRequest::Provider` *is* the projection `G ↾ Provider`, and for two roles
projection coincides with duality. Specifying binary this way costs nothing now —
projection onto two roles is duality — and means a later multiparty extension
adds cases to a function rather than replacing the framing.

What multiparty would need is both parties named per step, since "User sends" is
only unambiguous when there is exactly one other party. That is a notation
change, and it is where the arrow would earn its way back, since `A → B` is a
labelled edge where the symbol is the concept.

**Deadlock freedom is free for binary and not for multiparty.** Duality gives it
structurally for two parties. A well-formed global type gives communication
safety and protocol fidelity for many, and progress within one session, but
cyclic waits across several interleaved sessions need more. This RFC proposes
binary only, and that is the reason.

## What a hand-encoded session already gives

Before proposing syntax, the encoding was probed, because a session is a state
machine and state machines verify today. The Provider's view of
`User { op, count } → Provider { status, base } → end`:

```thermite
enum Message { Request { op: u32, count: u64 }, Reply { status: u32, base: u64 } }
enum Turn    { Awaiting { step: u32 }, Done { status: u32 } }

struct Session { step: u32 } inv step <= 2

fn advance(s: Session, m: Message) -> Turn
  req (s.step == 0 && m is Message::Request) || (s.step == 1 && m is Message::Reply)
  ens match result {
        Turn::Awaiting { step } => step == s.step + 1,
        Turn::Done { status }   => s.step == 1,
      }
  fx  pure
```

Every item certifies at **L3**, the contract is **non-vacuous** and kills 5 of 6
mutants, and a caller that replies at step 0 is rejected:

```
[FAIL] precondition not satisfied @ misuse_check.rs:48:5
```

**So protocol fidelity is enforceable today, with no new syntax.** The headline
claim below — a party that sends when it should receive fails to typecheck — is
already reachable as a precondition naming the legal message per step. That
changes what this RFC is: it proposes *sugar over something provable*, not a new
obligation, which is a much smaller thing to ask for and a much easier thing to
evaluate.

Two things the encoding does not give, and both are the reasons the sugar earns
its place.

**It does not compose.** The precondition is written by hand per protocol, and
nothing checks that the two parties' hand-written state machines are duals. A
declared protocol projects both sides from one source, which is where the safety
actually comes from.

**It does not force completion.** `abandon(s: Session) -> u64` — taking a session
at step 0 and simply dropping it — verifies. Affine types permit the drop, and no
contract can say "you must call `advance` again", because that is liveness. This
is the concrete evidence for the endpoint-is-a-resource requirement below: the
safety half is reachable today and the completion half is not.

## What it buys

- **Protocol conformance.** A party that sends when it should receive fails to
  typecheck.
- **Deadlock freedom, free.** For binary sessions, duality gives it structurally.
- **Session fidelity.** The conversation ends where `end` says it does.

For a decomposed Unix — filesystem server, network server, process server — this
is the difference between "the servers are isolated" and "the protocol between
them cannot desynchronise".

## A channel is shared state, and the protocol is its discipline

Worth stating plainly, because the effect row's treatment of channels only makes
sense once it is.

Two parties observe one channel, so a channel *is* shared state. It does not
appear in the effect row the way `shared shootdown: Shoot` does, and the reason
is not that it is less shared — it is that its discipline lives somewhere else.
For memory, the discipline is a lock (the shared-state-invariants RFC) or a
rely-guarantee pair (the interference-clauses RFC), and the effect row is what
carries it. For a channel, **the protocol type is the discipline**: at each step
the type names exactly one party that may send, so the interleaving that would
be a race is not expressible.

Three things follow, and each is a requirement rather than an observation.

**Endpoints must be resources, not merely affine.** The first draft listed this
as an open question that this RFC "probably depends on". It certainly does, in
both directions. Duplicating an endpoint would put two parties at one step, which
is the race the protocol is supposed to exclude. Dropping one abandons the peer
mid-session, blocked on a message that will never come — and affine types permit
dropping. So an endpoint is `resource` per
[resource types](0011-resource-types.md), and this RFC does not stand
without resource types.

**The effect row carries only `blocks`.** Per
[the surface conventions](0007-thermite-3.md), a `shared` thing is named in
the row and a `resource` thing is not, because ownership already establishes
exclusivity. An endpoint arrives as an owned parameter, so there is no conflict
for the row to decide and nothing to name. What remains is that the function can
wait, which is a control effect.

**An endpoint may not be stored in `shared` state.** This is the restriction the
other two rest on. If an endpoint were reachable from a `shared` declaration,
two functions could reach the same endpoint by name, ownership would no longer
establish exclusivity, and both the bare `blocks` row and the protocol's
one-sender-per-step guarantee would fail at once. The surface conventions flag
this as the condition under which the effect-row rule stops holding; enforcing it
belongs here, as a rejection at the `shared` declaration.

The payoff of getting this shape right is that an endpoint may cross a
concurrency boundary safely. Handing a session to another CPU is a move, not a
copy, so the protocol's guarantees travel with it — which is what makes a
decomposed kernel's servers relocatable rather than pinned.

## Metatheory

Session types (Honda, 1993; Honda, Vasconcelos and Kubo, 1998). Binary session
duality and its deadlock-freedom result are settled. Multiparty session types
(Honda, Yoshida and Carbone, 2008) generalise beyond two parties and are
substantially harder — this RFC proposes binary only.

## Open questions

- **Failure.** What is the protocol type of a partition that crashes mid-session?
  Real systems need a cancellation story and session types are traditionally weak
  on it. This is also where the endpoint-is-a-resource rule meets the `panic`
  question from [resource types](0011-resource-types.md): an aborting party drops its
  endpoint, and the peer's blocked wait is the observable consequence.
