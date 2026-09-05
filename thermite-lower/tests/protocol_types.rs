use std::process::Command;

use thermite_lower::{
    check_program, emit_protocol_witness, lower, lower_l1, lower_l3_artifact,
    replay_protocol_witness,
};
use thermite_syntax::parse;

const PROGRAM: &str = r#"
protocol PageRequest {
  User { op: u32, count: u64 },
  Provider { status: u32, base: u64 },
  end
}
fn pager(c: PageRequest::Provider) -> ()
  ! blocks
  requires true
  ensures true
{ c.receive(); c.send(200, 4096); }
fn app(c: PageRequest::User) -> ()
  ! blocks
  requires true
  ensures true
{ c.send(1, 2); c.receive(); }
"#;

fn parsed() -> thermite_syntax::Program {
    let parsed = parse(PROGRAM);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    parsed.program
}

#[test]
fn checked_program_binds_protocol_flow_evidence() {
    let program = parsed();
    let checked = check_program(&program).expect("protocol flow must check before lowering");
    assert_eq!(
        checked.protocol_flow().functions["pager"].transitions.len(),
        2
    );
    assert_eq!(checked.protocol_flow().functions["app"].completed, ["c"]);
}

#[test]
fn l1_protocol_carriers_compile_and_run() {
    let program = parsed();
    let emitted = lower_l1(&program).expect("checked protocol L1 lowering");
    assert!(emitted.contains("__thermite_protocol_send_2"));
    assert!(emitted.contains("__thermite_protocol_receive"));

    let fixture = std::env::temp_dir().join(format!("thermite-rfc13-l1-{}", std::process::id()));
    std::fs::create_dir_all(&fixture).unwrap();
    let source = fixture.join("protocol.rs");
    let binary = fixture.join("protocol-bin");
    let runnable = format!(
        "{emitted}\nfn main() {{ pager(PageRequest_Provider_Endpoint {{ step: 0 }}); app(PageRequest_User_Endpoint {{ step: 0 }}); }}\n"
    );
    std::fs::write(&source, &runnable).unwrap();
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("rustc must run");
    assert!(
        output.status.success(),
        "lowered protocol L1 did not compile: {}\n{runnable}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(Command::new(&binary).status().unwrap().success());
    std::fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn l3_protocol_carriers_expose_the_platform_boundary() {
    let emitted = lower(&parsed()).expect("checked protocol L3 lowering");
    assert!(emitted.contains("#[verifier::external_body]"));
    assert!(emitted.contains("c.__thermite_protocol_send_2(200, 4096)"));
    assert!(emitted.contains("c.__thermite_protocol_receive()"));
}

#[test]
fn l3_artifact_binds_the_checked_protocol_witness() {
    let artifact = lower_l3_artifact(&parsed(), "pager").expect("protocol L3 artifact");
    let witness = artifact
        .protocol_witness()
        .expect("protocol artifact must carry its checked witness");
    assert!(artifact
        .query_identity()
        .contains(&witness.checked_protocol_sha256));
    assert!(witness.definitions[0].compatible);
}

#[test]
fn protocol_witness_replay_rejects_tampering() {
    let program = parsed();
    let checked = check_program(&program).expect("checked protocol");
    let witness = emit_protocol_witness(&checked);
    replay_protocol_witness(&program, &witness).expect("untampered witness replays");

    let mut tampered = witness;
    tampered.functions[0].completed.clear();
    assert!(replay_protocol_witness(&program, &tampered).is_err());
}

#[test]
fn lowering_fails_closed_on_wrong_turn() {
    let parsed = parse(
        "protocol P { A { x: u32 }, B { y: u32 }, end } \
         fn bad(c: P::B) -> () ! blocks requires true ensures true { c.send(0); }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let error = lower_l1(&parsed.program).expect_err("invalid projection must not lower");
    assert!(error.to_string().contains("must `receive`"), "{error}");
}
