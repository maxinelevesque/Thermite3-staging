# The `day` stack

Deliberately empty until **RFC-15 step 4**.

`day` is installed as a Claude Code plugin, which ships its own commands, hooks
and MCP server — so this repository holds no `day` configuration to materialize.
The duplicated hooks and the project-scoped MCP entry that used to live in
tracked `.claude/settings.json` were removed as duplicates before RFC-15 began.

What lands here at step 4 is not config but **vocabulary**: the nine witness
subjects that `day doctor` needs in order to compose, which today ride in
`.claims/` because it is the only channel that exists. RFC-15 §2 is the argument
for why those are a different population from the config in `opt-in/claude/` and
`opt-in/crosslink/`, and why separating them is a step of the RFC rather than a
cleanup afterwards.

Until then `just use day` reports that there is nothing to materialize, which is
the honest answer rather than an error.
