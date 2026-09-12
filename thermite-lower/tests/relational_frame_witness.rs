use std::io::Write;
use std::process::{Command, Stdio};

use thermite_lower::{canonical_relational_frame_witness, lean_relational_frame_replay_source};
use thermite_syntax::parse;

const PROGRAM: &str = "struct State { n: u64 } keeps n < 10
shared state: State
fn bump() -> u64 ! read(state.n), write(state.n)
  requires true ensures result < 10
{ state.n = state.n + 1; state.n }";

#[test]
fn production_body_replays_in_the_lean_kernel() {
    let parsed = parse(PROGRAM);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let witness = canonical_relational_frame_witness(&parsed.program).unwrap();
    let replay = lean_relational_frame_replay_source(&parsed.program, &witness).unwrap();
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
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "Lean replay failed:\n{}\n{}\n{replay}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("THERMITE_RELATIONAL_FRAME_REPLAY_ACCEPTED_V1"));
}
