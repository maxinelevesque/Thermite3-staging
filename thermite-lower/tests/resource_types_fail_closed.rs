use std::process::Command;
use thermite_lower::{check_program, contains_rfc11, lower, lower_l1, lower_l3_artifact};
use thermite_syntax::parse;

#[test]
fn l1_and_l3_emit_explicit_abandonment_with_a_bound_resource_witness() {
    let parsed = parse(
        "resource(heap) struct Grant { id: u64 }\n\
         fn discard(g: Grant) -> u64\n\
           ! forgets(heap)\n\
           requires true\n\
           ensures result == 0\n\
         { forget(g); 0 }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);

    let checked = check_program(&parsed.program).expect("resource flow must check");
    assert_eq!(checked.resource_flow().direct_forgets["discard"].len(), 1);
    let l1 = lower_l1(&parsed.program).expect("L1 has explicit resource abandonment");
    assert!(l1.contains("drop(g);"), "explicit drop absent: {l1}");
    assert!(
        !l1.contains("clone()"),
        "resource lowering must not clone: {l1}"
    );
    let fixture = std::env::temp_dir().join(format!("thermite-rfc11-l1-{}", std::process::id()));
    std::fs::create_dir_all(&fixture).unwrap();
    let source = fixture.join("resource.rs");
    let binary = fixture.join("resource-bin");
    let runnable =
        format!("{l1}\nfn main() {{ let g = Grant {{ id: 1 }}; assert_eq!(discard(g), 0); }}\n");
    std::fs::write(&source, runnable).unwrap();
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("rustc must run");
    assert!(
        output.status.success(),
        "lowered L1 did not compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(Command::new(&binary).status().unwrap().success());
    let l3 = lower(&parsed.program).expect("L3 resource lowering is flow-witness gated");
    assert!(
        l3.contains("let _ = g; // RFC-11 checked forget"),
        "explicit L3 forget sink absent: {l3}"
    );
    let verus_source = fixture.join("resource_verus.rs");
    std::fs::write(&verus_source, &l3).unwrap();
    if let Ok(verus) = Command::new("which").arg("verus").output() {
        if verus.status.success() {
            let binary = String::from_utf8_lossy(&verus.stdout).trim().to_string();
            let output = Command::new(binary)
                .arg(&verus_source)
                .current_dir(&fixture)
                .output()
                .unwrap();
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                output.status.success() && combined.contains("verified, 0 errors"),
                "Verus rejected RFC-11 L3:\n{combined}\n{l3}"
            );
        }
    }
    let artifact = lower_l3_artifact(&parsed.program, "discard").unwrap();
    let witness = artifact
        .resource_witness()
        .expect("resource L3 artifact must bind its flow witness");
    assert!(artifact
        .query_identity()
        .contains(&witness.checked_resource_sha256));
    std::fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn expression_nested_forget_is_detected_at_the_certification_boundary() {
    let parsed = parse(
        "fn release(x: u64) -> u64\n\
         ! forgets(heap)\n\
         requires true\n\
         ensures true\n\
         { let y: u64 = if x > 0 { forget(x); x } else { x }; y }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    assert!(
        contains_rfc11(&parsed.program),
        "the whole-program certificate boundary must see expression-nested RFC-11 syntax"
    );
    assert!(
        check_program(&parsed.program).is_err(),
        "the unchecked forget must fail before lowering or certification"
    );
}
