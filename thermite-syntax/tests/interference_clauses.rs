use thermite_syntax::{parse, tokenize, Item, SyntaxError, TokKind};

fn function_with(block: &str) -> String {
    format!(
        "fn ack(s: &mut u64) -> u64 \
         ! pure requires true ensures result >= final(s) \
         {block} {{ 0 }}"
    )
}

#[test]
fn reserved_words_and_ordered_block_parse_to_typed_relations() {
    let (tokens, errors) = tokenize("interleaves asks promises");
    assert!(errors.is_empty());
    assert!(matches!(tokens[0].kind, TokKind::Interleaves));
    assert!(matches!(tokens[1].kind, TokKind::Asks));
    assert!(matches!(tokens[2].kind, TokKind::Promises));

    let parsed = parse(&function_with(
        "interleaves { asks final(s) >= s; promises final(s) >= s; }",
    ));
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let Item::Fn(function) = &parsed.program.items[0] else {
        panic!("expected function")
    };
    let interference = function
        .contract
        .interference
        .as_ref()
        .expect("typed interference contract");
    assert_eq!(interference.asks.text, "final(s) >= s");
    assert_eq!(interference.promises.text, "final(s) >= s");
    assert!(interference.span.len > 0);
}

#[test]
fn missing_and_misordered_relations_are_structured_errors() {
    let missing_asks = parse(&function_with("interleaves { promises final(s) >= s; }"));
    assert!(matches!(
        missing_asks.errors.first(),
        Some(SyntaxError::InterferenceClauseOrder { clause, .. }) if clause == "asks"
    ));

    let missing_promises = parse(&function_with("interleaves { asks final(s) >= s; }"));
    assert!(matches!(
        missing_promises.errors.first(),
        Some(SyntaxError::MissingInterferenceClause { clause, .. }) if clause == "promises"
    ));

    let repeated = parse(&function_with(
        "interleaves { asks final(s) >= s; promises final(s) >= s; asks true; }",
    ));
    assert!(matches!(
        repeated.errors.first(),
        Some(SyntaxError::InterferenceClauseOrder { .. })
    ));
}

#[test]
fn relation_words_have_no_production_outside_the_block() {
    for word in ["asks", "promises"] {
        let parsed = parse(&format!(
            "fn bad() -> bool ! pure requires true ensures true {word} true {{ true }}"
        ));
        assert!(
            !parsed.is_clean(),
            "{word} unexpectedly parsed outside block"
        );
        let function = parsed.program.items.iter().find_map(|item| match item {
            Item::Fn(function) => Some(function),
            _ => None,
        });
        assert!(function.is_none(), "broken item must not enter the AST");
    }
}

#[test]
fn handler_priorities_are_preserved_but_concurrent_groups_are_symmetric() {
    let parsed = parse("handlers { timer at 1, ipi at 2 }\nconcurrent workers { left, right }");
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let Item::Concurrent(handlers) = &parsed.program.items[0] else {
        panic!("expected handlers composition")
    };
    assert_eq!(handlers.roots, ["timer", "ipi"]);
    assert_eq!(handlers.handler_priorities.as_deref(), Some(&[1, 2][..]));

    let Item::Concurrent(workers) = &parsed.program.items[1] else {
        panic!("expected ordinary concurrent composition")
    };
    assert_eq!(workers.handler_priorities, None);
}

#[test]
fn functions_without_interference_keep_none() {
    let parsed = parse("fn old() -> bool ! pure requires true ensures result { true }");
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let Item::Fn(function) = &parsed.program.items[0] else {
        panic!("expected function")
    };
    assert!(function.contract.interference.is_none());
}
