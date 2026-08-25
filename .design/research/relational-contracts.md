# Relational contracts

<!--
tier: research
status: design record, research trajectory. Not proposed, not sequenced.
purpose: record the direction behind rungs 8-11 whole, so it can be examined and
         rejected now rather than one clause at a time. Cited by
         .design/syntax/effect-algebra.md for the placement of `random`,
         `blocks` and `accrues(M)`, and for the footprint projection REQ-12
         supplies to §5.1's relational frame lemma.
governs: (no code — a research record; no routes, no pin over source)
thesis-refs:
  - thermite-design.md §1
  - thermite-design.md §4.1
  - thermite-design.md §7
-->

**Status: design record, research trajectory. Not proposed, not sequenced.**
Nothing in this document is filed upstream. Most syntax shown here does not
exist — `forge check` will reject it. It is written the way the sequenced
documents are written: so the direction can be examined whole, and rejected now
rather than one clause at a time.

**Kind:** a family of clauses (`hides`, `varies`, `distributes`, `couples`,
`matches`) plus one language-level theorem (the relational frame lemma), one
assurance-vocabulary extension (hyper-arity in the refutation coordinate), and
one characterization of the certificate algebra (rung 10). Everything rests on
machinery the language already has: the effect basis, the admissibility
criterion, mutation scoring, and the assurance tuple.

**Position on the ladder:** this document is the design record behind rungs 8,
9, 10 and 11, and it explains why they are one piece of work seen from four
angles rather than four pieces of work.

---

## 1. Background: what a relational contract is

Every clause the language has today is a **trace property**: a predicate on one
execution. `requires P` constrains the state a run starts from; `ensures Q`
constrains the state it reaches; the row constrains what a run touches. A run
satisfies the clause or it does not, one run at a time.

Some of the properties the ladder wants cannot be stated that way.
*Noninterference* — "the low-observable behaviour does not depend on the
secret" — is not a fact about any single run. No trace violates it; a **pair**
of traces violates it: two runs agreeing on everything public, disagreeing on a
secret, producing distinguishable output. *Determinism*, *constant-time*,
*sensitivity bounds*, and *refinement* have the same shape. In the literature
these are **hyperproperties** (Clarkson–Schneider 2010): predicates on *sets*
of traces rather than on traces.

The vocabulary this document uses:

| shape | quantifier form | example | witness of violation |
| --- | --- | --- | --- |
| trace | one run | `ensures Q` | one run |
| **k-safety** | all k-tuples of runs | noninterference, determinism, constant-time, Lipschitz bounds | a concrete k-tuple |
| **forall-exists** | every run has a matching run | generalized noninterference, refinement, deniability | none finite — see §7 |

The quantifier shape is not a stylistic fact. It determines, mathematically,
what forms of evidence and refutation can exist for a clause, which is why it
must eventually appear in the assurance tuple (§7).

## 2. Background: the walls

The metatheory of hyperproperty verification contains severe impossibility
results, and this document is written inside them rather than against them.
Stated once, so every later scoping decision has its reason on record:

| result | consequence for this document |
| --- | --- |
| k-safety reduces to ordinary safety of the k-fold self-composition | everything in Tier A and most of Tier B is SMT-shaped work, not new logic |
| model checking cost grows one exponential per quantifier alternation (provably) | forall-exists clauses are never checked by search; they carry witnesses |
| HyperLTL satisfiability is Σ¹₁-complete — beyond the arithmetical hierarchy; no sound and complete proof system exists | there is no complete method to retreat to; soundness plus compositionality is the ceiling, so the design optimizes for those |
| synthesis is undecidable already at two universal trace quantifiers | "derive the implementation from the relational contract" is not on any roadmap |
| forall-exists clauses are not refutable from finitely many observed runs | the `empirical` refutation tier is structurally unavailable to them — a vocabulary rule, §7 |

> The walls do not say relational contracts are hopeless. They say the natural
> notions of "complete" and "decidable" for this structure are not the
> classical ones. The sound methods that remain are compositional, and
> compositionality is the property this language already optimizes for.

One further fact sets the tone. The **subset-closed** hyperproperties — those
preserved by shrinking the trace set, which include all k-safety — are exactly
the ones ordinary overapproximation verifies soundly (Mastroeni–Pasqua). A
kernel's interesting properties live almost entirely in that fragment. The
project is standing in the luckiest part of the impossibility landscape, and
the design should exploit that rather than merely note it.

## 3. The organizing principle

Every construction in this document is one maneuver, repeated:

> **Find the relational lifting whose laws are the composition you need; check
> that its side conditions are first-order; prove its soundness once in Lean;
> let the solver discharge instances.**

This is the existing admissibility criterion — *"a primitive effect is
admissible when it generates a frame condition expressible in the prover's
logic"* — applied one level up. A frame condition is a fact about one run. Its
**relational lifting** is the corresponding fact about a pair of runs: "two
runs agreeing here continue to agree there." The lifted criterion:

> **A relational clause is admissible when it generates coupling
> side-conditions expressible in the prover's logic.**

A *coupling* is an alignment of two (or k) runs under which the relational
claim decomposes into per-step, first-order obligations. For deterministic
state the coupling is lockstep execution of the self-composition and the
side-conditions are equalities the frame conditions generate mechanically. For
probabilistic effects the coupling is a measure-theoretic object whose
soundness is proved once, offline, and whose per-instance residue is a
bijection check (§6.2). In every case the division of labour is the one the
assurance model already names: the solver checks first-order instances
(`solver`), Lean licenses the route (`lean-lemma`) or re-checks the theorem
(`lean-checked`).

The maneuver has a precedent inside the language. The conflict table is not
stipulated; it is computed from the state equations, per operation pair.
Everything below extends that provenance discipline upward: the security
theorem, the reproducibility theorem, and the certificate algebra should each
be *derived from declared structure* by machinery proved sound once — because
in a language written principally by agents, a rule with a derivation can be
regenerated and checked, while a rule stipulated by taste can only be trusted.

---

## 4. Surface conventions for relational clauses

Following the anchor's rules — every clause a third-person-singular verb, full
words, the subject elided — with one addition for naming the paired run.

**`other(x)`** denotes the value of `x` in the paired run, mirroring how
`final(s)` names the paired *state* in a two-state relation. A clause
mentioning `other` is thereby a two-run clause, the way a clause inside
`interleaves` is thereby a concurrent one: the vocabulary marks the arity, so
no separate annotation is needed and misuse is a parse error rather than a
convention.

The clause family, in one table:

| clause | reads as | arity | shape |
| --- | --- | --- | --- |
| `hides R` | *(this item) hides R* | 2 | k-safety |
| `hides R in cost` | *hides R in its cost* | 2 | k-safety, graded |
| `varies result by f in x` | *varies its result by f in x* | 2 | quantitative k-safety |
| `distributes D` | *distributes D* | 1 over distributions | distributional |
| `couples other by w` | *couples the other run by w* | 2 | witness, not a claim |
| `matches some other { … }` | *matches some other run* | forall-exists | see §8.1 |

`couples` is deliberately in the family though it asserts nothing: it names
the witness that discharges its siblings, the way a proof block names a proof.
A relational clause without an admissible discharge route is exactly the
undischarged `asks` problem — a free assumption that makes a proof easier and
is checked against nothing — and the tuple must say so (§7).

---

## 5. Tier A — settled metatheory

The soundness arguments for everything in this tier exist in the literature.
The work is engineering: one Lean development, emitter work, no research risk.

### 5.1 The relational frame lemma

The single theorem underneath the tier, stated as language metatheory and
proved once:

> **If `f ! ρ`, then any two runs of `f` from states agreeing on the footprint
> of ρ produce equal results and reach states agreeing everywhere outside ρ's
> write-footprint.**

This is the relational reading of the row. It is proved by self-composition —
the product of the program with itself, a construction the SMT pipeline
already understands, with coupling invariants generated from the frame
conditions. (Benton 2004; Barthe–D'Argenio–Rezk 2004. Settled.)

What it mints, with no new surface syntax and no per-function proof:

```
fn hash(x: Bytes) -> Digest ! pure
// derived: determinism — hash(x) equal across any two runs

fn step(w: World) -> World ! state(entropy)
// derived: reproducibility — equal seeds replay the run
```

The row already distinguishes `state(entropy)` from `random`, and the design
record observes that the row "would say whether a program is reproducible."
The lemma is what upgrades that observation to a theorem, per artifact, read
off the source. Bulla's determinism apparatus consumes it directly, and §9.5
composes it further.

Assurance: derived facts carry the row's own tuple. Nothing weakens, because
nothing was added — the fact was always in the row; the lemma makes it legible.

### 5.2 Noninterference for static policy: `hides`

Grade regions with a security lattice; one new clause:

```
region audit_log : low
region key_store : high

fn respond(q: Query) -> Reply
  ! read(key_store), write(audit_log)
  requires wellformed(q)
  ensures  valid(result)
  hides    key_store
```

Meaning: any two runs agreeing on everything except `key_store` produce equal
results and equal low-region writes. Discharge is the self-composition again;
the coupling invariant — low projections equal — is generated from the frame
conditions, which is the concrete payoff of the admissibility criterion: **the
frames are the coupling invariants.** The product program's shared state is
cut to the declared footprints, which is what keeps the verification
conditions from doubling in the obvious bad way.

This is 2-safety, subset-closed, inside the good fragment. seL4's information
flow proof is the existence proof at kernel scale for the static case; the
novelty here is only that the proof is per-clause and compositional rather
than whole-system.

Tuple: `all / complete(2) / solver @ to_boundary` — where `complete(2)` is
hyper-arity refutation (§7): a countermodel is a concrete *pair* of runs,
which the solver can actually produce and a reader can actually inspect.

### 5.3 Constant-time from the cost grading

Because cost is already an effect over a monoid (`accrues`), secret-
independence of cost is `hides` aimed at the accumulator:

```
fn compare(mac: Tag, expected: Tag) -> bool
  ! read(key_store), accrues(cost)
  hides key_store in cost
```

Any two runs differing only in `key_store` accrue equal cost. This is the
constant-time property; the methodology is deployed industrially (the
Barthe et al. constant-time line; Jasmin/EasyCrypt). It is the strongest
shippable piece of rung 9 — not worst-case execution time, but *timing
reveals nothing* — and it is available years before WCET is.

The honest cost: a leakage model per backend, stating which cost the
accumulator models (instruction counts, memory-access traces, both). That
model is a platform assumption and lives in the frozen registry, so the tuple
closes `@ to_platform(p)` with `p` naming the model. A constant-time claim
without its leakage model named is unreadable rather than merely weaker — the
same sentence the assurance model already says about boundaries, because it
is the same situation.

---

## 6. Tier B — attainable research

Each item has a clear path and a worked precedent in an adjacent tool; none
has shipped in this form. Risk is integration-shaped, not idea-shaped.

### 6.1 Noninterference derived from the commutation table

The conflict table is computed per operation pair from the state equations —
a theorem, not an axiom. The security theorem admits the same derivation, one
relational level up. Grade the *operations* of a theory by the lattice, and
compute per-pair independence:

```
effect state(r) at level(r)

// derived, per operation pair, from equations plus lattice:
//   low get ∥ high put  →  independent    (the noninterference row)
//   low get ∥ low  put  →  dependent      (a flow, as it should be)
```

An item's `hides` then discharges compositionally from the table rather than
by a monolithic relational proof per function. The categorical content
(security level as a modality on the theory; the theorem falling out of the
graded structure — the Kavvos/DCC line) compiles down to the derivation
engine the language already runs for data races.

Why it is research: nobody has an effect-row language where the security
theorem has the same *provenance* as the data-race-freedom theorem. That
provenance claim — one derivation engine, two theorems — is the publishable
core, and it is also the maintainable core: an agent regenerating the table
regenerates both.

### 6.2 Distributional discharge by coupling: filling `random(D)`

The parameter was reserved so that gaining distributional proofs would not be
a breaking change. What fills it is a change of discharge strategy, not a
solver upgrade:

> **Distributions are never pushed through the solver. Couplings are.**

A coupling proof (the pRHL/EasyCrypt discipline) discharges a distributional
claim by aligning two runs through a bijection on the sample space; the
measure theory lives in the once-proved soundness lemma, and the per-instance
residue is first-order. The one-time pad, in full:

```
fn otp(m: Bits<n>) -> Bits<n>
  ! random(uniform(Bits<n>))
  distributes uniform(Bits<n>)
  hides m
  couples other by (k -> k xor m xor other(m))
```

Solver obligations: the alignment is a bijection (xor involution — trivially
first-order), and under it the outputs are equal. Lean obligation, once:
*bijective coupling of uniform samples implies equal output distributions* —
a `lean-lemma` licensing the route, with the solver remaining trusted per
query. This is precisely the existing trust vocabulary; no new tier is
invented, which is evidence the vocabulary was drawn correctly.

The design record's own table already contains the bookkeeping fact this
rests on: `random ∥ random` accepts "because independent samples commute
(Fubini)" — commutativity of the distribution monad. `couples` is the
relational lifting of the same structure. Rung 11's discharge is attainable
without the solver ever learning measure theory, which was the stated blocker.

### 6.3 Sensitivity: `varies`

Quantitative two-run bounds over declared metrics:

```
fn rebalance(load: Map<Cpu, nat>) -> Plan
  ! pure
  varies result by lipschitz(2) in load
```

Runs whose inputs differ by δ produce outputs within 2δ — robustness as a
contract. Metatheory: the Fuzz/RelCost lineage (graded relational liftings
over a metric); discharge is a coupling with first-order residue. Aimed at
the cost accumulator instead of the result, the same clause states *relative
cost* — "the patched path is within ε of baseline" — which is the contract
rung 6 wants when it starts trading precision for speed, stated in advance
of needing it.

### 6.4 Why the agent workload wants this tier specifically

A per-trace contract has a structural ceiling against an optimizer: it
constrains each run in isolation, so a body correct *on the runs the contract
can see* passes. Mutation scoring fights this per contract; relational
clauses remove the ceiling per *shape*:

| relational clause | what it forbids an optimizer from doing |
| --- | --- |
| `hides secret` | special-casing observed inputs — the violating pair is the case not special-cased |
| `varies … lipschitz(k)` | hiding a cliff between test points |
| determinism from rows | smuggling state through an undeclared channel |
| `matches some other` | behaviour explainable only by having peeked |

A hyperproperty constrains the shape of the function, not the trajectory of a
run, so it is closed under adversarial pressure from the code's author. In a
language written principally by agents, that is not an ornament; it is the
contract discipline matched to the threat model: trust the checker, not the
writer. Tier A's frame lemma plays for the code author the role parametricity
plays for the API client — *cannot tell, therefore cannot depend.*

---

## 7. The assurance model gains hyper-arity

One vocabulary extension, filable early because it commits to no mechanism:

```
refutation ::= complete(k) | incomplete(k) | empirical
             | traces(k, n) | witness | abort | none
```

`complete(2)` for `hides`: a countermodel is a pair of runs. `traces(2, n)`
for a bounded relational check. And one hard rule imported from the
impossibility results rather than from taste:

> **A forall-exists clause never carries `empirical`.** No finite set of
> observed runs refutes it, so a seeded generator attacking it is theatre.
> Its honest refutation value is `witness`: the certificate contains the
> coupling, strategy, or prophecy discharging the existential, and trust
> attaches to checking the witness, not to searching for it.

This is the existing rule — a prose summary that cannot be expanded into
tuples is not a claim — applied to quantifier shape, where the inexpandable
summary is forbidden by mathematics rather than by editorial policy. The
scoring note from the interference-clauses record generalizes with it: an
`asks` with no composition site, an undischarged `couples`, and a `matches`
with no witness are the same defect at three arities — a free assumption
reported at full assurance — and the tuple is where all three become visible.

---

## 8. Tier C — frontier

Genuinely open. Recorded so the surface can reserve space the way `random`
reserved its parameter: gaining these later must not be a breaking change.

### 8.1 Forall-exists contracts: `matches`

Generalized noninterference, refinement, and deniability share the shape
*every run has a matching run such that…*:

```
fn schedule(rq: RunQueue) -> Pick
  ! read(key_store), read(rq)
  matches some other {
    given  other(key_store) != key_store;
    holds  low(other(result)) == low(result);
  }
```

"Whatever was observed is consistent with a different secret." The witness is
a *function from runs to runs*, and sound methods are incomplete without
**prophecy**: the Coenen et al. line shows prophecy variables restore
completeness for ω-regular witness classes — Abadi–Lamport's refinement-
mapping completeness theorem resurfacing as the fundamental theorem of
forall-exists verification.

The language has already touched this boundary without naming it. The
interference-clauses lowering found one conjunct that does not map to
persistent sharding: stability — the epoch, constant per round, changing
between rounds. That is the first symptom of a gradient this document makes
explicit: **monotone facts need only history; per-round facts need ghost
instances indexed by round; full forall-exists needs prophecy.** A `matches`
clause whose certificate carries an explicit checked witness is sound and
shippable at any point; a complete method is the research program.

### 8.2 Dynamic-authority noninterference

The open half of rung 8. In this document's terms: the lattice stops being
global and becomes protocol state, so `hides` must be indexed by an epoch of
the authority relation — *the epoch problem again, one level up.* The
stability conjunct that would not lower and the declassification problem in
information flow are the same obstruction: a fact constant-per-round in a
system whose rounds are program-controlled. A ghost-instance-per-round
mechanism built for rely-guarantee is therefore expected to carry
epoch-indexed `hides` as well. If it does, that is a genuinely novel result —
compositional dynamic-policy noninterference in a deployed verifier — and it
was reached by solving a concurrency bookkeeping problem, which is the kind
of accident worth engineering for.

### 8.3 The certificate algebra, characterized (rung 10)

The aggregation notes already state the structure without the words:
refutation *fibered over* scope; trust splitting into residual risk that
*grows under composition* and discharged evidence that carries no liability.
Named: certificates are graded by an ordered monoid; aggregation is a lax
monoidal functor; and because the grading varies with scope, the full object
is a **fibration of graded monoids** over the scope order. The worked
precedent is differential privacy, whose (ε, δ) budget is exactly such a
grading and whose composition theorems are exactly the graded laws — proof
that the algebra can be made to work across code written by parties who
never met.

The current policy — per-clause tuples, weakest link per axis, no composite —
is the discrete grading, and it is *sound*. The open question becomes
well-posed: **what is the finest lax refinement of the per-axis meet that
remains sound?** The definitions are a paper; the right monoid for the trust
axis is a research question. One further piece falls out: the boundary
coordinate composes by a refinement order — claim A at boundary b refines
claim A′ at b′ iff it factors through the added assumptions — which is
Blackwell-style post-processing, and a precongruence, which is the property
aggregation needs from it.

### 8.4 `random` with a hardness parameter

The honesty constraint on the atom already notes that unpredictability
"needs… a hardness notion." The frontier form: `random(prg(f))`, discharged
by a constructive *reduction* — an adversary violating the clause breaks `f`
— with the hardness assumption living in the frozen registry beside the MMIO
model, where the boundary coordinate already knows how to name an assumption
about the world. CertiCrypt/EasyCrypt show reductions are formalizable;
composing them with a platform registry is unexplored. Five-to-ten-year
frontier; surface cost of readiness, zero, because the parameter position
already exists.

---

## 9. What the composition buys

The tiers are multiplicative, not additive. Five products, each currently
nonexistent anywhere.

### 9.1 Isolation meaning all channels, on real hardware

Rungs 3–8 plus §5.2 and §5.3 compose into: two tenants, multiple CPUs,
shared lock-free kernel structures, where *tenant A learns nothing about
tenant B* covers functional results, scheduling observations, **and accrued
cost**, with the leakage model named in the registry. No shipped system
carries that conjunction — seL4's information-flow proof is static-policy,
unicore, timing outside the model. The composition works because every
property discharges through the *same* relational frame lemma: the rows
shrink the product state; data-race freedom makes the verified interleaving
semantics the one the silicon executes; the cost grading rides the same
coupling as the functional claim. One lifting, four instantiations, glued by
§8.3.

### 9.2 Crypto and systems verification in one certificate chain

Today the cipher's proof lives in one tool-culture (couplings, reductions)
and the kernel's in another (invariants, separation), and the interface is
prose. §§5.3, 6.2, 8.4 in one language dissolve the wall: the distributional
contract, the constant-time contract, and the kernel's ordinary use of the
cipher are clauses in one certificate algebra, and the chain runs from "the
platform's cost model is the leakage model" and "this function is a PRF" to
"the observable behaviour of the box reveals nothing about keys" — as one
machine-checkable tuple chain with every assumption named. The pieces exist
in separate tools; the composition algebra is the missing artifact, and it
is rung 10.

### 9.3 Contracts closed under adversarial optimization

§6.4, restated as the design's telos: a specification surface where
delegating to an untrusted optimizer is safe, because the contracts quantify
over sets of runs and therefore cannot be satisfied by overfitting to the
runs a checker sees. For a language whose authors are models, hyperproperty
contracts are the type system for trusting the checker and not the writer.

### 9.4 Assurance as a build artifact

§8.3 plus §7 make certificates compose across organizational boundaries the
way types compose across modules: a capsule vendor ships per-clause tuples;
the build composes them through the algebra; the artifact carries a
*computed* tuple whose residual risk grew by the algebra's laws, weakest
boundary named, recomputed on rebuild, diffable. Regulated industries
assemble this today as prose and diagrams, by hand, re-audited from scratch
per change — assurance cases that cannot be expanded into tuples. A safety
case that recomputes on rebuild is the artifact certification regimes have
wanted for decades without an algebra to define it. Second-order effect:
certificates become tradeable, because composition is mechanical — market
infrastructure, disguised as a lax monoidal functor.

### 9.5 Deterministic replay as a theorem

§5.1 composed with data-race freedom and the crash clause: a program whose
row proves reproducibility, on a scheduler whose interleavings are proven
irrelevant, with crash states covered, is a system where record-replay is a
*theorem* — replaying inputs replays the execution, across a crash boundary,
on multiple cores. Certified time-travel debugging of a kernel; and the
soundness precondition for every seeded-generator tier the assurance model
runs, since attacking executable semantics empirically presupposes that
seeded means something.

---

## 10. Sequencing

By the anchor's own philosophy — file what is cheap and reversible; show the
rest so the direction can be rejected early:

| order | item | why it goes here |
| --- | --- | --- |
| 1 | §7 hyper-arity | a page of vocabulary; commits to no mechanism; future-proofs every later tuple |
| 2 | §5.1 frame lemma | one Lean development; everything else instantiates it; pays for itself via §9.5 immediately |
| 3 | §5.2 + §5.3 `hides` | makes rung 8 scheduled rather than aspirational; static case only |
| 4 | §6.1 derived NI | after the effect-rows RFC lands, since it extends the same table |
| 5 | §6.2 couplings | the item that makes the language cited outside kernel work |
| 6 | §6.3 `varies` | with rung 6, which is when relative-cost contracts have call sites |
| — | §8.x | recorded, unfiled; surface reservations only |

The dependency to name, matching the interference-clauses record: `other()`
needs the same treatment `final()` needs — defined for the parameter kinds
the clauses quantify over. A small extension to existing vocabulary rather
than a new concept, but a change, and it should be named.

## 11. The rule, restated

The walls from §2 are load-bearing in this design, not decoration. No
completeness for forall-exists — so `matches` carries witnesses and the tuple
says `witness`, never `empirical`. No absolute claims — every chain
terminates in a registry, and the boundary coordinate names it. No full
automation, ever — Σ¹₁ says so — which here manifests as: the agent proposes,
the coupling is the creative act, the checker disposes.

> A claim that cannot be expanded into its assumptions is not a claim. The
> relational metatheory is what lets the interesting ones be expanded.

## References

- Clarkson, Schneider. *Hyperproperties.* JCS 2010.
- Alpern, Schneider. *Defining liveness.* IPL 1985.
- Finkbeiner, Rabe, Sánchez. *Algorithms for model checking HyperLTL and HyperCTL\*.* CAV 2015.
- Fortin, Kuijer, Totzke, Zimmermann. *HyperLTL satisfiability is Σ¹₁-complete, HyperCTL\* satisfiability is Σ²₁-complete.* MFCS 2021.
- Mastroeni, Pasqua. *Verifying bounded subset-closed hyperproperties.* SAS 2018 (and the hyper abstract interpretation line).
- Benton. *Simple relational correctness proofs for static analyses and program transformations.* POPL 2004.
- Barthe, D'Argenio, Rezk. *Secure information flow by self-composition.* CSFW 2004.
- Murray et al. *seL4: from general purpose to a proof of information flow enforcement.* S&P 2013.
- Barthe, Grégoire, Zanella Béguelin. *Formal certification of code-based cryptographic proofs.* POPL 2009 (CertiCrypt; EasyCrypt line for pRHL).
- Barthe et al. *System-level non-interference for constant-time cryptography.* CCS 2014; Almeida et al., *Jasmin.* CCS 2017.
- Reed, Pierce. *Distance makes the types grow stronger.* ICFP 2010 (Fuzz); Çiçek et al., *Relational cost analysis.* POPL 2017 (RelCost).
- Barthe, Katsumata, Sato et al. — graded relational liftings for differential privacy (the span-lifting line, e.g. LICS 2019).
- Abadi, Lamport. *The existence of refinement mappings.* TCS 1991.
- Coenen, Finkbeiner, Sánchez, Tentrup. *Verifying hyperliveness.* CAV 2019 (prophecy for forall-exists).
- Jones. *Tentative steps toward a development method for interfering programs.* TOPLAS 1983; Vafeiadis, Parkinson, *RGSep.* CONCUR 2007; Jung et al., *Iris.* JFP 2018.
- Plotkin, Power. *Algebraic operations and generic effects.* ACS 2003.
- Kavvos. *Modalities, cohesion, and information flow.* POPL 2019; Abadi et al., *A core calculus of dependency.* POPL 1999.
- Dardinier, Müller. *Hyper Hoare Logic.* PLDI 2024.
- McIver, Morgan et al. *The Science of Quantitative Information Flow.* Springer 2020 (hyperdistributions; Blackwell refinement).
- Hermida, Jacobs. *Structural induction and coinduction in a fibrational setting.* Inf. Comput. 1998; Katsumata, *Parametric effect monads and semantics of effect systems.* POPL 2014.
