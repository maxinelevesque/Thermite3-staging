use thermite_syntax::{parse, Effect, EffectRow, Item, RegionPath, Type};

#[test]
fn shared_nested_types_and_region_paths_parse() {
    let parsed = parse(
        "struct Queue { head: u64 }\n\
         struct Scheduler { runqueue: Queue, timers: u64 }\n\
         shared scheduler: Scheduler\n\
         fn read_head() -> u64\n\
           ! read(scheduler.runqueue.head)\n\
           requires nothing\n\
           ensures result == 0\n\
         { 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);

    let shared = match &parsed.program.items[2] {
        Item::SharedDecl(shared) => shared,
        other => panic!("expected shared declaration, got {other:?}"),
    };
    assert_eq!(shared.name, "scheduler");
    assert_eq!(shared.ty, Type::Named("Scheduler".into()));

    let function = match &parsed.program.items[3] {
        Item::Fn(function) => function,
        other => panic!("expected function, got {other:?}"),
    };
    let EffectRow::Set(effects) = &function.contract.effects else {
        panic!("expected non-pure row")
    };
    let Effect::Read(path) = &effects[0] else {
        panic!("expected read effect")
    };
    assert_eq!(
        path,
        &RegionPath {
            segments: vec!["scheduler".into(), "runqueue".into(), "head".into()]
        }
    );
}

#[test]
fn concurrent_composition_parses_separately_from_function_contracts() {
    let parsed = parse(
        "concurrent shootdown { ack, complete, }\n\
         fn ack() -> u64 ! pure requires nothing ensures result == 0 { 0 }\n\
         fn complete() -> u64 ! pure requires nothing ensures result == 0 { 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let composition = match &parsed.program.items[0] {
        Item::Concurrent(composition) => composition,
        other => panic!("expected concurrent declaration, got {other:?}"),
    };
    assert_eq!(composition.name, "shootdown");
    assert_eq!(composition.roots, vec!["ack", "complete"]);
}
