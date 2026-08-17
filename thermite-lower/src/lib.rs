//! `thermite-lower` — lowering Thermite AST to Verus-annotated Rust source, plus
//! L1 runtime-check compilation.
//!
//! In the v0.1 kernel DAG (REQ-2) this crate depends on `thermite-syntax` and
//! `thermite-spec`. The L3 emission stage (`lower`) lands in `lower.rs` (issue
//! #4) per the route table; the L1 runtime-check stage (`l1::lower_l1`) is the
//! sibling `l1.rs` (`.design/lower/l1-runtime-checks.md`); the L2 Kani-harness
//! stage (`l2::lower_l2`) is the sibling `l2.rs` (`.design/lower/l2-kani.md`,
//! issue #9 / v0.2) and reuses the L1 executable lowering; effect subsumption is
//! a separate dispatch. This crate's own error type (`LowerError`) is born in
//! `lower.rs` with its first fallible function `lower` (workspace.md REQ-3) and
//! is shared by `l1::lower_l1` and `l2::lower_l2`.
//!
//! Governing design: `.design/scaffold/workspace.md` (crate topology),
//! `.design/lower/verus-lowering.md` (the L3 emission contract).
//!
//! ## REQ status
//!
//! <!-- generated:reqs view=thermite-lower-scaffold-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-SCAFFOLD-LOWER-COMPILE | shipped | `thermite-lower/src/lib.rs` | Lower clean compile |  |
//! | REQ-SCAFFOLD-LOWER-DAG | shipped | `thermite-lower/src/lib.rs` | Lower dependency DAG |  |
//! | REQ-SCAFFOLD-LOWER-RESULT | shipped | `thermite-lower/src/lib.rs` | Lower result discipline |  |
//! | REQ-SCAFFOLD-LOWER-WORKSPACE | shipped | `thermite-lower/src/lib.rs` | Lower workspace topology |  |
//! <!-- /generated:reqs -->

pub mod checked;
pub mod effects;
pub mod l1;
pub mod l2;
pub mod locks;
pub mod lower;
pub mod witness;

pub use checked::{
    check_program, AccessMode, CheckedCloseEdge, CheckedHolding, CheckedProgram,
    CheckedSharedPlace, CloseReason, DEFAULT_SEMANTIC_WORK_BUDGET,
};
pub use effects::{analyze_effects, check_effects, subsumes, EffectAnalysis, EffectWarning};
pub use l1::{lower_l1, lower_l1_with_lock_provider};
pub use l2::{bound_string, lower_l2, lower_l2_artifact, L2Artifact};
pub use locks::{program_uses_holding, LockProvider};
pub use lower::{
    lower, lower_contract_expr, lower_equivalence_obligation,
    lower_equivalence_obligation_with_shared, lower_exec_body, lower_exec_expr, lower_l3_library,
    lower_l3_library_with_lock_provider, spec_fn_param_type_map, L3Export, L3ExportVisibility,
    L3LibraryTarget, LowerError,
};
pub use witness::{
    canonical_ast_projection, emit_witness, equivalence_shared_observations, lean_replay_source,
    replay_witness, CanonicalAstProjection, SharedObservation, TraversalWitness, WitnessError,
};
