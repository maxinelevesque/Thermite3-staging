//! Compile-time effect-row subsumption (`fx`) — the static half of Thermite's
//! effect system (`thermite-design.md` §4.1: "Effect rows compose: a caller's
//! row must subsume every callee's row, checked at compile time").
//!
//! A caller's `fx` row must subsume every callee's row. `fx pure` permits
//! nothing, so a `pure` function that calls an effectful one is a compile-time
//! rejection. This component is the compile-time check only; the runtime
//! syscall sandbox (§4.1 "killed at the syscall boundary") is deferred to issue
//! #21 (`goal.md` excluded-from-kernel). R-SPEC-5: the v0.1 form is implemented;
//! the deferred form (sandbox) is not built. `effects.rs` has no codegen
//! path, only a checking path (REQ-6 / AC-6).
//!
//! Governing design: `.design/lower/effect-subsumption.md` (REQ-1..REQ-6).
//!
//! ## The lattice (REQ-1)
//!
//! The effects form a lattice over the powerset of the eleven atoms in
//! `thermite_syntax::ast::Effect` (`Read`/`Write`/`Net`/`Alloc`/`Time`/`Rand`/
//! `Panic`/`Diverge`/`Term`/`Owns`/`Forgets`), ordered by subset inclusion. `EffectRow::Pure` ≡ the empty
//! set `{}` is the bottom: it permits nothing and is subsumed by everything. The
//! join of two rows is set union. v0.1 subsumption is atom-kind level
//! (path-insensitive): a `Write(_)` caller subsumes any `Write(_)` callee (OQ-1;
//! path-granular subsumption is a deferred refinement that needs a path lattice
//! the v0.1 kernel does not build).
//!
//! ## REQ status
//!
//! <!-- generated:reqs view=thermite-lower-effects-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-LOWER-EFFECTS-CHECK | shipped | `thermite-lower/src/effects.rs` | Effect checker entry and call graph |  |
//! | REQ-LOWER-EFFECTS-ERROR | shipped | `thermite-lower/src/effects.rs` | Effect checker structured rejection |  |
//! | REQ-LOWER-EFFECTS-LATTICE | shipped | `thermite-lower/src/effects.rs` | Effect-row lattice |  |
//! | REQ-LOWER-EFFECTS-MAXIMAL-ROW-BOUNDARY | shipped | `thermite-lower/src/effects.rs` | Effect maximal-row boundary |  |
//! | REQ-LOWER-EFFECTS-SANDBOX-SCOPE | shipped | `thermite-lower/src/effects.rs` | Effect runtime sandbox scope |  |
//! | REQ-LOWER-EFFECTS-SUBSUMPTION | shipped | `thermite-lower/src/effects.rs` | Effect-row subsumption |  |
//! <!-- /generated:reqs -->

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use thermite_syntax::ast::{
    Block, Effect, EffectRow, Expr, IndexArg, Item, LoopKind, Program, Stmt,
};
use thermite_syntax::lexer::Span;
use thermite_syntax::RegionPath;
use thermite_syntax::Type;

use crate::lower::LowerError;

/// The maximum recursive-descent depth before the body walk returns
/// `LowerError::TooDeep`. Mirrors `lower.rs`'s `MAX_EMIT_DEPTH` discipline (and
/// `thermite-syntax`'s parser guard): a single shared counter bounds the
/// `Expr`-tree walk so a pathological (or adversarial, post-recovery) AST returns
/// a structured error rather than overflowing the native stack (REQ-3 / AC-5).
/// Fixed constant (determinism, `goal.md` R-CODE-5).
const MAX_WALK_DEPTH: usize = 256;

/// The eleven atomic effect kinds (REQ-1), the carriers of subsumption. This is
/// the path-insensitive projection of `thermite_syntax::ast::Effect`: `Read(p)`,
/// `Write(p)`, `Net(d)` collapse to `Read`/`Write`/`Net` regardless of the path/
/// domain argument (OQ-1; v0.1 subsumption is atom-kind level). The remaining
/// six atoms (`Alloc`/`Time`/`Rand`/`Panic`/`Diverge`/`Term`) are argument-free;
/// `Owns` and `Forgets` retain region paths before this kind projection.
/// `Term` is the #106 terminal-control atom (`fx term` → the `ioctl` seccomp
/// grant, runtime-sandbox.md REQ-7), the 9th atom that widened the proved bitset
/// from `u8` to `u16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectKind {
    Read,
    Write,
    Net,
    Alloc,
    Time,
    Rand,
    Panic,
    Diverge,
    Term,
    Owns,
    Forgets,
}

impl EffectKind {
    /// The bit position of this atom kind in the 11-atom `u16` bitset shared with
    /// the Verus-verified core (`thermite_verified`, epic #60 / #106): Read=0,
    /// Write=1, Net=2, Alloc=3, Time=4, Rand=5, Panic=6, Diverge=7, Term=8. This
    /// is the representation port of `.design/verified/self-verification.md`
    /// (REQ-5): `subsumes` projects each `EffectRow` to its mask via this bit and
    /// delegates the subset test to `thermite_verified::subsumes_masks`, the
    /// plain-Rust mirror of the `verus`-proved exec body. Widened `u8`→`u16` for
    /// the 9th atom `Term` (#106), so the proved bitset is now `u16`.
    fn bit(self) -> u16 {
        let index: u32 = match self {
            EffectKind::Read => 0,
            EffectKind::Write => 1,
            EffectKind::Net => 2,
            EffectKind::Alloc => 3,
            EffectKind::Time => 4,
            EffectKind::Rand => 5,
            EffectKind::Panic => 6,
            EffectKind::Diverge => 7,
            EffectKind::Term => 8,
            EffectKind::Owns => 9,
            EffectKind::Forgets => 10,
        };
        1u16 << index
    }

    /// The atom-kind of a concrete `Effect`, dropping the path/domain argument
    /// (REQ-2, OQ-1; path-insensitive in v0.1).
    fn of(effect: &Effect) -> EffectKind {
        match effect {
            Effect::Read(_) => EffectKind::Read,
            Effect::Write(_) => EffectKind::Write,
            Effect::Net(_) => EffectKind::Net,
            Effect::Alloc => EffectKind::Alloc,
            Effect::Time => EffectKind::Time,
            Effect::Rand => EffectKind::Rand,
            Effect::Panic => EffectKind::Panic,
            Effect::Diverge => EffectKind::Diverge,
            Effect::Term => EffectKind::Term,
            Effect::Owns(_) => EffectKind::Owns,
            Effect::Forgets(_) => EffectKind::Forgets,
        }
    }
}

/// The 11-atom `u16` bitset of an effect row (one bit per `EffectKind`, see
/// `EffectKind::bit`): the representation port shared with the Verus-verified
/// core (`.design/verified/self-verification.md` REQ-5). `mask(Pure) = 0`;
/// `mask(Set(v))` ORs in `EffectKind::of(e).bit()` for each `e`. Path-insensitive
/// (OQ-1), the projection `effects` already performs. Widened `u8`→`u16`
/// for the 9th atom `Term` (#106).
fn mask(row: &EffectRow) -> u16 {
    match row {
        EffectRow::Pure => 0,
        EffectRow::Set(v) => v.iter().fold(0u16, |m, e| m | EffectKind::of(e).bit()),
    }
}

/// The subsumption relation (REQ-2): `caller` subsumes `callee` iff
/// `effects(callee) ⊆ effects(caller)`. `Pure` (the bottom, `{}`) subsumes only
/// `Pure`; a row of all atoms (the top) subsumes every row. Reflexive by
/// construction (a set is a subset of itself). This is the accept relation
/// `check_effects` asserts at every resolved call site.
///
/// Self-verification (epic #60, `.design/verified/self-verification.md` REQ-5):
/// the bit-level subset decision is delegated to
/// `thermite_verified::subsumes_masks`, the plain-Rust mirror of the
/// `verus`-proved exec body (`(callee & !caller) == 0`). This function projects
/// each `EffectRow` to its 11-atom mask (`mask`) and hands the masks to the
/// verified core; the exhaustive 4194304-pair (2048×2048) equivalence test
/// (`tests/effects_verified.rs`) anchors this projection to the proved subset
/// relation `thermite_verified::spec_subsumes_mask`. Behavior matches the
/// former set-membership form (the masks encode the same atom-kind sets).
pub fn subsumes(caller: &EffectRow, callee: &EffectRow) -> bool {
    thermite_verified::subsumes_masks(mask(caller), mask(callee))
}

/// Region-sensitive difference used by RFC-9 call propagation. Unlike the
/// compatibility `subsumes` projection, carrier effects retain their complete
/// paths. A declaration on an ancestor region is a valid frame upper bound for
/// an operation on a descendant region.
fn missing_footprint(caller: &EffectRow, callee: &EffectRow) -> Vec<Effect> {
    let declared = match caller {
        EffectRow::Pure => &[][..],
        EffectRow::Set(effects) => effects.as_slice(),
    };
    let inferred = match callee {
        EffectRow::Pure => &[][..],
        EffectRow::Set(effects) => effects.as_slice(),
    };
    let mut missing: Vec<Effect> = inferred
        .iter()
        .filter(|effect| {
            !declared
                .iter()
                .any(|bound| effect_is_covered(bound, effect))
        })
        .cloned()
        .collect();
    missing.sort();
    missing.dedup();
    missing
}

fn effect_is_covered(bound: &Effect, operation: &Effect) -> bool {
    match (bound, operation) {
        (Effect::Read(outer), Effect::Read(inner))
        | (Effect::Write(outer), Effect::Write(inner))
        | (Effect::Net(outer), Effect::Net(inner)) => inner.segments.starts_with(&outer.segments),
        _ => bound == operation,
    }
}

/// What a callee name resolves to in the program's effect call graph (REQ-3).
enum Callee {
    /// A declared `fn` with its own `fx` row.
    Fn,
    /// A `spec fn` (no `fx`; pure by construction — §4.2 spec sublanguage is
    /// total/effect-free) or a registry combinator. Always subsumed.
    Pure,
    /// A name that is neither a declared `FnItem`, `SpecFnItem`, nor a
    /// combinator. A no-op for subsumption: the #2 validator owns unknown-name
    /// rejection (AC-5), so the effect checker does not panic or error here.
    Unresolved,
}

/// A declared operation that fixed-point inference did not find in the body or
/// any transitive callee. Warnings are data, not lowerer-side stderr output, so
/// Forge and other structured consumers can choose their release policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectWarning {
    pub function: String,
    pub excess: Vec<Effect>,
    pub span: Span,
}

impl std::fmt::Display for EffectWarning {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let excess: Vec<String> = self
            .excess
            .iter()
            .map(|effect| format!("{effect:?}"))
            .collect();
        write!(
            formatter,
            "effect row of `{}` is over-conservative at byte {}..{}: excess effect(s) [{}]",
            self.function,
            self.span.start,
            self.span.end(),
            excess.join(", ")
        )
    }
}

/// RFC-9's complete effect-analysis product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAnalysis {
    pub direct_footprints: BTreeMap<String, BTreeSet<Effect>>,
    pub calls: BTreeMap<String, BTreeSet<String>>,
    pub footprints: BTreeMap<String, BTreeSet<Effect>>,
    pub warnings: Vec<EffectWarning>,
}

/// The compile-time effect-subsumption check (REQ-3). Builds a name→`Contract.effects`
/// map over the program's `FnItem`s (noting `SpecFnItem` names and registry
/// combinators as pure), then walks every `FnItem` body's `Expr` tree. For each
/// `Call`/`MethodCall` whose callee resolves to a declared `FnItem`, it asserts
/// the caller's `fx` subsumes the callee's `fx` (REQ-2), accumulating one
/// `EffectNotSubsumed` per violation rather than failing on the first (§2.4
/// actionable feedback). Calls to a `spec fn` / combinator are pure ⇒
/// always permitted; an unresolved callee is a no-op (AC-5). Never panics: a
/// pathological body returns `LowerError::TooDeep` (REQ-4 / AC-5).
pub fn check_effects(program: &Program) -> Result<(), Vec<LowerError>> {
    crate::CheckedProgram::build(program).map(|_| ())
}

/// Infer exact transitive footprints to a deterministic least fixed point and
/// compare them with declared frame bounds. Foreign boundary functions seed
/// inference with their declared row; in-language functions seed it with
/// recognized direct intrinsics and then union resolved callees until stable.
pub fn analyze_effects(program: &Program) -> Result<EffectAnalysis, Vec<LowerError>> {
    crate::CheckedProgram::build(program).map(|checked| checked.effects().clone())
}

pub(crate) fn analyze_effects_unchecked(
    program: &Program,
) -> Result<EffectAnalysis, Vec<LowerError>> {
    let resource_forgets = thermite_spec::ResourceEnv::build(program)
        .ok()
        .and_then(|resources| thermite_spec::check_resource_flow(program, &resources).ok())
        .map(|report| report.direct_forgets)
        .unwrap_or_default();
    // name → declared `fx` row, over the `FnItem`s. `SpecFnItem` names are noted
    // as pure (they carry no `fx`). On a duplicate name the first declaration
    // wins (deterministic; duplicate-name rejection is the #2 validator's job).
    let mut fn_rows: BTreeMap<&str, &EffectRow> = BTreeMap::new();
    let mut spec_names: BTreeMap<&str, ()> = BTreeMap::new();
    for item in &program.items {
        match item {
            Item::Fn(f) => {
                fn_rows
                    .entry(f.name.as_str())
                    .or_insert(&f.contract.effects);
            }
            Item::SpecFn(s) => {
                spec_names.entry(s.name.as_str()).or_insert(());
            }
            // Basis Stage 1a (`.design/basis/01-adts.md`): a `struct`/`enum`
            // item declares no callable `fn`/`spec fn` name and carries no `fx`
            // row — the neutral value for this name-collection pass is a no-op.
            // (The item is gated at the validator before effect-check runs;
            // dead-in-1a.)
            Item::Struct(_) | Item::Enum(_) => {}
            // Forge-tier item (stage1-forge-tier.md REQ-3): no v1 effect consumer yet
            // (increments 2b-3); inert here, mirroring the ADT-decl arm.
            Item::Forge(_)
            | Item::EffectDecl(_)
            | Item::SharedDecl(_)
            | Item::Concurrent(_)
            | Item::LockDecl(_) => {}
        }
    }

    let mut errors: Vec<LowerError> = Vec::new();
    let mut calls: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut footprints: BTreeMap<String, BTreeSet<Effect>> = BTreeMap::new();
    for item in &program.items {
        // Only `fn` items have an `fx` row and so can be a checked caller; a
        // `spec fn` is pure by construction and makes no effectful calls.
        if let Item::Fn(f) = item {
            let direct = RefCell::new(BTreeSet::new());
            let called = RefCell::new(BTreeSet::new());
            let resolve = |name: &str| -> Callee {
                if fn_rows.contains_key(name) {
                    called.borrow_mut().insert(name.to_string());
                    Callee::Fn
                } else if let Some(effect) = intrinsic_effect(name) {
                    direct.borrow_mut().insert(effect);
                    Callee::Pure
                } else if spec_names.contains_key(name) || thermite_spec::lookup(name).is_some() {
                    Callee::Pure
                } else {
                    Callee::Unresolved
                }
            };
            // A boundary fn (ffi-boundary.md REQ-2) has `body: None`; its body is
            // foreign and makes no in-language calls to subsume (its own `fx` row
            // is trusted by fiat, OQ-4; the row is still checked at the call site,
            // because the boundary fn's declared `fx` is in `fn_rows` above). Only
            // an in-language body is walked for callee subsumption.
            if let Some(body) = &f.body {
                // Run the depth-bounded general walk before RFC-10's specialized
                // collectors. If the AST exceeds the shared budget, return the
                // structured `TooDeep` result before any recursive specialist
                // can consume native stack.
                check_block(
                    body,
                    &f.contract.effects,
                    &f.name,
                    f.span,
                    &resolve,
                    0,
                    &mut errors,
                );
                if errors
                    .iter()
                    .any(|error| matches!(error, LowerError::TooDeep { .. }))
                {
                    return Err(errors);
                }
                collect_holding_effects(body, &mut direct.borrow_mut());
                if let Some(regions) = resource_forgets.get(&f.name) {
                    direct
                        .borrow_mut()
                        .extend(regions.iter().cloned().map(Effect::Forgets));
                }
                let shared_roots: BTreeSet<String> = program
                    .items
                    .iter()
                    .filter_map(|item| match item {
                        Item::SharedDecl(shared) => Some(shared.name.clone()),
                        _ => None,
                    })
                    .collect();
                let regions = thermite_spec::RegionIndex::build(program).ok();
                let mut locals: BTreeSet<String> =
                    f.params.iter().map(|param| param.name.clone()).collect();
                collect_shared_place_effects(
                    body,
                    &shared_roots,
                    regions.as_ref(),
                    &mut locals,
                    &mut Vec::new(),
                    &mut direct.borrow_mut(),
                    &f.name,
                    f.span,
                    &mut errors,
                );
                calls.insert(f.name.clone(), called.into_inner());
                footprints.insert(f.name.clone(), direct.into_inner());
            } else {
                calls.insert(f.name.clone(), BTreeSet::new());
                footprints.insert(f.name.clone(), row_effects(&f.contract.effects));
            }
        }
    }

    if errors
        .iter()
        .any(|error| matches!(error, LowerError::TooDeep { .. }))
    {
        return Err(errors);
    }

    let direct_footprints = footprints.clone();

    // Monotone union over finite effect sets. Source-order maps and sets make
    // both convergence and diagnostics deterministic, including recursive SCCs.
    loop {
        let previous = footprints.clone();
        for (caller, callees) in &calls {
            let footprint = footprints.entry(caller.clone()).or_default();
            for callee in callees {
                if let Some(callee_footprint) = previous.get(callee) {
                    footprint.extend(callee_footprint.iter().cloned());
                }
            }
        }
        if footprints == previous {
            break;
        }
    }

    let function_names: BTreeSet<String> = fn_rows.keys().map(|name| (*name).to_string()).collect();
    for item in &program.items {
        let Item::Fn(function) = item else { continue };
        if let Some(body) = &function.body {
            check_holding_callees(
                body,
                &function_names,
                &footprints,
                thermite_spec::RegionIndex::build(program).ok().as_ref(),
                &function.name,
                &mut errors,
            );
        }
    }

    let mut warnings = Vec::new();
    for item in &program.items {
        let Item::Fn(function) = item else { continue };
        if function.body.is_none() {
            continue;
        }
        let inferred = footprints.get(&function.name).cloned().unwrap_or_default();
        let inferred_row = EffectRow::Set(inferred.iter().cloned().collect());
        let missing = missing_footprint(&function.contract.effects, &inferred_row);
        if !missing.is_empty() {
            errors.push(LowerError::EffectNotSubsumed {
                caller: function.name.clone(),
                callee: "inferred transitive footprint".into(),
                missing,
                span: function.span,
            });
        }
        let mut excess: Vec<Effect> = row_effects(&function.contract.effects)
            .into_iter()
            .filter(|declared| {
                !inferred
                    .iter()
                    .any(|effect| effect_is_covered(declared, effect))
            })
            .collect();
        excess.sort();
        if !excess.is_empty() {
            warnings.push(EffectWarning {
                function: function.name.clone(),
                excess,
                span: function.span,
            });
        }
    }

    if errors.is_empty() {
        let region_span = program
            .items
            .iter()
            .find_map(|item| match item {
                Item::SharedDecl(item) => Some(item.span),
                Item::Concurrent(item) => Some(item.span),
                _ => None,
            })
            .unwrap_or(Span::new(0, 0));
        // Region honesty is a property of every program, not an opt-in feature
        // activated by `shared` or `concurrent` metadata. Building the index
        // unconditionally makes a named state effect with no declaration fail
        // closed, including legacy programs that contain neither new item form.
        let regions = thermite_spec::RegionIndex::build(program).map_err(|region_errors| {
            region_errors
                .into_iter()
                .map(|error| LowerError::EffectAnalysis {
                    detail: format!("region resolution: {error:?}"),
                    span: region_span,
                })
                .collect::<Vec<_>>()
        })?;
        for (function, footprint) in &footprints {
            let mut accessed = footprint.clone();
            if let Some(declared) = program.items.iter().find_map(|item| match item {
                Item::Fn(item) if item.name == *function => Some(&item.contract.effects),
                _ => None,
            }) {
                accessed.extend(row_effects(declared));
            }
            for effect in &accessed {
                let Some(path) = thermite_spec::effect_path(effect) else {
                    continue;
                };
                for (lock, guarded) in regions.locks() {
                    if regions.overlaps(path, guarded)
                        && !accessed.contains(&Effect::Owns(lock.to_string()))
                    {
                        errors.push(LowerError::EffectAnalysis {
                            detail: format!(
                                "function `{function}` accesses guarded region `{path}` without `owns({lock})`"
                            ),
                            span: region_span,
                        });
                    }
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        for item in &program.items {
            let Item::Fn(function) = item else { continue };
            if let Some(body) = &function.body {
                check_holding_order(body, &regions, &mut Vec::new(), &function.name, &mut errors);
            }
        }
        let handler_roots: BTreeSet<String> = program
            .items
            .iter()
            .find_map(|item| match item {
                Item::Concurrent(item) if item.name == "__handlers" => {
                    Some(item.roots.iter().cloned().collect())
                }
                _ => None,
            })
            .unwrap_or_default();
        let handler_locks: BTreeSet<String> = handler_roots
            .iter()
            .filter_map(|name| footprints.get(name))
            .flat_map(|effects| effects.iter())
            .filter_map(|effect| match effect {
                Effect::Owns(lock) if lock != "interrupts" => Some(lock.clone()),
                _ => None,
            })
            .collect();
        for (function, footprint) in &footprints {
            if handler_roots.contains(function) {
                continue;
            }
            for lock in &handler_locks {
                if footprint.contains(&Effect::Owns(lock.clone()))
                    && !footprint.contains(&Effect::Owns("interrupts".to_string()))
                {
                    errors.push(LowerError::EffectAnalysis { detail: format!("normal-context function `{function}` owns handler-visible lock `{lock}` without `owns(interrupts)`"), span: region_span });
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        if program
            .items
            .iter()
            .any(|item| matches!(item, Item::Concurrent(_)))
        {
            let conflicts = thermite_spec::effect_commutation::concurrent_conflicts(
                program,
                &regions,
                &footprints,
            )
            .map_err(|detail| {
                vec![LowerError::EffectAnalysis {
                    detail,
                    span: Span::new(0, 0),
                }]
            })?;
            if !conflicts.is_empty() {
                return Err(conflicts
                    .into_iter()
                    .map(|conflict| LowerError::EffectAnalysis {
                        span: program
                            .items
                            .iter()
                            .find_map(|item| match item {
                                Item::Concurrent(item)
                                    if item.name == conflict.composition => Some(item.span),
                                _ => None,
                            })
                            .unwrap_or(Span::new(0, 0)),
                        detail: format!(
                            "concurrent `{}` roots `{}` and `{}` conflict: {:?} versus {:?}; overlap {:?}",
                            conflict.composition,
                            conflict.left_root,
                            conflict.right_root,
                            conflict.left_effect,
                            conflict.right_effect,
                            conflict.overlap
                        ),
                    })
                    .collect());
            }
        }
        // Revision-2 differential seam: the optimized recursive analysis and
        // the canonical semantic-inventory interpretation must agree before a
        // CheckedProgram exists. This keeps two derivations only while making
        // phase skew a structured rejection on every program, not a corpus-only
        // test discovery.
        let canonical = crate::witness::canonical_ast_projection(program).map_err(|error| {
            vec![LowerError::EffectAnalysis {
                detail: format!("canonical effect projection failed: {error:?}"),
                span: Span::new(0, 0),
            }]
        })?;
        let rendered_direct: BTreeMap<String, Vec<String>> = direct_footprints
            .iter()
            .map(|(function, effects)| {
                (
                    function.clone(),
                    effects.iter().map(|effect| format!("{effect:?}")).collect(),
                )
            })
            .collect();
        let rendered_calls: BTreeMap<String, Vec<String>> = calls
            .iter()
            .map(|(function, callees)| (function.clone(), callees.iter().cloned().collect()))
            .collect();
        ensure_canonical_effect_agreement(
            &rendered_direct,
            &rendered_calls,
            &canonical.direct_footprints,
            &canonical.calls,
        )
        .map_err(|error| vec![error])?;
        Ok(EffectAnalysis {
            direct_footprints,
            calls,
            footprints,
            warnings,
        })
    } else {
        Err(errors)
    }
}

fn ensure_canonical_effect_agreement(
    optimized_direct: &BTreeMap<String, Vec<String>>,
    optimized_calls: &BTreeMap<String, Vec<String>>,
    canonical_direct: &BTreeMap<String, Vec<String>>,
    canonical_calls: &BTreeMap<String, Vec<String>>,
) -> Result<(), LowerError> {
    if optimized_direct == canonical_direct && optimized_calls == canonical_calls {
        Ok(())
    } else {
        Err(LowerError::EffectAnalysis {
            detail: "optimized effect analysis diverged from the canonical semantic inventory"
                .into(),
            span: Span::new(0, 0),
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_shared_place_effects(
    block: &Block,
    shared_roots: &BTreeSet<String>,
    regions: Option<&thermite_spec::RegionIndex>,
    locals: &mut BTreeSet<String>,
    held: &mut Vec<String>,
    direct: &mut BTreeSet<Effect>,
    function: &str,
    span: Span,
    errors: &mut Vec<LowerError>,
) {
    let outer_locals = locals.clone();
    for stmt in &block.stmts {
        match stmt {
            Stmt::Let { name, init, .. } => {
                collect_shared_expr(
                    init,
                    false,
                    shared_roots,
                    regions,
                    locals,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                );
                locals.insert(name.clone());
            }
            Stmt::Assign { target, value } => {
                collect_shared_expr(
                    target,
                    true,
                    shared_roots,
                    regions,
                    locals,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                );
                collect_shared_expr(
                    value,
                    false,
                    shared_roots,
                    regions,
                    locals,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                );
            }
            Stmt::Return(Some(expr)) | Stmt::Expr(expr) | Stmt::Forget { value: expr, .. } => {
                collect_shared_expr(
                    expr,
                    false,
                    shared_roots,
                    regions,
                    locals,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                )
            }
            Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
            Stmt::If { cond, then, else_ } => {
                collect_shared_expr(
                    cond,
                    false,
                    shared_roots,
                    regions,
                    locals,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                );
                let mut branch = locals.clone();
                collect_shared_place_effects(
                    then,
                    shared_roots,
                    regions,
                    &mut branch,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                );
                if let Some(other) = else_ {
                    let mut branch = locals.clone();
                    collect_shared_place_effects(
                        other,
                        shared_roots,
                        regions,
                        &mut branch,
                        held,
                        direct,
                        function,
                        span,
                        errors,
                    );
                }
            }
            Stmt::Loop(loop_) => {
                if let LoopKind::While(cond) = &loop_.kind {
                    collect_shared_expr(
                        cond,
                        false,
                        shared_roots,
                        regions,
                        locals,
                        held,
                        direct,
                        function,
                        span,
                        errors,
                    );
                }
                let mut nested = locals.clone();
                collect_shared_place_effects(
                    &loop_.body,
                    shared_roots,
                    regions,
                    &mut nested,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                );
            }
            Stmt::Holding { lock, body, .. } => {
                held.push(lock.clone());
                let mut nested = locals.clone();
                collect_shared_place_effects(
                    body,
                    shared_roots,
                    regions,
                    &mut nested,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                );
                held.pop();
            }
        }
    }
    if let Some(tail) = &block.tail {
        collect_shared_expr(
            tail,
            false,
            shared_roots,
            regions,
            locals,
            held,
            direct,
            function,
            span,
            errors,
        );
    }
    *locals = outer_locals;
}

#[allow(clippy::too_many_arguments)]
fn collect_shared_expr(
    expr: &Expr,
    write: bool,
    shared_roots: &BTreeSet<String>,
    regions: Option<&thermite_spec::RegionIndex>,
    locals: &BTreeSet<String>,
    held: &[String],
    direct: &mut BTreeSet<Effect>,
    function: &str,
    span: Span,
    errors: &mut Vec<LowerError>,
) {
    if let Some(path) = shared_place_path(expr, shared_roots, locals) {
        if let Some(regions) = regions {
            if let Err(error) = regions.resolve(&path) {
                errors.push(LowerError::EffectAnalysis {
                    detail: format!(
                        "function `{function}` has invalid shared place `{path}`: {error:?}"
                    ),
                    span,
                });
                return;
            }
        }
        let matching_locks: Vec<_> = regions
            .into_iter()
            .flat_map(|regions| regions.locks())
            .filter(|(_, guard)| regions.is_some_and(|regions| regions.overlaps(&path, guard)))
            .map(|(lock, _)| lock.to_string())
            .collect();
        for lock in matching_locks {
            if !held.iter().any(|held_lock| held_lock == &lock) {
                errors.push(LowerError::EffectAnalysis {
                    detail: format!("function `{function}` accesses shared place `{path}` outside `holding {lock}`"),
                    span,
                });
            }
        }
        if !write
            && regions
                .and_then(|regions| regions.resolve(&path).ok())
                .is_some_and(|ty| !shared_read_is_copy(&ty))
        {
            errors.push(LowerError::EffectAnalysis {
                detail: format!("function `{function}` moves non-Copy shared place `{path}`; use an explicit `.clone()` inside its holding scope"),
                span,
            });
        }
        direct.insert(if write {
            Effect::Write(path)
        } else {
            Effect::Read(path)
        });
        return;
    }
    if let Expr::MethodCall {
        receiver,
        name,
        args,
    } = expr
    {
        if let Some(path) = shared_place_path(receiver, shared_roots, locals) {
            if name != "clone" {
                errors.push(LowerError::EffectAnalysis {
                    detail: format!("function `{function}` calls unsupported method `{name}` on shared place `{path}`; only explicit `.clone()` is admitted without typed receiver-method effects"),
                    span,
                });
            }
            record_shared_clone(&path, regions, held, direct, function, span, errors);
            for arg in args {
                collect_shared_expr(
                    arg,
                    false,
                    shared_roots,
                    regions,
                    locals,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                );
            }
            return;
        }
    }
    if let Expr::Ref { expr: receiver, .. } = expr {
        if let Some(path) = shared_place_path(receiver, shared_roots, locals) {
            errors.push(LowerError::EffectAnalysis {
                detail: format!("function `{function}` creates an escaping reference to shared place `{path}`; shared-derived borrows cannot cross invariant close"),
                span,
            });
            record_shared_clone(&path, regions, held, direct, function, span, errors);
            return;
        }
    }
    let mut visit = |child: &Expr| {
        collect_shared_expr(
            child,
            false,
            shared_roots,
            regions,
            locals,
            held,
            direct,
            function,
            span,
            errors,
        )
    };
    match expr {
        Expr::Call { callee, args } => {
            visit(callee);
            for arg in args {
                visit(arg);
            }
        }
        Expr::MethodCall {
            receiver,
            name,
            args,
        } => {
            let _ = name;
            visit(receiver);
            for arg in args {
                visit(arg);
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. } => visit(receiver),
        Expr::Ref { expr: receiver, .. } => visit(receiver),
        Expr::Closure { params, body } => {
            let mut nested = locals.clone();
            nested.extend(params.iter().cloned());
            collect_shared_expr(
                body,
                false,
                shared_roots,
                regions,
                &nested,
                held,
                direct,
                function,
                span,
                errors,
            );
        }
        Expr::Binary { lhs, rhs, .. } => {
            visit(lhs);
            visit(rhs);
        }
        Expr::Index { base, index } => {
            visit(base);
            match index {
                IndexArg::Single(e) | IndexArg::RangeTo(e) | IndexArg::RangeFrom(e) => visit(e),
                IndexArg::Range(lo, hi) => {
                    visit(lo);
                    visit(hi);
                }
            }
        }
        Expr::Tuple(items) => {
            for item in items {
                visit(item);
            }
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                visit(value);
            }
        }
        Expr::Is { scrutinee, .. } => visit(scrutinee),
        Expr::Quantifier { domain, body, .. } => {
            visit(domain);
            visit(body);
        }
        Expr::Match { scrutinee, arms } => {
            visit(scrutinee);
            for arm in arms {
                let mut arm_locals = locals.clone();
                collect_pattern_bindings(&arm.pattern, &mut arm_locals);
                if let Some(guard) = &arm.guard {
                    collect_shared_expr(
                        guard,
                        false,
                        shared_roots,
                        regions,
                        &arm_locals,
                        held,
                        direct,
                        function,
                        span,
                        errors,
                    );
                }
                collect_shared_expr(
                    &arm.body,
                    false,
                    shared_roots,
                    regions,
                    &arm_locals,
                    held,
                    direct,
                    function,
                    span,
                    errors,
                );
            }
        }
        Expr::If { cond, then, else_ } => {
            visit(cond);
            let mut nested = locals.clone();
            collect_shared_place_effects(
                then,
                shared_roots,
                regions,
                &mut nested,
                &mut held.to_vec(),
                direct,
                function,
                span,
                errors,
            );
            let mut nested = locals.clone();
            collect_shared_place_effects(
                else_,
                shared_roots,
                regions,
                &mut nested,
                &mut held.to_vec(),
                direct,
                function,
                span,
                errors,
            );
        }
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => {}
    }
}

fn collect_pattern_bindings(pattern: &thermite_syntax::Pattern, out: &mut BTreeSet<String>) {
    use thermite_syntax::{Pattern, SlicePat};
    match pattern {
        Pattern::Binding(name) => {
            out.insert(name.clone());
        }
        Pattern::Slice(parts) => {
            for part in parts {
                match part {
                    SlicePat::Pat(pattern) => collect_pattern_bindings(pattern, out),
                    SlicePat::Rest(name) => {
                        out.insert(name.clone());
                    }
                }
            }
        }
        Pattern::Enum { fields, .. } | Pattern::Or(fields) => {
            for field in fields {
                collect_pattern_bindings(field, out);
            }
        }
        Pattern::Struct { fields, .. } => {
            for (_, field) in fields {
                collect_pattern_bindings(field, out);
            }
        }
        Pattern::Wildcard | Pattern::Literal(_) => {}
    }
}

fn shared_read_is_copy(ty: &Type) -> bool {
    match ty {
        Type::Prim(_) | Type::Unit | Type::Ref { .. } => true,
        Type::Tuple(items) => items.iter().all(shared_read_is_copy),
        Type::Named(_)
        | Type::Slice(_)
        | Type::String
        | Type::Vec(_)
        | Type::Map(_, _)
        | Type::Box(_)
        | Type::Option(_)
        | Type::Result(_, _)
        | Type::Generic { .. } => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn record_shared_clone(
    path: &RegionPath,
    regions: Option<&thermite_spec::RegionIndex>,
    held: &[String],
    direct: &mut BTreeSet<Effect>,
    function: &str,
    span: Span,
    errors: &mut Vec<LowerError>,
) {
    if let Some(regions) = regions {
        if let Err(error) = regions.resolve(path) {
            errors.push(LowerError::EffectAnalysis {
                detail: format!(
                    "function `{function}` has invalid shared place `{path}`: {error:?}"
                ),
                span,
            });
            return;
        }
        for (lock, _) in regions
            .locks()
            .filter(|(_, guard)| regions.overlaps(path, guard))
        {
            if !held.iter().any(|held_lock| held_lock == lock) {
                errors.push(LowerError::EffectAnalysis { detail: format!("function `{function}` accesses shared place `{path}` outside `holding {lock}`"), span });
            }
        }
    }
    direct.insert(Effect::Read(path.clone()));
}

fn shared_place_path(
    expr: &Expr,
    shared_roots: &BTreeSet<String>,
    locals: &BTreeSet<String>,
) -> Option<RegionPath> {
    fn segments(expr: &Expr, out: &mut Vec<String>) -> bool {
        match expr {
            Expr::Path(path) if path.len() == 1 => {
                out.push(path[0].clone());
                true
            }
            Expr::Field { receiver, name } if segments(receiver, out) => {
                out.push(name.clone());
                true
            }
            _ => false,
        }
    }
    let mut path = Vec::new();
    if !segments(expr, &mut path)
        || path.is_empty()
        || locals.contains(&path[0])
        || !shared_roots.contains(&path[0])
    {
        return None;
    }
    Some(RegionPath { segments: path })
}

fn collect_holding_effects(block: &Block, direct: &mut BTreeSet<Effect>) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Holding { lock, body, .. } => {
                direct.insert(Effect::Owns(lock.clone()));
                collect_holding_effects(body, direct);
            }
            Stmt::If { cond, then, else_ } => {
                collect_holding_expr(cond, direct);
                collect_holding_effects(then, direct);
                if let Some(other) = else_ {
                    collect_holding_effects(other, direct);
                }
            }
            Stmt::Loop(loop_) => {
                if let LoopKind::While(cond) = &loop_.kind {
                    collect_holding_expr(cond, direct);
                }
                collect_holding_effects(&loop_.body, direct);
            }
            Stmt::Let { init, .. } => collect_holding_expr(init, direct),
            Stmt::Assign { target, value } => {
                collect_holding_expr(target, direct);
                collect_holding_expr(value, direct);
            }
            Stmt::Return(Some(expr)) | Stmt::Expr(expr) | Stmt::Forget { value: expr, .. } => {
                collect_holding_expr(expr, direct)
            }
            Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        }
    }
    if let Some(tail) = &block.tail {
        collect_holding_expr(tail, direct);
    }
}

fn collect_holding_expr(expr: &Expr, direct: &mut BTreeSet<Effect>) {
    let mut visit = |child: &Expr| collect_holding_expr(child, direct);
    match expr {
        Expr::If { cond, then, else_ } => {
            visit(cond);
            collect_holding_effects(then, direct);
            collect_holding_effects(else_, direct);
        }
        Expr::Call { callee, args } => {
            visit(callee);
            for arg in args {
                visit(arg);
            }
        }
        Expr::MethodCall { receiver, args, .. } => {
            visit(receiver);
            for arg in args {
                visit(arg);
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. }
        | Expr::Closure { body: receiver, .. }
        | Expr::Is {
            scrutinee: receiver,
            ..
        } => visit(receiver),
        Expr::Binary { lhs, rhs, .. } => {
            visit(lhs);
            visit(rhs);
        }
        Expr::Index { base, index } => {
            visit(base);
            match index {
                IndexArg::Single(e) | IndexArg::RangeTo(e) | IndexArg::RangeFrom(e) => visit(e),
                IndexArg::Range(lo, hi) => {
                    visit(lo);
                    visit(hi);
                }
            }
        }
        Expr::Tuple(items) => {
            for item in items {
                visit(item);
            }
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                visit(value);
            }
        }
        Expr::Quantifier { domain, body, .. } => {
            visit(domain);
            visit(body);
        }
        Expr::Match { scrutinee, arms } => {
            visit(scrutinee);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    visit(guard);
                }
                visit(&arm.body);
            }
        }
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => {}
    }
}

fn check_holding_order(
    block: &Block,
    regions: &thermite_spec::RegionIndex,
    held: &mut Vec<String>,
    function: &str,
    errors: &mut Vec<LowerError>,
) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Holding { lock, body, span } => {
                if regions.guarded_region(lock).is_none() {
                    errors.push(LowerError::EffectAnalysis {
                        detail: format!("function `{function}` holds unknown lock `{lock}`"),
                        span: *span,
                    });
                } else if let Some(outer) = held.last() {
                    if outer == lock {
                        errors.push(LowerError::EffectAnalysis {
                            detail: format!("function `{function}` reentrantly holds `{lock}`"),
                            span: *span,
                        });
                    } else if !regions.is_after(lock, outer) {
                        errors.push(LowerError::EffectAnalysis { detail: format!("function `{function}` holds `{outer}` and takes `{lock}` without `lock {lock} ... after {outer}`"), span: *span });
                    }
                }
                held.push(lock.clone());
                check_holding_order(body, regions, held, function, errors);
                held.pop();
            }
            Stmt::If { cond, then, else_ } => {
                visit_expr_blocks(cond, &mut |nested| {
                    check_holding_order(nested, regions, held, function, errors)
                });
                check_holding_order(then, regions, held, function, errors);
                if let Some(other) = else_ {
                    check_holding_order(other, regions, held, function, errors);
                }
            }
            Stmt::Loop(loop_) => {
                if let LoopKind::While(cond) = &loop_.kind {
                    visit_expr_blocks(cond, &mut |nested| {
                        check_holding_order(nested, regions, held, function, errors)
                    });
                }
                check_holding_order(&loop_.body, regions, held, function, errors);
            }
            Stmt::Let { init, .. } => visit_expr_blocks(init, &mut |nested| {
                check_holding_order(nested, regions, held, function, errors)
            }),
            Stmt::Assign { target, value } => {
                visit_expr_blocks(target, &mut |nested| {
                    check_holding_order(nested, regions, held, function, errors)
                });
                visit_expr_blocks(value, &mut |nested| {
                    check_holding_order(nested, regions, held, function, errors)
                });
            }
            Stmt::Return(Some(expr)) | Stmt::Expr(expr) | Stmt::Forget { value: expr, .. } => {
                visit_expr_blocks(expr, &mut |nested| {
                    check_holding_order(nested, regions, held, function, errors)
                })
            }
            Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        }
    }
    if let Some(tail) = &block.tail {
        visit_expr_blocks(tail, &mut |nested| {
            check_holding_order(nested, regions, held, function, errors)
        });
    }
}

fn check_holding_callees(
    block: &Block,
    function_names: &BTreeSet<String>,
    footprints: &BTreeMap<String, BTreeSet<Effect>>,
    regions: Option<&thermite_spec::RegionIndex>,
    function: &str,
    errors: &mut Vec<LowerError>,
) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Holding { lock, body, span } => {
                let called = RefCell::new(BTreeSet::new());
                let resolve = |name: &str| {
                    if function_names.contains(name) {
                        called.borrow_mut().insert(name.to_string());
                    }
                    Callee::Pure
                };
                let mut ignored = Vec::new();
                check_block(
                    body,
                    &EffectRow::Pure,
                    function,
                    *span,
                    &resolve,
                    0,
                    &mut ignored,
                );
                for callee in called.into_inner() {
                    if let Some(effects) = footprints.get(&callee) {
                        if let Some(regions) = regions {
                            if let Some(guard) = regions.guarded_region(lock) {
                                if !thermite_spec::effect_commutation::footprint_frames_region(
                                    effects, guard, regions,
                                ) {
                                    errors.push(LowerError::EffectAnalysis { detail: format!("function `{function}` calls `{callee}` while holding `{lock}`, but the callee footprint does not frame guarded region `{guard}`"), span: *span });
                                }
                            }
                        }
                        for owned in effects.iter().filter_map(|effect| match effect {
                            Effect::Owns(owned) => Some(owned),
                            _ => None,
                        }) {
                            if owned == lock {
                                errors.push(LowerError::EffectAnalysis { detail: format!("function `{function}` calls `{callee}` while holding `{lock}`, but the callee transitively owns the same lock"), span: *span });
                            } else if regions.is_some_and(|regions| !regions.is_after(owned, lock))
                            {
                                errors.push(LowerError::EffectAnalysis { detail: format!("function `{function}` calls `{callee}` while holding `{lock}`, but the callee transitively takes `{owned}` without `lock {owned} ... after {lock}`"), span: *span });
                            }
                        }
                    }
                }
                check_holding_callees(body, function_names, footprints, regions, function, errors);
            }
            Stmt::If { cond, then, else_ } => {
                visit_expr_blocks(cond, &mut |nested| {
                    check_holding_callees(
                        nested,
                        function_names,
                        footprints,
                        regions,
                        function,
                        errors,
                    )
                });
                check_holding_callees(then, function_names, footprints, regions, function, errors);
                if let Some(other) = else_ {
                    check_holding_callees(
                        other,
                        function_names,
                        footprints,
                        regions,
                        function,
                        errors,
                    );
                }
            }
            Stmt::Loop(loop_) => {
                if let LoopKind::While(cond) = &loop_.kind {
                    visit_expr_blocks(cond, &mut |nested| {
                        check_holding_callees(
                            nested,
                            function_names,
                            footprints,
                            regions,
                            function,
                            errors,
                        )
                    });
                }
                check_holding_callees(
                    &loop_.body,
                    function_names,
                    footprints,
                    regions,
                    function,
                    errors,
                );
            }
            Stmt::Let { init, .. } => visit_expr_blocks(init, &mut |nested| {
                check_holding_callees(
                    nested,
                    function_names,
                    footprints,
                    regions,
                    function,
                    errors,
                )
            }),
            Stmt::Assign { target, value } => {
                visit_expr_blocks(target, &mut |nested| {
                    check_holding_callees(
                        nested,
                        function_names,
                        footprints,
                        regions,
                        function,
                        errors,
                    )
                });
                visit_expr_blocks(value, &mut |nested| {
                    check_holding_callees(
                        nested,
                        function_names,
                        footprints,
                        regions,
                        function,
                        errors,
                    )
                });
            }
            Stmt::Return(Some(expr)) | Stmt::Expr(expr) | Stmt::Forget { value: expr, .. } => {
                visit_expr_blocks(expr, &mut |nested| {
                    check_holding_callees(
                        nested,
                        function_names,
                        footprints,
                        regions,
                        function,
                        errors,
                    )
                })
            }
            Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        }
    }
    if let Some(tail) = &block.tail {
        visit_expr_blocks(tail, &mut |nested| {
            check_holding_callees(
                nested,
                function_names,
                footprints,
                regions,
                function,
                errors,
            )
        });
    }
}

fn visit_expr_blocks<'a>(expr: &'a Expr, visit: &mut impl FnMut(&'a Block)) {
    match expr {
        Expr::If { cond, then, else_ } => {
            visit_expr_blocks(cond, visit);
            visit(then);
            visit(else_);
        }
        Expr::Call { callee, args } => {
            visit_expr_blocks(callee, visit);
            for arg in args {
                visit_expr_blocks(arg, visit);
            }
        }
        Expr::MethodCall { receiver, args, .. } => {
            visit_expr_blocks(receiver, visit);
            for arg in args {
                visit_expr_blocks(arg, visit);
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. }
        | Expr::Closure { body: receiver, .. }
        | Expr::Is {
            scrutinee: receiver,
            ..
        } => visit_expr_blocks(receiver, visit),
        Expr::Binary { lhs, rhs, .. } => {
            visit_expr_blocks(lhs, visit);
            visit_expr_blocks(rhs, visit);
        }
        Expr::Index { base, index } => {
            visit_expr_blocks(base, visit);
            match index {
                IndexArg::Single(expr) | IndexArg::RangeTo(expr) | IndexArg::RangeFrom(expr) => {
                    visit_expr_blocks(expr, visit)
                }
                IndexArg::Range(lo, hi) => {
                    visit_expr_blocks(lo, visit);
                    visit_expr_blocks(hi, visit);
                }
            }
        }
        Expr::Tuple(items) => {
            for item in items {
                visit_expr_blocks(item, visit);
            }
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                visit_expr_blocks(value, visit);
            }
        }
        Expr::Quantifier { domain, body, .. } => {
            visit_expr_blocks(domain, visit);
            visit_expr_blocks(body, visit);
        }
        Expr::Match { scrutinee, arms } => {
            visit_expr_blocks(scrutinee, visit);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    visit_expr_blocks(guard, visit);
                }
                visit_expr_blocks(&arm.body, visit);
            }
        }
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => {}
    }
}

pub(crate) fn row_effects(row: &EffectRow) -> BTreeSet<Effect> {
    match row {
        EffectRow::Pure => BTreeSet::new(),
        EffectRow::Set(effects) => effects.iter().cloned().collect(),
    }
}

/// Closed mapping for effectful intrinsic call names that the expression walk
/// can identify without type reconstruction. Ambiguous names remain for the
/// typed lowering hook rather than being guessed here.
pub(crate) fn intrinsic_effect(name: &str) -> Option<Effect> {
    match name {
        "__string_literal"
        | "__owned_constructor"
        | "push"
        | "push_byte"
        | "insert"
        | "concat"
        | "slice"
        | "to_string"
        | "split"
        | "trim" => Some(Effect::Alloc),
        _ => None,
    }
}

/// Canonical effect of a syntactic free-call path. This is shared by the
/// production analysis and the checked-traversal projection so constructor
/// syntax has one interpretation table.
pub(crate) fn call_path_effect(path: &[String]) -> Option<Effect> {
    owned_constructor_effect(path).or_else(|| path.last().and_then(|name| intrinsic_effect(name)))
}

pub(crate) fn owned_constructor_effect(path: &[String]) -> Option<Effect> {
    match path {
        [owner, operation]
            if (matches!(owner.as_str(), "Vec" | "Map" | "String" | "Box")
                && operation == "new")
                || (owner == "String" && operation == "from_byte") =>
        {
            Some(Effect::Alloc)
        }
        _ => None,
    }
}

/// Walk a block's statements (and its tail), checking every `Call`/`MethodCall`
/// against the caller's row (REQ-3). `depth` bounds the recursion (AC-5).
#[allow(
    clippy::too_many_arguments,
    reason = "the walk threads caller row, name, span, resolver, depth, and the error sink through one recursive family; bundling them into a context struct would not reduce the surface"
)]
fn check_block(
    block: &Block,
    caller_fx: &EffectRow,
    caller_name: &str,
    caller_span: Span,
    resolve: &dyn Fn(&str) -> Callee,
    depth: usize,
    errors: &mut Vec<LowerError>,
) {
    if depth >= MAX_WALK_DEPTH {
        errors.push(LowerError::TooDeep {
            limit: MAX_WALK_DEPTH,
            span: caller_span,
        });
        return;
    }
    let d = depth + 1;
    for stmt in &block.stmts {
        match stmt {
            Stmt::Let { init, .. } => check_expr(
                init,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            ),
            Stmt::Assign { target, value } => {
                check_expr(
                    target,
                    caller_fx,
                    caller_name,
                    caller_span,
                    resolve,
                    d,
                    errors,
                );
                check_expr(
                    value,
                    caller_fx,
                    caller_name,
                    caller_span,
                    resolve,
                    d,
                    errors,
                );
            }
            Stmt::Return(Some(e)) | Stmt::Expr(e) | Stmt::Forget { value: e, .. } => {
                check_expr(e, caller_fx, caller_name, caller_span, resolve, d, errors)
            }
            Stmt::Return(None) => {}
            Stmt::If { cond, then, else_ } => {
                check_expr(
                    cond,
                    caller_fx,
                    caller_name,
                    caller_span,
                    resolve,
                    d,
                    errors,
                );
                check_block(
                    then,
                    caller_fx,
                    caller_name,
                    caller_span,
                    resolve,
                    d,
                    errors,
                );
                if let Some(e) = else_ {
                    check_block(e, caller_fx, caller_name, caller_span, resolve, d, errors);
                }
            }
            Stmt::Loop(l) => {
                // A `while <cond> { .. }` evaluates `<cond>` at runtime before
                // each iteration; `lower.rs` lowers it (`LoopKind::While(c)` arm),
                // so a `Call` in the condition is a reachable callee and is
                // checked (§4.1: a caller subsumes every callee's row). The
                // `loop` keyword has no condition. The `invs`/`dec` clauses are
                // spec/contract positions (§4.2 spec sublanguage is pure by
                // construction), so they are not walked, matching `check_expr`'s
                // documented "Loop/spec clauses are not walked" discipline.
                if let LoopKind::While(cond) = &l.kind {
                    check_expr(
                        cond,
                        caller_fx,
                        caller_name,
                        caller_span,
                        resolve,
                        d,
                        errors,
                    );
                }
                check_block(
                    &l.body,
                    caller_fx,
                    caller_name,
                    caller_span,
                    resolve,
                    d,
                    errors,
                );
            }
            Stmt::Holding { body, .. } => check_block(
                body,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            ),
            // break/continue are loop-control statements with no sub-expression
            // and no callee (#93): they contribute no effect to the row walk
            // (the layer-neutral value, verus-lowering.md REQ-12).
            Stmt::Break | Stmt::Continue => {}
        }
    }
    if let Some(tail) = &block.tail {
        check_expr(
            tail,
            caller_fx,
            caller_name,
            caller_span,
            resolve,
            d,
            errors,
        );
    }
}

/// Walk an expression tree, checking every `Call`/`MethodCall` whose callee
/// resolves to a declared `FnItem` against the caller's row (REQ-2/REQ-3).
/// `depth` bounds the recursion (AC-5). A `while` loop condition is runtime
/// code and is walked (in the `Stmt::Loop` arm of `check_block`); the loop's
/// `inv`/`dec` spec clauses are not walked, since contract/spec positions are
/// pure by construction (§4.2).
fn check_expr(
    expr: &Expr,
    caller_fx: &EffectRow,
    caller_name: &str,
    caller_span: Span,
    resolve: &dyn Fn(&str) -> Callee,
    depth: usize,
    errors: &mut Vec<LowerError>,
) {
    if depth >= MAX_WALK_DEPTH {
        errors.push(LowerError::TooDeep {
            limit: MAX_WALK_DEPTH,
            span: caller_span,
        });
        return;
    }
    let d = depth + 1;
    match expr {
        Expr::Call { callee, args } => {
            // Resolve the callee by its path's last segment (the frontend is
            // registry-free; combinator/fn calls are plain `Expr::Call` with a
            // `Path` callee; `ast.rs` module doc / lower.rs precedent).
            if let Expr::Path(segs) = callee.as_ref() {
                if owned_constructor_effect(segs).is_some() {
                    check_call(
                        "__owned_constructor",
                        caller_fx,
                        caller_name,
                        caller_span,
                        resolve,
                        errors,
                    );
                } else if let Some(name) = segs.last() {
                    check_call(name, caller_fx, caller_name, caller_span, resolve, errors);
                }
            }
            check_expr(
                callee,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            );
            for a in args {
                check_expr(a, caller_fx, caller_name, caller_span, resolve, d, errors);
            }
        }
        Expr::MethodCall {
            receiver,
            name,
            args,
        } => {
            // A method call `recv.m(..)` resolves by the method name `m`. Only a
            // resolved `FnItem` triggers a subsumption check; an unresolved
            // method name (intrinsics like `.len()`) is a no-op (AC-5).
            if intrinsic_effect(name).is_some() {
                check_call(
                    "__owned_constructor",
                    caller_fx,
                    caller_name,
                    caller_span,
                    resolve,
                    errors,
                );
            } else {
                check_call(name, caller_fx, caller_name, caller_span, resolve, errors);
            }
            check_expr(
                receiver,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            );
            for a in args {
                check_expr(a, caller_fx, caller_name, caller_span, resolve, d, errors);
            }
        }
        Expr::Field { receiver, .. } => check_expr(
            receiver,
            caller_fx,
            caller_name,
            caller_span,
            resolve,
            d,
            errors,
        ),
        Expr::Closure { body, .. } => check_expr(
            body,
            caller_fx,
            caller_name,
            caller_span,
            resolve,
            d,
            errors,
        ),
        Expr::Match { scrutinee, arms } => {
            check_expr(
                scrutinee,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            );
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    check_expr(
                        guard,
                        caller_fx,
                        caller_name,
                        caller_span,
                        resolve,
                        d,
                        errors,
                    );
                }
                check_expr(
                    &arm.body,
                    caller_fx,
                    caller_name,
                    caller_span,
                    resolve,
                    d,
                    errors,
                );
            }
        }
        Expr::If { cond, then, else_ } => {
            check_expr(
                cond,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            );
            check_block(
                then,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            );
            check_block(
                else_,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            );
        }
        Expr::Binary { lhs, rhs, .. } => {
            check_expr(lhs, caller_fx, caller_name, caller_span, resolve, d, errors);
            check_expr(rhs, caller_fx, caller_name, caller_span, resolve, d, errors);
        }
        Expr::Index { base, index } => {
            check_expr(
                base,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            );
            match index {
                IndexArg::Single(e) | IndexArg::RangeTo(e) | IndexArg::RangeFrom(e) => {
                    check_expr(e, caller_fx, caller_name, caller_span, resolve, d, errors)
                }
                IndexArg::Range(a, b) => {
                    check_expr(a, caller_fx, caller_name, caller_span, resolve, d, errors);
                    check_expr(b, caller_fx, caller_name, caller_span, resolve, d, errors);
                }
            }
        }
        Expr::Cast { expr, .. } | Expr::Ref { expr, .. } => check_expr(
            expr,
            caller_fx,
            caller_name,
            caller_span,
            resolve,
            d,
            errors,
        ),
        // Basis Stage 1a (`.design/basis/01-adts.md`): the ADT expressions are
        // dead-in-1a (gated at the validator before effect-check), but the
        // walk descends into their sub-expressions: a call carrying an
        // effect could sit in a struct-literal field value, an `is` scrutinee,
        // or a deref operand, so subsumption must not silently skip it.
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                check_expr(
                    value,
                    caller_fx,
                    caller_name,
                    caller_span,
                    resolve,
                    d,
                    errors,
                );
            }
        }
        Expr::Is { scrutinee, .. } => check_expr(
            scrutinee,
            caller_fx,
            caller_name,
            caller_span,
            resolve,
            d,
            errors,
        ),
        Expr::Deref(inner) => check_expr(
            inner,
            caller_fx,
            caller_name,
            caller_span,
            resolve,
            d,
            errors,
        ),
        // The prefix `!` (#92): descend into the operand so an effect-bearing call
        // under a `!` (e.g. `!has_net_access()`) is still subsumption-checked.
        Expr::Unary { expr, .. } => check_expr(
            expr,
            caller_fx,
            caller_name,
            caller_span,
            resolve,
            d,
            errors,
        ),
        // Cluster C9-B (`.design/basis/10-recursion-tuples.md` REQ-8, #109): a
        // tuple construction's effects are the union of its elements' effects, so
        // every element is effect-walked; a projection's effects are its
        // receiver's (the projection itself is pure). An effect-bearing call inside
        // a tuple element / under a projection is still subsumption-checked.
        Expr::Tuple(elems) => {
            for e in elems {
                check_expr(e, caller_fx, caller_name, caller_span, resolve, d, errors);
            }
        }
        Expr::TupleProj { receiver, .. } => check_expr(
            receiver,
            caller_fx,
            caller_name,
            caller_span,
            resolve,
            d,
            errors,
        ),
        // A string literal is a leaf (`.design/basis/07-strings.md` REQ-1): no
        // sub-expressions, no calls — it contributes no effect-row obligation, so
        // it joins the no-op leaf arm alongside `IntLit`/`BoolLit`. (Materializing
        // a literal into an owned `String` carries `fx alloc`, but that is keyed at
        // the lowering/constructing site in `lower.rs`, not at this effect walk —
        // the bare literal node has no callee to subsume.)
        // A raw quantified formula (`.design/stage2-stratified-cage.md` REQ-0): its
        // effects are the union of the domain's and body's, so an effect-bearing
        // call inside either is still subsumption-checked. The binder itself is pure.
        Expr::Quantifier { domain, body, .. } => {
            check_expr(
                domain,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            );
            check_expr(
                body,
                caller_fx,
                caller_name,
                caller_span,
                resolve,
                d,
                errors,
            );
        }
        Expr::StrLit(_) => check_call(
            "__string_literal",
            caller_fx,
            caller_name,
            caller_span,
            resolve,
            errors,
        ),
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) => {}
    }
}

/// Resolve a single call-site callee name and, if it is a declared `FnItem`,
/// assert the caller's row subsumes it (REQ-2/REQ-3), pushing an
/// `EffectNotSubsumed` with the exact `missing` set on failure (REQ-4). A `spec
/// fn` / combinator callee is pure ⇒ always subsumed; an unresolved callee is a
/// no-op (AC-5).
fn check_call(
    name: &str,
    _caller_fx: &EffectRow,
    _caller_name: &str,
    _caller_span: Span,
    resolve: &dyn Fn(&str) -> Callee,
    _errors: &mut Vec<LowerError>,
) {
    match resolve(name) {
        // Merely resolving the call records its graph edge in `analyze_effects`.
        // Authorization happens once, after the least fixed point, so an
        // overdeclared callee cannot pollute its callers' inferred footprints.
        Callee::Fn => {}
        Callee::Pure | Callee::Unresolved => {}
    }
}

#[cfg(test)]
mod differential_tests {
    use super::*;

    #[test]
    fn canonical_differential_rejection_branch_is_live() {
        let optimized = BTreeMap::from([("f".to_string(), vec!["Alloc".to_string()])]);
        let canonical = BTreeMap::from([("f".to_string(), Vec::new())]);
        let calls = BTreeMap::from([("f".to_string(), Vec::new())]);
        let error =
            ensure_canonical_effect_agreement(&optimized, &calls, &canonical, &calls).unwrap_err();
        assert!(error.to_string().contains("diverged from the canonical"));
    }
}
