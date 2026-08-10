---
rfc: 15
title: Harness-agnostic tooling — a bare clone, and a stack you choose
status: draft
supersedes: []
introduces: []
---

# RFC-15: Harness-agnostic tooling — a bare clone, and a stack you choose

| | |
|---|---|
| **Status** | Draft, unfiled. Staged behind the RFC-6 anchor |
| **Supersedes** | — |
| **Baseline** | `dollspace-gay/Thermite @ 84d276e7` |
| **Scope** | Repository layout and CI, not the language |
| **Relation** | Independent of RFC-6..14; touches no `.th` file |

This is the only RFC in the set that proposes nothing about Thermite the
language. It is about what a clone of Thermite *is* — and specifically about
the fact that a clone currently arrives wired to one vendor's agent harness,
with CI configured to fail if that wiring is missing.

## 0. What this asks

Three things.

1. A clone is **bare**: no `.claude/`, no `.crosslink/`, no `.mcp.json`, no
   `.claims/` active at the root.
2. A `justfile` materializes a contributor's own choice — `just stack claude`,
   `just stack day` — from packs kept in-tree.
3. The gates that currently *require* a harness become conditional on one being
   installed, so the bare clone is green.

## 1. The problem is not tidiness

Thirty-nine tracked files under `.claude/`, `.crosslink/` and `.mcp.json` come
out of upstream at `84d276e7`. That number alone is an aesthetic complaint, and
aesthetic complaints do not justify an RFC. What justifies it is that the
arrangement has three measured costs, all of them observed rather than
projected.

### 1.1 CI hard-requires the control plane

`.github/workflows/ci.yml`'s `checks` job runs `gates/control-plane-check.py`,
which reads `.claude/settings.json` and asserts three wirings are present:
`PostToolUse` on `Read` → `spec-discipline.py`, and `PreToolUse` on
`Write`/`Edit` → `spec-discipline.py` and `anti-pattern-gate.py`.

Run it against a tree that carries `tooling/` and no `.claude/`:

```
UNPARSEABLE  .claude/settings.json  file not found — no hook is wired,
             so every gate below is dormant
exit 1
```

A bare clone is therefore **red on arrival**. This is the single fact that makes
the RFC non-trivial: the layout change cannot be made without deciding what this
gate means, or the neutral clone becomes the one that fails CI — which inverts
the argument for having it.

A second harness gate sits in the same job: the skill-budget gate,
`cargo run -p thermite-skill -- --check-budget`, a 6,000-token budget over the
generated `THERMITE.skill.md`.

### 1.2 Adding a second harness drifts an upstream design doc

`.design/tooling/control-plane.md` pins a content digest over its governed
files, and that set includes the whole of `.claude/settings.json`. So *any*
addition to the file moves the hash, not only a change to the three wirings the
document audits.

Wiring a second tool — here `day`, two purely additive `SessionStart` entries,
sixteen lines — moved it. `control-plane-check.py` exits 0 either side of that
move, so the audited control plane never changed; the pin simply cannot tell the
difference between "the control plane changed" and "someone else's tool is also
installed".

### 1.3 The cost lands on branches that carry none of it

This is the sharpest form, because it reaches work that has nothing to do with
harnesses at all.

`actions/checkout` builds the PR **merge** commit. A branch based on a fork's
`main` therefore inherits whatever that `main` carries — including harness
wiring the branch itself never touched. Measured: a branch removing the in-tree
bootable kernel, touching no harness file, tested green on `doc-drift`
standalone and red in CI, on exactly one drifted document — the control-plane
doc, arriving through the merge.

The process layer is not merely *carried* on a fork's main. It is **imposed** on
every branch that bases against it, including the branches whose entire purpose
is to be cherry-picked upstream, where none of it may travel.

## 2. Two populations, not one

The instinct is to treat this as one pile of "agent stuff" to be moved. It is
two, and they fail differently.

**Config** — `.claude/`, `.crosslink/`, `.mcp.json` — is *inert files a stack
copies into place*. Hooks, agent definitions, rules, server declarations. A
contributor either has the tool or does not. Nothing in the repository needs to
read these to function; only the harness does. This is the population CI depends
on, and the population §1 is entirely about.

**Process vocabulary** — atoms, teloi, tensions — is different in kind. It is
not inert: `day` reads it to run at all. It currently travels through `.claims/`,
which conflates two unlike things:

* reasoning worth sharing with anyone who clones the repository, and
* vocabulary the tool needs in order to compose.

`day doctor` reports 18 atoms. Nine are real. The other nine are **witness
subjects** — `atom/implement implementation`, `atom/stage-gate gate-result`, and
seven more — each carrying an identical empty body, each declared `in[] out[]`,
existing only so the graph composes. They are published to `.claims/` today for
a specific reason: `.claims/` is the only channel that exists, so vocabulary
rides the sharing layer because nothing else will carry it.

A `day pack install` separates the two. `.claims/` goes back to being only the
first thing — reasoning — and the nine witness atoms come out of it. **Their
removal is a step of this RFC**, not a cleanup afterwards, because their
presence is the evidence for the distinction the RFC draws.

## 3. Proposal

### 3.1 The bare clone

No `.claude/`, `.crosslink/`, `.mcp.json` or `.claims/` at the repository root.
The packs move under a neutral path — `packs/claude/`, `packs/day/` — where they
are ordinary tracked files that no tool reads by accident.

### 3.2 `just stack`

```
just stack claude    # materializes .claude/ and .mcp.json from packs/claude/
just stack day       # installs day's vocabulary; materializes .claims/
just stack           # lists what is available and what is currently installed
```

Materialized paths are gitignored. A contributor using neither tool runs
neither recipe and sees a repository with no agent surface at all.

### 3.3 The gates become conditional

`control-plane-check.py` gains one branch: if no stack is installed, the three
wirings are **not applicable** rather than **missing**, and it exits 0. Installed
but mis-wired stays exit 1 — that is the defect the gate exists to catch, and it
must keep catching it. The distinction is "no control plane is claimed" versus
"a control plane is claimed and is broken".

The same treatment applies to the skill-budget gate.

`doc-drift`'s route for the control-plane doc narrows from the whole of
`.claude/settings.json` to the pack that produces it, so a second tool's wiring
is outside the digest and §1.2 stops being possible.

## 4. Residual trust

Per `telos/residual-trust-is-named`, what this does **not** discharge:

* **Pack drift.** Once `.claude/settings.json` is generated rather than tracked,
  nothing pins that the generated file matches the pack. The control-plane gate
  checks the materialized result, so a stale pack is caught only for contributors
  who ran the recipe — which is the same class of hole `crosslink #93` opened,
  relocated rather than closed.
* **Vocabulary versioning.** `day pack install` makes the atom graph a tool
  input, and this RFC says nothing about what happens when a repository's pack
  and a contributor's `day` disagree.
* **The bare clone is untested by CI unless CI tests it.** The proposal is only
  meaningfully verified if a job runs the gates against a tree with no stack
  installed. That job does not exist and this RFC does not add it.

## 5. Why this is worth a maintainer's attention

The honest framing is that most of §1 is a cost *this fork* pays, because this
fork is the one that installed a second harness. A maintainer with one harness
and no second tool sees none of §1.2 and none of §1.3.

The argument is that this is exactly the position a project takes before the
second contributor arrives with different tooling, and the cost of the layout is
paid by them rather than by the maintainer — which is precisely the cost a
maintainer is least likely to observe. §1.1 is not conditional on any of that:
`control-plane-check.py` exits 1 on a bare clone today, for anyone, including a
contributor who simply has not installed Claude Code.

## 6. Sequence

1. Packs move to `packs/`, `justfile` recipes land, materialized paths
   gitignored. No gate changes yet; the tracked files stay in place so nothing
   goes red.
2. Gates become stack-conditional, with a CI job exercising the bare tree.
3. Tracked `.claude/`, `.crosslink/`, `.mcp.json` are deleted; the bare clone
   becomes real.
4. `day pack install` lands; the nine witness atoms leave `.claims/`.

Steps 1 and 2 are reversible and independently reviewable. Step 3 is the one
that cannot be un-decided cheaply, and it deliberately comes after the CI job
that proves the bare tree is green.

## 7. A telos proposed alongside this

This RFC is the first work in the set that no existing telos covers. The five in
play are about the language and about landing it upstream; none of them says
anything about who can clone the repository and work in it.

> **`telos/the-clone-is-neutral`** — a contributor's tooling is their choice, and
> the repository neither assumes it nor requires it. Config for a harness is an
> installable pack, never a tracked root; a gate may verify a harness that is
> *claimed*, never demand that one be present. The test is that a clone with no
> agent tooling installed is green.

Its tension with `telos/it-lands-upstream` should be recorded rather than
resolved: neutrality is a change to the maintainer's own repository layout, and
a proposal that rearranges a maintainer's tooling to serve a contributor who has
not arrived yet is a harder sell than one that adds a language feature.
