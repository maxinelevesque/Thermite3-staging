# Claim-closure draft slices

Files in this directory are non-authoritative version-1 JSON recipes for the
frozen claim/evidence migration. `gates/claim-closure-author.py --check-drafts`
checks every slice against the live shipped registry, executes its witness and
counterfeits, and derives its discriminator and receipt.

CI uses `--check-draft-shard INDEX/COUNT`. Every shard first requires the exact
complete draft population, then selects a disjoint subset by a stable hash of
the execution identity. Requirements sharing an executable verifier/oracle,
the built-in formal verifier, or an exact-population extractor stay together so
the existing deterministic execution cache and identity-collision checks are
preserved. The eight CI children collectively perform the same positive and
counterfeit executions as `--check-drafts`; a stable aggregate requires every
child to succeed.

Drafts do not close requirements. The schema-version-2 registry and ledger are
authoritative; these recipes independently reproduce the claims, witnesses, and
closures they contain. Coordinated rematerialization remains available only
when the drafts cover all 566 frozen shipped IDs plus every live shipped
addition exactly.

Local coordinated rematerialization is serial by default:

```sh
bash dev/install-g4-tools.sh
uv run python gates/claim-closure-author.py --materialize
```

For the same deterministic eight-shard ownership used by CI, use bounded local
parallelism:

```sh
bash dev/install-g4-tools.sh
uv run python gates/claim-closure-author.py --materialize --jobs 8
```

Complete materialization fails before running any claim probes when the pinned
CaDiCaL/drat-trim pair is absent. The lower harness automatically supplies the
repository-local `target/g4-tools/bin` paths when explicit `THERMITE_EPR_*`
overrides are not set.

Each worker receives whole execution-identity groups. Publication occurs only
after the coordinator proves that all eight outputs are complete, pairwise
disjoint, and equal to the frozen-plus-live population, then runs the same
canonical registry/inventory/ledger render used by the serial path. Worker
counts from two through eight control shard scheduling without changing
ownership. Because local shards share Cargo/Lean build trees, fixed oracle
timeouts, and an observable cache warm-up order, executable and formal probes
are admitted one at a time in the canonical serial entry order;
exact-population checks may still use the wider worker pool. CI retains eight
isolated runners. Omitting `--jobs` is the serial fallback.

The core lowering oracle allows 360 seconds per selected test case. This is a
harness resource margin, not a product proof budget: it prevents known
120–180-second cold cache paths from being classified as semantic failures on a
loaded local host, while every expected exit and counterfeit distinction remains
unchanged.

All `formal_theorem` entries use `W-FORMAL-LEAN-AXIOM-PROBE`. The closed
witness identity for formal claims is the built-in Lean/axiom verifier itself;
the theorem subject and kernel observation still derive a distinct
per-requirement discriminator. Splitting equivalent formal identities across
invented witness names is rejected.
