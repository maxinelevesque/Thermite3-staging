use thermite_lower::lower_l3_artifact;

fn parse(source: &str) -> thermite_syntax::Program {
    let parsed = thermite_syntax::parse(source);
    assert!(parsed.is_clean(), "fixture parse: {:?}", parsed.errors);
    parsed.program
}

#[test]
fn artifact_binds_isolated_source_item_effects_and_classifier() {
    let program = parse("fn id(x: u32) -> u32 ! time requires x < 100 ensures result == x { x }");
    let artifact = lower_l3_artifact(&program, "id").expect("checked Verus artifact");
    assert!(artifact.source().contains("fn id("));
    assert_eq!(artifact.item(), "id");
    assert_eq!(
        artifact.effect_row(),
        Some(&thermite_syntax::EffectRow::Set(vec![
            thermite_syntax::Effect::Time
        ]))
    );
    assert_eq!(artifact.classifier_fragment(), "thermite-verus-v1");
    assert!(artifact
        .query_identity()
        .starts_with("thermite-verus-query-v1:id:sha256:"));
}

#[test]
fn artifact_identity_changes_with_the_exact_query_and_rejects_missing_item() {
    let first = parse("fn f(x: u32) -> u32 ! pure requires x < 10 ensures result == x { x }");
    let second = parse("fn f(x: u32) -> u32 ! pure requires x < 20 ensures result == x { x }");
    let first = lower_l3_artifact(&first, "f").unwrap();
    let second = lower_l3_artifact(&second, "f").unwrap();
    assert_ne!(first.query_identity(), second.query_identity());
    assert!(lower_l3_artifact(
        &parse("spec fn g(x: u32) -> u32 measures x { x }"),
        "missing"
    )
    .is_err());
}
