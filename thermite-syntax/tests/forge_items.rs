//! Parse + AST + address round-trip tests for the Stage-1 forge-tier surface
//! items (`.design/stage1-forge-tier.md` REQ-3 / AC-7, increment 2a): `prop fn`,
//! `lemma`, `proof for`, `witness`, and the `?pN` proof holes inside proof blocks.
//!
//! These items are PARSE-only in this increment (their semantic consumers are the
//! covenant engine 2b, the tactic battery 2c, the proof view 2e, the lemma library
//! 3); the tests here are their consumers — they assert the parsed AST shape and
//! the semantic addresses (`f.proof.ensures#k`, `?pN`). R-CHAR-3: expected shapes are
//! hand-derived from the grammar, not copied from the parser's output. `tests/` is
//! ungated, so `unwrap`/`panic` are fine.

use thermite_syntax::{
    addresses_of, parse, resolve, AddrKind, ForgeItem, HoleContext, Item, SyntaxError,
};

/// Parse `src`, assert it is clean, and return the single forge item it contains.
fn single_forge_item(src: &str) -> ForgeItem {
    let result = parse(src);
    assert!(
        result.is_clean(),
        "expected a clean parse of {src:?}, got: {:?}",
        result.errors
    );
    assert_eq!(result.program.items.len(), 1, "expected exactly one item");
    match result.program.items.into_iter().next().unwrap() {
        Item::Forge(forge) => forge,
        other => panic!("expected a forge item, got {other:?}"),
    }
}

/// The set of address strings `addresses_of` produces for `src`.
fn address_set(src: &str) -> Vec<String> {
    let result = parse(src);
    assert!(result.is_clean(), "parse errors: {:?}", result.errors);
    addresses_of(&result.program)
        .into_iter()
        .map(|e| e.addr)
        .collect()
}

// ---------------------------------------------------------------------------
// prop fn
// ---------------------------------------------------------------------------

#[test]
fn prop_fn_parses_to_propfn_item() {
    let src = "prop fn sorted(xs: Vec<u64>) -> bool { true }";
    let forge = single_forge_item(src);
    let ForgeItem::PropFn(p) = forge else {
        panic!("expected a PropFn, got {forge:?}");
    };
    assert_eq!(p.name, "sorted");
    assert_eq!(p.params.len(), 1);
    assert_eq!(p.params[0].name, "xs");
    assert!(p.measures.is_none());
}

#[test]
fn prop_fn_is_addressed_by_name() {
    let src = "prop fn sorted(xs: Vec<u64>) -> bool { true }";
    let addrs = address_set(src);
    assert_eq!(addrs, vec!["sorted".to_string()]);
    // The address round-trips through `resolve`.
    let result = parse(src);
    let entry = resolve(&result.program, "sorted").expect("resolve sorted");
    assert_eq!(entry.kind, AddrKind::Forge);
}

// ---------------------------------------------------------------------------
// lemma
// ---------------------------------------------------------------------------

#[test]
fn lemma_parses_with_req_ens_and_proof_block() {
    let src = "lemma add_id(a: u64) requires true ensures a == a proof { omega }";
    let forge = single_forge_item(src);
    let ForgeItem::Lemma(l) = forge else {
        panic!("expected a Lemma, got {forge:?}");
    };
    assert_eq!(l.name, "add_id");
    assert_eq!(l.params.len(), 1);
    assert_eq!(l.ensures.len(), 1);
    // The proof block captures verbatim tactic text (not structurally parsed) and
    // has no open holes here.
    assert_eq!(l.proof.text, "omega");
    assert!(l.proof.holes.is_empty());
}

#[test]
fn lemma_proof_block_captures_proof_holes() {
    let src = "lemma l(a: u64) requires true ensures a == a proof { induction a; ?p0 }";
    let forge = single_forge_item(src);
    let ForgeItem::Lemma(l) = forge else {
        panic!("expected a Lemma, got {forge:?}");
    };
    assert_eq!(l.proof.holes.len(), 1);
    assert_eq!(l.proof.holes[0].number, 0);
    assert_eq!(l.proof.holes[0].context, HoleContext::Proof);
    // The verbatim text still includes the `?p0` source.
    assert!(l.proof.text.contains("?p0"));
}

#[test]
fn lemma_proof_hole_is_addressed() {
    let src = "lemma l(a: u64) requires true ensures a == a proof { ?p0 }";
    let addrs = address_set(src);
    assert!(addrs.contains(&"l".to_string()), "lemma root: {addrs:?}");
    assert!(
        addrs.contains(&"l.proof.?p0".to_string()),
        "proof-hole address: {addrs:?}"
    );
    let result = parse(src);
    let entry = resolve(&result.program, "l.proof.?p0").expect("resolve proof hole");
    assert_eq!(entry.kind, AddrKind::ProofHole);
}

#[test]
fn lemma_missing_ens_is_a_structured_error() {
    let src = "lemma l(a: u64) requires true proof { omega }";
    let result = parse(src);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, SyntaxError::MissingClause { clause, .. } if clause == "ensures")),
        "expected a MissingClause(ensures), got: {:?}",
        result.errors
    );
}

// ---------------------------------------------------------------------------
// proof for f { ens#k by { ... } }
// ---------------------------------------------------------------------------

#[test]
fn proof_item_parses_obligations() {
    let src = "proof for binary_search { ensures#0 by { omega } ensures#1 by { nlinarith } }";
    let forge = single_forge_item(src);
    let ForgeItem::Proof(p) = forge else {
        panic!("expected a Proof, got {forge:?}");
    };
    assert_eq!(p.target, "binary_search");
    assert_eq!(p.obligations.len(), 2);
    assert_eq!(p.obligations[0].clause.keyword, "ensures");
    assert_eq!(p.obligations[0].clause.index, Some(0));
    assert_eq!(p.obligations[0].proof.text, "omega");
    assert_eq!(p.obligations[1].clause.index, Some(1));
    assert_eq!(p.obligations[1].proof.text, "nlinarith");
}

#[test]
fn proof_clause_ordinal_overflow_is_rejected_without_truncation() {
    let value = u64::from(u32::MAX) + 1;
    let result = parse(&format!("proof for f {{ ensures#{value} by {{ omega }} }}"));
    assert!(result.errors.iter().any(|error| matches!(
        error,
        SyntaxError::ClauseOrdinalOverflow { value: parsed, .. }
            if *parsed == u128::from(value)
    )));
}

#[test]
fn proof_obligation_clause_and_hole_addresses() {
    let src = "fn f(n: u32) -> u32 ! pure requires true ensures result == n ensures result >= n ensures n <= result { n }\n\
               proof for f { ensures#2 by { ?p3 } }";
    let addrs = address_set(src);
    assert!(
        addrs.contains(&"f.proof.ensures#2".to_string()),
        "obligation address: {addrs:?}"
    );
    assert!(
        addrs.contains(&"f.proof.ensures#2.?p3".to_string()),
        "proof-hole address: {addrs:?}"
    );
    let result = parse(src);
    assert_eq!(
        resolve(&result.program, "f.proof.ensures#2").unwrap().kind,
        AddrKind::Forge
    );
    assert_eq!(
        resolve(&result.program, "f.proof.ensures#2.?p3")
            .unwrap()
            .kind,
        AddrKind::ProofHole
    );
}

#[test]
fn orphan_and_wrong_kind_proofs_have_no_semantic_addresses() {
    for src in [
        "proof for missing { ensures#0 by { omega } }",
        "spec fn not_exec(n: u32) -> u32 measures n { n }\n\
         proof for not_exec { ensures#0 by { omega } }",
    ] {
        let result = parse(src);
        assert!(
            result.is_clean(),
            "surface parse remains syntactic: {:?}",
            result.errors
        );
        assert!(
            addresses_of(&result.program)
                .iter()
                .all(|entry| !entry.addr.contains(".proof.")),
            "an unbound proof target must not mint an address: {:?}",
            addresses_of(&result.program)
        );
    }
}

// ---------------------------------------------------------------------------
// witness { inhabit (...); falsify N; }
// ---------------------------------------------------------------------------

#[test]
fn witness_block_parses_inhabit_and_falsify() {
    let src = "witness { inhabit (1, 2); falsify 50_000; }";
    let forge = single_forge_item(src);
    let ForgeItem::Witness(w) = forge else {
        panic!("expected a Witness, got {forge:?}");
    };
    assert_eq!(w.inhabits.len(), 1);
    assert_eq!(w.inhabits[0].args.len(), 2);
    assert_eq!(w.falsifies.len(), 1);
    assert_eq!(w.falsifies[0].budget, 50_000);
}

#[test]
fn witness_is_addressed_by_number() {
    let src = "witness { inhabit (1); falsify 10; }";
    let addrs = address_set(src);
    assert_eq!(addrs, vec!["witness#1".to_string()]);
    let result = parse(src);
    assert_eq!(
        resolve(&result.program, "witness#1").unwrap().kind,
        AddrKind::Forge
    );
}

// ---------------------------------------------------------------------------
// hole gating (AC-7)
// ---------------------------------------------------------------------------

#[test]
fn body_hole_inside_proof_block_is_structured_error() {
    // A proof block admits only proof holes `?pN`; a body hole `?N` there is the
    // mirror error of ProofHoleOutsideProofBlock.
    let src = "lemma l(a: u64) requires true ensures a == a proof { ?0 }";
    let result = parse(src);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, SyntaxError::BodyHoleInProofBlock { number: 0, .. })),
        "expected BodyHoleInProofBlock, got: {:?}",
        result.errors
    );
}

#[test]
fn nested_braces_in_proof_block_are_balanced() {
    // A proof block tracks brace depth so a nested `by { … }` / `calc { … }` does
    // not prematurely close it.
    let src = "lemma l(a: u64) requires true ensures a == a proof { calc { a == a by { refl } } }";
    let forge = single_forge_item(src);
    let ForgeItem::Lemma(l) = forge else {
        panic!("expected a Lemma");
    };
    assert!(l.proof.text.contains("calc"));
    assert!(l.proof.text.contains("refl"));
}

#[test]
fn forge_items_do_not_disturb_v1_items_in_a_mixed_program() {
    // A program mixing a v1 `fn` with forge items parses; the v1 fn keeps
    // its ordinary address and the forge items add theirs.
    let src = "fn id(x: u64) -> u64 ! pure requires true ensures result == x { x }\n\
               lemma l(a: u64) requires true ensures a == a proof { omega }\n\
               witness { inhabit (1); falsify 5; }";
    let addrs = address_set(src);
    assert!(addrs.contains(&"id".to_string()));
    assert!(addrs.contains(&"l".to_string()));
    assert!(addrs.contains(&"witness#1".to_string()));
}
