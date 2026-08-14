use thermite_syntax::{
    parse, semantic_inventory, walk_semantic, ChildRole, Expr, Item, ResourceLimit, SemanticEvent,
    UnaryOp, WorkBudget,
};

#[test]
fn canonical_inventory_covers_conditions_guards_patterns_and_expression_blocks() {
    let parsed = parse("fn f(x: bool) -> u64 ! pure requires x ensures result == 1 { let y = if x { 1 } else { 2 }; match y { n if x => n, _ => 1 } }");
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    let inventory = semantic_inventory(&parsed.program, WorkBudget(10_000)).unwrap();
    let roles: Vec<_> = inventory.edges.iter().map(|edge| edge.role).collect();
    assert!(roles.contains(&ChildRole::Condition));
    assert!(roles.contains(&ChildRole::Guard));
    assert!(roles.contains(&ChildRole::Pattern));
    assert!(roles.contains(&ChildRole::Then));
    assert!(roles.contains(&ChildRole::Else));
    assert!(roles.contains(&ChildRole::Tail));
    let events = walk_semantic(&inventory, WorkBudget(20_000)).unwrap();
    assert_eq!(events.len(), inventory.kinds.len() * 2);
    assert!(matches!(events.first(), Some(SemanticEvent::Enter { .. })));
    assert!(matches!(events.last(), Some(SemanticEvent::Leave { .. })));
}

#[test]
fn resource_exhaustion_is_structured_and_non_accepting() {
    let parsed = parse(
        "fn f(x: bool) -> bool ! pure requires x ensures result { if x { true } else { false } }",
    );
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    assert!(matches!(
        semantic_inventory(&parsed.program, WorkBudget(1)),
        Err(ResourceLimit { .. })
    ));
}

#[test]
fn exact_sufficient_budgets_separate_support_from_resource_availability() {
    let parsed = parse(
        "fn f(x: bool) -> bool ! pure requires x ensures result { if x { true } else { false } }",
    );
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    let generous = semantic_inventory(&parsed.program, WorkBudget(10_000)).unwrap();
    let nodes = generous.kinds.len();
    assert_eq!(
        semantic_inventory(&parsed.program, WorkBudget(nodes)).unwrap(),
        generous
    );
    assert!(matches!(
        semantic_inventory(&parsed.program, WorkBudget(nodes - 1)),
        Err(ResourceLimit { required_at_least, .. }) if required_at_least == nodes
    ));
    assert_eq!(
        walk_semantic(&generous, WorkBudget(nodes * 2))
            .unwrap()
            .len(),
        nodes * 2
    );
    assert!(matches!(
        walk_semantic(&generous, WorkBudget(nodes * 2 - 1)),
        Err(ResourceLimit { required_at_least, .. }) if required_at_least == nodes * 2
    ));
}

#[test]
fn deep_finite_expression_walk_uses_no_native_recursion() {
    let mut parsed = parse("fn f() -> bool ! pure requires true ensures result { true }");
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    let Item::Fn(function) = &mut parsed.program.items[0] else {
        unreachable!()
    };
    let mut expr = Expr::BoolLit(true);
    for _ in 0..10_000 {
        expr = Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(expr),
        };
    }
    function.body.as_mut().unwrap().tail = Some(Box::new(expr));
    let inventory = semantic_inventory(&parsed.program, WorkBudget(20_000)).unwrap();
    let events = walk_semantic(&inventory, WorkBudget(40_000)).unwrap();
    assert_eq!(events.len(), inventory.kinds.len() * 2);
    std::mem::forget(parsed);
}
