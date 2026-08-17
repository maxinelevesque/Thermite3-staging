use thermite_lower::{lower_l1_artifact, L1Route};

fn parse(source: &str) -> thermite_syntax::Program {
    let parsed = thermite_syntax::parse(source);
    assert!(
        parsed.is_clean(),
        "fixture must parse clean: {:?}",
        parsed.errors
    );
    parsed.program
}

#[test]
fn checked_artifact_binds_source_item_and_runtime_route() {
    let program = parse("fn id(x: u32) -> u32 ! pure requires x < 100 ensures result == x { x }");
    let artifact = lower_l1_artifact(&program, "id").expect("checked L1 artifact");
    assert!(artifact.source().contains("fn id("));
    assert_eq!(artifact.item(), "id");
    assert_eq!(artifact.effect_row(), &thermite_syntax::EffectRow::Pure);
    assert!(artifact
        .wrapper_identity()
        .starts_with("thermite-l1-wrapper-v1:id:sha256:"));
    assert_eq!(artifact.classifier_fragment(), "thermite-l1-runtime-v1");
    assert_eq!(artifact.route(), &L1Route::Runtime);
}

#[test]
fn route_classification_preserves_slag_ffi_and_divergence() {
    let slag = parse(
        "#[slag(reason = \"vendored\", owner = \"agent:forge-7\", review = \"required\")] \
         fn s(x: u32) -> u32 ! pure requires x < 100 ensures result == x { x }",
    );
    let slag = lower_l1_artifact(&slag, "s").expect("slag artifact");
    assert_eq!(slag.route(), &L1Route::Slag);
    assert_eq!(slag.classifier_fragment(), "thermite-l1-slag-v1");

    let boundary = parse(
        "#[boundary(\"ext::read\")] \
         fn b(x: u32) -> u32 ! pure requires x < 100 ensures result == x ;",
    );
    let boundary = lower_l1_artifact(&boundary, "b").expect("boundary artifact");
    assert_eq!(
        boundary.route(),
        &L1Route::Boundary {
            target: "ext::read".to_string()
        }
    );
    assert_eq!(boundary.classifier_fragment(), "thermite-l1-boundary-v1");

    let diverge = parse(
        "fn spin(n: u64) -> u64 ! diverge requires n <= 100 ensures result == 0 \
         { if n == 0 { 0 } else { spin(n - 1) } }",
    );
    let diverge = lower_l1_artifact(&diverge, "spin").expect("diverge artifact");
    assert_eq!(diverge.route(), &L1Route::Diverge);
    assert_eq!(diverge.classifier_fragment(), "thermite-l1-diverge-v1");
}

#[test]
fn artifact_refuses_non_function_and_empty_boundary_target() {
    let program = parse("spec fn ghost(x: u32) -> u32 measures x { x }");
    assert!(lower_l1_artifact(&program, "ghost").is_err());

    let boundary = parse(
        "#[boundary(\"\")] fn b(x: u32) -> u32 ! pure requires x < 100 ensures result == x ;",
    );
    assert!(lower_l1_artifact(&boundary, "b").is_err());
}
