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

All `formal_theorem` entries use `W-FORMAL-LEAN-AXIOM-PROBE`. The closed
witness identity for formal claims is the built-in Lean/axiom verifier itself;
the theorem subject and kernel observation still derive a distinct
per-requirement discriminator. Splitting equivalent formal identities across
invented witness names is rejected.
