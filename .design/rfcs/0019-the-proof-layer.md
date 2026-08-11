---
rfc: 19
title: The proof layer — what convinces the verifier, separated from what runs
status: draft
supersedes: []
introduces: []
---

# RFC-19: The proof layer — what convinces the verifier, separated from what runs

| | |
|---|---|
| **Status** | Draft, unfiled. A design pass, not a proposal. §0's five questions were answered 2026-08-11; the answers are recorded in §0 and applied through the document |
| **Supersedes** | — |
| **Baseline** | `staging @ b79b4005`; surface at the [RFC-7](0007-thermite-3.md) endpoint |
| **Position** | Cross-cutting. Assumes RFC-6; orthogonal to RFC-8..14 |
| **Relation** | Names a construct RFC-7 §6, `.design/stage1-forge-tier.md` REQ-3, and `.design/forge/spec-review.md` each built one face of |

The verification surface at the RFC-7 endpoint is large enough that a single
`.th` file carries four kinds of material with four different audiences. This
document asks where the seam between them falls, and finds that the answer is
already implemented three times under three names.

**Contents:** [§0 What this asked, and what was answered](#0-what-this-asked-and-what-was-answered) ·
[§1 The seam is already built](#1-the-seam-is-already-built) ·
[§2 The rule](#2-the-rule) · [§3 What the rule decides](#3-what-the-rule-decides) ·
[§4 Clause labels](#4-clause-labels) · [§5 The surface](#5-the-surface) ·
[§6 The witness block](#6-the-witness-block) ·
[§7 Two invariants](#7-two-invariants-that-make-the-separation-safe) ·
[§8 Where it lives](#8-where-it-lives) ·
[§9 Separating a specification from an implementation](#9-separating-a-specification-from-an-implementation) ·
[§10 Costs](#10-costs) · [§11 Open questions](#11-open-questions) ·
[Appendix](#appendix-what-the-probes-established)

---

## 0. What this asked, and what was answered

A direction check on one rule and its consequences. It was answered on
2026-08-11. Nothing is filed and no requirement is introduced — the answers set
direction for a later proposal, and this document remains a design pass.

**1. Is the rule right?** A construct belongs to the proof layer when nothing
outside the verifier depends on it. §2 states it and §3 applies it.

> **Yes, with the evidence claim narrowed.** The rule and both tests stand. What
> is scoped back is §2's closing sentence, which offered the agreement between
> the rule and `forge review`'s projection as evidence that the rule "is a
> description rather than a preference". That agreement is real but holds over a
> smaller domain than the sentence implied: `forge review` structurally never
> reads a body, so every body-keyed construct — loop `keeps`, loop `measures` —
> is a case where the two do not both apply, and §3.4 is precisely where the rule
> is overridden on other grounds. §2 now says what the agreement covers.

**2. Is the promotion the right shape?** The forge tier already parses
`prop fn`, `lemma`, `proof for f` and `witness`. This asks whether that item
set is a forge-tier feature or the language's proof layer, gated by routing
rather than by grammar.

> **It is recognition rather than migration.** The probe settles the question in
> a way the framing did not anticipate: all four forms parse in an *ordinary*
> file with no attribute and no feature flag, and emit addresses
> (`sum.proof.ensures#1:Forge`, `witness#1:Forge`). There is no gate to move them
> through. The forge-tier framing is already nominal, and the honest statement is
> that the item set is the language's today and always has been — what is missing
> is that nothing says so, and nothing checks the bindings (§7).

**3. Are clause labels wanted?** §4. They replace the positional `ensures#k`
with a name, and they are what makes body-keyed proof material addressable.

> **Yes — optional, and required on any clause the proof layer references.** §4
> as written. Optional rather than mandatory follows RFC-9's argument: a
> mechanical pass over 547 sites would emit `ensures_1`, which is the ordinal
> with extra steps. Required-where-referenced is what gives out-of-line material
> something stable to bind to. This is where essentially all of the
> implementation cost sits, and it is load-bearing for the rest: §3.2's argument
> that a positional address cannot key committed proof material means the
> out-of-line forms have nothing to bind to without it.

**4. Does the covenant belong in the source at all?** §6 finds one directive in
it that measures tool effort rather than stating a fact, and moves it out.

> **The budget leaves the source**, as §6.3 works it: `admits` and `excludes`
> stay as checked claims, the budget becomes `--falsify <N>` beside `--rlimit`,
> and the counts and seed stay in the certificate as an oracle-excluded receipt
> sibling to `burn`. The decisive evidence is mechanical rather than aesthetic:
> keeping the budget in source is what forced `covenant_evidence` into
> `oracle_subset`, so `conformance/covenant/max_correct.cert.json` now pins
> `"falsify_generated": 2002` — a measured result frozen as though it were a
> specification. §10 prices the resulting golden change; sequencing it is an
> implementation question this document does not settle.

**5. Which separation of specification from implementation is wanted?** §9 gives
three, and only one of them needs a module system. The first draft of this
document said all three did, which was wrong.

> **(a) only — the body moves out.** §9's own argument decides it: (a) and (b)
> give the same physical separation and the same authority boundary and differ
> only in which artifact is primary, and for a language whose thesis makes
> contracts the only artifact needing intent review, the contract is the one to
> keep primary. (b) is retained in §9 as a considered alternative rather than a
> proposal, because its cost is a retirement — `surface-grammar.md` REQ-2,
> "absence of a required clause is a parse error", would have to go, which makes
> `fn f(x: u32) -> u32 { x }` legal Thermite and turns THERMITE.skill.md §8's
> verification-is-the-default polarity from a grammar property into a link-time
> check. (c) stays blocked and stays analysis.
>
> §9(a)'s one open choice is settled with it: the declaration carries a marker
> rather than the bodyless-`fn` gate relaxing, and the marker is the keyword
> **`contract fn`**. See §9(a).

## 1. The seam is already built

Three subsystems implement one idea, and none of them names it.

### 1.1 `opaque spec fn` seals a spec body behind its interface

From [RFC-7 §6](0007-thermite-3.md#6-specification-vocabulary):

```thermite
opaque spec fn plan_ok(p: Plan) -> bool
  ensures   !result || p.count > 0     // the interface, visible while sealed
  measures  p.count
{ ...expensive, quantifies over p.regions... }
```

`opaque` splits one item into an interface consumers reason from and a body they
do not see. That is the separation, applied to a single spec function.

### 1.2 The forge tier's item set is out-of-line proof, and it parses today

`.design/stage1-forge-tier.md` REQ-3 specifies four item forms, and
`thermite-syntax/src/ast.rs` carries them as `Item::Forge(ForgeItem)`:

| form | AST | what it is |
|---|---|---|
| `prop fn NAME(..) -> T { .. }` | `PropFnItem` | a proposition definition |
| `lemma NAME(..) requires .. ensures .. proof { .. }` | `LemmaItem` | a named lemma with a proof block |
| `proof for f { ensures#k by { .. } }` | `ProofItem` | proof discharging a named clause of an existing `f` |
| `witness { inhabit (..); falsify N; }` | `WitnessBlock` | the covenant for the function it follows |

`ProofItem` is the construct this document is about. It carries a `target: Ident`
and a list of `ProofObligation`, each a `ClauseSelector` (`ensures#k`) plus a
`ProofBlock`. Proof material for `f`, written outside `f`, bound to `f` by
address.

The forms parse in an ordinary file with no attribute and no feature flag, which
is REQ-3's "syntax is always-on; gating is forge-side via holes and routing"
holding in practice. The appendix records the probe.

### 1.3 `forge review` projects the seam deterministically

`.design/forge/spec-review.md` REQ-1 is SHIPPED in `forge/src/review.rs`. It
extracts, per `fn`, the verbatim `requires`, every `ensures`, the `!` row, and
the *declaration* of every `spec fn` the contract references. It reads
`contract`, `name`, `params`, `ret` and `measures`, and never reads `body`.

The doc calls this out: "Exclusion is structural, not heuristic." The reviewed
surface is defined by which fields the projection reads.

So the compiler already computes the boundary between the surface a reviewer
must read and everything else. What it cannot do is let an author write the
program with that boundary already cut.

### 1.4 Two subsystems agree on where the line falls

`forge review` and `thermite-syntax/src/address.rs` were built for unrelated
reasons — one for the §7 intent-review slot, one for the per-item proof cache.
Their partitions of the surface are close to complementary:

| construct | in `forge review`'s projection | addressed by `address.rs` |
|---|---|---|
| `requires` / `ensures` on a `fn` | yes | as a forge-tier clause selector, `f.proof.ensures#k` |
| the `!` row | yes | no |
| `spec fn` declaration | yes (declaration only) | root only, `AddrKind::SpecFn` |
| `spec fn` body | no | no |
| `fn` body | no | no |
| loop `keeps` / `measures` | no | yes, `f.loop#1.keeps#2` / `.dec` |
| body hole `?N` | no | yes, `AddrKind::Hole` |
| `prop fn` / `lemma` / `proof for` / `witness` | no | yes, `AddrKind::Forge` |
| proof hole `?pN` | no | yes, `AddrKind::ProofHole` |
| `struct` / `enum` and their `keeps` | no | no (`addresses_of` skips `Item::Struct`/`Item::Enum`) |

The reviewed surface is the contract plus spec-fn declarations. The addressed
surface is everything a proof is written *about*. The two overlap only where the
forge tier already reaches across, at `f.proof.ensures#k`.

`validate_segments` in `address.rs` already admits that reach:

```rust
if matches!(seg, "measures" | "proof" | "ensures" | "requires" | "keeps") {
```

and, for numbered segments, `"loop" | "keeps" | "ensures" | "requires"`. The
address grammar for an out-of-line proof layer exists.

## 2. The rule

> A construct belongs to the **proof layer** when nothing outside the verifier
> depends on it.

Two tests make it mechanical.

**Erasure.** The construct generates no code and contributes no runtime check.
This is necessary and not sufficient: a contract erases at L3 and still belongs
with the item, because callers depend on it.

**Deletion.** Delete the construct. If what the program does and what it promises
are both unchanged, and the only difference is that an item stops certifying,
the construct is proof material.

Applied to the surface at the RFC-7 endpoint:

| construct | erases | who depends on it | layer |
|---|---|---|---|
| `fn` body | no | callers, at runtime | program |
| `requires` / `ensures` | at L3; enforced at L1 under `#[slag]` | callers | contract |
| a clause label (§4) | yes | readers and reviewers | contract |
| the `!` row | yes | callers, by the composition law | contract |
| `keeps` on a struct or enum | yes | every holder of the type | contract |
| `survives` (RFC-14) | yes | callers, across a crash | contract |
| `interleaves { asks, promises }` (RFC-12) | yes | peers, at the composition site | contract |
| `shared` / `lock` / `resource` / `protocol` / `handlers` / `effect` | yes | the program, which obeys them | contract |
| `spec fn` declaration | yes | the contract that names it | contract |
| `spec fn` body | yes | the verifier, unless `opaque` | spec |
| `by unfold(p)` on an `ensures` | yes | the verifier | **proof** |
| `lemma`, `prop fn` | yes | the verifier | **proof** |
| `proof for f { .. }` | yes | the verifier | **proof** |
| `admits` / `excludes` (§6) | yes | the verifier's covenant gate | **proof** |
| loop `keeps` / `measures` | yes | the verifier | **proof by the rule; see §3.2 and §4** |

The rule and `forge review`'s projection give the same answer everywhere they
both apply, which is evidence that the rule describes something the compiler
already computes rather than stating a preference.

The scope of that evidence is worth being exact about, because the sentence is
easy to read as stronger than it is. `forge review` structurally never reads a
body, so the domain where both apply is the contract and spec-declaration
surface — the rows above `spec fn` body in the table. Every body-keyed construct
is outside it: the projection has no opinion on loop `keeps` or loop `measures`,
which is why the last row of the table defers to §3.2 and §4 rather than to the
projection, and why §3.4 records the one case where the rule is overridden on
address-stability grounds. Corroboration over the contract surface is what is
claimed; corroboration everywhere is not.

## 3. What the rule decides

### 3.1 `by` retires from the contract clause

At the RFC-7 endpoint a proof hint is written inside a contract clause:

```thermite
  ensures match result { Ok(p) => plan_ok(p), Err(e) => true }
          by unfold(plan_ok)
```

and the same hint is written out of line by the forge tier:

```thermite
proof for allocate {
  ensures#1 by { unfold(plan_ok) };
}
```

These are one construct with two spellings, which is the case pillar 3 rules out
(thermite-design.md §2.3, one way to do everything). The out-of-line spelling is
the one to keep, on three counts.

**It reaches further.** The inline form attaches to the clause it is written in.
The out-of-line form takes any `ClauseSelector`, so a hint can target
`requires#2` or a clause of a `#[slag]` function whose body carries no proof at
all.

**It keeps the reviewed surface clean.** `forge review` REQ-1 emits the verbatim
`Clause.text`. With `by` inline, the intent-review prompt for `allocate` contains
`unfold(plan_ok)`, which is not a claim about the function and is not what the
reviewer is being asked about.

**It partitions the proof cache.** thermite-design.md §5.3: "an edit to `f`
cannot invalidate `g`'s certificate unless `g`'s contract references `f`'s
contract." With `by` inline, editing a hint edits a clause, and a clause is part
of the contract. Out of line, editing a hint invalidates one certificate and
touches no contract, so the cache key partitions along the layer boundary.

Cost: a reader of `ensures` no longer sees the hint that discharges it. That is
the locality pillar 5 protects, and it is a real loss. The reply is that the hint
is addressed to the verifier, and `forge goal <f> --proof` is where a reader who
wants it looks.

### 3.2 A positional address cannot key committed proof material

By the rule, a loop `keeps` is proof material: nothing outside the function
depends on it, it erases, and deleting it changes only whether the function
certifies. `forge review` excludes it from the reviewed surface.

Moving it out is unsafe under the address scheme as it stands.
`semantic-addressing.md` REQ-2 numbers loops in source-text order within the
function, and REQ-5 scopes address stability to three cases: an unrelated item
edit, an edit to a loop's body statements, and a rename. Adding a loop to a
function renumbers every later loop in it. An out-of-line `f.loop#2.keeps#1`
therefore retargets when an author inserts a loop above it, and the failure mode
is a proof hint attaching to the wrong invariant.

The same hazard reaches contract clauses. `proof for isqrt_class { ensures#2 by
{ … } }` is in the corpus today, and inserting an `ensures` above the second one
silently moves what that proof discharges.

So the constraint is not about loops. It is that **an ordinal is a coordinate,
and proof material committed to a file needs a name.** §4 supplies one, and with
it the restriction lifts in both places.

### 3.3 The `measures` question is smaller than it looks

A termination measure is a witness, and termination itself is a row property:
`! diverge` is what a caller inherits. That suggests `measures` is proof material
and the row is the contract.

For a recursive `fn` the language already reads it this way. RFC-7's probe table
records a recursive `fn` with no measure and no `diverge` as rejected "naming the
`fx diverge` exemption", and the same function with `fx diverge` as accepted at
L1. Absence of a measure plus `diverge` in the row is already the spelling for
"this does not terminate".

For a loop it does not. The same table records "a loop with `fx diverge` on the
enclosing fn and no measure" as `missing the mandatory 'dec' clause`, and RFC-7
calls that out as worth a defect report: an idle loop is only writable by
supplying a measure that cannot decrease, and the false measure stays in the
source.

That defect is loop-local and independent of this document. Extending the fn-level
`diverge` exemption to a loop inside a `diverge` function fixes it, and the proof
layer neither helps nor is needed. Recorded here so the two are not bundled.

### 3.4 What stays, and one case the rule decides against intuition

`interleaves { asks, promises }` looks like verifier material and is not.
[RFC-12](0012-interference-clauses.md) discharges `promises(a) ⟹ asks(b)`
pairwise at the composition site, so a peer depends on both clauses. They are the
concurrent analogue of `requires`/`ensures` and they sit with them.

## 4. Clause labels

A clause may carry a name, written between the keyword and the expression:

```thermite
fn sum(xs: &[u32]) -> u64
  ! pure
  requires bounded_input:  xs.len() <= 1_000_000
  ensures  matches_model:  result == spec_sum(xs)
  ensures  no_overflow:    result <= xs.len() as u64 * u32::MAX as u64
{
  while i < xs.len()
    keeps in_range:    i <= xs.len()
    keeps partial_sum: acc == spec_sum(&xs[..i])
    measures xs.len() - i
  { ... }
}
```

and inside a conjunct block:

```thermite
  requires {
    positive:  pages > 0;
    in_bounds: pages <= 1024;
  }
```

### The slot exists

`Clause` already carries an annotation field, `bv: Option<BvTag>`, described in
`ast.rs` as "the first clause-level annotation in `thermite-syntax`" and carrying
the property a label needs:

> The tag sits outside `text`, so semantic addresses remain unchanged.

The surface position follows the same precedent: `ensures@bv64 result + 0 ==
result` places the annotation between the keyword and the expression. `:` is
otherwise unused in expression position — the grammar spends it only in `Param`
and `LetStmt` — so one token of lookahead separates a labelled clause from an
unlabelled one.

### The ordinal disappears

Labels are unique **within the item**, not within the enclosing block. The
address of a labelled clause is then `<item>.<label>`, with no ordinal segment at
all:

```thermite
proof for sum {
  no_overflow by { ... };
  partial_sum by { induction(i) };
}
```

`partial_sum` is a loop invariant discharged from outside the body, and the
address does not mention the loop. That is what lifts §3.2: the coordinate is
gone rather than renamed, so inserting a loop or an `ensures` moves nothing.

`ClauseSelector { keyword: Ident, index: Option<u32> }` gains a label variant,
and `validate_segments` admits an identifier segment. The reserved segment words
— `proof`, `ensures`, `requires`, `keeps`, `measures` — are refused as labels so
an address cannot be read two ways.

### Optional, with one rule that binds

A label is optional in general and **required on any clause the proof layer
references.** Ordinals stay legal for tools that resolve them against a fresh
parse (`forge edit`, `forge goal`) and are refused in committed proof material.

Mandatory labelling was considered and rejected on RFC-9's argument about its own
migration: "unlike a rename, this one cannot be automated." A good label is a
judgement about what a clause means, and across 547 clause sites a mechanical
pass would emit `ensures_1`, `ensures_2` — the ordinal with extra steps, and
worse for reading like a name. Under the rule above, adoption follows need: a
clause acquires a label when someone writes a proof against it, which is when its
meaning is known.

### A label is contract, not annotation

By §2's rule, the question is who depends on it, and the answer is a reader. So
labels enter `forge review`'s projection and are reviewable text, and a wrong
label is a defect of the same kind as a wrong `ensures`.

This is the cost labels buy their readability with, and it is worth stating
plainly because it runs against `telos/surface-serves-agents` rather than with
it. Given

```thermite
  requires input_nonempty: pages > 0
```

the clause states positivity and the label says nonemptiness. Read quickly, the
label tells a reviewer the contract rejects empty inputs, which it does not say.
An ordinal cannot mislead because it asserts nothing. A label can, and the
reviewed surface is where that lands.

## 5. The surface

One function, with the layers separated. Every form below is either the RFC-7
endpoint surface, `stage1-forge-tier.md` REQ-3, or the additions in §4 and §6.

The program, carrying its full contract:

```thermite
opaque spec fn plan_ok(p: Plan) -> bool
  ensures   nonempty_plan: !result || p.count > 0
  measures  p.count
{ ... }

fn allocate(pages: u64) -> Result<Grant, u64>
  ! write(heap), cost(12 * pages + 40)
  requires {
    positive:  pages > 0;
    in_bounds: pages <= 1024;
  }
  ensures grant_sized: match result {
    Ok(g)  => g.len == pages * 4096 && plan_ok(current_plan()),
    Err(e) => true,
  }
{
  let mut remaining: u64 = pages;
  while remaining > 0
    keeps    within_request: remaining <= pages
    measures remaining
  { ... }
  ...
}
```

The proof layer:

```thermite
lemma plan_ok_survives_grant(p: Plan, g: Grant)
  requires plan_ok(p)
  ensures  plan_ok(extend(p, g))
  proof { simp [Thermite.denote]; omega }

proof for allocate {
  grant_sized    by { unfold(plan_ok); cite(plan_ok_survives_grant) };
  within_request by { induction(remaining) };
}

witness for allocate {
  admits (1);
  admits (1024);

  positive  excludes (0);
  in_bounds excludes (1025);
}
```

Read the first block alone and the function's interface is complete: what it
takes, what it returns, what it touches, what it requires and what it ensures.
Read the second alone and nothing about the program's behaviour is available.
That asymmetry is the property §7 pins down.

Two spellings change from `stage1-forge-tier.md` REQ-3, both because RFC-6's
naming rules apply once these forms are language-level:

- `witness for f { .. }` rather than a bare `witness { .. }` following the
  function it covenants. `isqrt_class.th` shows why the positional binding is
  fragile: its `witness` sits between the function body and a `proof for` block,
  so inserting an item between them moves the covenant to a different function
  with no diff to the covenant. A named target makes it addressable as
  `f.witness`.
- The block's directives change, for the reasons in §6.

## 6. The witness block

The covenant is the evidence gathered before any proof runs.
`stage1-forge-tier.md` REQ-4 gives it two directives, and they are different
kinds of thing:

| directive | says | a claim about the item? |
|---|---|---|
| `inhabit (1)` | this tuple satisfies `requires` | yes, but the elided subject is the tuple, not the item |
| `falsify 50_000` | search this many random inputs | no. It is a budget |

Both are worth changing, for separate reasons.

### 6.1 The subject rule

RFC-7 rule 2 elides a clause's subject because it is always the item. A covenant
directive is sometimes about the item and sometimes about one clause, so the
subject has to become visible:

> **Subject elided means the item. Subject named means that clause.**

```thermite
witness for allocate {
  admits (1);
  admits (1024);

  positive  excludes (0);
  in_bounds excludes (1025);
}
```

*allocate admits (1)*. *positive excludes (0)*. Both read as sentences with the
subject where the surface conventions put it.

| form | checked how |
|---|---|
| `admits (t)` | evaluate `requires` at `t`; it must hold. Today's `inhabit`, renamed |
| `L excludes (t)` | evaluate every conjunct at `t`; all but `L` hold and `L` fails |

`inhabit` is a type theorist's word for a property of a type, in a slot where
every other Thermite clause is a third-person verb about the item — the objection
RFC-7 used to reject `dual`. It survives today because the forge tier was never
held to RFC-6's rules, and the promotion in §1.2 is where that applies.

`excludes` needs no new machinery: it is `admits`'s executable evaluator run per
conjunct. No solver.

### 6.2 What `excludes` is for

`Certificate.contract_quality.vacuous_precondition` is one boolean for the whole
precondition. A conjunct implied by its neighbours contributes nothing and makes
the contract read stronger than it is, and nothing notices today. `excludes` is
the per-conjunct version: a tuple satisfying every other conjunct and failing
this one witnesses that this one excludes something.

It also answers, checked, the question a label on an `inhabit` would have
answered by assertion. `admits (1)` together with `positive excludes (0)` pins
the boundary of `positive` between 0 and 1.

Most of these should not be written by hand. `falsify` already drives a
SplitMix64 generator against the executable semantics, and REQ-3's Q3 default is
that "witnesses may be generator-synthesized but at least one must be
author-stated." The same split applies: forge searches for an excluding tuple per
conjunct, a hit records the conjunct as independent, and a miss within budget is
surfaced as advisory — matching `strengthening-probes.md`'s "ADVISORY, not a
gate", since a gate here would fail every contract in the corpus. An author
`excludes` is a pin: it fixes the boundary as documentation and turns a search
into a regression test.

### 6.3 The budget leaves the source

Thermite already has a rule for where a measurement of tool effort lives, and it
applies it in two places:

| quantity | lives where | in `oracle_subset`? | stated reason |
|---|---|---|---|
| SMT resource budget | `--rlimit`, with `DEFAULT_RLIMIT` | no | — |
| proof size, cited lemmas (burn receipt) | certificate only | excluded | "re-authoring a proof legitimately changes its committed-token count without changing what was proven" |
| falsify budget | source syntax | included | "weakening a falsify budget or dropping a witness changes these numbers" |

The burn receipt is the worked example of the principle: a number measuring how
hard the tool worked is not part of what was proven, so it stays out of the
oracle. The falsify budget measures the same kind of thing and is handled the
opposite way on both axes.

Being written in source is what forced the oracle to widen.
`covenant_evidence` joins `oracle_subset` to police an author knob, and the
consequence is visible in a committed golden — `conformance/covenant/max_correct.cert.json`
carries `"falsify_generated": 2002`, which is two author witnesses plus the
author's `falsify 2000`. Raising that number on a program that is unchanged and
still correct regenerates the golden. The corpus figures (2000, 5000, 2000, 10)
have no semantic content; they record what each example needed while it was being
written.

**The objection.** `--rlimit` is safe as a flag because lowering it is
self-punishing: the budget runs out, the item times out, the verdict is worse.
Lowering `falsify` is self-rewarding: fewer inputs searched, fewer refutations,
and the covenant passes. The two are not symmetric, which is the reason to think
the budget was put in source deliberately.

**What resolves it.** The covenant is an economy gate rather than a rung on the
ladder. Its two outcomes carry different information: a refutation is decisive
(`CovenantRefuted` is `LadderAction::HardFail` in `degrade.rs`, "never a
degrade", the same treatment as `Counterexample`), while finding nothing carries
no assurance at all. `falsify_refuted: 0` grants permission to burn proof effort
and nothing more. A weakened budget therefore cannot buy assurance, because there
was none to buy — the proof still has to succeed on its own terms. What the
budget guards is proof spend, which makes it an economy parameter of exactly the
kind `--rlimit` is.

So:

- **the claims stay in source** — `admits` and `excludes`, every directive a
  checked fact about the item or one of its clauses;
- **the budget becomes `--falsify <N>`**, with a `DEFAULT_FALSIFY`, beside
  `--rlimit`;
- **the counts and seed stay in the certificate as a covenant receipt**, oracle
  *excluded*, sibling to `burn`, for the reason `burn` already gives;
- **a refutation stays decisive and sticky**, which is already true and already
  independent of where the budget is written.

The block that remains is uniform: nothing in it is a knob.

**The alternative considered.** Keeping authorial control by making the budget a
claim — stating what the search must achieve rather than how long it runs, as
`covers pages` — reads better and states a property rather than a setting. It
fails on definition: the range of a `u64` cannot be covered. The bounded version,
covering the boundaries that matter, is what the `excludes` set already states.

## 7. Two invariants that make the separation safe

Separating proof from program is only sound if the contract stays complete
without it. Two rules, both mechanically checkable.

**A proof-layer item may not strengthen a contract.** A `proof for f` discharges
obligations `f` already states. It cannot add an `ensures`, weaken a `requires`,
or introduce a row atom.

This is what keeps thermite-design.md §12's position intact — "contracts as the
only artifact that needs intent review". If proof material could add an
obligation, then `forge review`'s projection would be an incomplete account of
what `f` claims, and the pre-screened spec layer would stop being the whole
reviewable surface. The invariant is what makes the review slot's structural
exclusion of bodies still correct once bodies of proof exist elsewhere.

**No executable position may name a proof-layer item.** A `fn` body cannot call a
`lemma` or a `prop fn`. Name resolution enforces it, and the consequence is the
deletion test made total: remove every proof-layer item from a program and the
compiled artifact is identical.

Neither invariant holds today. The probe in the appendix found that `proof for
no_such_function { … }` parses clean and addresses cleanly, so the binding is
checked in neither direction — the same shape as the unchecked effect row RFC-9
was written against. Target resolution is the first thing this layering owes.

**This one is a defect now, not an obligation of the layering, and it is filed
separately.** An orphan `proof for` is wrong at the current baseline whether or
not anything in this document is adopted: rename a function and the out-of-line
proof committed against it silently detaches, with nothing reporting the break.
It is tracked as its own issue with the reproducing probe attached, so that
fixing it does not wait on a direction check. What that issue cannot yet say is
how bad it is — the probes here reach the parse and address layers only, and no
Verus or Lean run was made anywhere in this work, so whether a detached proof
merely rots (its target then fails to certify, which is loud and safe) or is
counted as discharging something (which would not be) is unestablished and needs
a certification run to separate.

**What a missing proof layer costs.** An item whose obligations do not discharge
is L0 and is reported as such. The failure mode of a proof layer that is absent,
stale or unwritten is that the program does not certify, never that it certifies
without evidence. The contract stays mandatory and inline, so it cannot be
otherwise.

## 8. Where it lives

Proof-layer items bind by semantic address, so physical placement is not a
language question. `proof for allocate` names `allocate`; nothing about the
binding depends on the two being in one file.

That makes three arrangements available without three mechanisms:

| arrangement | when it fits |
|---|---|
| same file, after the program items | a small program; the default |
| a sidecar file per module | proof bulk exceeds program bulk, which the forge tier's proof blocks make ordinary |
| a directory, by role | a project that reviews contracts and proofs on different cadences |

The choice is a project convention. This matches the position RFC-15 and RFC-18
took for the repository — file by role, and do not require a layout the language
depends on.

One consequence to state: a sidecar arrangement means a reader of `allocate.th`
cannot see whether a proof exists. `forge check` reports the level, so the
absence is visible in the verdict rather than in the source.

## 9. Separating a specification from an implementation

Everything above separates proof from program. Separating a *specification* from
an *implementation* is a different question, and it has three answers rather than
one. An earlier draft of this section gave only the third and said the whole
question was blocked on a module system. That was wrong: two of the three need no
module system at all, because they bind by name the way `proof for` does.

| | binds by | obligation | needs a module system | in scope |
|---|---|---|---|---|
| **(a) the body moves out** | name — `body for f` | none new | no | **yes** |
| **(b) the contract moves out** | name — `contract for f` | none new | no | no — considered alternative |
| **(c) the implementation refines a model** | a named abstract specification | refinement | yes | no — blocked |

§0's answer to question 5 takes (a) only. (b) is kept below because the
comparison is what justifies the choice, not because it is proposed; (c) stays
analysis until there is a module system to write it against.

### (a) The body moves out

The contract stays welded to the signature and the implementation becomes the
sidecar:

```thermite
// sum.th — the interface, and the whole reviewed surface
spec fn spec_sum(xs: &[u32]) -> u64
  measures xs.len()
{ match xs { [] => 0, [head, ..t] => head as u64 + spec_sum(t) } }

contract fn sum(xs: &[u32]) -> u64
  ! pure
  requires bounded_input: xs.len() <= 1_000_000
  ensures  matches_model: result == spec_sum(xs)
```

```thermite
// sum.body.th
body for sum {
  let mut acc: u64 = 0;
  let mut i: usize = 0;
  while i < xs.len()
    keeps in_range:    i <= xs.len()
    keeps partial_sum: acc == spec_sum(&xs[..i])
    measures xs.len() - i
  { acc = acc + xs[i] as u64; i = i + 1; }
  acc
}
```

`spec_sum`'s body stays in the interface file because it is the specification
rather than an implementation of one; `opaque` is the opt-out when the body is
expensive.

**Most of this ships.** `.design/boundary/ffi-boundary.md` REQ-1..REQ-7 are all
SHIPPED, and what they ship is a `fn` with a mandatory contract and no body:
`#[boundary("crate::path")] fn NAME(..) requires … ensures … ! … ;` parses,
`FnItem { boundary: Some(_), body: None }` is the AST shape, `closure.rs`,
`mutation.rs`, `verified_build.rs` and `address.rs` all branch on the absent
body, and REQ-7 states the semantic rule this needs — a caller `g` "certifies
independent of `f`'s foreign body". The merge pass has a precedent too:
`thermite-syntax/src/desugar.rs` exists as the post-parse pass the refinement
sugar needed, "so downstream stages see only the v1 clause shapes".

What is missing is the `body for f` item itself, which the probe confirms is a
parse error at item dispatch.

Two things it must not do. It must not reuse `#[boundary]`, which means *foreign,
unproven, capped at L1* and is in `oracle_subset`; a Thermite body supplied
elsewhere should certify normally. And the bodyless-`fn` parse gate is a real
decision rather than wiring: today the `;` body is gated on `boundary.is_some()`,
a per-item check, and a `body for` declaration cannot be validated at item-parse
time. Relaxing the gate and enforcing at file level costs pillar 5's per-item
independence; a marker on the declaration keeps it.

**The marker is a keyword, and the keyword is `contract`.** The declaration reads
`contract fn sum(..)`, and the gate stays per-item: a `contract fn` legally has
no body, every other `fn` still requires one, and both facts are decidable at the
item where they are written. Relaxing the gate was the alternative and was
rejected on that ground — it makes a bodyless `fn` an error only once the whole
compilation unit is known, which is the independence pillar 5 states.

Three things recommend `contract` over the other candidates.

*It completes a vocabulary that already exists.* The sidecar items are nouns
naming what they carry — `body for f`, `proof for f`, `witness for f`, and §9(b)'s
`contract for f`. Those four nouns are the four kinds of material this document
opens by observing that one file carries. `contract fn` makes the primary item
name its noun too, and the rule becomes uniform: **the noun says what this is;
it is either welded to the signature or `for`-bound elsewhere.** The same noun in
both positions is a feature — `contract fn f` is the contract welded on, and
§9(b)'s `contract for f` is the same contract moved out.

*It names the meaning rather than the file layout.* `separate`, `split` and
`outlined` all describe where the bytes went, which is the mechanism and not the
idea. What the declaration means is that this item's obligations are settled here
and the work is performed elsewhere against them.

*It stays clear of the contrast set.* The words with the strongest recognition
are the dangerous ones, because they already mean *no implementation* or
*unproven*, and both are taken here by constructs that mean something materially
different. `abstract` says "has no implementation" in Java, Scala and C#, and
collides with §9(c)'s abstract *model* in this same document. `partial` is a near
mechanical match in C# — declaration and body in separate files, merged at
compile time — but in a language with `measures` and totality checking, a
*partial function* is one that may not terminate. `extern` and `deferred` read as
foreign or not-yet-written. A `contract fn` is none of these: its body exists,
it is Thermite, and it certifies normally.

`contract` is not currently a reserved word — `keyword_kind` in
`thermite-syntax/src/lexer.rs` does not list it — so this is an addition rather
than a re-use. `opaque` is not reserved either, which is worth stating since §5
and §1.1 write `opaque spec fn` throughout: that spelling is RFC-7 §6's proposed
surface rather than shipped grammar, so `contract fn` would be the first
item-leading modifier of its kind to land, not the second.

### (b) The contract moves out

```thermite
// sum.th
fn sum(xs: &[u32]) -> u64
{ let mut acc: u64 = 0; ... acc }
```
```thermite
// sum.contract.th
contract for sum {
  ! pure
  requires bounded_input: xs.len() <= 1_000_000
  ensures  matches_model: result == spec_sum(xs)
}
```

This is the arrangement with costs. `surface-grammar.md` REQ-2 — "absence of a
required clause is a parse error" — has to go, which means `fn f(x: u32) -> u32
{ x }` parses. The polarity THERMITE.skill.md §8 describes, where "verification is
the default and free" and non-verification "costs more keystrokes and
visibility", stops being a grammar property and becomes a link-time check. And
`#[slag]` and `#[boundary]`'s shared claim that "the contract is STILL mandatory"
loses its referent.

(a) and (b) give the same physical separation and the same authority boundary: an
agent handed the sidecar cannot weaken what is in the other file either way. They
differ in which artifact is primary. For a language whose thesis makes contracts
the only artifact needing intent review, the contract is the one to keep primary,
which is (a).

### (c) The implementation refines a model

```thermite
specification Allocator {
  spec fn free_count(a: Allocator) -> nat

  fn allocate(pages: u64) -> Result<Grant, u64>
    ! write(heap)
    requires positive: pages > 0
    ensures  ...
}

module PageAllocator refines Allocator { ... }
```

`refines` names the obligation. `implements` names a different one, since an
interface is discharged by types and a refinement by proof, and the misnaming is
the kind `telos/surface-serves-agents` costs most.

This is the one that is blocked. `surface-grammar.md` REQ-1 admits three
top-level item forms plus the basis `struct`/`enum`, and lists `impl`, `trait`,
`use`, `mod` and `macro` as constructs with no production. There is no module
system to refine into, adding one is a larger proposal than any of RFC-8..14, and
no worked example exists in this corpus.

## 10. Costs

**The proof-layer surface is mostly promotion.** Every form in §5's second block
is already specified and parses. The work is moving the item set from forge-tier
gating to the language, with routing rather than grammar deciding which engine
discharges them.

**The `by` retirement costs the corpus nothing.** Across `conformance/` and
`examples/`, `by` appears in a clause position once, and it is already the
out-of-line spelling, in `conformance/forge/isqrt_class.th`. Every other
occurrence is prose in a comment. The inline form appears only in RFC-7 §6, so
the break is against a proposed surface rather than a shipped one.

**Labels are additive.** `Clause` gains a field beside `bv`; `ClauseSelector`
gains a variant; the parser takes an optional `Ident :` prefix in clause and
conjunct position; `validate_segments` admits identifier segments. One new check,
label uniqueness within the item. `Clause.text` is untouched, so every existing
address keeps resolving.

**The covenant changes are small and touch four goldens.** `inhabit` → `admits`
is 10 sites in 4 files, mechanical. `excludes` is one new directive over the same
evaluator. `witness for f` touches the witness production and `address.rs`'s
`witness#N` numbering. Removing `covenant_evidence` from `oracle_subset` is a
narrowing where R-SPEC-2 is additive-only: the 7 frozen v1 goldens carry
`covenant_evidence: None` and stay byte-identical, and the four covenant goldens
(`max_correct`, `max_buggy`, `max_no_witness`, `isqrt_class`) change. That belongs
in the change rather than surfacing as a red CI run, which is the discipline RFC-9
applied to its own certificate break.

**Two invariants are new checks.** The no-strengthening rule reads a proof item's
obligations against its target's contract; the no-naming rule is a
name-resolution restriction. Neither needs a solver, and target resolution (§7)
does not exist today at all — and, per §7, is filed as a defect against the
current baseline rather than costed here.

**`contract fn` is one reserved word and one branch.** `contract` joins
`keyword_kind`, and the `;`-body gate in the parser changes from
`boundary.is_some()` to admitting the `contract` modifier as well, which keeps it
a per-item check.

Reserving the word is the only part with corpus reach, and the reach is measured
rather than assumed: across the 65 `.th` files in `conformance/` and `examples/`,
the bare token `contract` occurs 11 times in 9 files, and **every occurrence is
inside a `//` comment** — prose describing a contract, never an identifier in
code position. Identifiers that merely contain the substring, such as
`bv_weak_contract`, lex as single identifiers and are unaffected. So the
migration cost of reserving it is zero at this baseline, which is the one thing
that would otherwise have to be weighed against `telos/the-corpus-still-certifies`.

**Nothing about certification changes.** Levels, verdicts, the ladder and the
oracle's verdict semantics are untouched. This document proposes where material
is written, not what is proved about it.

## 11. Open questions

Four of the five below were answered on 2026-08-11 and are recorded with their
answers. One remains open, and it acquired a live dependency while this document
was being written.

- **Does the bodyless-`fn` gate relax, or does the declaration carry a marker?**
  §9(a). The first costs per-item parse independence; the second adds a token.

  > **Resolved: the declaration carries a marker, and the marker is the keyword
  > `contract`.** Per-item decidability is the deciding property, and §9(a)
  > carries the full argument for the spelling.

- **Does `opaque` become the default for `spec fn`?** §1.1 makes `opaque` the
  spec-world instance of the same separation. If the layering is adopted, whether
  a sealed body is the default and `transparent` the modifier is a question this
  document raises and does not answer. RFC-7's asymmetry argument — assuming an
  opaque predicate is free, establishing one costs an unfold — is the input.

  > **Deferred, and a probe is owed before it is decided.** The asymmetry argues
  > for opaque-by-default, but flipping the default silently invalidates any
  > existing proof that relied on a transparent body, and under
  > `telos/the-corpus-still-certifies` that is a claim requiring a certification
  > run rather than an argument. No Verus run was made anywhere in this document.
  > Measure the corpus impact first. Note this is not blocking: nothing else here
  > depends on the answer, because `opaque` is written explicitly today either
  > way — and, per §9(a), is not reserved grammar yet in any case.

- **Does the no-strengthening rule need an escape?** A proof needing an auxiliary
  invariant the contract does not state has to put it somewhere. The answer is
  probably a `lemma`, which states its own `requires`/`ensures` and strengthens
  nothing. Whether that covers every case has no worked example here, because no
  verified subsystem in this corpus has yet needed one.

  > **Resolved: no escape. `lemma` covers it.** A `lemma` states its own
  > obligations and strengthens nothing, and no verified subsystem in this corpus
  > has yet needed more. Adding a sanctioned escape to an invariant before a case
  > demands it is how the invariant stops being checked, which is the failure
  > `telos/a-clause-is-checked` names. When a real case forces one, that case is
  > its justification and can be evaluated on its own terms.

- **Should a missing `excludes` witness ever become a gate?** §6.2 makes it
  advisory, on the strengthening-probe precedent and because a gate would fail
  the whole corpus.

  > **Resolved: advisory now, with the promotion condition written down rather
  > than left open.** It becomes a gate when the advisory has run on real code
  > and the corpus is clean under it — that is the condition, and meeting it is
  > what re-opens the question. An advisory with no stated path to becoming a
  > check tends to stay advisory permanently, which is the same failure the
  > previous item guards against from the other side.

- **Do proof-cache keys survive the split?** thermite-design.md §5.3 keys the
  per-item cache on an item's content. Once an item's contract, body and proof can
  sit in three files, the key has to compose from three places, and the layer
  boundary is what should make that composition clean rather than fragile.

  > **Still open — and it now has a live dependency.** The composition is not
  > worked out here and this document does not attempt it. What is recorded is
  > that the key is changing underneath the question: `fix/proof-cache-effect-row`
  > adds the declared effect row to the proof-cache key (REQ-1e) at the same
  > baseline this document measures. That change is independent and correct on its
  > own terms, but it sets precedent for how the key composes, and §9(a) would
  > later have the key drawing from two files rather than one. Whoever settles the
  > composition should read that change first rather than discover it afterwards.

---

## Appendix: what the probes established

Parser probes run against `thermite-syntax` at `staging @ b79b4005`, through a
temporary integration test since no parse-only binary exists. These measure the
parse and address layers; no Verus or Lean run was made, so every certification
statement in this document is cited from a REQ status table or a corpus header
rather than produced here.

| probe | verdict |
|---|---|
| `fn` + `witness { … }` + `proof for sum { ensures#1 by { … } }` in one ordinary file, no attribute, no feature flag | clean parse, 3 items; addresses `sum:Fn`, `witness#1:Forge`, `sum.proof.ensures#1:Forge` |
| `proof for no_such_function { ensures#1 by { … } }` | **clean parse.** The target is resolved by nothing; addressed as `no_such_function.proof.ensures#1` |
| `witness for sum { … }` | parse error — the proposed spelling does not exist |
| bodyless `fn` with no attribute | parse error, "a non-`#[boundary]` fn requires a `{ }` body" (`ffi-boundary.md` REQ-3, deliberate) |
| `#[boundary("core::sum")] fn sum(..) requires … ensures … ! … ;` | clean parse; `FnItem { body: None }` |
| `body for sum { … }` | parse error at item dispatch — the item form does not exist |
| `by` in a clause position across `conformance/` and `examples/` | one occurrence, already the out-of-line form (`isqrt_class.th`); all others are prose in comments |
| `inhabit` sites in the corpus | 10, across 4 `witness` blocks |
| `contract` and `opaque` as reserved words | neither appears in `keyword_kind` (`thermite-syntax/src/lexer.rs`). `contract fn` (§9(a)) is an addition rather than a re-use, and `opaque spec fn` is RFC-7 §6's proposed surface rather than shipped grammar |

Two of these decided sections rather than confirming them. The unresolved
`proof for` target is why §7 names target resolution as the first thing owed; the
absence of `body for` against the presence of everything it needs is what §9(a)
measures.
