use std::io::Write;
use std::process::{Command, Stdio};

use thermite_lower::{canonical_relational_frame_witness, lean_relational_frame_replay_source};
use thermite_syntax::parse;

const PROGRAM: &str = "struct State { n: u64 } keeps n < 10
shared state: State
fn bump() -> u64 ! read(state.n), write(state.n)
  requires true ensures result < 10
{ state.n = state.n + 1; state.n }";

fn lean_output(replay: &str) -> std::process::Output {
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
        .write_all(replay.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn production_body_replays_in_the_lean_kernel() {
    let parsed = parse(PROGRAM);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let witness = canonical_relational_frame_witness(&parsed.program).unwrap();
    let replay = lean_relational_frame_replay_source(&parsed.program, &witness).unwrap();
    let output = lean_output(&replay);
    assert!(
        output.status.success(),
        "Lean replay failed:\n{}\n{}\n{replay}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("THERMITE_RELATIONAL_FRAME_REPLAY_ACCEPTED_V1"));
    assert!(
        thermite_lower::emit_relational_transport_receipts(&parsed.program, &witness)
            .unwrap()
            .is_empty(),
        "shared-region body must remain source-only until region transport is proved"
    );
}

#[test]
fn production_pure_function_receives_exact_t2_transport() {
    let parsed = parse("fn identity(x: u64) -> u64 ! pure requires true ensures result == x { x }");
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let witness = canonical_relational_frame_witness(&parsed.program).unwrap();
    let receipts =
        thermite_lower::emit_relational_transport_receipts(&parsed.program, &witness).unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].function, "identity");
    assert_eq!(receipts[0].scope, thermite_lower::RelationalScope::EndToEnd);
    assert_eq!(
        receipts[0].projections,
        [
            thermite_lower::RelationalProjection::Result,
            thermite_lower::RelationalProjection::Outcome,
            thermite_lower::RelationalProjection::Termination,
        ]
    );
    assert!(
        !receipts[0]
            .projections
            .contains(&thermite_lower::RelationalProjection::WriteFrame),
        "the return transport theorem does not upgrade source region framing"
    );
    let replay = thermite_lower::lean_relational_transport_replay_source(
        &parsed.program,
        &witness,
        &receipts,
    )
    .unwrap();
    let output = lean_output(&replay);
    assert!(
        output.status.success(),
        "Lean transport replay failed:\n{}\n{}\n{replay}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("THERMITE_RELATIONAL_TRANSPORT_ACCEPTED_V1"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("sorryAx"));

    let mut mutants = Vec::new();
    let mut mutant = receipts.clone();
    mutant[0].lowered_artifact_sha256.push('0');
    mutants.push(mutant);
    let mut mutant = receipts.clone();
    mutant[0].theorem = "Thermite.forged".into();
    mutants.push(mutant);
    let mut mutant = receipts.clone();
    mutant[0].projections.pop();
    mutants.push(mutant);
    let mut mutant = receipts.clone();
    mutant[0].canonical_ast_sha256.push('0');
    mutants.push(mutant);
    for mutant in mutants {
        assert!(thermite_lower::replay_relational_transport_receipts(
            &parsed.program,
            &witness,
            &mutant,
        )
        .is_err());
    }

    let changed =
        parse("fn identity(x: u64) -> u64 ! pure requires true ensures result == x { x + x }");
    assert!(changed.is_clean(), "parse errors: {:?}", changed.errors);
    assert!(thermite_lower::replay_relational_transport_receipts(
        &changed.program,
        &witness,
        &receipts,
    )
    .is_err(), "source-valid evidence cannot cross to a different production body");
}
