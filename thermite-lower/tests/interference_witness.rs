use std::io::Write;
use std::process::{Command, Stdio};
use thermite_lower::{
    canonical_interference_projection, check_program, emit_interference_witness,
    lean_interference_replay_source, replay_interference_witness, score_promise_trace_mutations,
    InterferenceWitness, PromiseTraceMutationOutcome, WitnessError,
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

fn nested_region_fixture() -> thermite_syntax::Program {
    let parsed = parse(
        "struct State { n: u64 }\n\
         shared counter: State\n\
         concurrent pair { left, right }\n\
         #[boundary(\"ext::left\")] fn left(a: &mut State) -> u64 ! write(counter) requires true ensures true \
           interleaves { asks final(a.n) >= a.n; promises final(a.n) >= a.n; };\n\
         #[boundary(\"ext::right\")] fn right(b: &mut State) -> u64 ! write(counter) requires true ensures true \
           interleaves { asks final(b.n) >= b.n; promises final(b.n) >= b.n; };",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    parsed.program
}

fn inferred_write_fixture(write_body: bool) -> thermite_syntax::Program {
    let body = if write_body {
        "{ counter = counter + 1; counter }"
    } else {
        "{ 0 }"
    };
    let parsed = parse(&format!(
        "shared counter: u64\n\
         fn increment() -> u64 ! read(counter), write(counter) requires true ensures true \
           interleaves {{ asks final(counter) >= counter; promises final(counter) >= counter; }} {body}"
    ));
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
fn interference_witness_and_promise_mutation_replay_are_deterministic_and_source_bound() {
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
    assert!(first
        .functions
        .iter()
        .all(|function| function.observed_writes == ["counter"]));
    let score = score_promise_trace_mutations(&first);
    assert_eq!(score.killed, 6);
    assert_eq!(score.survived, 0);
    assert_eq!(score.unsupported, 0);
    assert!(score
        .cases
        .iter()
        .all(|case| case.outcome == PromiseTraceMutationOutcome::Killed));
    let output = lean_output(&canonical, &first);
    assert!(
        output.status.success(),
        "Lean rejected canonical interference:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("THERMITE_RFC12_INTERFERENCE_REPLAY_ACCEPTED_V1"));
    assert!(stdout.contains("THERMITE_RFC12_PROMISE_MUTATION_REPLAY_ACCEPTED_V1"));
    assert!(
        stdout.contains("depends on axioms: [propext]")
            && !stdout.contains("sorryAx")
            && !stdout.contains("Classical.choice")
            && !stdout.contains("Quot.sound"),
        "unexpected axiom report: {stdout}"
    );
}

#[test]
fn promise_mutation_scoring_reports_unsupported_observables_explicitly() {
    let program = inferred_write_fixture(false);
    let witness = emit_interference_witness(&check_program(&program).unwrap());

    let score = score_promise_trace_mutations(&witness);
    assert_eq!(score.killed, 0);
    assert_eq!(score.survived, 0);
    assert_eq!(score.unsupported, 3);
    assert!(score
        .cases
        .iter()
        .all(|case| { case.outcome == PromiseTraceMutationOutcome::Unsupported }));
}

#[test]
fn promise_trace_uses_inferred_in_language_shared_writes() {
    let program = inferred_write_fixture(true);
    let witness = emit_interference_witness(&check_program(&program).unwrap());
    assert_eq!(witness.functions[0].observed_writes, ["counter"]);

    let score = score_promise_trace_mutations(&witness);
    assert_eq!((score.killed, score.survived, score.unsupported), (3, 0, 0));
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
fn lean_uses_region_overlap_for_nested_relation_coverage() {
    let program = nested_region_fixture();
    let checked = check_program(&program).expect("Rust accepts nested relation coverage");
    let witness = emit_interference_witness(&checked);
    assert_eq!(witness.requirements[0].overlaps, ["counter"]);
    assert!(witness.functions.iter().all(|function| {
        function.asks.iter().all(|atom| atom.place == "counter.n")
            && function
                .promises
                .iter()
                .all(|atom| atom.place == "counter.n")
    }));

    let canonical = canonical_interference_projection(&program).unwrap();
    let output = lean_output(&canonical, &witness);
    assert!(
        output.status.success(),
        "Lean must match Rust's ancestor/descendant region-overlap coverage:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let mut uncovered = witness;
    for function in &mut uncovered.functions {
        for atom in &mut function.asks {
            atom.place = "other.n".to_string();
        }
        for atom in &mut function.promises {
            atom.place = "other.n".to_string();
        }
    }
    let forged_canonical = thermite_lower::CanonicalInterferenceProjection {
        canonical_ast_sha256: uncovered.canonical_ast_sha256.clone(),
        checked_interference_sha256: uncovered.checked_interference_sha256.clone(),
        functions: uncovered.functions.clone(),
        requirements: uncovered.requirements.clone(),
        obligations: uncovered.obligations.clone(),
    };
    assert!(
        !lean_output(&forged_canonical, &uncovered).status.success(),
        "Lean accepted a relation on a disjoint region"
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
