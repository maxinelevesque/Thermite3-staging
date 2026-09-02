use std::io::Write;
use std::process::{Command, Stdio};

use thermite_lower::{
    canonical_resource_projection, check_program, emit_resource_witness,
    lean_resource_replay_source, replay_resource_witness, CanonicalResourceProjection,
    ResourceFlowWitness, WitnessError,
};
use thermite_syntax::parse;

const SOURCE: &str = "
resource(heap) struct HeapGrant { id: u64 }
resource(device.port) struct PortGrant { id: u64 }
resource struct Bundle { heap: HeapGrant, port: PortGrant }
fn discard(b: Bundle, c: bool) -> u64
  ! forgets(heap), forgets(device.port)
  requires true
  ensures result == 0
{
  while c keeps true measures 1 { break; }
  if c { forget(b); } else { forget(b); }
  0
}";

fn fixture() -> thermite_syntax::Program {
    let parsed = parse(SOURCE);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    parsed.program
}

fn lean_output(
    canonical: &CanonicalResourceProjection,
    witness: &ResourceFlowWitness,
) -> std::process::Output {
    let source = lean_resource_replay_source(canonical, witness);
    let lean_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../lean");
    static BUILT: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        let status = Command::new("lake")
            .args(["build", "Thermite.ResourceFlow"])
            .current_dir(&lean_root)
            .status()
            .expect("lake must be installed");
        assert!(status.success(), "resource-flow Lean module must build");
    });
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
fn resource_witness_is_deterministic_bound_and_replayed_by_lean() {
    let program = fixture();
    let checked = check_program(&program).unwrap();
    let first = emit_resource_witness(&checked);
    let second = emit_resource_witness(&checked);
    assert_eq!(first, second);
    assert_eq!(
        ResourceFlowWitness::from_json(&first.canonical_json().unwrap()).unwrap(),
        first
    );
    replay_resource_witness(&program, &first).unwrap();

    let canonical = canonical_resource_projection(&program).unwrap();
    let output = lean_output(&canonical, &first);
    assert!(
        output.status.success(),
        "Lean rejected canonical resource flow:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("THERMITE_RFC11_RESOURCE_REPLAY_ACCEPTED_V1"));
    assert!(
        stdout.contains("does not depend on any axioms"),
        "unexpected axioms: {stdout}"
    );
}

#[test]
fn rust_replay_rejects_digest_and_flow_mutations_by_named_field() {
    let program = fixture();
    let original = emit_resource_witness(&check_program(&program).unwrap());

    let mut source = original.clone();
    source.canonical_ast_sha256.push('0');
    assert_eq!(
        replay_resource_witness(&program, &source).unwrap_err(),
        WitnessError::Mismatch {
            field: "resource_canonical_ast_sha256"
        }
    );

    let mut checked = original.clone();
    checked.checked_resource_sha256.push('0');
    assert_eq!(
        replay_resource_witness(&program, &checked).unwrap_err(),
        WitnessError::Mismatch {
            field: "checked_resource_sha256"
        }
    );

    let mut flow = original;
    flow.functions[0].forgets[0].priced_regions.pop();
    assert_eq!(
        replay_resource_witness(&program, &flow).unwrap_err(),
        WitnessError::Mismatch {
            field: "resource_functions"
        }
    );
}

#[test]
fn lean_rejects_semantically_invalid_but_structurally_matching_mutations() {
    let program = fixture();
    let original = emit_resource_witness(&check_program(&program).unwrap());
    let canonical = canonical_resource_projection(&program).unwrap();

    let mut cases = Vec::new();

    let mut returning = original.clone();
    returning.functions[0].returning_edges[0]
        .live
        .push("forged".into());
    cases.push(returning);

    let mut join = original.clone();
    join.functions[0].joins[0].incoming[0].push("forged".into());
    cases.push(join);

    let mut loop_ = original.clone();
    loop_.functions[0].loops[0]
        .back_edges
        .push(vec!["forged".into()]);
    cases.push(loop_);

    let mut forget = original;
    forget.functions[0].forgets[0].priced_regions.pop();
    cases.push(forget);

    let mut duplicate_disposition = cases[0].clone();
    // Restore the returning edge changed in case 0, then duplicate a terminal
    // abandonment while changing canonical and witness together. Digest/shape
    // equality alone must not make this formally acceptable.
    duplicate_disposition.functions[0].returning_edges[0]
        .live
        .clear();
    let duplicate = duplicate_disposition.functions[0].forgets[0].clone();
    duplicate_disposition.functions[0].forgets.push(duplicate);
    cases.push(duplicate_disposition);

    for (index, witness) in cases.into_iter().enumerate() {
        let matching = CanonicalResourceProjection {
            canonical_ast_sha256: canonical.canonical_ast_sha256.clone(),
            checked_resource_sha256: canonical.checked_resource_sha256.clone(),
            functions: witness.functions.clone(),
        };
        assert!(
            !lean_output(&matching, &witness).status.success(),
            "semantic resource mutation {index} was accepted"
        );
    }
}
