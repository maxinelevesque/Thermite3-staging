---
rfc: 16
title: What a pin should pin — narrowing doc-drift from bytes to claims
status: draft
supersedes: []
introduces: []
---

# RFC-16: What a pin should pin — narrowing doc-drift from bytes to claims

| | |
|---|---|
| **Status** | Draft, unfiled |
| **Supersedes** | — |
| **Baseline** | `dollspace-gay/Thermite @ 84d276e7` |
| **Scope** | `tooling/doc-drift.py` and the route table, not the language |
| **Relation** | Generalizes RFC-15 §3.3, which narrows one route as a special case |

`doc-drift` asks a good question — *might this document now be false?* — and
answers a different one: *did the governed bytes move?* This proposes closing
that gap, and argues from a measured false-positive rate rather than from
irritation.

## 0. What this asks

Three changes, in increasing order of ambition and decreasing order of
confidence.

1. A route may pin a **region or an extraction** of a governed file, not only
   the whole file.
2. Where a document's claim is executable, it is checked rather than pinned.
3. Where a document quotes the surface, it transcludes rather than restates.

## 1. The gate measures the wrong thing

`_digest` is `sha256` over `read_bytes()` of each governed file, aggregated
across the route's whole-file globs. There is no way to express "digest only the
part I depend on." So the pin fires on any change to any governed file:
reformatting, a comment, an unrelated function, a third-party tool appending its
own configuration.

The document's claims are not consulted, because the tool has no representation
of them. It has a hash.

## 2. Evidence: forty-one findings in one day

A single working day on this fork produced **41 drift findings across 4 root
events**. Classified by whether the document's claims were actually falsified:

| | findings | |
|---|---|---|
| **Confirmed true** | 28 | anchor branch; documents describing `req`/`ens`/`fx` after RFC-6 deleted that surface |
| **Confirmed false** | 13 | claims verified still true |

An earlier revision of this document left 7 of those 13 **undetermined**, because
resolving them meant reading each one, and disclosed that §3 would weaken if a
meaningful share turned out to be true. They have since been read. All 7 are
false positives, and the test was narrower than reading prose: for each, ask what
the anchor actually *changed* in the files that document governs. Every changed
line in `verified_build.rs`, `composition.rs`, `cli.rs`, `closure.rs` and
`thermite-syntax/src/lib.rs` is the RFC-6 rename — including the ones that do not
look like it, which are clause names built as data (`format!("{}.ens#{}")` →
`.ensures#`) and one module doc comment. No behaviour moved, and none of the
seven documents quotes an address segment, so none was falsified.

The gate is not useless — 28 real findings in one firing is a good day for any
tripwire, and this proposal does not touch that path. The problem is the 6.

### The confirmed-false six, and why they are not bad luck

**Four came from a formatting change.** A commit deleted a comment orphaned by an
earlier arm removal and let `rustfmt` re-wrap an export list. That moved the
digests of `09-option-result.md`, `11-ergonomics.md`, `vacuity-triage.md` and
`workspace.md`. The export list holds the same 46 names before and after,
set-identical; no claim in any of the four documents was affected.

**Two came from a second tool's configuration.** `.claude/settings.json` gained
sixteen purely additive lines wiring a second agent harness. That moved
`control-plane.md`'s digest — and `control-plane-check.py`, which checks the
three wirings the document actually audits, **exits 0 on both sides of the
move**. The same finding then recurred through a pull request's merge commit on
a branch that touched no harness file at all.

That second pair is the sharpest evidence available: on the same file, at the
same commit, **the executable check said the claim was true and the hash said
DRIFT.** The check was right.

## 3. The false positives are structural, which is why this is fixable

The thirteen do not scatter. They come from three shapes, which are one root
cause wearing three faces — **the pin digests more than the document depends
on**:

* **(i) A whole-file pin over a file that accumulates third-party content.**
  Configuration files grow entries that belong to other tools. Any pin over the
  whole file conflates "the audited thing changed" with "somebody else's thing
  is also installed."
* **(ii) A whole-file pin over source, where formatting and comments move bytes
  without moving meaning.** Rust source is re-wrapped by `rustfmt` as a matter of
  course; comments are edited constantly.
* **(iii) A whole-file pin over source touched by a semantics-preserving global
  rename.** RFC-6 renamed the clause surface; six documents about verified
  builds, the CLI and closure analysis drifted without any of their claims
  moving, because the rename passed through files they govern.

Both are properties of the *pin's shape*, not of the change that tripped it. And
neither shape overlaps with the 28 true findings, which came from a semantic
change — a rename that genuinely invalidated prose. **The false-positive sources
can be removed without touching the path that produced every true finding.**
That is the whole argument.

## 4. Why the response is worse than the interruption

The response to a false positive is a re-pin with a prose note. That is
self-certification, and it is the only affordable response — re-auditing a
document to clear a finding caused by `rustfmt` is not a good use of anyone's
attention.

Recorded honestly, from the day that produced the evidence above: all six false
positives were cleared by re-pinning, and in none of them was the governed
document re-read to confirm its claims still held. The reasoning was from the
diff — "comments and line breaks only" — which is sound, and is also precisely
the ritual.

This is the real cost. A pin-everything design maximizes *nominal* recall. Its
*effective* recall is the fraction of re-pins where someone genuinely re-audits,
and a false-positive rate high enough to make the response reflexive drives that
fraction toward zero. **A gate whose response has become a ritual has already
lost the recall it was built to guarantee.**

## 5. Proposal

### 5.1 Pin a region or an extraction

A route may name what it depends on:

```toml
[[route]]
crate_pattern = ".claude/settings.json"
design        = ".design/tooling/control-plane.md"
pin_extract   = "hooks"     # digest the extracted hook entries, not the file
```

```rust
// doc:begin(.design/syntax/parser.md#REQ-2)
fn parse_contract(...) { ... }
// doc:end
```

An absent `pin_extract` and absent anchors mean the whole file, so every existing
route keeps its current behaviour and this ships without a flag day. Shape (i) is
answered by extraction, shape (ii) by anchors — a re-wrap outside the anchored
region stops mattering.

### 5.2 Prefer a check to a pin

Where a document's claim can be executed, it should be. This project already
does this in four places — `control-plane-check.py`, the req-registry, the
conformance corpus, and the golden files — and the `control-plane` case above is
a direct demonstration that the check outperforms the pin on the same claim.

A content pin is what you reach for when the claim *cannot* be expressed
executably. It should not be the default for claims that can. Each pin converted
to a check removes a false-positive source permanently rather than tuning it.

### 5.3 Transclude the surface rather than restating it

The 28 true findings are real, but they are also structurally predictable: a
surface rename invalidates every document quoting the surface, simultaneously
and forever. Ninety of 145 tracked `.md` files in this fork spell the pre-RFC-6
clause keywords.

Documents should transclude surface examples from the conformance corpus rather
than restate them, so a rename regenerates rather than invalidates. The machinery
exists — `THERMITE.skill.md` is generated and budget-checked in CI.

This is the least certain of the three. It changes how documents are written, not
merely how they are checked, and a transcluded example is harder to read in
context than an inline one.

## 6. Residual trust

* **Narrowing trades recall for precision.** A change outside a pinned region can
  still falsify a document, and this proposal would miss what today's design
  catches. §4 is the argument that the trade is favourable; it is not a proof,
  and someone who re-audits diligently on every finding is strictly worse off
  under this RFC.
* **An extraction is code.** `pin_extract = "hooks"` means a function that pulls
  the hook entries out, and that function can be wrong in a way a whole-file hash
  cannot. It needs its own oracle tests, in the fixture convention `doc-drift`
  and `control-plane-check` already share.
* **Anchors rot.** A `doc:begin` comment can be moved, deleted, or wrapped around
  the wrong thing, and nothing proposed here detects an anchor that no longer
  surrounds what the document discusses.
* **The false-positive rate is measured on one fork over one day.** 41 findings
  is enough to show the shapes are real and not enough to size them. A project
  whose docs are pinned more narrowly, or which reformats less, would see a
  different ratio — and the 28 true findings all came from a single event, a
  surface rename, which is not a routine occurrence.

## 7. Sequence

1. `pin_extract` and `doc:begin`/`doc:end`, with oracle tests, defaulting to
   current behaviour. Nothing is re-pinned; nothing goes red.
2. Narrow the two known shape-(i)/(ii) routes — `control-plane.md` and the
   formatting-sensitive source routes — and confirm the six findings above
   would no longer fire.
3. Convert pins to checks where a claim is executable, one route at a time.
4. Transclusion, if §5.3 survives review.

Steps 1 and 2 are independently useful and reversible. Step 3 is a long tail
rather than a project. Step 4 is a proposal to argue about, not a plan.
