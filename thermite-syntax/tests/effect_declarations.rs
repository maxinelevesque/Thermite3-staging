use thermite_syntax::{
    effect_basis::{resolve_declaration, BasisEntry, Instance, Operation, Theory},
    parse, EffectPrimitive, Item,
};

#[test]
fn declaration_parses_and_resolves_to_basis_primitives() {
    let parsed = parse("effect platform(d) = state(d) + io(sigma_d)");
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let Item::EffectDecl(declaration) = &parsed.program.items[0] else {
        panic!("expected an effect declaration");
    };
    assert_eq!(declaration.name, "platform");
    assert_eq!(declaration.param, "d");
    assert_eq!(
        declaration.combination,
        vec![
            EffectPrimitive::State("d".into()),
            EffectPrimitive::Io("sigma_d".into()),
        ]
    );

    let BasisEntry::Combination(entries) = resolve_declaration(declaration) else {
        panic!("expected a two-entry combination");
    };
    assert_eq!(entries[0].theory, Theory::State(Instance("d".into())));
    assert_eq!(
        entries[0].operations,
        [Operation::Get, Operation::Put].into_iter().collect()
    );
    assert_eq!(entries[1].theory, Theory::Io("sigma_d".into()));
}

#[test]
fn unknown_primitive_is_a_structured_error_listing_the_basis() {
    let parsed = parse("effect platform(d) = mystery(d)");
    assert_eq!(parsed.errors.len(), 1);
    let message = parsed.errors[0].to_string();
    assert!(message.contains("mystery"));
    for primitive in ["state", "accrues", "exception", "partiality", "io"] {
        assert!(message.contains(primitive), "{message}");
    }
}

#[test]
fn declaration_is_an_item_and_does_not_consume_the_following_function() {
    let parsed = parse(
        "effect failure(e) = exception + accrues(e)\n\
         fn id(x: u64) -> u64 ! pure requires true ensures result == x { x }",
    );
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert!(matches!(parsed.program.items[0], Item::EffectDecl(_)));
    assert!(matches!(parsed.program.items[1], Item::Fn(_)));
}
