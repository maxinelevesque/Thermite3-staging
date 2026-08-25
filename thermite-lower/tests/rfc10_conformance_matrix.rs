use thermite_lower::{
    check_program, emit_witness, lower, lower_l1_with_lock_provider, lower_l2, replay_witness,
    LockProvider,
};
use thermite_syntax::{parse, semantic_inventory, ChildRole, SemanticFact, WorkBudget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Position {
    Initializer,
    AssignmentValue,
    ReturnValue,
    Tail,
    IfCondition,
    MatchScrutinee,
    MatchGuard,
    CallArgument,
    TupleElement,
    LoopTest,
}

const POSITIONS: [Position; 10] = [
    Position::Initializer,
    Position::AssignmentValue,
    Position::ReturnValue,
    Position::Tail,
    Position::IfCondition,
    Position::MatchScrutinee,
    Position::MatchGuard,
    Position::CallArgument,
    Position::TupleElement,
    Position::LoopTest,
];

impl Position {
    fn canonical_role(self) -> ChildRole {
        match self {
            Self::Initializer => ChildRole::Initializer,
            Self::AssignmentValue => ChildRole::Value,
            Self::ReturnValue => ChildRole::ReturnValue,
            Self::Tail => ChildRole::Tail,
            Self::IfCondition | Self::LoopTest => ChildRole::Condition,
            Self::MatchScrutinee => ChildRole::Scrutinee,
            Self::MatchGuard => ChildRole::Guard,
            Self::CallArgument => ChildRole::Argument,
            Self::TupleElement => ChildRole::TupleElement,
        }
    }
}

const ESCAPING_BORROW_EXCLUSIONS: [(Position, &str); 8] = [
    (Position::Initializer, "slot requires a u64 value"),
    (Position::AssignmentValue, "slot requires a u64 value"),
    (Position::IfCondition, "condition requires a scalar value"),
    (Position::MatchScrutinee, "outer probe returns u64"),
    (
        Position::MatchGuard,
        "guard requires bool/scalar comparison",
    ),
    (Position::CallArgument, "id parameter is u64"),
    (Position::TupleElement, "outer probe returns u64"),
    (Position::LoopTest, "test requires bool/scalar comparison"),
];

const L2_SHARED_STATE_EXCLUSION: &str =
    "RFC-10 shared-state L2 Kani harness is not implemented; L3 replay remains authoritative";

#[derive(Clone, Copy, Debug)]
enum RejectingPayload {
    DirectReentrancy,
    ReverseOrdering,
    TransitiveReentrancy,
    UnauthorisedRead,
}

const REJECTING_PAYLOADS: [RejectingPayload; 4] = [
    RejectingPayload::DirectReentrancy,
    RejectingPayload::ReverseOrdering,
    RejectingPayload::TransitiveReentrancy,
    RejectingPayload::UnauthorisedRead,
];

fn body(position: Position, payload: &str) -> String {
    match position {
        Position::Initializer => format!("let x: u64 = {payload}; x"),
        Position::AssignmentValue => format!("let mut x: u64 = 0; x = {payload}; x"),
        Position::ReturnValue => format!("return {payload};"),
        Position::Tail => payload.to_string(),
        Position::IfCondition => format!("if {payload} == 0 {{ }} 0"),
        Position::MatchScrutinee => format!("match {payload} {{ _ => 0 }}"),
        Position::MatchGuard => {
            format!("match 0 {{ _ if {payload} == 0 => 0, _ => 0 }}")
        }
        Position::CallArgument => format!("id({payload})"),
        Position::TupleElement => format!("({payload}, 0).0"),
        Position::LoopTest => {
            format!("while {payload} == 0 keeps true measures 1 {{ break; }} 0")
        }
    }
}

fn source(position: Position, inner: &str) -> String {
    let payload = format!("if true {{ holding gate {{ {inner} state.n }} 0 }} else {{ 0 }}");
    format!(
        "struct State {{ n: u64 }} keeps n < 10\n\
         shared state: State\n\
         lock gate guards state\n\
         fn id(x: u64) -> u64 ! pure requires true ensures result == x {{ x }}\n\
         fn probe() -> u64 ! owns(gate), read(state.n), write(state.n)\n\
           requires true ensures result < 10\n\
         {{ {} }}",
        body(position, &payload)
    )
}

fn source_with_payload(position: Position, declarations: &str, payload: &str) -> String {
    format!(
        "struct State {{ n: u64, m: u64 }} keeps n < 10\n\
         shared state: State\n\
         {declarations}\n\
         fn id(x: u64) -> u64 ! pure requires true ensures result == x {{ x }}\n\
         fn probe() -> u64 ! owns(gate), owns(other), read(state.n), write(state.n)\n\
           requires true ensures result < 10\n\
         {{ {} }}",
        body(position, payload)
    )
}

fn affine_source(position: Position, action: &str) -> String {
    let payload = format!("if true {{ holding gate {{ {action} }} 0 }} else {{ 0 }}");
    format!(
        "struct State {{ text: String }} keeps text.len() <= 20\n\
         shared state: State\n\
         lock gate guards state\n\
         fn id(x: u64) -> u64 ! pure requires true ensures result == x {{ x }}\n\
         fn probe() -> u64 ! owns(gate), read(state.text)\n\
           requires true ensures result == 0\n\
         {{ {} }}",
        body(position, &payload)
    )
}

fn assert_l2_shared_state_exclusion(position: Position, source: &str) {
    let parsed = parse(source);
    let error = lower_l2(&parsed.program).expect_err("RFC-10 shared-state L2 is a typed exclusion");
    assert!(
        error
            .to_string()
            .contains("RFC-10 shared-state L2 Kani harness"),
        "{position:?}: {L2_SHARED_STATE_EXCLUSION}: {error}"
    );
}

fn assert_position_is_a_canonical_holding_ancestor(position: Position, source: &str) {
    let parsed = parse(source);
    let inventory = semantic_inventory(&parsed.program, WorkBudget(100_000)).unwrap();
    let holding = inventory
        .facts
        .iter()
        .position(|fact| matches!(fact, SemanticFact::Holding { lock } if lock == "gate"))
        .expect("generated cell contains holding gate");
    let mut cursor = thermite_syntax::NodeId(holding as u32);
    let mut ancestor_roles = Vec::new();
    while let Some(edge) = inventory.edges.iter().find(|edge| edge.child == cursor) {
        ancestor_roles.push(edge.role);
        cursor = edge.parent;
    }
    assert!(
        ancestor_roles.contains(&position.canonical_role()),
        "{position:?} must be grounded in canonical role {:?}: {ancestor_roles:?}",
        position.canonical_role()
    );
}

fn rejecting_source(position: Position, payload: RejectingPayload) -> String {
    match payload {
        RejectingPayload::DirectReentrancy => source(position, "holding gate { } "),
        RejectingPayload::ReverseOrdering => source_with_payload(
            position,
            "lock other guards state.m\nlock gate guards state.n after other",
            "if true { holding gate { holding other { } state.n } 0 } else { 0 }",
        ),
        RejectingPayload::TransitiveReentrancy => {
            let nested = "if true { holding gate { leaf(); state.n } 0 } else { 0 }";
            let source = source_with_payload(
                position,
                "lock gate guards state.n\nlock other guards state.m",
                nested,
            );
            source.replace(
                "fn id(x: u64)",
                "fn leaf() -> u64 ! owns(gate) requires true ensures result == 0 { holding gate { } 0 }\nfn id(x: u64)",
            )
        }
        RejectingPayload::UnauthorisedRead => {
            let source = source(position, "");
            source.replace("holding gate {  state.n } 0", "state.n")
        }
    }
}

fn expected_rejection(payload: RejectingPayload) -> &'static str {
    match payload {
        RejectingPayload::DirectReentrancy => "reentrantly holds `gate`",
        RejectingPayload::ReverseOrdering => "takes `other` without",
        RejectingPayload::TransitiveReentrancy => "callee transitively owns the same lock",
        RejectingPayload::UnauthorisedRead => "outside `holding gate`",
    }
}

fn provider() -> LockProvider {
    LockProvider {
        name: "matrix-provider".into(),
        rust_source: "use std::cell::UnsafeCell;\nstruct MatrixStorage(UnsafeCell<State>);\nunsafe impl Sync for MatrixStorage {}\nstatic STATE: MatrixStorage = MatrixStorage(UnsafeCell::new(State { n: 0 }));\nfn __thermite_shared_state() -> &'static mut State { unsafe { &mut *STATE.0.get() } }\nfn __thermite_lock_acquire_gate() {}\nfn __thermite_lock_release_gate() {}\n".into(),
        verus_source: "fn __thermite_lock_acquire_gate() -> (state: State) ensures state.well_formed() { State { n: 0 } }\nfn __thermite_close_gate(state: &mut State) requires state.well_formed() {}\n".into(),
        proves_exclusive_acquire: true,
        proves_restore_before_release: true,
        states_interrupt_policy: true,
    }
}

fn compile_and_run_l1(position: Position, label: &str, mut source: String) -> bool {
    source.push_str("\nfn main() { assert_eq!(probe(), 0); }\n");
    let dir = std::env::temp_dir().join(format!(
        "thermite-rfc10-matrix-{}-{position:?}-{label}",
        std::process::id()
    ));
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
    assert!(built.success(), "{position:?}/{label}");
    std::process::Command::new(&binary_path)
        .status()
        .unwrap()
        .success()
}

#[test]
fn generated_holding_position_matrix_agrees_across_phases() {
    for position in POSITIONS {
        let source = source(position, "state.n = 1;");
        assert_position_is_a_canonical_holding_ancestor(position, &source);
        let parsed = parse(&source);
        assert!(
            parsed.is_clean(),
            "{position:?}: {:?}\n{source}",
            parsed.errors
        );
        let checked = check_program(&parsed.program)
            .unwrap_or_else(|errors| panic!("{position:?}: {errors:?}\n{source}"));
        assert_eq!(checked.holdings().len(), 1, "{position:?}");
        assert!(
            checked
                .shared_places()
                .iter()
                .all(|place| !place.authorizing_locks.is_empty()),
            "{position:?}"
        );
        let witness = emit_witness(&checked);
        replay_witness(&parsed.program, &witness)
            .unwrap_or_else(|error| panic!("{position:?}: {error:?}"));
        let l3 = lower(&parsed.program).unwrap_or_else(|error| panic!("{position:?}: {error}"));
        assert!(l3.contains("__thermite_lock_acquire_gate"), "{position:?}");
        let l1 = lower_l1_with_lock_provider(&parsed.program, &provider())
            .unwrap_or_else(|error| panic!("{position:?}: {error}"));
        assert!(l1.contains("__thermite_lock_acquire_gate"), "{position:?}");
        assert_l2_shared_state_exclusion(position, &source);
        assert!(compile_and_run_l1(position, "restored", l1), "{position:?}");
    }
}

#[test]
fn deep_finite_expression_payload_reaches_every_position() {
    let mut inner = "state.n".to_string();
    for _ in 0..24 {
        inner = format!("if true {{ {inner} }} else {{ 0 }}");
    }

    for position in POSITIONS {
        let source = source(position, &format!("state.n = 1; let deep: u64 = {inner};"));
        let parsed = parse(&source);
        assert!(parsed.is_clean(), "{position:?}: {:?}", parsed.errors);
        let checked = check_program(&parsed.program)
            .unwrap_or_else(|errors| panic!("{position:?}: {errors:?}\n{source}"));
        replay_witness(&parsed.program, &emit_witness(&checked))
            .unwrap_or_else(|error| panic!("{position:?}: {error:?}"));
        lower(&parsed.program).unwrap_or_else(|error| panic!("{position:?}: {error}"));
        lower_l1_with_lock_provider(&parsed.program, &provider())
            .unwrap_or_else(|error| panic!("{position:?}: {error}"));
        assert_l2_shared_state_exclusion(position, &source);
    }
}

#[test]
fn generated_affine_payload_matrix_records_compatible_cells_and_exclusions() {
    for position in POSITIONS {
        let source = affine_source(position, "let moved: String = state.text;");
        let parsed = parse(&source);
        assert!(
            parsed.is_clean(),
            "non-Copy/{position:?}: {:?}\n{source}",
            parsed.errors
        );
        let error = check_program(&parsed.program).unwrap_err();
        assert!(
            format!("{error:?}").contains("moves non-Copy shared place"),
            "non-Copy/{position:?}: {error:?}\n{source}"
        );
        assert_l2_shared_state_exclusion(position, &source);
    }

    for (position, body) in [
        (Position::Tail, "holding gate { &state.text }"),
        (
            Position::ReturnValue,
            "return if true { holding gate { &state.text } } else { holding gate { &state.text } };",
        ),
    ] {
        let escaping = parse(&format!(
            "struct State {{ text: String }} keeps text.len() <= 20\n\
             shared state: State\n\
             lock gate guards state\n\
             fn escape() -> &String ! owns(gate), read(state.text)\n\
               requires true ensures true {{ {body} }}"
        ));
        assert!(escaping.is_clean(), "{position:?}: {:?}", escaping.errors);
        let error = check_program(&escaping.program).unwrap_err();
        assert!(
            format!("{error:?}").contains("escaping reference to shared place"),
            "escaping-borrow/{position:?}: {error:?}"
        );
    }

    assert_eq!(ESCAPING_BORROW_EXCLUSIONS.len(), 8);
    for (position, reason) in ESCAPING_BORROW_EXCLUSIONS {
        assert_ne!(position, Position::Tail);
        assert!(!reason.is_empty(), "{position:?} exclusion must be typed");
    }
}

#[test]
fn generated_invariant_breaking_payload_fails_at_every_l1_close() {
    for position in POSITIONS {
        let source = source(position, "state.n = 10;");
        let parsed = parse(&source);
        assert!(parsed.is_clean(), "{position:?}: {:?}", parsed.errors);
        let checked = check_program(&parsed.program)
            .unwrap_or_else(|errors| panic!("{position:?}: {errors:?}\n{source}"));
        replay_witness(&parsed.program, &emit_witness(&checked)).unwrap();
        let l1 = lower_l1_with_lock_provider(&parsed.program, &provider()).unwrap();
        assert!(
            !compile_and_run_l1(position, "broken-invariant", l1),
            "{position:?}: an invariant-breaking close must abort"
        );
    }
}

#[test]
fn generated_negative_payloads_fail_in_every_compatible_position() {
    for position in POSITIONS {
        for payload in REJECTING_PAYLOADS {
            let source = rejecting_source(position, payload);
            let parsed = parse(&source);
            assert!(
                parsed.is_clean(),
                "{position:?}/{payload:?}: {:?}\n{source}",
                parsed.errors
            );
            let error = check_program(&parsed.program).unwrap_err();
            assert!(
                format!("{error:?}").contains(expected_rejection(payload)),
                "{position:?}/{payload:?}: {error:?}\n{source}"
            );
        }
    }
}
