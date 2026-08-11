---
rfc: 20
title: A template for RFC components, and the scope rule it carries
status: draft
supersedes: []
introduces: []
---

# RFC-20: A template for RFC components, and the scope rule it carries

| | |
|---|---|
| **Status** | Draft |
| **Baseline** | `staging @ b79b4005` — 18 RFCs in `.design/rfcs/` |
| **Position** | Process. Applies to every RFC written after it lands |
| **Depends on** | [RFC-5](0005-rfc-process.md), which made RFCs files with front matter |

## 0. What this asks

Adopt one section skeleton for RFC components, and one rule about what an RFC
is allowed to contain. The skeleton is convenience. The rule is the point.

## 1. The problem

[RFC-5](0005-rfc-process.md) settled that an RFC is a file with front matter,
and the gate enforces the fields. Below the front matter the documents diverge:
of eighteen, fifteen open with a Status table and three do not; eleven have a
section named some variant of *Proposal*, five open with *What this asks*, six
carry *Metatheory*, three name *Residual trust*. A reader who knows one RFC does
not thereby know where to look in another, and an author starts from a blank
file each time.

That is the mild half. The sharper half is a failure this repository has already
produced.

RFC-8 states a desired end state for the effect algebra. A `language-probe`
measured what the toolchain does today and found, among other things, that
effect-row subsumption compares labels and ignores the region — so `! write(db)`
over a `! write(log)` callee certifies. That measurement was then written *into*
RFC-8 as a correction: a section reporting the probe, a note on the conflict
table saying it was "not a reading of current behaviour", a paragraph explaining
that the frame rule could not be observed.

None of that belonged there. Every one of those findings describes the work
RFC-8 exists to propose. A design document that absorbs the current state stops
proposing anything: it argues with the tree it is trying to change, and the
proposal reads as an apology. Sixty-three lines went in and fifty-three came
back out.

The failure was not carelessness about tone. It was the absence of a stated rule
about what an RFC contains, which left the question to be answered per document
by whoever was writing.

## 2. Proposal

### 2.1 The scope rule

> **An RFC states a desired end state. Current behaviour enters only as evidence
> that a path exists, or as a finding that no path does.**

Three consequences, in the order they come up:

* A measurement showing today's toolchain lacks something the RFC proposes is
  **not** a correction to the RFC. It is the motivation for it, and it belongs
  in *The problem* if it belongs in the document at all.
* A measurement showing an end state is **unreachable** is the one kind that
  changes the proposal, and it changes it by narrowing or withdrawing the claim,
  not by annotating it.
* A measurement that merely refines what is true today belongs in the probe
  record, not in the RFC. `atom/language-probe` produces a `probe-verdict`; that
  artifact is the input to a design pass, not a rider on the design.

A claim about the present tense inside an RFC is a claim that will rot. Where one
is load-bearing — a baseline, a count, a named defect the RFC exists to fix — it
carries the commit it was measured at, as the Status table's **Baseline** row
already does.

### 2.2 The skeleton

Sections are omitted when they have nothing to say, and never reordered. Nothing
here is new: it is the shape the fifteen conforming RFCs already share, written
down.

```markdown
---
rfc: <n>
title: <sentence, not a slug>
status: draft | proposed | accepted | superseded
supersedes: []
introduces: []          # REQ ids; each must exist in .design/reqs/registry.toml
---

# RFC-<n>: <title>

| | |
|---|---|
| **Status** | <state, and what it waits on> |
| **Baseline** | `<repo> @ <sha>` — the tree every present-tense claim is measured against |
| **Position** | <where this sits in a sequence, if it is in one> |
| **Depends on** | <RFC links, or "—"> |

## 0. What this asks
One paragraph. A maintainer who reads only this knows what they are agreeing to.

## 1. The problem
Why the end state is worth reaching. Present-tense evidence lives HERE, dated by
the Baseline row.

## 2. Proposal
The end state. Written in the indicative, not the conditional.

## 3. Metatheory
What the proposal costs the proof theory. "None new" is a real answer and is
worth stating when true.

## 4. Residual trust
What this does NOT discharge, per `telos/residual-trust-is-named`. Omitting this
section is a claim that nothing is left over, which is rarely true.

## 5. Sequence
The order the parts land in, and which are irreversible.

## 6. What is deferred, and to where
Named, so a reader does not mistake an omission for an oversight.
```

### 2.3 Where the template lives

Not in `.design/rfcs/`. `gates/rfc-check.py` globs `*.md` in that directory and
requires valid front matter and a numbered filename of every file it finds, so a
`TEMPLATE.md` there is a gate failure. It goes to `.design/templates/rfc.md`,
which no gate scans, and this RFC's §2.2 is its content until it moves.

## 3. Metatheory

None new. This changes what documents look like, not what the language proves.

## 4. Residual trust

* **The skeleton is convention, not a gate.** `rfc-check.py` validates front
  matter; it does not read section headings, and this RFC does not propose that
  it start. A document can conform to the letter and still bury its proposal.
* **The scope rule is a judgement, and judgement drifts.** "Evidence that a path
  exists" and "an annotation on the design" are distinguishable in the clear
  cases and arguable at the margin. The rule narrows the argument; it does not
  end it.
* **The eighteen existing RFCs are not migrated.** They are the record of what
  was proposed and when. Bringing them to the skeleton would rewrite that record
  for tidiness, which [RFC-5](0005-rfc-process.md) already declined to do for
  front matter.

## 5. Sequence

1. This RFC lands.
2. `.design/templates/rfc.md` is added, carrying §2.2 verbatim.
3. RFCs written after that start from it. Existing RFCs are untouched.

## 6. What is deferred, and to where

**A heading gate.** A checker that asserts section presence and order is
buildable and is not proposed. The skeleton earns its keep by being easier than
a blank file; if it needs enforcement to survive, the more interesting finding is
that authors disagree with it, and that is worth hearing before it is compelled.

**A template for design docs under `.design/<area>/`.** They have their own
shape — REQ tables, ACs, content pins — and a different audience. Out of scope
here.
