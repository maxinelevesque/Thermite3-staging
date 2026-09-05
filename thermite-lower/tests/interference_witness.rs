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
         #[boundary(\"ext::left\")] fn left(a: &mut u64) -> u64 ! write(counter) requires true ensures final(a) >= 0 \
           interleaves { asks final(a) >= a; promises final(a) >= a; };\n\
         #[boundary(\"ext::right\")] fn right(b: &mut u64) -> u64 ! write(counter) requires true ensures final(b) >= 0 \
           interleaves { asks final(b) >= b; promises final(b) >= b; };",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    parsed.program
}

fn handler_fixture() -> thermite_syntax::Program {
    let parsed = parse(
        "shared counter: u64\n\
         handlers { low at 1, high at 2 }\n\
         #[boundary(\"ext::low\")] fn low(a: &mut u64) -> u64 ! write(counter) requires true ensures true \
           interleaves { asks final(a) >= a; promises final(a) >= a; };\n\
         #[boundary(\"ext::high\")] fn high(b: &mut u64) -> u64 ! write(counter) requires true ensures true \
           interleaves { asks final(b) >= b; promises final(b) >= b; };",
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
        requirements: forged.requirements.clone(),
        obligations: forged.obligations.clone(),
    };
    assert!(
        !lean_output(&canonical, &forged).status.success(),
        "Lean accepted a guarantee that no longer implies its peer rely"
    );
}

#[test]
fn lean_derives_graph_completeness_and_rejects_a_reversed_edge() {
    let program = fixture();
    let mut forged = emit_interference_witness(&check_program(&program).unwrap());
    forged.obligations[0] = forged.obligations[1].clone();
    let canonical = thermite_lower::CanonicalInterferenceProjection {
        canonical_ast_sha256: forged.canonical_ast_sha256.clone(),
        checked_interference_sha256: forged.checked_interference_sha256.clone(),
        functions: forged.functions.clone(),
        requirements: forged.requirements.clone(),
        obligations: forged.obligations.clone(),
    };
    assert!(
        !lean_output(&canonical, &forged).status.success(),
        "Lean accepted an obligation graph with a reversed/duplicated edge"
    );
}

#[test]
fn lean_derives_handler_direction_from_priorities() {
    let program = handler_fixture();
    let original = emit_interference_witness(&check_program(&program).unwrap());
    let canonical = canonical_interference_projection(&program).unwrap();
    assert!(lean_output(&canonical, &original).status.success());
    assert_eq!(original.obligations.len(), 1);
    assert_eq!(original.obligations[0].guarantor, "high");

    let mut forged = original;
    let guarantor = forged.obligations[0].guarantor.clone();
    forged.obligations[0].guarantor = forged.obligations[0].relying.clone();
    forged.obligations[0].relying = guarantor;
    let forged_canonical = thermite_lower::CanonicalInterferenceProjection {
        canonical_ast_sha256: forged.canonical_ast_sha256.clone(),
        checked_interference_sha256: forged.checked_interference_sha256.clone(),
        functions: forged.functions.clone(),
        requirements: forged.requirements.clone(),
        obligations: forged.obligations.clone(),
    };
    assert!(
        !lean_output(&forged_canonical, &forged).status.success(),
        "Lean accepted the impossible low-priority-to-high-priority edge"
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

    let mut requirement = original.clone();
    requirement.requirements[0].left_priority = Some(9);
    assert_eq!(
        replay_interference_witness(&program, &requirement).unwrap_err(),
        WitnessError::Mismatch {
            field: "interference_requirements"
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
