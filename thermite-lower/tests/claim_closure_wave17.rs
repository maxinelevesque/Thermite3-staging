fn pins(text: &str, expected: &[&str]) {
    for pin in expected {
        assert!(text.contains(pin), "missing claim pin {pin}");
    }
}

fn ordered(text: &str, expected: &[&str]) {
    let mut cursor = 0;
    for pin in expected {
        let offset = text[cursor..]
            .find(pin)
            .unwrap_or_else(|| panic!("missing ordered claim pin {pin}"));
        cursor += offset + pin.len();
    }
}

#[test]
fn skill_combinator_section_is_registry_driven() {
    pins(
        include_str!("../../thermite-skill/src/generate.rs"),
        &[
            "fn render_combinators()",
            "thermite_spec::all()",
            "example_for(sig.name)",
        ],
    );
}

#[test]
fn skill_committed_output_is_freshness_checked() {
    pins(
        include_str!("../../thermite-skill/tests/skill.rs"),
        &[
            "fn committed_skill_is_fresh()",
            "generate()",
            "THERMITE.skill.md",
        ],
    );
}

#[test]
fn skill_curated_prose_has_deterministic_renderers() {
    let generator = include_str!("../../thermite-skill/src/generate.rs");
    ordered(
        generator,
        &[
            "fn render_forge()",
            "fn render_ladder()",
            "fn render_slag()",
        ],
    );
    pins(generator, &["fn grammar_forge_slag_coverage()"]);
}

#[test]
fn skill_grammar_rendering_is_variant_exhaustive() {
    pins(
        include_str!("../../thermite-skill/src/generate.rs"),
        &[
            "fn render_type_arm(ty: &Type)",
            "fn render_item_arm(item: &Item)",
            "fn render_expr_arm(expr: &Expr)",
            "fn render_pattern_arm(pat: &Pattern)",
            "fn render_effect_arm(effect: &Effect)",
            "fn renderers_are_exhaustive_no_wildcard()",
        ],
    );
}

#[test]
fn skill_surface_freshness_is_compile_forced() {
    pins(
        include_str!("../../thermite-skill/src/generate.rs"),
        &[
            "fn type_inventory()",
            "fn expr_inventory()",
            "fn item_inventory()",
            "fn pattern_inventory()",
            "fn effect_inventory()",
            "fn renderers_are_exhaustive_no_wildcard()",
        ],
    );
}

#[test]
fn skill_curated_prose_is_bound_to_committed_output() {
    pins(
        include_str!("../../thermite-skill/tests/skill.rs"),
        &["fn committed_skill_is_fresh()", "generate()"],
    );
    pins(
        include_str!("../../thermite-skill/src/generate.rs"),
        &["fn grammar_forge_slag_coverage()"],
    );
    pins(
        include_str!("../../THERMITE.skill.md"),
        &[
            "## 3. Forge methods",
            "## 4. Verification ladder",
            "## 5. Slag rules",
        ],
    );
}

#[test]
fn skill_scheme_section_is_registry_driven() {
    pins(
        include_str!("../../thermite-skill/src/generate.rs"),
        &[
            "fn render_schemes()",
            "schemes::all()",
            "scheme_example_for(sig.name)",
            "fn scheme_coverage()",
        ],
    );
}

#[test]
fn skill_v2_forge_tier_carries_closed_agent_guidance() {
    let generator = include_str!("../../thermite-skill/src/generate.rs");
    pins(
        generator,
        &[
            "fn render_forge_tier()",
            "The seven verdicts",
            "CovenantRefuted",
            "KernelBudget",
            "auto|nlsat|verus|lean|forge|bv",
            "witness { inhabit (args); falsify N; }",
            "burn receipt",
        ],
    );
    pins(
        include_str!("../../thermite-skill/tests/skill.rs"),
        &[
            "fn forge_tier_section_present()",
            "fn skill_is_under_budget()",
        ],
    );
}

#[test]
fn spec_combinators_carry_executable_l1_bodies() {
    pins(
        include_str!("../../thermite-spec/src/combinators.rs"),
        &[
            "pub struct CombinatorSig",
            "pub l1: &'static str",
            "fn forall_in(s: &[u32]",
            "fn count_where(s: &[u32]",
        ],
    );
}

#[test]
fn spec_combinator_shape_is_structural_and_typed() {
    pins(
        include_str!("../../thermite-spec/src/combinators.rs"),
        &[
            "pub enum ArgKind",
            "pub enum ResultKind",
            "pub name: &'static str",
            "pub arity: usize",
            "pub arg_kinds: &'static [ArgKind]",
            "pub result: ResultKind",
        ],
    );
}

#[test]
fn spec_combinators_carry_frozen_verus_l3_bodies() {
    pins(
        include_str!("../../thermite-spec/src/combinators.rs"),
        &[
            "pub verus_l3: &'static str",
            "spec fn forall_in",
            "#[trigger] p(s[i])",
            "spec fn permutation_of",
        ],
    );
    pins(
        include_str!("../src/lower.rs"),
        &["fn emit_combinator_defs", ".verus_l3"],
    );
}

#[test]
fn spec_effect_commutation_is_computed_from_basis_operations() {
    pins(
        include_str!("../../thermite-spec/src/effect_commutation.rs"),
        &[
            "pub enum Commutation",
            "pub fn commutes(",
            "fn entries_commute(",
            "fn state_operations_commute(",
            "Commutation::Reject",
        ],
    );
}

#[test]
fn spec_effect_conflicts_consume_overlap_and_commutation() {
    pins(
        include_str!("../../thermite-spec/src/effect_commutation.rs"),
        &[
            "pub fn concurrent_conflicts(",
            "regions.overlap",
            "effects_commute(left_effect, right_effect, regions)",
            "conflicts.push(ConcurrentConflict",
        ],
    );
    pins(
        include_str!("../../thermite-spec/tests/verified_effect_rows.rs"),
        &["concurrent_consumer_reports_ancestry_conflicts_and_accepts_siblings"],
    );
}

#[test]
fn spec_effect_rows_are_checked_against_body_inference() {
    pins(
        include_str!("../src/effects.rs"),
        &[
            "pub fn analyze",
            "EffectWarning",
            "missing_footprint",
            "inferred",
            "fixed point",
        ],
    );
    pins(
        include_str!("../tests/effects.rs"),
        &["production_analysis_rejects_concurrent_ancestry_conflict"],
    );
}

#[test]
fn spec_schemes_declare_flat_step_shapes_and_arity() {
    pins(
        include_str!("../../thermite-spec/src/schemes.rs"),
        &[
            "pub enum StepShape",
            "pub struct SchemeSig",
            "pub scrutinee_args: usize",
            "pub step_shape: StepShape",
            "pub fn arity(self) -> usize",
        ],
    );
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &["fn check_scheme(", "SchemeStepShape"],
    );
}

#[test]
fn shared_noncopy_escape_is_rejected_while_copy_and_clone_survive() {
    pins(
        include_str!("../tests/shared_state_invariants.rs"),
        &[
            "Copy read is allowed",
            "explicit clone produces an owned value",
            "moves non-Copy shared place",
            "escaping reference to shared place",
        ],
    );
    pins(
        include_str!("../src/effects.rs"),
        &[
            "moves non-Copy shared place",
            "escaping reference to shared place",
        ],
    );
}

#[test]
fn shared_access_requires_matching_lexical_holding() {
    pins(
        include_str!("../tests/shared_state_invariants.rs"),
        &["owns", "holding", "wrong", "lock"],
    );
    pins(include_str!("../src/lower.rs"), &["holding", "shared"]);
}

#[test]
fn shared_declarations_resolve_as_shadowable_place_roots() {
    pins(
        include_str!("../tests/shared_state_invariants.rs"),
        &[
            "shared",
            "shadow",
            "lexical_bindings_shadow_shared_roots",
            "shared_place_copy_clone_and_escape_rules_are_affine",
        ],
    );
    pins(include_str!("../src/effects.rs"), &["shared", "RegionPath"]);
}

#[test]
fn spec_validator_accepts_only_registered_flat_calls() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "pub fn validate(program: &Program)",
            "combinators::lookup",
            "spec_fns",
            "fn walk_call(",
        ],
    );
}

#[test]
fn spec_validator_admits_flat_adt_builtins() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "fn walk_expr_inner",
            "Expr::Match",
            "Expr::Field",
            "Expr::Is",
            "Expr::Deref",
        ],
    );
}

#[test]
fn spec_validator_prechecks_match_exhaustiveness_and_reachability() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "fn check_match_exhaustiveness",
            "NonExhaustiveMatch",
            "UnreachableArm",
            "collect_covered_variants",
        ],
    );
}

#[test]
fn spec_validator_enforces_variant_casing_before_disambiguation() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "InvalidVariantCasing",
            "is_ascii_uppercase",
            "casing_errors",
        ],
    );
    pins(
        include_str!("../../thermite-spec/tests/divergence_adt_validate.rs"),
        &["divergence_lowercase_variant_bypasses_exhaustiveness"],
    );
}

#[test]
fn spec_validator_checks_adt_fields_and_variants() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "fn check_field",
            "fn check_variant_ref",
            "UnknownField",
            "UnknownVariant",
            "struct_fields",
            "variant_to_enum",
        ],
    );
}

#[test]
fn spec_validator_collection_methods_are_a_closed_cage() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "const BUILTIN_METHODS",
            "\"contains\"",
            "\"get\"",
            "\"len\"",
            "BUILTIN_METHODS.contains",
        ],
    );
    pins(
        include_str!("../tests/collections_conformance.rs"),
        &["Vec", "capacity", "get"],
    );
}

#[test]
fn spec_validator_bounds_every_expression_descent() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "const MAX_RECURSION_DEPTH: usize = 64",
            "fn descend(",
            "ExpressionTooDeep",
            "self.depth += 1",
            "self.depth -= 1",
            "self.descend(span, |s| s.walk_expr_inner(expr, span))",
        ],
    );
}
