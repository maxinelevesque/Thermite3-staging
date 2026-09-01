use thermite_lower::{analyze_effects, check_effects, LowerError};
use thermite_syntax::{parse, Effect, RegionPath};

fn program(source: &str) -> thermite_syntax::Program {
    let parsed = parse(source);
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    parsed.program
}

const PREFIX: &str = "resource(heap) struct Grant { id: u64 }\n\
 fn dispose(g: Grant) -> u64 ! forgets(heap) requires true ensures result == 0 \
 { forget(g); 0 }\n";

#[test]
fn forget_footprints_are_inferred_directly_and_transitively() {
    let program = program(&format!(
        "{PREFIX}fn forward(g: Grant) -> u64 ! forgets(heap) requires true ensures result == 0 \
         {{ dispose(g) }}"
    ));
    let analysis = analyze_effects(&program).expect("priced flow must pass effect analysis");
    let forget = Effect::Forgets(RegionPath::from("heap"));
    assert!(analysis.direct_footprints["dispose"].contains(&forget));
    assert!(analysis.footprints["forward"].contains(&forget));
}

#[test]
fn transitive_forget_requires_the_caller_atom() {
    let program = program(&format!(
        "{PREFIX}fn forward(g: Grant) -> u64 ! pure requires true ensures result == 0 \
         {{ dispose(g) }}"
    ));
    let errors = check_effects(&program).unwrap_err();
    assert!(errors.iter().any(|error| matches!(
        error,
        LowerError::EffectNotSubsumed { missing, .. }
            if missing.contains(&Effect::Forgets(RegionPath::from("heap")))
    )));
}
