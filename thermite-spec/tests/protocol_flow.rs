use thermite_spec::{check_protocols, ProtocolAction, ProtocolErrorKind};
use thermite_syntax::parse;

fn program(body: &str, effects: &str) -> thermite_syntax::Program {
    let source = format!(
        r#"
protocol PageRequest {{
  User {{ op: u32, count: u64 }},
  Provider {{ status: u32, base: u64 }},
  end
}}
fn pager(c: PageRequest::Provider) -> ()
  ! {effects}
  requires true
  ensures true
{{ {body} }}
"#
    );
    let parsed = parse(&source);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    parsed.program
}

#[test]
fn accepts_the_provider_projection_to_completion() {
    let report = check_protocols(&program("c.receive(); c.send(0, 0);", "blocks"))
        .expect("valid projected flow");
    let flow = &report.functions["pager"];
    assert_eq!(flow.transitions.len(), 2);
    assert_eq!(flow.completed, ["c"]);
    let definition = &report.definitions["PageRequest"];
    assert!(definition.compatible);
    assert!(matches!(
        definition.projections["Provider"][0],
        ProtocolAction::Receive(_)
    ));
    assert!(matches!(
        definition.projections["User"][0],
        ProtocolAction::Send(_)
    ));
}

#[test]
fn rejects_wrong_role_and_unfinished_return() {
    let errors = check_protocols(&program("c.send(0, 0);", "blocks"))
        .expect_err("provider cannot send the user's first turn");
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::WrongTurn));
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::UnfinishedEndpoint));
}

#[test]
fn rejects_payload_mismatch_duplication_and_missing_blocks() {
    let errors = check_protocols(&program("c.receive(); c.send(0); let d = c;", "pure"))
        .expect_err("all three protocol violations must be caught");
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::PayloadMismatch));
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::DuplicateEndpoint));
    assert!(errors.iter().any(|error| {
        error.kind == ProtocolErrorKind::WrongTurn && error.detail.contains("blocks")
    }));
}

#[test]
fn rejects_payload_type_mismatch() {
    let errors = check_protocols(&program("c.receive(); c.send(true, 0);", "blocks"))
        .expect_err("a bool cannot inhabit the declared u32 payload field");
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::PayloadMismatch));
}

#[test]
fn rejects_typed_variable_payload_mismatch() {
    let parsed = parse(
        r#"
protocol P { A { value: u32 }, B { reply: u32 }, end }
fn bad(c: P::A, value: bool) -> () ! blocks requires true ensures true
{ c.send(value); c.receive(); }
"#,
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = check_protocols(&parsed.program)
        .expect_err("a typed bool variable cannot inhabit a u32 payload field");
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::PayloadMismatch));
}

#[test]
fn rejects_protocol_actions_in_unmodeled_control_flow() {
    let parsed = parse(
        r#"
protocol P { A { value: u32 }, B { reply: u32 }, end }
fn bad(c: P::A, choose: bool) -> () ! blocks requires true ensures true
{ if choose { c.send(0); } else { c.send(1); } c.receive(); }
"#,
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = check_protocols(&parsed.program)
        .expect_err("v1 must not flatten two branch paths into one proof witness");
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::UnsupportedControlFlow));
}

#[test]
fn receive_payload_requires_and_checks_an_explicit_projected_type() {
    let valid = program(
        "let request: (u32, u64) = c.receive_payload(); c.send(0, 0);",
        "blocks",
    );
    check_protocols(&valid).expect("the receiver may bind its projected payload tuple");

    let invalid = program(
        "let request: bool = c.receive_payload(); c.send(0, 0);",
        "blocks",
    );
    let errors =
        check_protocols(&invalid).expect_err("a receive binding cannot lie about payload type");
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::PayloadMismatch));
}

#[test]
fn rejects_endpoint_in_shared_state() {
    let parsed = parse("protocol P { A { x: u32 }, B { y: u32 }, end } shared channel: P::A");
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = check_protocols(&parsed.program).expect_err("shared endpoint must fail closed");
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::SharedEndpoint));
}

#[test]
fn repeat_tail_requires_an_explicit_exit_choice() {
    let parsed = parse(
        r#"
protocol Stream { User { op: u32 }, Provider { value: u64 }, repeat | end }
fn serve(c: Stream::Provider) -> () ! blocks requires true ensures true
{ c.receive(); c.send(0); }
"#,
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = check_protocols(&parsed.program).expect_err("repeat tail cannot be abandoned");
    assert!(errors
        .iter()
        .any(|error| error.kind == ProtocolErrorKind::UnfinishedEndpoint));
}

#[test]
fn repeat_tail_distinguishes_chooser_from_peer() {
    let parsed = parse(
        r#"
protocol Stream { User { op: u32 }, Provider { value: u64 }, repeat | end }
fn user(c: Stream::User) -> () ! blocks requires true ensures true
{ c.send(0); c.receive(); c.end(); }
fn provider(c: Stream::Provider) -> () ! blocks requires true ensures true
{ c.receive(); c.send(0); c.receive_end(); }
"#,
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    check_protocols(&parsed.program).expect("dual repeat/end projections must complete");
}
