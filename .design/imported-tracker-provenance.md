# Imported tracker provenance

Several research and verification documents were imported with issue numbers
from an earlier tracker. Those numbers are historical provenance, not links to
issues in `maxinelevesque/Thermite3-staging`. This table is the canonical local
disposition for the imported references named by the Post-Thermite-3 roadmap.

| Imported reference | Meaning in the imported work | Current local disposition |
|---|---|---|
| `#169` | Lowering-soundness epic for the frozen verified subset | Completed. The resulting architecture and remaining trust are recorded in `.design/verified/thermite-semantics.md`; no new implementation issue is required. |
| `#173` | Proof-assistant and verified-validator architecture fork | Resolved in favor of Lean 4, Mathlib, Lean-SMT/CVC5 reconstruction, and a verified-validator architecture. The decision is recorded in `.design/verified/thermite-semantics.md`; no new issue is required. |
| `#175` | Formal-methods state-of-the-art survey and terminology correction | Completed as `.design/research/formal-methods-sota.md`. Future proof-producing SMT or source/Rust boundary work must receive its own current issue when promoted. |
| `#273` | Preregistered contract-carrying composition experiment | Rehomed as local issue #155, governed by `.design/research/composition-experiment.md`. |

The imported numbers remain quoted in historical documents because they explain
the provenance and sequencing of those records. They must not be interpreted as
current local dependencies, used in `Closes` metadata, or silently reassigned to
future issues that happen to receive the same number.

RFC-14 was staged without a durable local tracker in the imported roadmap. Its
current research home is local issue #154, governed by
`.design/rfcs/0014-crash-clause.md`. Issues #154 and #155 are research-tier
inventory only; their existence does not place them on the assurance-minimum
product spine.
