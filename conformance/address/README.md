# Semantic-address fixtures

Each `<name>.addresses.json` file describes the addresses expected for
`conformance/<name>.th`. These fixtures test
`thermite-syntax/src/address.rs` against the scheme defined in
`.design/syntax/semantic-addressing.md`.

The `addresses` array lists valid addresses in document order. Entries for
`keeps` and `measures` nodes also include the source text that the address must resolve
to. The `must_error` array contains invalid or out-of-range addresses that must
produce a structured error.

Addresses are one-based and follow source order within their enclosing item.
For `binary_search`, `keeps#2` resolves to `forall_below` and `keeps#3` resolves to
`forall_from`. This corrects the reversed labels in the illustrative
`thermite-design.md` §4.3 example.

Expected values are derived from the source programs and the design, not from
the address resolver's output (goal.md R-CHAR-3).
