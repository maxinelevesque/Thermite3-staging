use std::io::Write;
use std::process::{Command, Stdio};
use thermite_lower::{
    canonical_interference_projection, check_program, emit_interference_witness,
    lean_interference_replay_source, replay_interference_witness, InterferenceWitness,
    WitnessError,
};
use thermite_syntax::parse;

fn fixture() -> thermite_syntax::Program {
    let parsed = parse(
        "shared counter: u64\n\
         concurrent pair { left, right }\n\
         fn left(a: &mut u64) -> u64 ! write(counter) requires true ensures final(a) >= 0 \
           interleaves { asks final(a) >= a; promises final(a) >= a; } { 0 }\n\
         fn right(b: &mut u64) -> u64 ! write(counter) requires true ensures final(b) >= 0 \
           interleaves { asks final(b) >= b; promises final(b) >= b; } { 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    parsed.program
}

fn lean_output(
    canonical: &thermite_lower::CanonicalInterferenceProjection,
    witness: &InterferenceWitness,
) -> std::process::Output {
    let source = lean_interference_replay_source(canonical, witness);
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
fn interference_witness_is_deterministic_and_source_bound() {
    let program = fixture();
    let checked = check_program(&program).expect("checked RFC-12 program");
    let first = emit_interference_witness(&checked);
    let second = emit_interference_witness(&checked);
    assert_eq!(first, second);
    assert_eq!(
        InterferenceWitness::from_json(&first.canonical_json().unwrap()).unwrap(),
        first
    );
    replay_interference_witness(&program, &first).expect("canonical witness replays");
    let canonical = canonical_interference_projection(&program).unwrap();
    assert_eq!(canonical.functions, first.functions);
    assert_eq!(canonical.obligations, first.obligations);
    let output = lean_output(&canonical, &first);
    assert!(
        output.status.success(),
        "Lean rejected canonical interference:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("THERMITE_RFC12_INTERFERENCE_REPLAY_ACCEPTED_V1"));
    assert!(
        stdout.contains("depends on axioms: [propext]")
            && !stdout.contains("sorryAx")
            && !stdout.contains("Classical.choice")
            && !stdout.contains("Quot.sound"),
        "unexpected axiom report: {stdout}"
    );
}

#[test]
fn lean_rejects_a_peer_implication_mutation_even_when_shapes_match() {
    let program = fixture();
    let original = emit_interference_witness(&check_program(&program).unwrap());
    let mut forged = original.clone();
    forged.functions[0].promises.clear();
    let canonical = thermite_lower::CanonicalInterferenceProjection {
        canonical_ast_sha256: forged.canonical_ast_sha256.clone(),
        checked_interference_sha256: forged.checked_interference_sha256.clone(),
        functions: forged.functions.clone(),
        obligations: forged.obligations.clone(),
    };
    assert!(
        !lean_output(&canonical, &forged).status.success(),
        "Lean accepted a guarantee that no longer implies its peer rely"
    );
}

#[test]
fn replay_rejects_digest_relation_and_edge_tampering() {
    let program = fixture();
    let original = emit_interference_witness(&check_program(&program).unwrap());

    let mut source = original.clone();
    source.canonical_ast_sha256.push('0');
    assert_eq!(
        replay_interference_witness(&program, &source).unwrap_err(),
        WitnessError::Mismatch {
            field: "interference_canonical_ast_sha256"
        }
    );

    let mut digest = original.clone();
    digest.checked_interference_sha256.push('0');
    assert_eq!(
        replay_interference_witness(&program, &digest).unwrap_err(),
        WitnessError::Mismatch {
            field: "checked_interference_sha256"
        }
    );

    let mut relation = original.clone();
    relation.functions[0].asks.clear();
    assert_eq!(
        replay_interference_witness(&program, &relation).unwrap_err(),
        WitnessError::Mismatch {
            field: "interference_functions"
        }
    );

    let mut edge = original;
    edge.obligations[0].guarantor = "forged".to_string();
    assert_eq!(
        replay_interference_witness(&program, &edge).unwrap_err(),
        WitnessError::Mismatch {
            field: "interference_obligations"
        }
    );
}
