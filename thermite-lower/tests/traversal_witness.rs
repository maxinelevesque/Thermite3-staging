use thermite_lower::witness::{emit_witness_with_budget, required_witness_budget};
use thermite_lower::{
    canonical_ast_projection, check_program, emit_witness, replay_witness, CanonicalAstProjection,
    TraversalWitness, WitnessError,
};
use thermite_syntax::{parse, WorkBudget};

fn lean_output(ast: &CanonicalAstProjection, witness: &TraversalWitness) -> std::process::Output {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let replay = thermite_lower::lean_replay_source(ast, witness);
    let lean_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../lean");
    let mut child = Command::new("lake")
        .args(["env", "lean", "--stdin", "--threads=1"])
        .current_dir(lean_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("lake/lean must be installed for the completeness matrix");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(replay.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn lean_replays(source: &str) {
    let parsed = parse(source);
    assert!(parsed.is_clean(), "{source}: {:?}", parsed.errors);
    let checked = check_program(&parsed.program)
        .unwrap_or_else(|errors| panic!("fixture must check: {source}: {errors:?}"));
    let witness = emit_witness(&checked);
    let ast = canonical_ast_projection(&parsed.program).unwrap();
    let output = lean_output(&ast, &witness);
    assert!(
        output.status.success(),
        "Lean rejected supported fixture:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const SOURCE: &str = "struct State { n: u64 } keeps n < 10
     shared state: State
     lock state_lock guards state
     fn read() -> u64 ! owns(state_lock), read(state.n)
       requires true ensures result < 10
     { holding state_lock { state.n } }";

#[test]
fn witness_is_deterministic_json_and_replays_against_source() {
    let parsed = parse(SOURCE);
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    let checked = check_program(&parsed.program).unwrap();
    let first = emit_witness(&checked);
    let second = emit_witness(&checked);
    assert_eq!(first, second);
    let json = first.canonical_json().unwrap();
    assert_eq!(TraversalWitness::from_json(&json).unwrap(), first);
    replay_witness(&parsed.program, &first).expect("faithful witness replays");
}

#[test]
fn bounded_producer_succeeds_at_exact_budget_and_exhausts_one_below() {
    let parsed = parse(SOURCE);
    let exact = required_witness_budget(&parsed.program).unwrap();
    let produced = emit_witness_with_budget(&parsed.program, exact).unwrap();
    replay_witness(&parsed.program, &produced).unwrap();

    let insufficient = WorkBudget(exact.0 - 1);
    assert!(matches!(
        emit_witness_with_budget(&parsed.program, insufficient),
        Err(WitnessError::Construction(errors))
            if matches!(errors.as_slice(), [thermite_lower::LowerError::ResourceLimit {
                budget,
                required_at_least,
            }] if *budget == insufficient.0 && *required_at_least == exact.0)
    ));
}

#[test]
fn wire_format_rejects_truncation_version_skew_and_same_shape_payload_change() {
    let parsed = parse(SOURCE);
    let witness = emit_witness(&check_program(&parsed.program).unwrap());
    let json = witness.canonical_json().unwrap();

    assert!(matches!(
        TraversalWitness::from_json(&json[..json.len() - 1]),
        Err(WitnessError::Json(_))
    ));

    let mut skewed: serde_json::Value = serde_json::from_str(&json).unwrap();
    skewed["version"] = serde_json::json!(witness.version + 1);
    let skewed = TraversalWitness::from_json(&skewed.to_string()).unwrap();
    assert_eq!(
        replay_witness(&parsed.program, &skewed).unwrap_err(),
        WitnessError::Mismatch { field: "version" }
    );

    let mut changed: serde_json::Value = serde_json::from_str(&json).unwrap();
    changed["node_facts"][0] = serde_json::json!("None-but-mutated");
    let changed = TraversalWitness::from_json(&changed.to_string()).unwrap();
    assert_eq!(
        replay_witness(&parsed.program, &changed).unwrap_err(),
        WitnessError::Mismatch {
            field: "node_facts"
        }
    );
}

#[test]
fn independently_mutated_evidence_is_rejected_by_named_field() {
    let parsed = parse(SOURCE);
    let checked = check_program(&parsed.program).unwrap();
    let original = emit_witness(&checked);

    let mut omitted_edge = original.clone();
    omitted_edge.edges.pop();
    assert_eq!(
        replay_witness(&parsed.program, &omitted_edge).unwrap_err(),
        WitnessError::Mismatch { field: "edges" }
    );

    let mut forged_close = original.clone();
    forged_close.holdings[0].close_edges[0]
        .inner_to_outer
        .clear();
    assert_eq!(
        replay_witness(&parsed.program, &forged_close).unwrap_err(),
        WitnessError::Mismatch { field: "holdings" }
    );

    let mut missing_authority = original.clone();
    missing_authority.shared_places[0].authorizing_locks.clear();
    assert_eq!(
        replay_witness(&parsed.program, &missing_authority).unwrap_err(),
        WitnessError::Mismatch {
            field: "shared_places"
        }
    );
}

#[test]
fn lean_derives_lock_authority_order_and_all_close_edges_from_neutral_facts() {
    let parsed = parse(SOURCE);
    let witness = emit_witness(&check_program(&parsed.program).unwrap());
    let ast = canonical_ast_projection(&parsed.program).unwrap();
    assert!(lean_output(&ast, &witness).status.success());

    let mut forged_guard = ast.clone();
    forged_guard.lock_decls[0].guarded_region = "forged.region".into();
    assert!(!lean_output(&forged_guard, &witness).status.success());

    let mut omitted_place = ast.clone();
    omitted_place
        .events
        .iter_mut()
        .find(|event| event.entering && event.kind == "Place" && event.eligible)
        .unwrap()
        .eligible = false;
    assert!(!lean_output(&omitted_place, &witness).status.success());

    let mut forged_fallthrough = ast.clone();
    forged_fallthrough
        .events
        .iter_mut()
        .find(|event| !event.entering && event.kind == "Holding")
        .unwrap()
        .node = u32::MAX;
    assert!(!lean_output(&forged_fallthrough, &witness).status.success());

    let ordered = parse(
        "struct State { a: u64, b: u64 } keeps a < 10\n\
         shared state: State\n\
         lock alpha guards state.a\n\
         lock beta guards state.b after alpha\n\
         fn ordered() -> u64 ! owns(alpha), owns(beta) requires true ensures result == 0\n\
         { holding alpha { holding beta { } } 0 }",
    );
    let ordered_witness = emit_witness(&check_program(&ordered.program).unwrap());
    let ordered_ast = canonical_ast_projection(&ordered.program).unwrap();
    assert!(lean_output(&ordered_ast, &ordered_witness).status.success());
    let mut missing_order = ordered_ast.clone();
    missing_order
        .lock_decls
        .iter_mut()
        .find(|lock| lock.name == "beta")
        .unwrap()
        .after = None;
    assert!(!lean_output(&missing_order, &ordered_witness)
        .status
        .success());

    let control = parse(
        "struct State { n: u64 } keeps n < 10\n\
         shared state: State\n\
         lock gate guards state\n\
         fn via_return() -> u64 ! owns(gate) requires true ensures result == 1\n\
         { holding gate { return 1; } 0 }\n\
         fn via_break() -> u64 ! owns(gate) requires true ensures result == 2\n\
         { loop keeps true measures 1 as u64 { holding gate { break; } } 2 }\n\
         fn via_continue() -> u64 ! owns(gate) requires true ensures result == 3\n\
         { let mut i: u64 = 0; while i < 1 keeps i <= 1 measures 1 - i\n\
           { holding gate { i = i + 1; continue; } } 3 }",
    );
    let control_witness = emit_witness(&check_program(&control.program).unwrap());
    let control_ast = canonical_ast_projection(&control.program).unwrap();
    assert!(lean_output(&control_ast, &control_witness).status.success());
    for reason in ["Return", "Break", "Continue"] {
        let mut omitted = control_ast.clone();
        omitted
            .events
            .iter_mut()
            .find(|event| event.entering && event.kind == reason)
            .unwrap()
            .kind = "Other".into();
        assert!(!lean_output(&omitted, &control_witness).status.success());
    }
}

#[test]
fn witness_is_bound_to_literal_and_binding_changes_not_only_tree_shape() {
    let first = parse(SOURCE);
    let changed = parse(&SOURCE.replace("result < 10", "result < 9"));
    let witness = emit_witness(&check_program(&first.program).unwrap());
    assert_eq!(
        replay_witness(&changed.program, &witness).unwrap_err(),
        WitnessError::Mismatch {
            field: "canonical_ast_sha256"
        }
    );
}

#[test]
fn canonical_projection_agrees_on_alloc_calls_and_lexical_shadowing() {
    let fixtures = [
        "fn literal() -> String ! alloc requires true ensures true { \"hello\" }",
        "fn constructors() -> Vec<u64> ! alloc requires true ensures true { Vec::new() }",
        "struct State { n: u64 } keeps n < 10\n\
         shared state: State\n\
         lock gate guards state\n\
         fn shadow() -> u64 ! owns(gate) requires true ensures true\n\
         { let state: u64 = 1; holding gate { state } }",
        "fn helper() -> u64 ! alloc requires true ensures true { 1 }\n\
         fn caller(xs: Vec<u64>) -> u64 ! alloc requires true ensures true { xs.helper() }",
        "fn new() -> u64 ! pure requires true ensures true { 0 }\n\
         fn caller() -> Vec<u64> ! alloc requires true ensures true { Vec::new() }",
        "fn push() -> u64 ! pure requires true ensures true { 0 }\n\
         fn caller(xs: Vec<u64>) -> u64 ! alloc requires true ensures true { xs.push(1); 0 }",
    ];
    for source in fixtures {
        let parsed = parse(source);
        assert!(parsed.is_clean(), "{source}: {:?}", parsed.errors);
        let checked = check_program(&parsed.program)
            .unwrap_or_else(|errors| panic!("fixture must check: {source}: {errors:?}"));
        let witness = emit_witness(&checked);
        let ast = canonical_ast_projection(&parsed.program).unwrap();
        assert_eq!(
            witness.direct_footprints, ast.direct_footprints,
            "direct effects diverged for {source}"
        );
        assert_eq!(witness.calls, ast.calls, "calls diverged for {source}");
    }
}

#[test]
fn checked_shared_places_obey_the_same_lexical_scope_as_projection() {
    let source = r#"struct State { n: u64 } keeps n < 10
         shared state: State
         lock gate guards state
         fn real() -> u64 ! owns(gate), read(state.n) requires true ensures true
         { holding gate { state.n } }
         fn parameter(state: u64) -> u64 ! pure requires true ensures true { state }
         fn local() -> u64 ! pure requires true ensures true
         { let state: u64 = 1; state }"#;
    let parsed = parse(source);
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    let checked = check_program(&parsed.program).expect("shadowed locals are not shared places");
    let witness = emit_witness(&checked);
    let ast = canonical_ast_projection(&parsed.program).unwrap();
    assert_eq!(witness.shared_places.len(), 1);
    assert_eq!(
        witness
            .shared_places
            .iter()
            .map(|place| place.node)
            .collect::<Vec<_>>(),
        ast.events
            .iter()
            .filter(|event| event.entering && event.kind == "Place" && event.eligible)
            .map(|event| event.node)
            .collect::<Vec<_>>()
    );
    replay_witness(&parsed.program, &witness).expect("shadowed program replays");
}

#[test]
fn lock_free_shared_place_has_no_authority_requirement() {
    let parsed = parse("struct State { n: u64 } keeps n < 10\nshared state: State\nfn read() -> u64 ! read(state.n) requires true ensures result < 10 { state.n }");
    assert!(parsed.is_clean(), "{:?}", parsed.errors);
    let checked = check_program(&parsed.program).unwrap();
    let witness = emit_witness(&checked);
    let ast = canonical_ast_projection(&parsed.program).unwrap();
    assert_eq!(witness.shared_places.len(), 1);
    assert!(witness.shared_places[0].authorizing_locks.is_empty());
    assert!(ast.lock_decls.is_empty());
}

#[test]
fn complete_current_inventory_matrix_uses_the_universal_lean_theorem() {
    let fixtures = [
        SOURCE,
        "fn helper(x: u64) -> u64 ! pure requires true ensures true { x }\n\
         fn flow(x: u64) -> u64 ! pure requires true ensures true {\n\
           let f = |y| y;\n\
           let z = if x == 0 { helper(x) } else { match x { n if n == x => n, _ => 0 } };\n\
           while false keeps true measures 1 { break; }\n\
           if z == 0 { return f(z); }\n\
           z\n\
         }",
        "fn quantified(xs: &[u64]) -> bool ! pure\n\
           requires forall (i : Idx) in xs. xs[i] == xs[i]\n\
           ensures result\n\
         { forall_in(xs, |x| x == x) }",
        "fn new() -> u64 ! pure requires true ensures true { 0 }\n\
         fn push() -> u64 ! pure requires true ensures true { 0 }\n\
         fn owned(xs: Vec<u64>) -> String ! alloc requires true ensures true {\n\
           xs.push(1); let ys = Vec::new(); \"complete\"\n\
         }",
    ];
    for source in fixtures {
        lean_replays(source);
    }
}
