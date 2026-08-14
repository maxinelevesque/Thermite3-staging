use thermite_lower::effects::analyze_effects;
use thermite_lower::{
    lower, lower_l1, lower_l1_with_lock_provider, lower_l3_library,
    lower_l3_library_with_lock_provider, L3Export, L3ExportVisibility, L3LibraryTarget,
    LockProvider,
};
use thermite_spec::{effect_commutation::footprint_frames_region, RegionIndex};
use thermite_syntax::{parse, Effect};

#[test]
fn holding_is_inferred_and_propagates_transitively() {
    let parsed = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn leaf() -> u64 ! owns(gate) requires nothing ensures result == 0 { holding gate { } 0 }\n\
         fn caller() -> u64 ! owns(gate) requires nothing ensures result == 0 { leaf() }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_effects(&parsed.program).expect("exact ownership rows pass");
    assert!(analysis.footprints["leaf"].contains(&Effect::Owns("gate".into())));
    assert!(analysis.footprints["caller"].contains(&Effect::Owns("gate".into())));
    assert!(analysis.warnings.is_empty());
}

#[test]
fn owns_row_is_checked_in_both_directions() {
    let missing = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! pure requires nothing ensures result == 0 { holding gate { } 0 }",
    );
    assert!(analyze_effects(&missing.program).is_err());

    let excess = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate) requires nothing ensures result == 0 { 0 }",
    );
    let analysis = analyze_effects(&excess.program).expect("excess is a warning");
    assert_eq!(
        analysis.warnings[0].excess,
        vec![Effect::Owns("gate".into())]
    );
}

#[test]
fn guarded_access_requires_ownership_and_disjoint_access_does_not() {
    let bypass = parse(
        "struct S { guarded: u64, free: u64 } keeps guarded < 10\nshared state: S\nlock gate guards state.guarded\n\
         fn f() -> u64 ! read(state.guarded) requires nothing ensures result == 0 { 0 }",
    );
    let errors = analyze_effects(&bypass.program).expect_err("unlocked guarded read rejects");
    assert!(format!("{errors:?}").contains("without `owns(gate)`"));

    let disjoint = parse(
        "struct S { guarded: u64, free: u64 } keeps guarded < 10\nshared state: S\nlock gate guards state.guarded\n\
         fn f() -> u64 ! read(state.free) requires nothing ensures result == 0 { 0 }",
    );
    analyze_effects(&disjoint.program).expect("disjoint region remains accessible");
}

#[test]
fn nested_holding_requires_declared_order_and_rejects_reentrancy() {
    let reversed = parse(
        "struct S { a: u64, b: u64 } keeps a < 10\nshared state: S\n\
         lock a guards state.a\nlock b guards state.b after a\n\
         fn f() -> u64 ! owns(a), owns(b) requires nothing ensures result == 0 { holding b { holding a { } } 0 }",
    );
    assert!(analyze_effects(&reversed.program).is_err());

    let ordered = parse(
        "struct S { a: u64, b: u64 } keeps a < 10\nshared state: S\n\
         lock a guards state.a\nlock b guards state.b after a\n\
         fn f() -> u64 ! owns(a), owns(b) requires nothing ensures result == 0 { holding a { holding b { } } 0 }",
    );
    analyze_effects(&ordered.program).expect("declared order permits nesting");

    let reentrant = parse(
        "struct S { a: u64 } keeps a < 10\nshared state: S\nlock a guards state\n\
         fn f() -> u64 ! owns(a) requires nothing ensures result == 0 { holding a { holding a { } } 0 }",
    );
    assert!(analyze_effects(&reentrant.program).is_err());
}

#[test]
fn call_inside_holding_cannot_transitively_reacquire() {
    let parsed = parse(
        "struct S { a: u64 } keeps a < 10\nshared state: S\nlock gate guards state\n\
         fn leaf() -> u64 ! owns(gate) requires nothing ensures result == 0 { holding gate { } 0 }\n\
         fn caller() -> u64 ! owns(gate) requires nothing ensures result == 0 { holding gate { leaf(); } 0 }",
    );
    let errors = analyze_effects(&parsed.program).expect_err("transitive reentrancy rejects");
    assert!(format!("{errors:?}").contains("callee transitively owns the same lock"));
}

#[test]
fn handler_visible_lock_requires_interrupt_masking_in_normal_context() {
    let unmasked = parse(
        "struct S { a: u64, irq: u64 } keeps a < 10\nshared state: S\n\
         lock interrupts guards state.irq\nlock gate guards state.a after interrupts\n\
         fn irq() -> u64 ! owns(gate) requires nothing ensures result == 0 { holding gate { } 0 }\n\
         fn normal() -> u64 ! owns(gate) requires nothing ensures result == 0 { holding gate { } 0 }\n\
         handlers { irq at 1 }",
    );
    let errors = analyze_effects(&unmasked.program).expect_err("unmasked owner rejects");
    assert!(format!("{errors:?}").contains("without `owns(interrupts)`"));

    let masked = parse(
        "struct S { a: u64, irq: u64 } keeps a < 10\nshared state: S\n\
         lock interrupts guards state.irq\nlock gate guards state.a after interrupts\n\
         fn irq() -> u64 ! owns(gate) requires nothing ensures result == 0 { holding gate { } 0 }\n\
         fn normal() -> u64 ! owns(interrupts), owns(gate) requires nothing ensures result == 0 { holding interrupts { holding gate { } } 0 }\n\
         handlers { irq at 1 }",
    );
    analyze_effects(&masked.program).expect("explicit masking accepts");
}

fn test_provider() -> LockProvider {
    LockProvider {
        name: "test".into(),
        rust_source: "use std::cell::UnsafeCell;\nuse std::sync::atomic::{AtomicUsize, Ordering};\nstatic ACQ: AtomicUsize = AtomicUsize::new(0);\nstatic REL: AtomicUsize = AtomicUsize::new(0);\nstruct TestStorage(UnsafeCell<S>);\nunsafe impl Sync for TestStorage {}\nstatic STATE: TestStorage = TestStorage(UnsafeCell::new(S { n: 0 }));\nfn __thermite_shared_state() -> &'static mut S { unsafe { &mut *STATE.0.get() } }\nfn __thermite_lock_acquire_gate() { ACQ.fetch_add(1, Ordering::SeqCst); }\nfn __thermite_lock_release_gate() { REL.fetch_add(1, Ordering::SeqCst); }\n".into(),
        verus_source: String::new(),
        proves_exclusive_acquire: true,
        proves_restore_before_release: true,
        states_interrupt_policy: true,
    }
}

fn run_verus_source(name: &str, source: &str) -> Option<(bool, String)> {
    let binary = std::env::var("VERUS_BIN")
        .ok()
        .map(std::path::PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            std::process::Command::new("which")
                .arg("verus")
                .output()
                .ok()
                .filter(|output| output.status.success())
                .map(|output| {
                    std::path::PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
                })
        })?;
    let path = std::env::temp_dir().join(format!("{name}-{}.rs", std::process::id()));
    std::fs::write(&path, source).ok()?;
    let output = std::process::Command::new(binary)
        .arg(&path)
        .current_dir(std::env::temp_dir())
        .output()
        .ok()?;
    let _ = std::fs::remove_file(path);
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Some((output.status.success(), text))
}

#[test]
fn executable_holding_is_provider_gated_and_releases_once() {
    let parsed = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate), read(state.n), write(state.n) requires true ensures result == 1 { holding gate { state.n = state.n + 1; state.n } }",
    );
    analyze_effects(&parsed.program).expect("static checking is provider-free");
    assert!(format!("{:?}", lower_l1(&parsed.program).unwrap_err())
        .contains("requires an explicit target lock provider"));

    let mut source = lower_l1_with_lock_provider(&parsed.program, &test_provider()).unwrap();
    source.push_str("\nfn main() { assert_eq!(f(), 1); assert_eq!(ACQ.load(Ordering::SeqCst), 1); assert_eq!(REL.load(Ordering::SeqCst), 1); }\n");
    let dir = std::env::temp_dir().join(format!("thermite-rfc10-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source_path = dir.join("main.rs");
    let binary_path = dir.join("main");
    std::fs::write(&source_path, source).unwrap();
    let built = std::process::Command::new("rustc")
        .args([
            "--edition=2021",
            source_path.to_str().unwrap(),
            "-o",
            binary_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(built.success());
    assert!(std::process::Command::new(&binary_path)
        .status()
        .unwrap()
        .success());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn executable_holding_closes_once_on_all_rust_exit_kinds() {
    let parsed = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn via_return() -> u64 ! owns(gate), write(state.n) requires true ensures result == 1 { holding gate { state.n = 1; return 1; } 0 }\n\
         fn via_break() -> u64 ! owns(gate), write(state.n) requires true ensures result == 2 { loop keeps true measures 1 { holding gate { state.n = 2; break; } } 2 }\n\
         fn via_continue() -> u64 ! owns(gate), write(state.n) requires true ensures result == 3 { while keep_going() keeps true measures 1 { holding gate { state.n = 3; continue; } } 3 }\n\
         fn via_panic() -> u64 ! owns(gate), read(state.n), write(state.n) requires true ensures true { holding gate { state.n = 0; 1 / state.n } }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    analyze_effects(&parsed.program).expect("all exits retain lexical authority");

    let mut provider = test_provider();
    provider.rust_source.push_str(
        "static LOOPS: AtomicUsize = AtomicUsize::new(0);\nfn keep_going() -> bool { LOOPS.fetch_add(1, Ordering::SeqCst) == 0 }\n",
    );
    let mut source = lower_l1_with_lock_provider(&parsed.program, &provider).unwrap();
    source.push_str(
        "\nfn main() {\n\
         assert_eq!(via_return(), 1);\n\
         assert_eq!(via_break(), 2);\n\
         assert_eq!(via_continue(), 3);\n\
         assert!(std::panic::catch_unwind(via_panic).is_err());\n\
         assert_eq!(ACQ.load(Ordering::SeqCst), 4);\n\
         assert_eq!(REL.load(Ordering::SeqCst), 4);\n\
         }\n",
    );
    let dir = std::env::temp_dir().join(format!("thermite-rfc10-exits-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source_path = dir.join("main.rs");
    let binary_path = dir.join("main");
    std::fs::write(&source_path, source).unwrap();
    let built = std::process::Command::new("rustc")
        .args([
            "--edition=2021",
            source_path.to_str().unwrap(),
            "-o",
            binary_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(built.success());
    assert!(std::process::Command::new(&binary_path)
        .status()
        .unwrap()
        .success());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn executable_holding_checks_restoration_before_release() {
    let parsed = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn violate() -> u64 ! owns(gate), write(state.n) requires true ensures true { holding gate { state.n = 10; } 0 }",
    );
    analyze_effects(&parsed.program).expect("the open invariant may be temporarily mutated");
    let mut source = lower_l1_with_lock_provider(&parsed.program, &test_provider()).unwrap();
    source.push_str(
        "\nfn main() {\n\
         assert!(std::panic::catch_unwind(violate).is_err());\n\
         assert_eq!(ACQ.load(Ordering::SeqCst), 1);\n\
         assert_eq!(REL.load(Ordering::SeqCst), 0);\n\
         }\n",
    );
    let dir =
        std::env::temp_dir().join(format!("thermite-rfc10-restoration-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source_path = dir.join("main.rs");
    let binary_path = dir.join("main");
    std::fs::write(&source_path, source).unwrap();
    let built = std::process::Command::new("rustc")
        .args([
            "--edition=2021",
            source_path.to_str().unwrap(),
            "-o",
            binary_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(built.success());
    assert!(std::process::Command::new(&binary_path)
        .status()
        .unwrap()
        .success());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn l3_normalizes_close_before_early_edges_and_after_tail_evaluation() {
    let parsed = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn via_return() -> u64 ! owns(gate), write(state.n) requires true ensures true { holding gate { state.n = 1; return 1; } 0 }\n\
         fn via_break() -> u64 ! owns(gate), write(state.n) requires true ensures true { loop keeps true measures 1 { holding gate { state.n = 2; break; } } 2 }\n\
         fn via_continue() -> u64 ! owns(gate), write(state.n) requires true ensures true { while keep_going() keeps true measures 1 { holding gate { state.n = 3; continue; } } 3 }\n\
         fn via_tail() -> u64 ! owns(gate), read(state.n) requires true ensures true { holding gate { state.n } }",
    );
    analyze_effects(&parsed.program).expect("exit fixture is statically authorized");
    let source = lower(&parsed.program).unwrap();
    assert!(source.contains(
        "__thermite_close_gate(&mut __thermite_lock_capability_0);\n            return 1;"
    ));
    assert!(source.contains(
        "__thermite_close_gate(&mut __thermite_lock_capability_1);\n                break;"
    ));
    assert!(source.contains(
        "__thermite_close_gate(&mut __thermite_lock_capability_2);\n                continue;"
    ));
    assert!(
        source.contains("let __thermite_holding_value_0 = __thermite_lock_capability_3.n;"),
        "{source}"
    );
    assert!(
        source.contains("__thermite_close_gate(&mut __thermite_lock_capability_3);"),
        "{source}"
    );
}

#[test]
fn l3_artifacts_require_and_embed_a_verification_provider() {
    let parsed = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate), read(state.n), write(state.n) requires true ensures result == 1 { holding gate { state.n = 10; state.n = 1; state.n } }",
    );
    analyze_effects(&parsed.program).expect("provider-independent analysis passes");
    let exports = [L3Export {
        source_name: "f".into(),
        public_name: "f".into(),
        wrapped: false,
        visibility: L3ExportVisibility::Public,
    }];
    assert!(lower_l3_library(&parsed.program, &exports, L3LibraryTarget::Std).is_err());

    let mut provider = test_provider();
    assert!(lower_l3_library_with_lock_provider(
        &parsed.program,
        &exports,
        L3LibraryTarget::Std,
        &provider,
    )
    .is_err());
    provider.verus_source = "// target-owned verified lock and shared-storage declarations\n\
        fn __thermite_lock_acquire_gate() -> (state: S)\n\
            ensures state.well_formed()\n\
        { S { n: 0 } }\n\
        fn __thermite_close_gate(state: &mut S)\n\
            requires state.well_formed()\n\
        {}\n"
        .to_string();
    let source = lower_l3_library_with_lock_provider(
        &parsed.program,
        &exports,
        L3LibraryTarget::Std,
        &provider,
    )
    .unwrap();
    assert!(source.contains("target-owned verified lock"));
    assert!(
        source.contains("let mut __thermite_lock_capability_0 = __thermite_lock_acquire_gate();")
    );
    assert!(source.contains("__thermite_lock_capability_0.n"));
    assert!(source.contains("__thermite_close_gate(&mut __thermite_lock_capability_0);"));
    if let Some((ok, output)) = run_verus_source("rfc10-provider-positive", &source) {
        assert!(ok && output.contains("0 errors"), "{output}\n{source}");
    }

    let negative_source = lower_l3_library_with_lock_provider(
        &parsed.program,
        &exports,
        L3LibraryTarget::Std,
        &provider,
    )
    .unwrap()
    .replace(
        "__thermite_lock_capability_0.n = 1;",
        "__thermite_lock_capability_0.n = 10;",
    );
    if let Some((ok, output)) = run_verus_source("rfc10-provider-negative", &negative_source) {
        assert!(!ok, "mutated provider unexpectedly verified:\n{output}");
    }
}

#[test]
fn l3_provider_proves_restoration_on_early_control_flow() {
    let parsed = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn via_return() -> u64 ! owns(gate), write(state.n) requires true ensures result == 1 { holding gate { state.n = 1; return 1; } 0 }\n\
         fn via_break() -> u64 ! owns(gate), write(state.n) requires true ensures result == 2 { loop keeps true measures 1 as u64 { holding gate { state.n = 1; break; } } 2 }\n\
         fn via_continue() -> u64 ! owns(gate), write(state.n) requires true ensures result == 3 { let mut i: u64 = 0; while i < 1 keeps i <= 1 measures 1 - i { holding gate { state.n = 1; i = i + 1; continue; } } 3 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    analyze_effects(&parsed.program).expect("early edges are lexically authorized");
    let exports = ["via_return", "via_break", "via_continue"].map(|name| L3Export {
        source_name: name.into(),
        public_name: name.into(),
        wrapped: false,
        visibility: L3ExportVisibility::Public,
    });
    let mut provider = test_provider();
    provider.verus_source = "fn __thermite_lock_acquire_gate() -> (state: S)\n\
        ensures state.well_formed()\n\
        { S { n: 0 } }\n\
        fn __thermite_close_gate(state: &mut S)\n\
        requires state.well_formed()\n\
        {}\n"
        .to_string();
    let source = lower_l3_library_with_lock_provider(
        &parsed.program,
        &exports,
        L3LibraryTarget::Std,
        &provider,
    )
    .unwrap();
    if let Some((ok, output)) = run_verus_source("rfc10-provider-exits", &source) {
        assert!(ok && output.contains("0 errors"), "{output}\n{source}");
    }

    for (index, name) in ["return", "break", "continue"].iter().enumerate() {
        let mut seen = 0usize;
        let negative = source.replacen(".n = 1;", ".n = 10;", index + 1);
        // `replacen` mutates the prefix too; retain the last replacement as the
        // selected edge by restoring earlier occurrences.
        let negative = if index == 0 {
            negative
        } else {
            negative
                .lines()
                .map(|line| {
                    if line.contains(".n = 10;") {
                        seen += 1;
                        if seen <= index {
                            return line.replace(".n = 10;", ".n = 1;");
                        }
                    }
                    line.to_string()
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        if let Some((ok, output)) =
            run_verus_source(&format!("rfc10-provider-{name}-negative"), &negative)
        {
            assert!(
                !ok,
                "unrestored {name} edge unexpectedly verified:\n{output}"
            );
        }
    }
}

#[test]
fn incomplete_provider_attestation_rejects() {
    let parsed = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate) requires true ensures result == 0 { holding gate { } 0 }",
    );
    let mut provider = test_provider();
    provider.proves_restore_before_release = false;
    assert!(format!(
        "{:?}",
        lower_l1_with_lock_provider(&parsed.program, &provider).unwrap_err()
    )
    .contains("restoration before release"));
}

#[test]
fn shared_places_infer_effects_and_require_matching_lexical_holding() {
    let accepted = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate), read(state.n), write(state.n) requires true ensures result < 10 { holding gate { state.n = state.n + 1; state.n } }",
    );
    let analysis =
        analyze_effects(&accepted.program).expect("matching holding authorizes place access");
    assert!(analysis.footprints["f"].contains(&Effect::Read("state.n".into())));
    assert!(analysis.footprints["f"].contains(&Effect::Write("state.n".into())));

    let outside = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate), read(state.n) requires true ensures result < 10 { state.n }",
    );
    assert!(
        format!("{:?}", analyze_effects(&outside.program).unwrap_err())
            .contains("outside `holding gate`")
    );

    let wrong = parse(
        "struct S { n: u64 } keeps n < 10\nstruct T { n: u64 } keeps n < 10\nshared state: S\nshared other: T\nlock gate guards state\nlock wrong guards other\n\
         fn f() -> u64 ! owns(wrong), read(state.n) requires true ensures result < 10 { holding wrong { state.n } }",
    );
    assert!(
        format!("{:?}", analyze_effects(&wrong.program).unwrap_err())
            .contains("outside `holding gate`")
    );

    let unknown = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate), read(state.missing) requires true ensures result == 0 { holding gate { state.missing } }",
    );
    assert!(
        format!("{:?}", analyze_effects(&unknown.program).unwrap_err())
            .contains("UnknownRegionField")
    );
}

#[test]
fn ancestor_write_requires_every_overlapping_guard() {
    let parsed = parse(
        "struct Inner { x: u64, y: u64 }\nstruct Outer { inner: Inner } keeps inner.x < 10\n\
         shared state: Outer\nlock alpha guards state.inner.x\nlock beta guards state.inner.y\n\
         fn f() -> u64 ! owns(alpha), owns(beta), write(state.inner) requires true ensures result == 0 { holding alpha { state.inner = Inner { x: 0, y: 99 }; } 0 }",
    );
    let errors = analyze_effects(&parsed.program).expect_err("ancestor write overlaps both guards");
    assert!(format!("{errors:?}").contains("outside `holding beta`"));
}

#[test]
fn expression_nested_holding_is_inferred_and_closed_at_l3() {
    let parsed = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate), write(state.n) requires true ensures result < 3 { let x: u64 = if true { holding gate { state.n = 10; state.n = 1; } 1 } else { 2 }; x }",
    );
    let analysis = analyze_effects(&parsed.program).expect("nested holding contributes ownership");
    assert!(analysis.footprints["f"].contains(&Effect::Owns("gate".into())));
    let exports = [L3Export {
        source_name: "f".into(),
        public_name: "f".into(),
        wrapped: false,
        visibility: L3ExportVisibility::Public,
    }];
    let mut provider = test_provider();
    provider.verus_source = "fn __thermite_lock_acquire_gate() -> (state: S) ensures state.well_formed() { S { n: 0 } }\nfn __thermite_close_gate(state: &mut S) requires state.well_formed() {}\n".into();
    let source = lower_l3_library_with_lock_provider(
        &parsed.program,
        &exports,
        L3LibraryTarget::Std,
        &provider,
    )
    .unwrap();
    assert!(source.contains("let mut __thermite_lock_capability_0"));
    assert!(source.contains("__thermite_close_gate(&mut __thermite_lock_capability_0)"));
    if let Some((ok, output)) = run_verus_source("rfc10-expression-holding", &source) {
        assert!(ok && output.contains("0 errors"), "{output}\n{source}");
    }
    let broken = source.replace(".n = 1;", ".n = 10;");
    if let Some((ok, output)) = run_verus_source("rfc10-expression-holding-broken", &broken) {
        assert!(
            !ok,
            "broken expression holding unexpectedly verified: {output}"
        );
    }
}

#[test]
fn lexical_bindings_shadow_shared_roots() {
    let parameter = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f(state: u64) -> u64 ! pure requires true ensures result == state { state }",
    );
    let analysis = analyze_effects(&parameter.program).expect("parameter shadows shared root");
    assert!(analysis.footprints["f"].is_empty());

    let local = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! pure requires true ensures result == 1 { let state: u64 = 1; state }",
    );
    let analysis =
        analyze_effects(&local.program).expect("local shadows shared root after binding");
    assert!(analysis.footprints["f"].is_empty());

    let pattern = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! pure requires true ensures result == 1 { match 1 { state => state } }",
    );
    let analysis = analyze_effects(&pattern.program).expect("pattern shadows shared root");
    assert!(analysis.footprints["f"].is_empty());

    let closure = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! pure requires true ensures result == 1 { let c = |state| state; 1 }",
    );
    let analysis =
        analyze_effects(&closure.program).expect("closure parameter shadows shared root");
    assert!(analysis.footprints["f"].is_empty());
}

#[test]
fn shared_place_copy_clone_and_escape_rules_are_affine() {
    let scalar = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn read_n() -> u64 ! owns(gate), read(state.n) requires true ensures true { holding gate { state.n } }",
    );
    analyze_effects(&scalar.program).expect("Copy read is allowed");

    let cloned = parse(
        "struct S { text: String } keeps text.len() <= 20\nshared state: S\nlock gate guards state\n\
         fn copy_text() -> String ! owns(gate), read(state.text) requires true ensures true { holding gate { state.text.clone() } }",
    );
    analyze_effects(&cloned.program).expect("explicit clone produces an owned value");

    let assigned = parse(
        "struct S { text: String } keeps text.len() <= 20\nshared state: S\nlock gate guards state\n\
         fn replace_text() -> u64 ! owns(gate), read(state.text), write(state.text) requires true ensures true { holding gate { state.text = state.text.clone(); } 0 }",
    );
    analyze_effects(&assigned.program).expect("non-Copy shared place may be assigned in place");

    let moved = parse(
        "struct S { text: String } keeps text.len() <= 20\nshared state: S\nlock gate guards state\n\
         fn take_text() -> String ! owns(gate), read(state.text) requires true ensures true { holding gate { state.text } }",
    );
    assert!(
        format!("{:?}", analyze_effects(&moved.program).unwrap_err())
            .contains("moves non-Copy shared place")
    );

    let borrowed = parse(
        "struct S { text: String } keeps text.len() <= 20\nshared state: S\nlock gate guards state\n\
         fn borrow_text() -> &String ! owns(gate), read(state.text) requires true ensures true { holding gate { &state.text } }",
    );
    assert!(
        format!("{:?}", analyze_effects(&borrowed.program).unwrap_err())
            .contains("escaping reference to shared place")
    );

    let unknown_method = parse(
        "struct S { text: String } keeps text.len() <= 20\nshared state: S\nlock gate guards state\n\
         fn inspect() -> u64 ! owns(gate), read(state.text) requires true ensures true { holding gate { state.text.len() } }",
    );
    assert!(format!(
        "{:?}",
        analyze_effects(&unknown_method.program).unwrap_err()
    )
    .contains("unsupported method `len`"));
}

#[test]
fn transitive_lock_acquisition_obeys_declared_order() {
    let unordered = parse(
        "struct S { a: u64, b: u64 } keeps a < 10\nshared state: S\nlock alpha guards state.a\nlock beta guards state.b\n\
         fn inner() -> u64 ! owns(beta) requires true ensures result == 0 { holding beta { } 0 }\n\
         fn outer() -> u64 ! owns(alpha), owns(beta) requires true ensures result == 0 { holding alpha { inner(); } 0 }",
    );
    assert!(
        format!("{:?}", analyze_effects(&unordered.program).unwrap_err())
            .contains("callee transitively takes `beta`")
    );

    let ordered = parse(
        "struct S { a: u64, b: u64 } keeps a < 10\nshared state: S\nlock alpha guards state.a\nlock beta guards state.b after alpha\n\
         fn inner() -> u64 ! owns(beta) requires true ensures result == 0 { holding beta { } 0 }\n\
         fn outer() -> u64 ! owns(alpha), owns(beta) requires true ensures result == 0 { holding alpha { inner(); } 0 }",
    );
    analyze_effects(&ordered.program).expect("declared transitive lock order accepts");
}

#[test]
fn expression_nested_holding_obeys_all_lock_discipline() {
    let reentrant = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate) requires true ensures result == 1 { holding gate { let x: u64 = if true { holding gate { } 1 } else { 2 }; x } }",
    );
    assert!(
        format!("{:?}", analyze_effects(&reentrant.program).unwrap_err())
            .contains("reentrantly holds `gate`")
    );

    let reverse_order = parse(
        "struct S { a: u64, b: u64 } keeps a < 10\nshared state: S\nlock alpha guards state.a\nlock beta guards state.b after alpha\n\
         fn f() -> u64 ! owns(alpha), owns(beta) requires true ensures result == 1 { holding beta { let x: u64 = if true { holding alpha { } 1 } else { 2 }; x } }",
    );
    assert!(
        format!("{:?}", analyze_effects(&reverse_order.program).unwrap_err())
            .contains("takes `alpha` without")
    );

    let transitive_reentrant = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn leaf() -> u64 ! owns(gate) requires true ensures result == 0 { holding gate { } 0 }\n\
         fn f() -> u64 ! owns(gate) requires true ensures result == 1 { let x: u64 = if true { holding gate { leaf(); } 1 } else { 2 }; x }",
    );
    assert!(format!(
        "{:?}",
        analyze_effects(&transitive_reentrant.program).unwrap_err()
    )
    .contains("callee transitively owns the same lock"));
}

#[test]
fn statement_conditions_obey_holding_inference_and_discipline() {
    let if_reentrant = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate) requires true ensures result == 0 { holding gate { if if true { holding gate { } true } else { false } { } 0 } }",
    );
    assert!(
        format!("{:?}", analyze_effects(&if_reentrant.program).unwrap_err())
            .contains("reentrantly holds `gate`")
    );

    let while_reentrant = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate) requires true ensures result == 0 { holding gate { while if true { holding gate { } true } else { false } keeps true measures 1 { } 0 } }",
    );
    assert!(format!(
        "{:?}",
        analyze_effects(&while_reentrant.program).unwrap_err()
    )
    .contains("reentrantly holds `gate`"));

    let inferred = parse(
        "struct S { n: u64 } keeps n < 10\nshared state: S\nlock gate guards state\n\
         fn f() -> u64 ! owns(gate) requires true ensures result == 0 { if if true { holding gate { } true } else { false } { } 0 }",
    );
    let analysis = analyze_effects(&inferred.program).expect("condition holding is inferred");
    assert!(analysis.footprints["f"].contains(&Effect::Owns("gate".into())));
    assert!(lower(&inferred.program)
        .unwrap()
        .contains("__thermite_lock_acquire_gate"));
}

#[test]
fn relational_frame_composes_calls_inside_holding() {
    let parsed = parse(
        "struct S { guarded: u64, free: u64 } keeps guarded < 10\nshared state: S\nlock gate guards state.guarded\n\
         fn pure_id(x: u64) -> u64 ! pure requires true ensures result == x { x }\n\
         fn touch_free() -> u64 ! write(state.free) requires true ensures result == 0 { state.free = 1; 0 }\n\
         fn caller() -> u64 ! owns(gate), read(state.guarded), write(state.free) requires true ensures result < 10 { holding gate { let before: u64 = state.guarded; touch_free(); state.guarded } }",
    );
    let analysis =
        analyze_effects(&parsed.program).expect("disjoint callee composes under holding");
    assert!(analysis.footprints["pure_id"].is_empty());
    let regions = RegionIndex::build(&parsed.program).unwrap();
    assert!(footprint_frames_region(
        &analysis.footprints["touch_free"],
        &"state.guarded".into(),
        &regions,
    ));

    let undeclared = parse(
        "struct S { guarded: u64, free: u64 } keeps guarded < 10\nshared state: S\nlock gate guards state.guarded\n\
         fn touch_free() -> u64 ! pure requires true ensures result == 0 { state.free = 1; 0 }",
    );
    assert!(analyze_effects(&undeclared.program).is_err());

    let unrelated = parse(
        "struct S { guarded: u64, free: u64 } keeps guarded < 10\nshared state: S\nlock gate guards state.guarded\n\
         fn touch_guarded() -> u64 ! write(state.guarded) requires true ensures result == 0 { state.guarded = 10; 0 }",
    );
    assert!(analyze_effects(&unrelated.program).is_err());
}
