use std::collections::{BTreeMap, BTreeSet};

use thermite_spec::effect_commutation::{
    concurrent_conflicts, effects_commute, footprint_frames_region, Commutation,
};
use thermite_spec::{RegionError, RegionIndex};
use thermite_syntax::{parse, Effect, RegionPath, Type};

#[test]
fn nested_declared_fields_resolve_and_induce_ancestry() {
    let parsed = parse(
        "struct Queue { head: u64 }\n\
         struct Scheduler { runqueue: Queue, timers: u64 }\n\
         shared scheduler: Scheduler\n\
         fn ack() -> u64 ! read(scheduler.runqueue.head) requires nothing ensures result == 0 { 0 }\n\
         fn complete() -> u64 ! write(scheduler.timers) requires nothing ensures result == 0 { 0 }\n\
         concurrent shootdown { ack, complete }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let index = RegionIndex::build(&parsed.program).expect("region declarations resolve");
    let scheduler = RegionPath::from("scheduler");
    let runqueue = RegionPath {
        segments: vec!["scheduler".into(), "runqueue".into()],
    };
    let head = RegionPath {
        segments: vec!["scheduler".into(), "runqueue".into(), "head".into()],
    };
    let timers = RegionPath {
        segments: vec!["scheduler".into(), "timers".into()],
    };
    assert_eq!(
        index.resolve(&head),
        Ok(Type::Prim(thermite_syntax::PrimType::U64))
    );
    assert!(index.overlaps(&scheduler, &head));
    assert!(index.overlaps(&runqueue, &head));
    assert!(!index.overlaps(&runqueue, &timers));
}

#[test]
fn unknown_root_and_field_are_structured_errors() {
    let parsed = parse(
        "struct Scheduler { timers: u64 }\n\
         shared scheduler: Scheduler\n\
         fn f() -> u64 ! read(missing.field), write(scheduler.runqueue) requires nothing ensures result == 0 { 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = RegionIndex::build(&parsed.program).expect_err("invalid paths must reject");
    assert!(errors
        .iter()
        .any(|error| matches!(error, RegionError::UnknownRegionRoot { .. })));
    assert!(errors.iter().any(|error| matches!(error, RegionError::UnknownRegionField { field, .. } if field == "runqueue")));
}

#[test]
fn concurrent_consumer_reports_ancestry_conflicts_and_accepts_siblings() {
    let parsed = parse(
        "struct Queue { head: u64 }\n\
         struct Scheduler { runqueue: Queue, timers: u64 }\n\
         shared scheduler: Scheduler\n\
         fn left() -> u64 ! write(scheduler) requires nothing ensures result == 0 { 0 }\n\
         fn right() -> u64 ! read(scheduler.runqueue) requires nothing ensures result == 0 { 0 }\n\
         fn sibling() -> u64 ! write(scheduler.timers) requires nothing ensures result == 0 { 0 }\n\
         concurrent ancestry { left, right }\n\
         concurrent disjoint { right, sibling }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let regions = RegionIndex::build(&parsed.program).expect("regions resolve");
    let footprints = BTreeMap::from([
        (
            "left".into(),
            BTreeSet::from([Effect::Write("scheduler".into())]),
        ),
        (
            "right".into(),
            BTreeSet::from([Effect::Read("scheduler.runqueue".into())]),
        ),
        (
            "sibling".into(),
            BTreeSet::from([Effect::Write("scheduler.timers".into())]),
        ),
    ]);

    let conflicts = concurrent_conflicts(&parsed.program, &regions, &footprints)
        .expect("all root footprints supplied");
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].composition, "ancestry");
    assert_eq!(conflicts[0].left_root, "left");
    assert_eq!(conflicts[0].right_root, "right");
    assert_eq!(conflicts[0].overlap, Some(RegionPath::from("scheduler")));
}

#[test]
fn concurrent_consumer_fails_closed_without_an_inferred_root() {
    let parsed = parse(
        "fn left() -> u64 ! pure requires nothing ensures result == 0 { 0 }\n\
         fn right() -> u64 ! pure requires nothing ensures result == 0 { 0 }\n\
         concurrent pair { left, right }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let regions = RegionIndex::build(&parsed.program).expect("metadata resolves");
    let error = concurrent_conflicts(&parsed.program, &regions, &BTreeMap::new())
        .expect_err("unknown footprint must not be treated as pure");
    assert!(error.contains("no inferred footprint for `left`"));
}

#[test]
fn lock_guards_resolve_and_order_cycles_fail_closed() {
    let parsed = parse(
        "struct S { a: u64, b: u64 } keeps a < 10\n\
         shared state: S\n\
         lock first guards state.a\n\
         lock second guards state.b after first",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let index = RegionIndex::build(&parsed.program).expect("lock metadata resolves");
    assert_eq!(
        index.guarded_region("first"),
        Some(&RegionPath::from("state.a"))
    );
    assert!(index.is_after("second", "first"));

    let cyclic = parse(
        "struct S { a: u64 } keeps a < 10\nshared state: S\n\
         lock first guards state after second\nlock second guards state after first",
    );
    let errors = RegionIndex::build(&cyclic.program).expect_err("cycle must reject");
    assert!(errors
        .iter()
        .any(|e| matches!(e, RegionError::LockOrderCycle { .. })));
}

#[test]
fn ownership_conflicts_with_guarded_access_and_frame_uses_writes_only() {
    let parsed = parse(
        "struct S { guarded: u64, free: u64 } keeps guarded < 10\nshared state: S\nlock gate guards state.guarded",
    );
    let regions = RegionIndex::build(&parsed.program).unwrap();
    assert_eq!(
        effects_commute(
            &Effect::Owns("gate".into()),
            &Effect::Read("state.guarded".into()),
            &regions,
        ),
        Commutation::Reject
    );
    let footprint = BTreeSet::from([
        Effect::Read("state.guarded".into()),
        Effect::Write("state.free".into()),
    ]);
    assert!(footprint_frames_region(
        &footprint,
        &RegionPath::from("state.guarded"),
        &regions
    ));
    assert!(!footprint_frames_region(
        &footprint,
        &RegionPath::from("state.free"),
        &regions
    ));
}
