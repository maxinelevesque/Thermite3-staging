//! RFC-8 declaration pipeline boundary: declarations resolve during validation
//! and remain metadata rather than becoming a new verified-core constructor.

use thermite_lower::{check_effects, lower};
use thermite_spec::validate;
use thermite_syntax::{parse, Item};

#[test]
fn declaration_resolves_before_lowering_and_emits_no_verified_construct() {
    let parsed = parse(
        "effect platform(d) = state(d) + io(sigma_d)\n\
         fn id(x: u64) -> u64 ! pure requires true ensures result == x { x }",
    );
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert!(matches!(parsed.program.items[0], Item::EffectDecl(_)));

    validate(&parsed.program).expect("basis declaration must validate");
    check_effects(&parsed.program).expect("declaration metadata has no effect row of its own");
    let lowered = lower(&parsed.program).expect("declaration must not obstruct lowering");

    assert!(lowered.contains("fn id("));
    assert!(!lowered.contains("effect platform"));
    assert!(!lowered.contains("sigma_d"));
}
