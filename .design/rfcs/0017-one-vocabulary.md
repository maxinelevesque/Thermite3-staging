---
rfc: 17
title: One vocabulary — carrying the full words into the AST and the token kinds
status: draft
supersedes: []
introduces: []
---

# RFC-17: One vocabulary — carrying the full words into the AST and the token kinds

| | |
|---|---|
| **Status** | Draft, unfiled. Depends on RFC-6 landing first |
| **Supersedes** | — |
| **Baseline** | `dollspace-gay/Thermite @ 84d276e7`, plus RFC-6 |
| **Scope** | Rust identifiers only. No change to the surface, the grammar, or any `.th` file |
| **Relation** | Finishes what RFC-6 deliberately left undone |

RFC-6 moved the *surface* to full words and deliberately stopped there: the
`TokKind` variants keep their names "so the 53 downstream references do not
move", and the AST fields were never in scope. That was the right call for
reviewability. It leaves the toolchain speaking two vocabularies for one
concept, and this proposes finishing the job as a separate, mechanical step.

## 0. What this asks

Rename five token kinds and three AST fields, and nothing else.

| today | proposed |
|---|---|
| `TokKind::Req` | `TokKind::Requires` |
| `TokKind::Ens` | `TokKind::Ensures` |
| `TokKind::Fx` | `TokKind::Effects` |
| `TokKind::Inv` | `TokKind::Keeps` |
| `TokKind::Dec` | `TokKind::Measures` |
| `Contract.req` | `Contract.requires` |
| `Contract.ens` | `Contract.ensures` |
| `Contract.fx` | `Contract.effects` |
| `LemmaItem.req` | `LemmaItem.requires` |
| `LemmaItem.ens` | `LemmaItem.ensures` |
| `FnItem.dec` | `FnItem.measures` |
| `SpecFnItem.dec` | `SpecFnItem.measures` |
| `PropFnItem.dec` | `PropFnItem.measures` |
| `LoopNode.dec` | `LoopNode.measures` |
| `StructItem.inv` | `StructItem.keeps` |

The table originally held eight rows. The implementation found seven more, each
as a compile error rather than a reading, and `LemmaItem` is the clearest case. It carries the same two clause fields for the same reason, and leaving
it behind would have reinstated the split this proposal exists to close — a
`Contract` spelling `requires` beside a `LemmaItem` spelling `req`. Named here
rather than absorbed silently, because a rename that grows during implementation
is exactly the kind of scope change a reviewer should be told about.

## 1. The lexer table is the argument

After RFC-6, `keyword_kind` reads:

```rust
"requires" => TokKind::Req,
"ensures"  => TokKind::Ens,
"keeps"    => TokKind::Inv,
"measures" => TokKind::Dec,
```

Two of those are not abbreviations of the word on their left. `keeps` is not
short for `Inv`, and `measures` is not short for `Dec` — they are the *previous*
words, carried forward. A reader of the lexer now needs to know that `Inv` means
`keeps` and `Dec` means `measures`, which is precisely the mapping RFC-6 removed
from the surface and precisely the cost `telos/surface-serves-agents` names:
abbreviations do not merely fail to help, they misdirect.

The AST says the same thing more quietly:

```rust
pub struct Contract {
    pub req: Clause,
    pub ens: Vec<Clause>,
    pub fx: EffectRow,
}
```

## 2. Why this is not RFC-6 scope creep, and why it is worth doing anyway

RFC-6's case rests on the surface being read and written by language models. The
same argument applies to the AST with the same force — agents read
`thermite-syntax` constantly, and `Contract.fx` misdirects toward audio/visual
effects exactly as `fx` did in the surface. The reason to hold it back was
review cost, not disagreement.

That reason is now spent. RFC-6's diff is the one a maintainer has to accept as
a judgement about the language. This diff is a rename with no semantic content,
reviewable by its mechanical properties rather than by reading it.

Measured against the post-RFC-6 tree:

| | |
|---|---|
| `TokKind::{Req,Ens,Fx,Inv,Dec}` references | 49, across **2** files |
| `.req` / `.ens` / `.fx` field accesses | 278 |
| `req`/`ens`/`fx` in identifier position, any meaning | 3,196 across 186 files |

The first row is the load-bearing one, and it is far smaller than RFC-6's "53
downstream references" suggested — the variants concentrate in the lexer and the
parser. The third row is a superset that includes local variables, Express-style
`req` in unrelated code, and prose in comments; it is the reason this must be a
compiler-assisted rename rather than a textual one.

## 3. How it is checked

The rename is type-directed, which makes it the easiest possible thing to
verify: rename the variant or field, and the compiler enumerates every site.
There is no analogue of RFC-6's hard case — no string literals to parse, no
JSON population, no clause-name-as-data. A clause name in the AST is an
identifier, and `rustc` knows every one.

The evidence a reviewer should ask for is therefore narrow:

* `cargo check --workspace --all-targets` exits 0, which is the completeness
  proof — an unrenamed site does not compile;
* `cargo test --workspace` matches the pre-rename run by failing-test **name**,
  the same control RFC-6 used;
* `tests/golden/` is untouched, since Verus lowering never sees a Rust field
  name;
* the conformance corpus is untouched, since no `.th` file changes.

**A digest over a `Debug` rendering is NOT type-directed, and this RFC first
claimed no such case existed.** `forge/src/kernel_image.rs` computes a frozen
boundary digest as `sha256(format!("{:#?}", function.contract))`, and Rust's
derived `Debug` renders FIELD NAMES. Renaming `Contract`'s fields therefore moves
that digest — `3f525f14` to `756cd389` — with no clause, span or effect changed.
The compiler cannot see it; a CI job caught it. Re-pinned with the cause
recorded, as RFC-6 did for the same digest when clause SPANS moved it.

**A serialized form is the same failure, and it happened too.** `forge audit
--json` keys its boundary rows from `BoundaryContract`'s field names via serde,
so the rename re-keyed a manifest carrying `manifest_version: v1` — an external
contract. Two further structs were affected, `KernelCertificateBinding.fx` and
`SpecFnDecl.dec`. The fix keeps both properties rather than trading them: the
Rust fields carry the full word, and `#[serde(rename = "req")]` holds the wire
spelling. Reverting the fields would have reintroduced the split.

The lesson generalizes past this RFC: "the compiler enumerates every site" holds
only where the compiler is what reads the name. A digest over a rendering, a
serialized form, or a diagnostic string reads it too, and none of those
type-check. Both cases here were caught by CI rather than by `cargo check`, and
both were invisible to the local test run.

**One further population needs care and is not type-directed.** RFC-6 already
found it:
clause names built as data — `format!("{loop_addr}.measures")`, the
`ClauseSelector` family strings, and their diagnostics. Those are strings that
must agree with `validate_segments`, and the compiler cannot see them. They were
migrated in the RFC-6 implementation and this RFC does not touch them; they are
listed here so a reviewer knows they were considered and excluded, not missed.

## 4. Residual trust

* **`Effects` is a naming choice, not a derivation.** `TokKind::Fx` becomes
  `Effects` rather than `Bang` or `Row` because the row's *keyword* disappeared
  in RFC-6 — the surface spells it `!`. Nothing forces this name, and it is the
  one line of this RFC that is taste rather than mechanism.
* **The `.th` comments are still v2.** Thirty comment lines across twelve corpus
  files refer to `inv#3` and `fx alloc` inside already-migrated programs, because
  the migration rewrites clause-token spans and preserves comments by design.
  RFC-6's implementation cleaned these; nothing prevents them recurring, and no
  gate looks at comment text.
* **Three wire formats are held at v1 by attribute rather than by type.**
  `#[serde(rename)]` keeps the manifest keys stable, and nothing prevents a
  future field being added without one. The schema is not versioned against the
  struct; the attribute is the only thing holding them together.
* **One frozen digest moved and was re-pinned rather than derived.** The
  kernel boundary digest is `sha256` over a `Debug` rendering, so it encodes
  field names. Accepting the new value rests on the same argument RFC-6 used:
  the structural fingerprint over item names, clause text and the effect row is
  identical either side.
* **This RFC does not make the vocabulary enforceable.** After it, nothing stops
  a future field from being named `fx` again. A lint could, and this does not
  propose one.

## 5. Sequence

1. Land RFC-6. This RFC is unreviewable before then — the two vocabularies have
   to exist before the argument for collapsing them does.
2. Rename `TokKind` variants (2 files, 49 sites), compiler-checked.
3. Rename `Contract` fields (278 accesses), compiler-checked.
4. One commit, since a partial rename is strictly worse than either end state.
