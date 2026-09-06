use std::io::Write;
use std::process::{Command, Stdio};

use thermite_lower::{
    canonical_protocol_projection, check_program, emit_protocol_witness,
    lean_protocol_replay_source, lower, lower_l1, lower_l3_artifact, replay_protocol_witness,
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

fn lean_output(
    canonical: &thermite_lower::CanonicalProtocolProjection,
    witness: &thermite_lower::ProtocolWitness,
) -> std::process::Output {
    let source = lean_protocol_replay_source(canonical, witness);
    let lean_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../lean");
    let mut child = Command::new("lake")
        .args(["env", "lean", "--stdin", "--threads=1"])
        .current_dir(lean_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("lake/lean must be installed");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
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
    assert!(
        checked.effects().warnings.is_empty(),
        "endpoint operations must justify their required `blocks` effect"
    );
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
fn lean_replays_duality_and_completion_without_sorry() {
    let program = parsed();
    let checked = check_program(&program).unwrap();
    let witness = emit_protocol_witness(&checked);
    let canonical = canonical_protocol_projection(&program).unwrap();
    let output = lean_output(&canonical, &witness);
    assert!(
        output.status.success(),
        "Lean rejected canonical protocol witness:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("THERMITE_RFC13_PROTOCOL_REPLAY_ACCEPTED_V1"));
    assert!(
        !stdout.contains("sorryAx"),
        "unexpected axiom report: {stdout}"
    );
}

#[test]
fn lean_rejects_non_dual_projections_even_when_columns_match() {
    let program = parsed();
    let mut witness = emit_protocol_witness(&check_program(&program).unwrap());
    witness.definitions[0].projections[0].actions[0].kind = "send".into();
    witness.definitions[0].projections[1].actions[0].kind = "send".into();
    let canonical = thermite_lower::CanonicalProtocolProjection {
        canonical_ast_sha256: witness.canonical_ast_sha256.clone(),
        checked_protocol_sha256: witness.checked_protocol_sha256.clone(),
        definitions: witness.definitions.clone(),
        functions: witness.functions.clone(),
    };
    assert!(
        !lean_output(&canonical, &witness).status.success(),
        "Lean accepted two send projections at one turn"
    );
}

#[test]
fn lean_rejects_claimed_completion_without_transitions() {
    let program = parsed();
    let mut witness = emit_protocol_witness(&check_program(&program).unwrap());
    witness.functions[0].transitions.clear();
    let canonical = thermite_lower::CanonicalProtocolProjection {
        canonical_ast_sha256: witness.canonical_ast_sha256.clone(),
        checked_protocol_sha256: witness.checked_protocol_sha256.clone(),
        definitions: witness.definitions.clone(),
        functions: witness.functions.clone(),
    };
    assert!(
        !lean_output(&canonical, &witness).status.success(),
        "Lean accepted completion with no projected protocol actions"
    );
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

#[test]
fn received_payload_can_be_bound_at_its_projected_type() {
    let parsed = parse(
        "protocol P { A { x: u32, y: u64 }, B { z: u32 }, end } \
         fn b(c: P::B) -> () ! blocks requires true ensures true \
         { let request: (u32, u64) = c.receive_payload(); c.send(0); }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let emitted = lower_l1(&parsed.program).expect("typed receive must lower");
    assert!(emitted.contains("__thermite_protocol_receive_payload"));
    assert!(
        emitted.contains("c.__thermite_protocol_receive_payload()"),
        "{emitted}"
    );
}
