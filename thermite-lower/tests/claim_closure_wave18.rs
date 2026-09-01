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
fn validator_guarded_arms_do_not_close_matches_but_guards_are_walked() {
    let validator = include_str!("../../thermite-spec/src/validator.rs");
    pins(
        validator,
        &[
            "let guarded = arm.guard.is_some()",
            "self.walk_expr(guard, span)",
            "NonExhaustiveMatch",
        ],
    );
    pins(
        include_str!("../../thermite-spec/tests/divergence_c10_guarded_catchall.rs"),
        &["divergence_guarded_only_catchall_is_non_exhaustive"],
    );
}

#[test]
fn validator_or_patterns_union_variants_and_propagate_catchalls() {
    let validator = include_str!("../../thermite-spec/src/validator.rs");
    pins(
        validator,
        &[
            "fn pattern_is_catch_all",
            "Pattern::Or(alts) => alts.iter().any(pattern_is_catch_all)",
            "fn collect_covered_variants",
            "collect_covered_variants(alt, out)",
        ],
    );
}

#[test]
fn validator_combinator_closures_are_flat_but_named_spec_calls_survive() {
    let validator = include_str!("../../thermite-spec/src/validator.rs");
    pins(
        validator,
        &[
            "in_combinator_closure: bool",
            "SpecError::NestedCombinator",
            "self.spec_fns.contains(name)",
        ],
    );
    pins(
        include_str!("../../thermite-spec/tests/divergence_nesting.rs"),
        &["named_spec_fn_call_in_closure_accepts"],
    );
}

#[test]
fn validator_map_contract_methods_are_caged_and_mutators_are_absent() {
    let validator = include_str!("../../thermite-spec/src/validator.rs");
    pins(
        validator,
        &[
            "\"contains_key\"",
            "\"get\"",
            "\"len\"",
            "\"key_at\"",
            "\"value_at\"",
        ],
    );
    assert!(
        !validator.contains("\"insert\","),
        "mutating insert entered the spec cage"
    );
    assert!(
        !validator.contains("\"remove\","),
        "mutating remove entered the spec cage"
    );
}

#[test]
fn validator_direct_recursion_requires_decreases_unless_diverge() {
    let validator = include_str!("../../thermite-spec/src/validator.rs");
    pins(
        validator,
        &[
            "SpecError::MissingDecreases",
            "f.measures.is_none()",
            "!fn_is_diverge(f)",
            "block_calls_name(body, &f.name)",
        ],
    );
}

#[test]
fn validator_rejections_are_structured_and_depth_bounded() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "UnknownCombinator",
            "WrongArity",
            "WrongArgKind",
            "ForbiddenCall",
            "ExpressionTooDeep",
            "const MAX_RECURSION_DEPTH: usize = 64",
        ],
    );
}

#[test]
fn validator_scheme_cage_separates_top_level_and_nested_calls() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "if let Some(scheme) = schemes::lookup(name)",
            "if self.in_scheme_step || self.in_combinator_closure",
            "self.check_scheme(scheme, args, span)",
        ],
    );
}

#[test]
fn validator_scheme_errors_are_explicit_and_total() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &["NestedScheme", "SchemeWrongArity", "SchemeStepShape"],
    );
    pins(
        include_str!("../../thermite-spec/tests/scheme_validate.rs"),
        &[
            "reject_cases_yield_the_oracle_error",
            "nested_scheme_in_step",
            "unknown_scheme",
        ],
    );
}

#[test]
fn validator_scheme_step_checks_closure_arity_and_flat_body() {
    let validator = include_str!("../../thermite-spec/src/validator.rs");
    ordered(
        validator,
        &[
            "let step_arity = scheme.step_shape.arity()",
            "Expr::Closure { params, body }",
            "self.in_scheme_step = true",
            "self.walk_expr(body, span)",
        ],
    );
}

#[test]
fn validator_resolves_registered_schemes_before_combinators() {
    ordered(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "if let Some(scheme) = schemes::lookup(name)",
            "if let Some(sig) = combinators::lookup(name)",
        ],
    );
}

#[test]
fn validator_sealed_struct_literals_cannot_mint_capabilities() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "sealed_structs: HashSet<String>",
            "fn check_sealed_construction",
            "SpecError::SealedConstruction",
        ],
    );
    pins(
        include_str!("../../thermite-spec/tests/sealed_validate.rs"),
        &["sealed_structlit_launder_is_rejected", "SealedConstruction"],
    );
}

#[test]
fn syntax_effect_basis_generates_primitive_frames_and_keeps_given_atoms_distinct() {
    pins(
        include_str!("../../thermite-syntax/src/effect_basis.rs"),
        &[
            "pub enum Theory",
            "State(Instance)",
            "Accrues(String)",
            "Exception",
            "Partiality",
            "Io(String)",
            "pub enum GivenAtom",
            "Random",
            "Blocks",
            "pub fn frame_condition",
        ],
    );
}

#[test]
fn syntax_effect_declarations_resolve_only_to_the_fixed_basis() {
    pins(
        include_str!("../../thermite-syntax/src/effect_basis.rs"),
        &[
            "pub fn resolve_declaration",
            "EffectPrimitive::State",
            "EffectPrimitive::Io",
        ],
    );
    pins(
        include_str!("../../thermite-syntax/tests/effect_declarations.rs"),
        &[
            "declaration_parses_and_resolves_to_basis_primitives",
            "effect platform(d) = state(d) + io(sigma_d)",
            "unknown_primitive_is_a_structured_error_listing_the_basis",
        ],
    );
}

#[test]
fn syntax_effect_footprints_derive_reads_and_writes_from_operations() {
    pins(
        include_str!("../../thermite-syntax/src/effect_basis.rs"),
        &[
            "pub fn footprint(&self) -> Footprint",
            "entry.operations.contains(&Operation::Get)",
            "footprint.reads.insert",
            "entry.operations.contains(&Operation::Put)",
            "footprint.writes.insert",
            "footprint_and_label_mask_membership_agree",
        ],
    );
}

#[test]
fn syntax_effect_routes_expose_implemented_deferred_and_open_trust() {
    pins(
        include_str!("../../thermite-syntax/src/effect_basis.rs"),
        &[
            "pub enum ObligationRoute",
            "NoObligation(NoObligationPremise)",
            "Implemented(DischargeForm)",
            "Deferred(DischargeForm)",
            "Open(DischargeQuestion)",
            "pub const fn route_for_given",
        ],
    );
}

#[test]
fn syntax_holding_block_is_lexical_while_owns_is_declared() {
    pins(
        include_str!("../../thermite-syntax/tests/shared_state_invariants.rs"),
        &[
            "owns_and_holding_have_distinct_public_and_lexical_nodes",
            "Effect::Owns",
            "Stmt::Holding",
        ],
    );
}

#[test]
fn syntax_lock_declarations_name_guarded_regions_and_order() {
    pins(
        include_str!("../../thermite-syntax/tests/shared_state_invariants.rs"),
        &[
            "lock_declarations_guard_regions_and_record_order",
            "lock scheduler_lock guards scheduler",
            "after scheduler_lock",
            "second.after.as_deref()",
        ],
    );
}

#[test]
fn syntax_shared_declarations_anchor_effect_region_paths() {
    pins(
        include_str!("../../thermite-syntax/tests/verified_effect_rows.rs"),
        &[
            "shared_nested_types_and_region_paths_parse",
            "shared scheduler: Scheduler",
            "Effect::Read(path)",
            "scheduler.runqueue.head",
        ],
    );
}

#[test]
fn tv_body_reference_threads_state_and_projects_tuple_cells() {
    let encoder = include_str!("../../thermite-tv/src/exec_stmt_encode.rs");
    pins(
        encoder,
        &[
            "pub fn body_ref_state",
            "fn thread_stmt",
            "Stmt::Let",
            "Stmt::Assign",
            "Stmt::If",
            "Expr::Tuple(elems)",
            "format!(\"{result_name}.{i} == {cell}\")",
        ],
    );
}

#[test]
fn tv_body_reference_loudly_rejects_out_of_v1_statements() {
    let encoder = include_str!("../../thermite-tv/src/exec_stmt_encode.rs");
    pins(
        encoder,
        &[
            "re-shadowed binding",
            "mid-body early return",
            "Stmt::Loop(_)",
            "Stmt::Break",
            "Stmt::Continue",
            "RefEncodeError::Unsupported",
        ],
    );
    pins(
        include_str!("../../thermite-tv/tests/body_teeth.rs"),
        &["body_ref_state"],
    );
}

#[test]
fn tv_contract_forge_plugin_joins_production_and_reference_paths() {
    pins(
        include_str!("../../forge/src/contract_tv.rs"),
        &[
            "pub fn tv_file",
            "thermite_lower::lower_contract_expr",
            "equivalence_obligation",
            "ref_contract_pred",
            "pub fn run_generated",
        ],
    );
}

#[test]
fn tv_contract_generator_is_deterministic_typed_and_bounded() {
    pins(
        include_str!("../../thermite-tv/src/gen.rs"),
        &[
            "pub struct Rng",
            "pub fn generate_clauses(seed: u64, n: usize)",
            "The fixed typed vocabulary",
            "MAX_DEPTH",
        ],
    );
}

#[test]
fn tv_contract_obligation_compares_production_and_reference_equivalence() {
    pins(
        include_str!("../../thermite-tv/src/obligation.rs"),
        &[
            "pub fn equivalence_obligation",
            "let p_reference = ref_contract_pred",
            "assert(({p_production}) <==> ({p_reference}))",
        ],
    );
}

#[test]
fn tv_contract_reference_encoder_is_independent_and_total() {
    pins(
        include_str!("../../thermite-tv/src/ref_encode.rs"),
        &[
            "pub fn ref_contract_pred",
            "fn encode(expr: &Expr",
            "RefEncodeError::Unsupported",
        ],
    );
    let manifest = include_str!("../../thermite-tv/Cargo.toml");
    let dependencies = manifest
        .split_once("[dependencies]")
        .expect("thermite-tv manifest has a dependency section")
        .1;
    assert!(
        !dependencies.contains("thermite-lower"),
        "reference encoder depends on production lowering"
    );
}

#[test]
fn tv_contract_teeth_verify_faithful_and_catch_infidel_clauses() {
    pins(
        include_str!("../../thermite-tv/tests/teeth.rs"),
        &[
            "fn assert_faithful_verifies",
            "fn assert_infidel_caught",
            "assertion failed",
            "f1_comparison_faithful_verifies",
            "f1_comparison_infidel_caught",
        ],
    );
}
