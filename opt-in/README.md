# `opt-in/` — a contributor's tooling is their choice

Per `telos/the-clone-is-neutral` and RFC-15. Nothing under this directory is
required to build, test or verify Thermite. The path is named for the claim it
makes: everything below it is opted into, never assumed.

## Layout

Each stack **mirrors the repository root**, so materializing it is a
structure-preserving copy:

```
opt-in/claude/.claude/settings.json   ->  .claude/settings.json
opt-in/crosslink/.mcp.json            ->  .mcp.json
```

There is deliberately no manifest. A manifest is a second description of the
same thing, and every second copy in this repository has eventually drifted from
the first.

## Use

```
just use              # list stacks, and which are installed
just use claude       # materialize .claude/settings.json and .claude/agents/
just use crosslink    # materialize .crosslink/ and .mcp.json
just use day          # empty until RFC-15 step 4 — see opt-in/day/README.md
```

`just use <stack>` **refuses** when a target exists and differs from the stack,
printing the diff. Pass `--force` to overwrite.

## Why the config is currently tracked twice

RFC-15 §6 lands this in three steps, and step 1 is deliberately additive: the
stacks appear here while `.claude/`, `.crosslink/` and `.mcp.json` stay tracked
at the root, so no gate goes red. `control-plane-check.py` still *requires* that
wiring — a bare clone is red on arrival today (§1.1), and that only changes at
step 2, which needs agreement about what a gate may demand.

So between step 1 and step 3 there are two tracked copies of the same files.
That window is accepted, not overlooked: it keeps the uncontentious move
independent of the contentious gate change, which is what lets step 1 land on
its own. `just use` refusing on a difference is what makes the drift visible
while the window is open.
