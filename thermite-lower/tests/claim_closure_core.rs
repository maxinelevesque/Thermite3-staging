//! Focused executable observations for the core L3 lowering claim closures.

const SUM: &str = include_str!("../../conformance/sum.th");
const BINARY_SEARCH: &str = include_str!("../../conformance/binary_search.th");

fn lower(source: &str) -> String {
    let parsed = thermite_syntax::parse(source);
    assert!(
        parsed.errors.is_empty(),
        "parse errors: {:?}",
        parsed.errors
    );
    thermite_lower::lower(&parsed.program).expect("claim-closure fixture must lower")
}

#[test]
fn frame_and_function_signatures_are_observable() {
    let emitted = lower(SUM);
    assert!(emitted.starts_with("use vstd::prelude::*;\nverus! {\n"));
    assert!(emitted.ends_with("\n}\nfn main() {}\n"));
    assert!(emitted.contains("spec fn spec_sum(xs: Seq<u32>) -> nat"));
    assert!(emitted.contains("decreases xs.len()"));
    assert!(emitted.contains("fn sum(xs: &[u32]) -> (result: u64)"));
    assert!(emitted.contains("requires xs.len() <= 1000000,"));
    assert!(emitted.contains("ensures\n"));
}

#[test]
fn type_lowering_is_observable() {
    let sum = lower(SUM);
    assert!(sum.contains("spec fn spec_sum(xs: Seq<u32>) -> nat"));
    assert!(sum.contains("fn sum(xs: &[u32]) -> (result: u64)"));

    let search = lower(BINARY_SEARCH);
    assert!(search
        .contains("fn binary_search(haystack: &[u32], needle: u32) -> (result: Option<usize>)"));

    let unit_bool = lower("fn observe(flag: bool) -> () ! pure requires flag ensures true { }");
    assert!(unit_bool.contains("fn observe(flag: bool) -> (result: ())"));
}

#[test]
fn exec_expression_lowering_is_observable() {
    let sum = lower(SUM);
    for expected in [
        "let mut acc: u64 = 0;",
        "acc = acc + xs[i] as u64;",
        "i = i + 1;",
        "u32::MAX",
    ] {
        assert!(sum.contains(expected), "missing `{expected}`\n{sum}");
    }

    let search = lower(BINARY_SEARCH);
    for expected in [
        "let mid = lo + (hi - lo) / 2;",
        "if haystack[mid] == needle",
        "return Some(mid);",
        "return None;",
    ] {
        assert!(search.contains(expected), "missing `{expected}`\n{search}");
    }
}

#[test]
fn statement_and_loop_contracts_are_observable() {
    let sum = lower(SUM);
    for expected in [
        "while i < xs.len()",
        "invariant\n            i <= xs.len(),",
        "decreases xs.len() - i,",
    ] {
        assert!(sum.contains(expected), "missing `{expected}`\n{sum}");
    }

    let search = lower(BINARY_SEARCH);
    for expected in [
        "loop\n",
        "invariant\n            lo <= hi && hi <= haystack.len(),",
        "decreases hi - lo,",
        "if lo == hi",
    ] {
        assert!(search.contains(expected), "missing `{expected}`\n{search}");
    }
}

#[test]
fn spec_seq_views_are_observable() {
    let sum = lower(SUM);
    for expected in [
        "spec fn spec_sum(xs: Seq<u32>) -> nat",
        "xs.drop_first()",
        "spec_sum(xs@.subrange(0, i as int))",
    ] {
        assert!(sum.contains(expected), "missing `{expected}`\n{sum}");
    }

    let search = lower(BINARY_SEARCH);
    for expected in [
        "sorted(haystack@)",
        "haystack@[i as int] == needle",
        "forall_in(haystack@, |x: u32| x != needle)",
    ] {
        assert!(search.contains(expected), "missing `{expected}`\n{search}");
    }
}

#[test]
fn discovered_combinator_definitions_are_observable() {
    let emitted = lower(BINARY_SEARCH);
    for expected in [
        "spec fn sorted(s: Seq<u32>) -> bool",
        "spec fn forall_in(s: Seq<u32>, p: spec_fn(u32) -> bool)",
        "spec fn forall_below(s: Seq<u32>, n: int, p: spec_fn(u32) -> bool)",
        "spec fn forall_from(s: Seq<u32>, n: int, p: spec_fn(u32) -> bool)",
        "#[trigger] p(s[i])",
    ] {
        assert!(
            emitted.contains(expected),
            "missing `{expected}`\n{emitted}"
        );
    }
    assert!(!emitted.contains("spec fn count_where("));
}
