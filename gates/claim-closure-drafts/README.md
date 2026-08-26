# Claim-closure draft slices

Files in this directory are non-authoritative version-1 JSON recipes for the
frozen claim/evidence migration. `gates/claim-closure-author.py --check-drafts`
checks every slice against the live shipped registry, executes its witness and
counterfeits, and derives its discriminator and receipt.

Drafts do not close requirements. Registry and ledger schema version 1 continue
to forbid authoritative claims, witnesses, and closures. Coordinated
materialization is available only after the drafts cover all 566 frozen shipped
IDs plus every live shipped addition exactly.
