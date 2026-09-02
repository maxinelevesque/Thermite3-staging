use thermite_spec::{validate, ResourceEnv, SpecError};
use thermite_syntax::{parse, RegionPath, Type};

fn program(source: &str) -> thermite_syntax::Program {
    let parsed = parse(source);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    parsed.program
}

fn regions(names: &[&str]) -> Vec<RegionPath> {
    names.iter().map(|name| RegionPath::from(*name)).collect()
}

#[test]
fn direct_and_contagious_provenance_is_order_independent_and_multi_region() {
    let program = program(
        "resource(heap, device.port) struct Bundle { heap: HeapGrant, port: PortGrant }\n\
         resource(device.port) enum PortGrant { Open(u64), Closed }\n\
         resource(heap) struct HeapGrant { id: u64 }",
    );
    validate(&program).expect("exact contagious union must validate");
    let env = ResourceEnv::build(&program).unwrap();
    assert_eq!(
        env.declared("Bundle")
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        regions(&["device.port", "heap"])
    );
}

#[test]
fn every_owning_constructor_is_contagious_but_borrows_are_not() {
    for ty in [
        "Grant",
        "Box<Grant>",
        "Vec<Grant>",
        "Option<Grant>",
        "Result<Grant, u64>",
        "Result<u64, Grant>",
        "Map<Grant, u64>",
        "Map<u64, Grant>",
        "(Grant, u64)",
        "Wrapper<Grant>",
    ] {
        let program = program(&format!(
            "resource struct Aggregate {{ value: {ty} }}\n\
             resource(heap) struct Grant {{ id: u64 }}"
        ));
        validate(&program).unwrap_or_else(|errors| panic!("{ty} lost contagion: {errors:?}"));
        let env = ResourceEnv::build(&program).unwrap();
        assert_eq!(
            env.declared("Aggregate").unwrap(),
            env.declared("Grant").unwrap(),
            "{ty}"
        );
    }

    let direct_program = program("resource(heap) struct Grant { id: u64 }");
    let env = ResourceEnv::build(&direct_program).unwrap();

    assert!(env
        .provenance_of_type(&Type::Ref {
            mutable: false,
            inner: Box::new(Type::Named("Grant".into())),
        })
        .is_empty());
    assert!(env
        .provenance_of_type(&Type::Slice(Box::new(Type::Named("Grant".into()))))
        .is_empty());

    validate(&program(
        "struct Borrowed { reference: &Grant, slice: &[Grant] }\n\
         resource(heap) struct Grant { id: u64 }",
    ))
    .expect("borrowed views do not make their container resource-bearing");
}

#[test]
fn recursive_and_variant_payload_contagion_reaches_a_fixed_point() {
    let program = program(
        "resource struct Envelope { payload: Message }\n\
         resource struct Message { next: Option<Box<Message> >, body: Payload }\n\
         resource enum Payload { Empty, Grant(HeapGrant), Port { token: PortGrant } }\n\
         resource(heap) struct HeapGrant { id: u64 }\n\
         resource(device.port) struct PortGrant { id: u64 }",
    );
    validate(&program).expect("recursive fixed point and variant union must validate");
    let env = ResourceEnv::build(&program).unwrap();
    for name in ["Envelope", "Message", "Payload"] {
        assert_eq!(
            env.declared(name)
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            regions(&["device.port", "heap"]),
            "{name}"
        );
    }
}

#[test]
fn missing_empty_and_mismatched_markers_name_the_responsible_component() {
    let missing =
        program("struct Container { grant: Grant }\nresource(heap) struct Grant { id: u64 }");
    let errors = validate(&missing).unwrap_err();
    assert!(errors.iter().any(|error| matches!(
        error,
        SpecError::MissingResourceMarker { declaration, computed, sources, .. }
            if declaration == "Container"
                && computed == &regions(&["heap"])
                && sources == &vec!["field `grant`".to_string()]
    )));

    let empty = program("resource struct Empty { value: u64 }");
    assert!(validate(&empty).unwrap_err().iter().any(|error| matches!(
        error,
        SpecError::EmptyResourceMarker { declaration, .. } if declaration == "Empty"
    )));

    let mismatch = program(
        "resource(device) struct Container { grant: Grant }\n\
         resource(heap) struct Grant { id: u64 }",
    );
    assert!(validate(&mismatch)
        .unwrap_err()
        .iter()
        .any(|error| matches!(
            error,
            SpecError::ResourceProvenanceMismatch {
                declaration,
                declared,
                computed,
                sources,
                ..
            } if declaration == "Container"
                && declared == &regions(&["device"])
                && computed == &regions(&["heap"])
                && sources == &vec!["field `grant`".to_string()]
        )));
}
