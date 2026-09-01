# Language

## Three promises

Every function carries three promises, enforced as syntax. Leaving one out is a
compile error:

- `requires` — what must hold before the function is called (e.g. "the list is sorted").
- `ensures` — what the result guarantees (e.g. "if it returns an index, the element is there").
- `!` — what the function may touch (e.g. "nothing — pure", or "may read this file").

Loops add `keeps` (the loop invariant) and `measures` (the termination measure).

```thermite
fn sum(xs: &[u32]) -> u64
  requires xs.len() <= 1_000_000
  ensures result == spec_sum(xs)
  !  pure
{
  let mut acc: u64 = 0;
  let mut i: usize = 0;
  while i < xs.len()
    keeps acc == spec_sum(&xs[..i])
    measures xs.len() - i
  {
    acc = acc + xs[i] as u64;
    i = i + 1;
  }
  acc
}
```

## Resource types

`resource(region)` upgrades an affine struct or enum to a checked resource: an
owned value must be transferred, returned, or deliberately abandoned on every
path that returns. A bare `resource` declaration is contagious and derives its
regions from resource-bearing fields or variants. Owning containers propagate
the obligation; borrowed references do not.

Deliberate abandonment is explicit and priced in the effect row:

```thermite
resource(heap) struct Grant { id: u64 }

fn discard(g: Grant) -> u64
  requires true
  ensures result == 0
  ! forgets(heap)
{
  forget(g);
  0
}
```

`forget(value)` consumes exactly one live owned resource and requires every
region in that value's provenance as a `forgets(region)` effect. Certification
binds the checked flow to the canonical program, replays returning paths,
joins, loops, and abandonment footprints in Lean, and reports the remaining
trust in parsing, provenance resolution, witness extraction, and target
resource behavior.

## The specification language

Contract-position expressions stay inside a small fragment: a fixed set of
bounded quantifier combinators with frozen SMT triggers, and no raw `forall`.
The fragment is small enough to make the machine-checked soundness proof
([Verification](verification.md)) feasible, and to keep the whole surface
teachable to a model within a fixed token budget. `THERMITE.skill.md` is the
generated, budget-checked language reference.

## How a function is written

The workflow is incremental. Declare the contract with a hole where the body
goes (`?0`, a typed hole). `forge goal` shows what is given and what must hold;
`forge fill` puts code in the hole and re-checks, returning a counterexample on
failure. Repeat until `forge check` reports `ALL GOALS DISCHARGED ✓ certified
L3`. An item with an unfilled hole cannot be built or certified.

`forge build` compiles a certified program to a native binary, with the
`!`-derived syscall cage enabled ([Trust](trust.md)).
