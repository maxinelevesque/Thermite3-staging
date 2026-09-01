use thermite_syntax::{parse, Item, RegionPath, SyntaxError};

#[test]
fn resource_structs_and_enums_preserve_explicit_and_bare_provenance() {
    let source = r#"
resource(heap, device.port) struct Grant { value: u64 }
resource enum Envelope { Empty, Full(Grant) }
resource(net) enum Packet { Data(u64) }
"#;
    let parsed = parse(source);
    assert!(parsed.is_clean(), "unexpected errors: {:?}", parsed.errors);

    let Item::Struct(grant) = &parsed.program.items[0] else {
        panic!("expected resource struct")
    };
    let direct = grant.resource.as_ref().expect("direct resource marker");
    assert_eq!(
        direct.regions,
        vec![
            RegionPath::root("heap".into()),
            RegionPath {
                segments: vec!["device".into(), "port".into()]
            }
        ]
    );
    assert_eq!(
        &source[direct.span.start..direct.span.end()],
        "resource(heap, device.port)"
    );

    let Item::Enum(envelope) = &parsed.program.items[1] else {
        panic!("expected contagious resource enum")
    };
    let contagious = envelope.resource.as_ref().expect("bare resource marker");
    assert!(contagious.regions.is_empty());
    assert_eq!(
        &source[contagious.span.start..contagious.span.end()],
        "resource"
    );

    let Item::Enum(packet) = &parsed.program.items[2] else {
        panic!("expected direct resource enum")
    };
    assert_eq!(
        packet.resource.as_ref().unwrap().regions,
        vec![RegionPath::root("net".into())]
    );
}

#[test]
fn resource_remains_an_identifier_outside_item_modifier_position() {
    let parsed = parse(
        "fn resource(resource: u64) -> u64 ! pure requires true ensures result == resource { resource }",
    );
    assert!(parsed.is_clean(), "unexpected errors: {:?}", parsed.errors);
    assert_eq!(parsed.program.items[0].name(), "resource");
}

#[test]
fn malformed_resource_modifiers_have_structured_diagnostics() {
    let empty = parse("resource() struct Empty { value: u64 }");
    assert!(matches!(
        empty.errors.first(),
        Some(SyntaxError::EmptyResourceProvenance { .. })
    ));

    let duplicate = parse("resource(heap, heap) enum Duplicate { Unit }");
    assert!(matches!(
        duplicate.errors.first(),
        Some(SyntaxError::DuplicateResourceRegion { path, .. })
            if path == &RegionPath::root("heap".into())
    ));

    let wrong_target =
        parse("resource fn bad() -> u64 ! pure requires true ensures result == 0 { 0 }");
    assert!(matches!(
        wrong_target.errors.first(),
        Some(SyntaxError::ResourceModifierTarget { .. })
    ));
}
