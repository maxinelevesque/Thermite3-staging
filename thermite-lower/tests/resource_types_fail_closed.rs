use std::process::Command;
use thermite_lower::{check_program, lower, lower_l1, LowerError};
use thermite_syntax::parse;

#[test]
fn l1_emits_explicit_abandonment_while_l3_remains_fail_closed() {
    let parsed = parse(
        "resource(heap) struct Grant { id: u64 }\n\
         fn discard(g: Grant) -> u64\n\
           ! forgets(heap)\n\
           requires true\n\
           ensures result == 0\n\
         { forget(g); 0 }\n\
         fn main() -> ()\n\
           ! forgets(heap)\n\
           requires true\n\
           ensures true\n\
         { let g: Grant = Grant { id: 1 }; discard(g); return; }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);

    let assert_resource_refusal = |error: &LowerError| match error {
        LowerError::Unsupported { what, .. } => {
            assert!(
                what.contains("ownership flow"),
                "unexpected refusal: {what}"
            )
        }
        other => panic!("unexpected refusal: {other:?}"),
    };

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
    std::fs::write(&source, &l1).unwrap();
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
    std::fs::remove_dir_all(fixture).unwrap();
    assert_resource_refusal(&lower(&parsed.program).unwrap_err());
}
