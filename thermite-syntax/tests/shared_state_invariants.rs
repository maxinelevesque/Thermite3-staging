use thermite_syntax::{parse, Effect, EffectRow, Item, RegionPath, Stmt};

#[test]
fn lock_declarations_guard_regions_and_record_order() {
    let parsed = parse(
        "struct Sched { counter: u64 } keeps counter < 100\n\
         shared scheduler: Sched\n\
         lock scheduler_lock guards scheduler;\n\
         lock irq_lock guards scheduler.counter after scheduler_lock;",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);

    let Item::LockDecl(first) = &parsed.program.items[2] else {
        panic!("expected first lock declaration")
    };
    assert_eq!(first.name, "scheduler_lock");
    assert_eq!(first.guards, RegionPath::from("scheduler"));
    assert_eq!(first.after, None);

    let Item::LockDecl(second) = &parsed.program.items[3] else {
        panic!("expected ordered lock declaration")
    };
    assert_eq!(second.guards, RegionPath::from("scheduler.counter"));
    assert_eq!(second.after.as_deref(), Some("scheduler_lock"));
}

#[test]
fn owns_and_holding_have_distinct_public_and_lexical_nodes() {
    let parsed = parse(
        "struct Counter { n: u64 } keeps n < 100\n\
         shared counters: Counter\n\
         lock counter_lock guards counters\n\
         fn bump() -> u64\n\
           ! owns(counter_lock), write(counters.n)\n\
           requires nothing\n\
           ensures result == 1\n\
         { holding counter_lock { let n = 1; } 1 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);

    let Item::Fn(function) = &parsed.program.items[3] else {
        panic!("expected function")
    };
    let EffectRow::Set(effects) = &function.contract.effects else {
        panic!("expected effect set")
    };
    assert_eq!(effects[0], Effect::Owns("counter_lock".into()));
    let body = function.body.as_ref().expect("body");
    let Stmt::Holding { lock, body, .. } = &body.stmts[0] else {
        panic!("expected lexical holding statement")
    };
    assert_eq!(lock, "counter_lock");
    assert_eq!(body.stmts.len(), 1);
}

#[test]
fn legacy_inv_struct_spelling_is_rejected() {
    let parsed = parse("struct Counter { n: u64 } inv n < 100");
    assert!(
        !parsed.is_clean(),
        "legacy `inv` must not remain a live spelling"
    );
}
