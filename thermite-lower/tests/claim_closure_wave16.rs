fn pins(text: &str, expected: &[&str]) {
    for pin in expected {
        assert!(text.contains(pin), "missing claim pin {pin}");
    }
}

fn rejects(text: &str, forbidden: &[&str]) {
    for pin in forbidden {
        assert!(!text.contains(pin), "forbidden claim pin {pin}");
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
fn s3_kernel_trust_binds_theorem_axioms_and_solver_input() {
    pins(
        include_str!("../../forge/src/lean_smt_export.rs"),
        &[
            "pub struct ReconstructionEvidence",
            "pub theorem: String",
            "pub axioms: Vec<String>",
            "pub source_clause_sha256: Option<String>",
            "pub solver_query_sha256: Option<String>",
            "pub lrat_sha256: Option<String>",
            "RECONSTRUCTION_AXIOM_ALLOWLIST",
            "parse_and_validate_axioms",
        ],
    );
    pins(
        include_str!("../../forge/src/check.rs"),
        &["req8_arithmetic_and_bitwise_clauses_migrate_to_kernel_checked"],
    );
    pins(
        include_str!("../../gates/lean-axiom-probe.sh"),
        &["Thermite.PinReconstruction", "#print axioms"],
    );
}

#[test]
fn s3_g3_is_one_fail_closed_feature_matrix() {
    let gate = include_str!("../../gates/g3.sh");
    pins(
        gate,
        &[
            "set -euo pipefail",
            "--release --test bv_tag_parse",
            "--release --features bv --test bv_tag_parse",
            "thermite-lower tagged_",
            "--test bv_invariants",
            "bash gates/lean-axiom-probe.sh",
            "lean_smt_export::tests",
            "req8_arithmetic_and_bitwise_clauses_migrate_to_kernel_checked",
            "--test bv_lowering",
        ],
    );
    pins(
        include_str!("../../.github/workflows/ci.yml"),
        &["g3_children", "g3:"],
    );
}

#[test]
fn scaffold_forge_compile_root_is_concrete() {
    pins(
        include_str!("../../forge/src/main.rs"),
        &[
            "mod check;",
            "mod cli;",
            "mod manifest;",
            "fn main() -> ExitCode",
        ],
    );
    pins(include_str!("../../Cargo.toml"), &["\"forge\""]);
}

#[test]
fn scaffold_forge_dag_drives_every_library_phase() {
    let manifest = include_str!("../../forge/Cargo.toml");
    pins(
        manifest,
        &[
            "thermite-syntax = { path = \"../thermite-syntax\" }",
            "thermite-spec = { path = \"../thermite-spec\" }",
            "thermite-lower = { path = \"../thermite-lower\" }",
        ],
    );
    pins(
        include_str!("../../forge/src/cli.rs"),
        &["SyntaxError", "SpecError", "LowerError", "crate::check"],
    );
}

#[test]
fn scaffold_forge_result_is_typed_at_entrypoint() {
    pins(
        include_str!("../../forge/src/main.rs"),
        &["fn main() -> ExitCode", "cli::run()"],
    );
    pins(
        include_str!("../../forge/src/cli.rs"),
        &[
            "pub enum ForgeError",
            "pub fn run() -> ExitCode",
            "EXIT_VERIFICATION_FAILURE",
            "EXIT_ENVIRONMENT",
        ],
    );
}

#[test]
fn scaffold_forge_workspace_materializes_binary() {
    pins(
        include_str!("../../forge/Cargo.toml"),
        &["name = \"forge\"", "[[bin]]", "path = \"src/main.rs\""],
    );
    pins(
        include_str!("../../Cargo.toml"),
        &["members = [", "\"forge\""],
    );
}

#[test]
fn scaffold_lower_compile_root_is_concrete() {
    pins(
        include_str!("../src/lib.rs"),
        &[
            "pub mod checked;",
            "pub mod effects;",
            "pub mod lower;",
            "pub use lower::",
        ],
    );
    pins(include_str!("../../Cargo.toml"), &["\"thermite-lower\""]);
}

#[test]
fn scaffold_lower_dag_is_below_forge() {
    let manifest = include_str!("../Cargo.toml");
    pins(
        manifest,
        &[
            "thermite-syntax = { path = \"../thermite-syntax\" }",
            "thermite-spec = { path = \"../thermite-spec\" }",
        ],
    );
    rejects(manifest, &["path = \"../forge\""]);
}

#[test]
fn scaffold_lower_result_is_owned_and_reexported() {
    pins(
        include_str!("../src/lower.rs"),
        &[
            "pub enum LowerError",
            "pub fn lower(program: &Program) -> Result<String, LowerError>",
        ],
    );
    pins(
        include_str!("../src/lib.rs"),
        &["L3LibraryTarget, LowerError"],
    );
}

#[test]
fn scaffold_lower_workspace_materializes_library() {
    pins(
        include_str!("../Cargo.toml"),
        &["name = \"thermite-lower\""],
    );
    pins(
        include_str!("../src/lib.rs"),
        &["pub use lower::{", "lower, lower_contract_expr"],
    );
}

#[test]
fn scaffold_spec_compile_root_is_concrete() {
    pins(
        include_str!("../../thermite-spec/src/lib.rs"),
        &[
            "pub mod combinators;",
            "pub mod schemes;",
            "pub mod validator;",
        ],
    );
    pins(include_str!("../../Cargo.toml"), &["\"thermite-spec\""]);
}

#[test]
fn scaffold_spec_dag_is_below_lower_and_forge() {
    let manifest = include_str!("../../thermite-spec/Cargo.toml");
    pins(
        manifest,
        &["thermite-syntax = { path = \"../thermite-syntax\" }"],
    );
    rejects(
        manifest,
        &["path = \"../thermite-lower\"", "path = \"../forge\""],
    );
}

#[test]
fn scaffold_spec_result_is_owned_and_reexported() {
    pins(
        include_str!("../../thermite-spec/src/validator.rs"),
        &[
            "pub enum SpecError",
            "pub fn validate(program: &Program) -> Result<(), Vec<SpecError>>",
        ],
    );
    pins(
        include_str!("../../thermite-spec/src/lib.rs"),
        &["pub use validator::{validate, SpecError}"],
    );
}

#[test]
fn scaffold_spec_workspace_materializes_registry_validator_library() {
    pins(
        include_str!("../../thermite-spec/Cargo.toml"),
        &["name = \"thermite-spec\""],
    );
    pins(
        include_str!("../../thermite-spec/src/lib.rs"),
        &[
            "pub use combinators::",
            "pub use validator::{validate, SpecError}",
        ],
    );
}

#[test]
fn scaffold_syntax_compile_root_is_concrete() {
    pins(
        include_str!("../../thermite-syntax/src/lib.rs"),
        &["pub mod ast;", "pub mod lexer;", "pub mod parser;"],
    );
    pins(include_str!("../../Cargo.toml"), &["\"thermite-syntax\""]);
}

#[test]
fn scaffold_syntax_dag_is_internal_leaf() {
    let manifest = include_str!("../../thermite-syntax/Cargo.toml");
    pins(manifest, &["name = \"thermite-syntax\""]);
    rejects(manifest, &["path = \"../thermite-"]);
}

#[test]
fn scaffold_syntax_result_is_parser_owned() {
    let parser = include_str!("../../thermite-syntax/src/parser.rs");
    pins(
        parser,
        &[
            "pub enum SyntaxError",
            "pub struct ParseResult",
            "pub fn parse(src: &str) -> ParseResult",
        ],
    );
    pins(
        include_str!("../../thermite-syntax/src/lib.rs"),
        &["pub use parser::{parse, ParseResult, SyntaxError}"],
    );
}

#[test]
fn scaffold_syntax_workspace_materializes_leaf_library() {
    pins(
        include_str!("../../thermite-syntax/Cargo.toml"),
        &["name = \"thermite-syntax\""],
    );
    pins(
        include_str!("../../thermite-syntax/src/lib.rs"),
        &[
            "pub use ast::{",
            "pub use parser::{parse, ParseResult, SyntaxError}",
        ],
    );
}

#[test]
fn skill_ergonomics_desugars_are_taught_together() {
    pins(
        include_str!("../../thermite-skill/src/generate.rs"),
        &[
            "let (x, y) = e;",
            "for i in lo..hi keeps EXPR",
            "if let Pat = e { T } else { E }",
            "while let V(_) = e keeps .. measures ..",
            "Match guards `Pat if COND => EXPR`",
            "Or-patterns `p0 | p1 => EXPR`",
        ],
    );
}

#[test]
fn skill_match_guard_documents_non_completeness() {
    pins(
        include_str!("../../thermite-skill/src/generate.rs"),
        &[
            "fn render_expr_arm(expr: &Expr)",
            "Expr::Match { .. }",
            "an `if C` guard does NOT complete a match",
            "a guarded-only arm",
        ],
    );
}

#[test]
fn skill_or_pattern_is_rendered_and_inventoried() {
    let generator = include_str!("../../thermite-skill/src/generate.rs");
    pins(
        generator,
        &[
            "fn render_pattern_arm(pat: &Pattern)",
            "Pattern::Or(_) => SkillFragment",
            "fragment: \"p0 | p1 | ..\"",
            "fn pattern_inventory() -> Vec<Pattern>",
            "Pattern::Or(Vec::new())",
            "cover their UNION",
        ],
    );
}

#[test]
fn skill_generator_binary_dispatches_both_modes() {
    pins(
        include_str!("../../thermite-skill/src/main.rs"),
        &[
            "Some(\"--emit\")",
            "print!(\"{}\", generate())",
            "Some(\"--check-budget\")",
            "token_count(&generate())",
            "Outcome::Failure",
        ],
    );
}

#[test]
fn skill_generator_budget_is_deterministic_and_fixed() {
    pins(
        include_str!("../../thermite-skill/src/generate.rs"),
        &[
            "pub const SKILL_TOKEN_BUDGET: usize = 6000",
            "pub fn token_count(s: &str) -> usize",
            "(s.chars().count() * 2).div_ceil(7)",
            "fn budget_gate()",
        ],
    );
}

#[test]
fn skill_generator_sections_have_canonical_order() {
    let generator = include_str!("../../thermite-skill/src/generate.rs");
    pins(
        generator,
        &["pub fn generate() -> String", "out.push_str(HEADER)"],
    );
    ordered(
        generator,
        &[
            "out.push_str(&render_grammar())",
            "out.push_str(&render_combinators())",
            "out.push_str(&render_schemes())",
            "out.push_str(&render_forge())",
            "out.push_str(&render_ladder())",
            "out.push_str(&render_slag())",
        ],
    );
}

#[test]
fn skill_generator_budget_is_a_ci_gate() {
    pins(
        include_str!("../../.github/workflows/ci.yml"),
        &[
            "skill budget gate",
            "cargo run -p thermite-skill -- --check-budget",
        ],
    );
    pins(
        include_str!("../../thermite-skill/src/main.rs"),
        &[
            "if count <= SKILL_TOKEN_BUDGET",
            "Outcome::Success",
            "Outcome::Failure",
        ],
    );
}
