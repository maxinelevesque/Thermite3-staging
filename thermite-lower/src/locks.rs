//! RFC-10 executable lock-provider boundary.

use std::collections::BTreeSet;

use thermite_syntax::{Block, Expr, IndexArg, Item, LoopKind, Program, Stmt, Type};

use crate::LowerError;

/// Target-owned executable synchronization plus the evidence Thermite requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockProvider {
    pub name: String,
    pub rust_source: String,
    /// Provider declarations/proofs emitted inside the L3 `verus!` module.
    pub verus_source: String,
    pub proves_exclusive_acquire: bool,
    pub proves_restore_before_release: bool,
    pub states_interrupt_policy: bool,
}

impl LockProvider {
    pub fn validate(&self) -> Result<(), LowerError> {
        let mut missing = Vec::new();
        if !self.proves_exclusive_acquire {
            missing.push("exclusive acquisition");
        }
        if !self.proves_restore_before_release {
            missing.push("restoration before release");
        }
        if !self.states_interrupt_policy {
            missing.push("interrupt policy");
        }
        if self.name.trim().is_empty() {
            missing.push("provider identity");
        }
        if !missing.is_empty() {
            return Err(LowerError::Unsupported {
                what: format!(
                    "lock provider `{}` lacks evidence for {}",
                    self.name,
                    missing.join(", ")
                ),
                span: thermite_syntax::Span::new(0, 0),
            });
        }
        Ok(())
    }

    pub fn validate_l3(&self) -> Result<(), LowerError> {
        self.validate()?;
        if self.verus_source.trim().is_empty() {
            return Err(LowerError::Unsupported {
                what: format!(
                    "lock provider `{}` has no L3 verification integration",
                    self.name
                ),
                span: thermite_syntax::Span::new(0, 0),
            });
        }
        Ok(())
    }

    pub fn acquire_symbol(lock: &str) -> String {
        format!("__thermite_lock_acquire_{}", symbol_suffix(lock))
    }

    pub fn release_symbol(lock: &str) -> String {
        format!("__thermite_lock_release_{}", symbol_suffix(lock))
    }

    pub fn shared_symbol(root: &str) -> String {
        format!("__thermite_shared_{}", symbol_suffix(root))
    }

    pub fn close_symbol(lock: &str) -> String {
        format!("__thermite_close_{}", symbol_suffix(lock))
    }
}

/// Rewrite validated shared roots to target-provider accessor places. This is
/// deliberately post-checking: source semantics and diagnostics retain the
/// ordinary shared name, while executable Rust sees only the provider seam.
pub fn rewrite_shared_places(program: &Program) -> Program {
    let roots: BTreeSet<String> = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::SharedDecl(shared) => Some(shared.name.clone()),
            _ => None,
        })
        .collect();
    let mut rewritten = program.clone();
    for item in &mut rewritten.items {
        let Item::Fn(function) = item else { continue };
        let mut locals: BTreeSet<String> = function
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        if let Some(body) = &mut function.body {
            rewrite_block(body, &roots, &mut locals);
        }
    }
    rewritten
}

/// Materialize the semantic close operation on every edge that leaves a
/// `holding` scope. L3 uses this normalized tree so verification cannot mistake
/// a final-block assertion for coverage of `return`, loop control, or a tail
/// expression. L1 retains its drop guard as the unwind backstop.
pub fn normalize_holding_closes(program: &Program) -> Program {
    let mut normalized = program.clone();
    let mut next_temp = 0usize;
    for item in &mut normalized.items {
        let Item::Fn(function) = item else { continue };
        if let Some(body) = &mut function.body {
            normalize_close_block(body, &[], 0, &mut next_temp);
        }
    }
    normalized
}

/// Prepare the provider-backed L3 tree. Acquisition yields the one lexical
/// mutable capability used for all accesses to that shared root; close consumes
/// the same capability. This makes `well_formed` a genuine close precondition,
/// rather than something a fresh accessor could promise vacuously.
pub fn prepare_l3_shared(program: &Program) -> Result<Program, LowerError> {
    let regions =
        thermite_spec::RegionIndex::build(program).map_err(|errors| LowerError::Unsupported {
            what: format!("cannot prepare L3 shared storage: {errors:?}"),
            span: thermite_syntax::Span::new(0, 0),
        })?;
    let mut prepared = rewrite_shared_places(program);
    let mut next_binding = 0usize;
    let mut next_temp = 0usize;
    for item in &mut prepared.items {
        let Item::Fn(function) = item else { continue };
        if let Some(body) = &mut function.body {
            prepare_l3_block(body, &regions, &[], 0, &mut next_binding, &mut next_temp)?;
        }
    }
    Ok(prepared)
}

/// Verification-only declarations for the target lock seam. `forge check`
/// proves source functions against this explicit boundary contract; executable
/// artifact lowering still requires a concrete, validated [`LockProvider`].
pub fn verification_lock_provider_source(program: &Program) -> Result<String, LowerError> {
    let regions =
        thermite_spec::RegionIndex::build(program).map_err(|errors| LowerError::Unsupported {
            what: format!("cannot prepare verification lock seam: {errors:?}"),
            span: thermite_syntax::Span::new(0, 0),
        })?;
    let mut out = String::new();
    for item in &program.items {
        let Item::LockDecl(lock) = item else { continue };
        let root = lock
            .guards
            .segments
            .first()
            .ok_or_else(|| LowerError::Unsupported {
                what: format!("lock `{}` guards an empty region", lock.name),
                span: lock.span,
            })?;
        let shared = program
            .items
            .iter()
            .find_map(|item| match item {
                Item::SharedDecl(shared) if shared.name == *root => Some(shared),
                _ => None,
            })
            .ok_or_else(|| LowerError::Unsupported {
                what: format!("lock `{}` has no declared shared root `{root}`", lock.name),
                span: lock.span,
            })?;
        let Type::Named(root_ty) = &shared.ty else {
            return Err(LowerError::Unsupported {
                what: format!(
                    "verification lock `{}` requires a named shared-root type",
                    lock.name
                ),
                span: shared.span,
            });
        };
        let invariant =
            regions
                .invariant_region(&lock.guards)
                .ok_or_else(|| LowerError::Unsupported {
                    what: format!(
                        "lock `{}` does not guard an invariant-bearing region",
                        lock.name
                    ),
                    span: lock.span,
                })?;
        let suffix = invariant
            .segments
            .iter()
            .skip(1)
            .map(|s| format!(".{s}"))
            .collect::<String>();
        out.push_str(&format!(
            "#[verifier::external_body]\nfn {}() -> (state: {})\n    ensures state{}.well_formed()\n{{ unimplemented!() }}\n#[verifier::external_body]\nfn {}(state: &mut {})\n    requires state{}.well_formed()\n{{ unimplemented!() }}\n",
            LockProvider::acquire_symbol(&lock.name), root_ty, suffix,
            LockProvider::close_symbol(&lock.name), root_ty, suffix,
        ));
    }
    Ok(out)
}

#[derive(Clone)]
struct L3Binding {
    lock: String,
    root_accessor: String,
    name: String,
    loop_depth: usize,
}

fn prepare_l3_block(
    block: &mut Block,
    regions: &thermite_spec::RegionIndex,
    held: &[L3Binding],
    loop_depth: usize,
    next_binding: &mut usize,
    next_temp: &mut usize,
) -> Result<(), LowerError> {
    if let Some(tail) = &mut block.tail {
        rewrite_bound_accesses(tail, held);
        prepare_l3_expr(tail, regions, held, loop_depth, next_binding, next_temp)?;
    }
    let old = std::mem::take(&mut block.stmts);
    let mut out = Vec::with_capacity(old.len());
    for mut stmt in old {
        match &mut stmt {
            Stmt::Let { init, .. } => {
                rewrite_bound_accesses(init, held);
                prepare_l3_expr(init, regions, held, loop_depth, next_binding, next_temp)?;
            }
            Stmt::Assign { target, value } => {
                rewrite_bound_accesses(target, held);
                rewrite_bound_accesses(value, held);
                prepare_l3_expr(target, regions, held, loop_depth, next_binding, next_temp)?;
                prepare_l3_expr(value, regions, held, loop_depth, next_binding, next_temp)?;
            }
            Stmt::Return(value) => {
                if let Some(value) = value {
                    rewrite_bound_accesses(value, held);
                    prepare_l3_expr(value, regions, held, loop_depth, next_binding, next_temp)?;
                }
                append_l3_closes(&mut out, held.iter());
            }
            Stmt::Break | Stmt::Continue => append_l3_closes(
                &mut out,
                held.iter()
                    .filter(|binding| binding.loop_depth == loop_depth),
            ),
            Stmt::Expr(expr) => {
                rewrite_bound_accesses(expr, held);
                prepare_l3_expr(expr, regions, held, loop_depth, next_binding, next_temp)?;
            }
            Stmt::If { cond, then, else_ } => {
                rewrite_bound_accesses(cond, held);
                prepare_l3_expr(cond, regions, held, loop_depth, next_binding, next_temp)?;
                prepare_l3_block(then, regions, held, loop_depth, next_binding, next_temp)?;
                if let Some(other) = else_ {
                    prepare_l3_block(other, regions, held, loop_depth, next_binding, next_temp)?;
                }
            }
            Stmt::Loop(loop_) => {
                if let LoopKind::While(cond) = &mut loop_.kind {
                    rewrite_bound_accesses(cond, held);
                    prepare_l3_expr(cond, regions, held, loop_depth, next_binding, next_temp)?;
                }
                prepare_l3_block(
                    &mut loop_.body,
                    regions,
                    held,
                    loop_depth + 1,
                    next_binding,
                    next_temp,
                )?;
            }
            Stmt::Holding {
                lock, body, span, ..
            } => {
                let guarded =
                    regions
                        .guarded_region(lock)
                        .ok_or_else(|| LowerError::Unsupported {
                            what: format!("unknown L3 lock `{lock}`"),
                            span: *span,
                        })?;
                let root = guarded
                    .segments
                    .first()
                    .ok_or_else(|| LowerError::Unsupported {
                        what: format!("lock `{lock}` guards an empty region"),
                        span: *span,
                    })?;
                let binding = L3Binding {
                    lock: lock.clone(),
                    root_accessor: LockProvider::shared_symbol(root),
                    name: format!("__thermite_lock_capability_{}", *next_binding),
                    loop_depth,
                };
                *next_binding += 1;
                let mut nested = held.to_vec();
                nested.push(binding.clone());
                prepare_l3_block(body, regions, &nested, loop_depth, next_binding, next_temp)?;
                normalize_l3_fallthrough(body, &binding, next_temp);
                body.stmts.insert(
                    0,
                    Stmt::Let {
                        mutable: true,
                        name: binding.name.clone(),
                        ty: None,
                        init: Expr::Call {
                            callee: Box::new(Expr::Path(vec![LockProvider::acquire_symbol(lock)])),
                            args: Vec::new(),
                        },
                    },
                );
            }
        }
        out.push(stmt);
    }
    block.stmts = out;
    Ok(())
}

fn prepare_l3_expr(
    expr: &mut Expr,
    regions: &thermite_spec::RegionIndex,
    held: &[L3Binding],
    loop_depth: usize,
    next_binding: &mut usize,
    next_temp: &mut usize,
) -> Result<(), LowerError> {
    match expr {
        Expr::Call { callee, args } => {
            prepare_l3_expr(callee, regions, held, loop_depth, next_binding, next_temp)?;
            for arg in args {
                prepare_l3_expr(arg, regions, held, loop_depth, next_binding, next_temp)?;
            }
        }
        Expr::MethodCall { receiver, args, .. } => {
            prepare_l3_expr(receiver, regions, held, loop_depth, next_binding, next_temp)?;
            for arg in args {
                prepare_l3_expr(arg, regions, held, loop_depth, next_binding, next_temp)?;
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. } => {
            prepare_l3_expr(receiver, regions, held, loop_depth, next_binding, next_temp)?;
        }
        Expr::Closure { body, .. } => {
            prepare_l3_expr(body, regions, held, loop_depth, next_binding, next_temp)?;
        }
        Expr::Binary { lhs, rhs, .. } => {
            prepare_l3_expr(lhs, regions, held, loop_depth, next_binding, next_temp)?;
            prepare_l3_expr(rhs, regions, held, loop_depth, next_binding, next_temp)?;
        }
        Expr::Index { base, index } => {
            prepare_l3_expr(base, regions, held, loop_depth, next_binding, next_temp)?;
            match index {
                IndexArg::Single(e) | IndexArg::RangeTo(e) | IndexArg::RangeFrom(e) => {
                    prepare_l3_expr(e, regions, held, loop_depth, next_binding, next_temp)?;
                }
                IndexArg::Range(lo, hi) => {
                    prepare_l3_expr(lo, regions, held, loop_depth, next_binding, next_temp)?;
                    prepare_l3_expr(hi, regions, held, loop_depth, next_binding, next_temp)?;
                }
            }
        }
        Expr::Tuple(items) => {
            for item in items {
                prepare_l3_expr(item, regions, held, loop_depth, next_binding, next_temp)?;
            }
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                prepare_l3_expr(value, regions, held, loop_depth, next_binding, next_temp)?;
            }
        }
        Expr::Is { scrutinee, .. } => {
            prepare_l3_expr(
                scrutinee,
                regions,
                held,
                loop_depth,
                next_binding,
                next_temp,
            )?;
        }
        Expr::Quantifier { domain, body, .. } => {
            prepare_l3_expr(domain, regions, held, loop_depth, next_binding, next_temp)?;
            prepare_l3_expr(body, regions, held, loop_depth, next_binding, next_temp)?;
        }
        Expr::Match { scrutinee, arms } => {
            prepare_l3_expr(
                scrutinee,
                regions,
                held,
                loop_depth,
                next_binding,
                next_temp,
            )?;
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    prepare_l3_expr(guard, regions, held, loop_depth, next_binding, next_temp)?;
                }
                prepare_l3_expr(
                    &mut arm.body,
                    regions,
                    held,
                    loop_depth,
                    next_binding,
                    next_temp,
                )?;
            }
        }
        Expr::If { cond, then, else_ } => {
            prepare_l3_expr(cond, regions, held, loop_depth, next_binding, next_temp)?;
            prepare_l3_block(then, regions, held, loop_depth, next_binding, next_temp)?;
            prepare_l3_block(else_, regions, held, loop_depth, next_binding, next_temp)?;
        }
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => {}
    }
    Ok(())
}

fn normalize_l3_fallthrough(block: &mut Block, binding: &L3Binding, next_temp: &mut usize) {
    if let Some(tail) = block.tail.take() {
        let temp = format!("__thermite_holding_value_{}", *next_temp);
        *next_temp += 1;
        block.stmts.push(Stmt::Let {
            mutable: false,
            name: temp.clone(),
            ty: None,
            init: *tail,
        });
        block.stmts.push(l3_close_call(binding));
        block.tail = Some(Box::new(Expr::Path(vec![temp])));
    } else if !block_guaranteed_exits(block) {
        block.stmts.push(l3_close_call(binding));
    }
}

fn block_guaranteed_exits(block: &Block) -> bool {
    if block.tail.is_some() {
        return false;
    }
    block.stmts.last().is_some_and(|stmt| match stmt {
        Stmt::Return(_) | Stmt::Break | Stmt::Continue => true,
        Stmt::Holding { body, .. } => block_guaranteed_exits(body),
        Stmt::If {
            then,
            else_: Some(other),
            ..
        } => block_guaranteed_exits(then) && block_guaranteed_exits(other),
        _ => false,
    })
}

fn append_l3_closes<'a>(out: &mut Vec<Stmt>, bindings: impl Iterator<Item = &'a L3Binding>) {
    let bindings: Vec<_> = bindings.collect();
    out.extend(bindings.into_iter().rev().map(l3_close_call));
}

fn l3_close_call(binding: &L3Binding) -> Stmt {
    Stmt::Expr(Expr::Call {
        callee: Box::new(Expr::Path(vec![LockProvider::close_symbol(&binding.lock)])),
        args: vec![Expr::Ref {
            mutable: true,
            expr: Box::new(Expr::Path(vec![binding.name.clone()])),
        }],
    })
}

fn rewrite_bound_accesses(expr: &mut Expr, held: &[L3Binding]) {
    if let Expr::Call { callee, args } = expr {
        if args.is_empty() {
            if let Expr::Path(path) = callee.as_ref() {
                if path.len() == 1 {
                    if let Some(binding) = held
                        .iter()
                        .rev()
                        .find(|binding| binding.root_accessor == path[0])
                    {
                        *expr = Expr::Path(vec![binding.name.clone()]);
                        return;
                    }
                }
            }
        }
    }
    match expr {
        Expr::Call { callee, args } => {
            rewrite_bound_accesses(callee, held);
            for arg in args {
                rewrite_bound_accesses(arg, held);
            }
        }
        Expr::MethodCall { receiver, args, .. } => {
            rewrite_bound_accesses(receiver, held);
            for arg in args {
                rewrite_bound_accesses(arg, held);
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. } => rewrite_bound_accesses(receiver, held),
        Expr::Closure { body, .. } => rewrite_bound_accesses(body, held),
        Expr::Binary { lhs, rhs, .. } => {
            rewrite_bound_accesses(lhs, held);
            rewrite_bound_accesses(rhs, held);
        }
        Expr::Index { base, index } => {
            rewrite_bound_accesses(base, held);
            match index {
                IndexArg::Single(e) | IndexArg::RangeTo(e) | IndexArg::RangeFrom(e) => {
                    rewrite_bound_accesses(e, held)
                }
                IndexArg::Range(lo, hi) => {
                    rewrite_bound_accesses(lo, held);
                    rewrite_bound_accesses(hi, held);
                }
            }
        }
        Expr::Tuple(items) => {
            for item in items {
                rewrite_bound_accesses(item, held);
            }
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                rewrite_bound_accesses(value, held);
            }
        }
        Expr::Is { scrutinee, .. } => rewrite_bound_accesses(scrutinee, held),
        Expr::Quantifier { domain, body, .. } => {
            rewrite_bound_accesses(domain, held);
            rewrite_bound_accesses(body, held);
        }
        Expr::Match { scrutinee, arms } => {
            rewrite_bound_accesses(scrutinee, held);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    rewrite_bound_accesses(guard, held);
                }
                rewrite_bound_accesses(&mut arm.body, held);
            }
        }
        Expr::If { cond, then, else_ } => {
            rewrite_bound_accesses(cond, held);
            rewrite_bound_block_only(then, held);
            rewrite_bound_block_only(else_, held);
        }
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => {}
    }
}

fn rewrite_bound_block_only(block: &mut Block, held: &[L3Binding]) {
    for stmt in &mut block.stmts {
        match stmt {
            Stmt::Let { init, .. } => rewrite_bound_accesses(init, held),
            Stmt::Assign { target, value } => {
                rewrite_bound_accesses(target, held);
                rewrite_bound_accesses(value, held);
            }
            Stmt::Return(Some(value)) | Stmt::Expr(value) => rewrite_bound_accesses(value, held),
            Stmt::If { cond, then, else_ } => {
                rewrite_bound_accesses(cond, held);
                rewrite_bound_block_only(then, held);
                if let Some(other) = else_ {
                    rewrite_bound_block_only(other, held);
                }
            }
            Stmt::Loop(loop_) => rewrite_bound_block_only(&mut loop_.body, held),
            Stmt::Holding { body, .. } => rewrite_bound_block_only(body, held),
            Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        }
    }
    if let Some(tail) = &mut block.tail {
        rewrite_bound_accesses(tail, held);
    }
}

fn normalize_close_block(
    block: &mut Block,
    held: &[(&str, usize)],
    loop_depth: usize,
    next_temp: &mut usize,
) {
    let old = std::mem::take(&mut block.stmts);
    let mut normalized = Vec::with_capacity(old.len());
    for mut stmt in old {
        match &mut stmt {
            Stmt::Return(_) => {
                append_close_calls(&mut normalized, held.iter().map(|(lock, _)| *lock))
            }
            Stmt::Break | Stmt::Continue => append_close_calls(
                &mut normalized,
                held.iter()
                    .filter(|(_, acquired_depth)| *acquired_depth == loop_depth)
                    .map(|(lock, _)| *lock),
            ),
            Stmt::If { then, else_, .. } => {
                normalize_close_block(then, held, loop_depth, next_temp);
                if let Some(other) = else_ {
                    normalize_close_block(other, held, loop_depth, next_temp);
                }
            }
            Stmt::Loop(loop_) => {
                normalize_close_block(&mut loop_.body, held, loop_depth + 1, next_temp);
            }
            Stmt::Holding { lock, body, .. } => {
                let mut nested = held.to_vec();
                nested.push((lock.as_str(), loop_depth));
                normalize_close_block(body, &nested, loop_depth, next_temp);
                normalize_holding_fallthrough(body, lock, next_temp);
                body.stmts.insert(0, acquire_call(lock));
            }
            Stmt::Let { init, .. } => normalize_close_expr(init, held, loop_depth, next_temp),
            Stmt::Assign { target, value } => {
                normalize_close_expr(target, held, loop_depth, next_temp);
                normalize_close_expr(value, held, loop_depth, next_temp);
            }
            Stmt::Expr(expr) => normalize_close_expr(expr, held, loop_depth, next_temp),
        }
        normalized.push(stmt);
    }
    block.stmts = normalized;
}

fn normalize_close_expr(
    expr: &mut Expr,
    held: &[(&str, usize)],
    loop_depth: usize,
    next_temp: &mut usize,
) {
    match expr {
        Expr::Call { callee, args } => {
            normalize_close_expr(callee, held, loop_depth, next_temp);
            for arg in args {
                normalize_close_expr(arg, held, loop_depth, next_temp);
            }
        }
        Expr::MethodCall { receiver, args, .. } => {
            normalize_close_expr(receiver, held, loop_depth, next_temp);
            for arg in args {
                normalize_close_expr(arg, held, loop_depth, next_temp);
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. } => {
            normalize_close_expr(receiver, held, loop_depth, next_temp)
        }
        Expr::Closure { body, .. } => normalize_close_expr(body, held, loop_depth, next_temp),
        Expr::Binary { lhs, rhs, .. } => {
            normalize_close_expr(lhs, held, loop_depth, next_temp);
            normalize_close_expr(rhs, held, loop_depth, next_temp);
        }
        Expr::Index { base, index } => {
            normalize_close_expr(base, held, loop_depth, next_temp);
            match index {
                IndexArg::Single(e) | IndexArg::RangeTo(e) | IndexArg::RangeFrom(e) => {
                    normalize_close_expr(e, held, loop_depth, next_temp)
                }
                IndexArg::Range(lo, hi) => {
                    normalize_close_expr(lo, held, loop_depth, next_temp);
                    normalize_close_expr(hi, held, loop_depth, next_temp);
                }
            }
        }
        Expr::Tuple(items) => {
            for item in items {
                normalize_close_expr(item, held, loop_depth, next_temp);
            }
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                normalize_close_expr(value, held, loop_depth, next_temp);
            }
        }
        Expr::Is { scrutinee, .. } => normalize_close_expr(scrutinee, held, loop_depth, next_temp),
        Expr::Quantifier { domain, body, .. } => {
            normalize_close_expr(domain, held, loop_depth, next_temp);
            normalize_close_expr(body, held, loop_depth, next_temp);
        }
        Expr::Match { scrutinee, arms } => {
            normalize_close_expr(scrutinee, held, loop_depth, next_temp);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    normalize_close_expr(guard, held, loop_depth, next_temp);
                }
                normalize_close_expr(&mut arm.body, held, loop_depth, next_temp);
            }
        }
        Expr::If { cond, then, else_ } => {
            normalize_close_expr(cond, held, loop_depth, next_temp);
            normalize_close_block(then, held, loop_depth, next_temp);
            normalize_close_block(else_, held, loop_depth, next_temp);
        }
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => {}
    }
}

fn normalize_holding_fallthrough(block: &mut Block, lock: &str, next_temp: &mut usize) {
    if let Some(tail) = block.tail.take() {
        let temp = format!("__thermite_holding_value_{}", *next_temp);
        *next_temp += 1;
        block.stmts.push(Stmt::Let {
            mutable: false,
            name: temp.clone(),
            ty: None,
            init: *tail,
        });
        block.stmts.push(close_call(lock));
        block.tail = Some(Box::new(Expr::Path(vec![temp])));
    } else if !block_guaranteed_exits(block) {
        block.stmts.push(close_call(lock));
    }
}

fn append_close_calls<'a>(out: &mut Vec<Stmt>, locks: impl Iterator<Item = &'a str>) {
    let locks: Vec<_> = locks.collect();
    out.extend(locks.into_iter().rev().map(close_call));
}

fn close_call(lock: &str) -> Stmt {
    Stmt::Expr(Expr::Call {
        callee: Box::new(Expr::Path(vec![LockProvider::close_symbol(lock)])),
        args: Vec::new(),
    })
}

fn acquire_call(lock: &str) -> Stmt {
    Stmt::Expr(Expr::Call {
        callee: Box::new(Expr::Path(vec![LockProvider::acquire_symbol(lock)])),
        args: Vec::new(),
    })
}

fn rewrite_block(block: &mut Block, roots: &BTreeSet<String>, locals: &mut BTreeSet<String>) {
    let outer = locals.clone();
    for stmt in &mut block.stmts {
        match stmt {
            Stmt::Let { name, init, .. } => {
                rewrite_expr(init, roots, locals);
                locals.insert(name.clone());
            }
            Stmt::Assign { target, value } => {
                rewrite_expr(target, roots, locals);
                rewrite_expr(value, roots, locals);
            }
            Stmt::Return(Some(expr)) | Stmt::Expr(expr) => rewrite_expr(expr, roots, locals),
            Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
            Stmt::If { cond, then, else_ } => {
                rewrite_expr(cond, roots, locals);
                let mut branch = locals.clone();
                rewrite_block(then, roots, &mut branch);
                if let Some(other) = else_ {
                    let mut branch = locals.clone();
                    rewrite_block(other, roots, &mut branch);
                }
            }
            Stmt::Loop(loop_) => {
                if let LoopKind::While(cond) = &mut loop_.kind {
                    rewrite_expr(cond, roots, locals);
                }
                let mut nested = locals.clone();
                rewrite_block(&mut loop_.body, roots, &mut nested);
            }
            Stmt::Holding { body, .. } => {
                let mut nested = locals.clone();
                rewrite_block(body, roots, &mut nested);
            }
        }
    }
    if let Some(tail) = &mut block.tail {
        rewrite_expr(tail, roots, locals);
    }
    *locals = outer;
}

fn rewrite_expr(expr: &mut Expr, roots: &BTreeSet<String>, locals: &BTreeSet<String>) {
    if let Expr::Path(path) = expr {
        if path.len() == 1 && roots.contains(&path[0]) && !locals.contains(&path[0]) {
            let symbol = LockProvider::shared_symbol(&path[0]);
            *expr = Expr::Call {
                callee: Box::new(Expr::Path(vec![symbol])),
                args: Vec::new(),
            };
            return;
        }
    }
    match expr {
        Expr::Call { callee, args } => {
            rewrite_expr(callee, roots, locals);
            for arg in args {
                rewrite_expr(arg, roots, locals);
            }
        }
        Expr::MethodCall { receiver, args, .. } => {
            rewrite_expr(receiver, roots, locals);
            for arg in args {
                rewrite_expr(arg, roots, locals);
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. } => rewrite_expr(receiver, roots, locals),
        Expr::Closure { params, body } => {
            let mut nested = locals.clone();
            nested.extend(params.iter().cloned());
            rewrite_expr(body, roots, &nested);
        }
        Expr::Binary { lhs, rhs, .. } => {
            rewrite_expr(lhs, roots, locals);
            rewrite_expr(rhs, roots, locals);
        }
        Expr::Index { base, index } => {
            rewrite_expr(base, roots, locals);
            match index {
                IndexArg::Single(e) | IndexArg::RangeTo(e) | IndexArg::RangeFrom(e) => {
                    rewrite_expr(e, roots, locals)
                }
                IndexArg::Range(lo, hi) => {
                    rewrite_expr(lo, roots, locals);
                    rewrite_expr(hi, roots, locals);
                }
            }
        }
        Expr::Tuple(items) => {
            for item in items {
                rewrite_expr(item, roots, locals);
            }
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                rewrite_expr(value, roots, locals);
            }
        }
        Expr::Is { scrutinee, .. } => rewrite_expr(scrutinee, roots, locals),
        Expr::Quantifier { domain, body, .. } => {
            rewrite_expr(domain, roots, locals);
            rewrite_expr(body, roots, locals);
        }
        Expr::Match { scrutinee, arms } => {
            rewrite_expr(scrutinee, roots, locals);
            for arm in arms {
                let mut nested = locals.clone();
                collect_pattern_bindings(&arm.pattern, &mut nested);
                if let Some(guard) = &mut arm.guard {
                    rewrite_expr(guard, roots, &nested);
                }
                rewrite_expr(&mut arm.body, roots, &nested);
            }
        }
        Expr::If { cond, then, else_ } => {
            rewrite_expr(cond, roots, locals);
            let mut nested = locals.clone();
            rewrite_block(then, roots, &mut nested);
            let mut nested = locals.clone();
            rewrite_block(else_, roots, &mut nested);
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

fn symbol_suffix(lock: &str) -> String {
    lock.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub fn program_uses_holding(program: &Program) -> bool {
    program.items.iter().any(|item| match item {
        Item::Fn(function) => function.body.as_ref().is_some_and(block_uses_holding),
        _ => false,
    })
}

fn block_uses_holding(block: &Block) -> bool {
    enum Node<'a> {
        Block(&'a Block),
        Expr(&'a Expr),
    }
    let mut pending = vec![Node::Block(block)];
    while let Some(node) = pending.pop() {
        match node {
            Node::Block(block) => {
                if let Some(tail) = block.tail.as_deref() {
                    pending.push(Node::Expr(tail));
                }
                for stmt in &block.stmts {
                    match stmt {
                        Stmt::Holding { .. } => return true,
                        Stmt::If { cond, then, else_ } => {
                            pending.push(Node::Expr(cond));
                            pending.push(Node::Block(then));
                            if let Some(other) = else_ {
                                pending.push(Node::Block(other));
                            }
                        }
                        Stmt::Loop(loop_) => {
                            if let LoopKind::While(cond) = &loop_.kind {
                                pending.push(Node::Expr(cond));
                            }
                            pending.push(Node::Block(&loop_.body));
                        }
                        Stmt::Let { init, .. } => pending.push(Node::Expr(init)),
                        Stmt::Assign { target, value } => {
                            pending.push(Node::Expr(target));
                            pending.push(Node::Expr(value));
                        }
                        Stmt::Return(Some(expr)) | Stmt::Expr(expr) => {
                            pending.push(Node::Expr(expr))
                        }
                        Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
                    }
                }
            }
            Node::Expr(expr) => match expr {
                Expr::If { cond, then, else_ } => {
                    pending.push(Node::Expr(cond));
                    pending.push(Node::Block(then));
                    pending.push(Node::Block(else_));
                }
                Expr::Call { callee, args } => {
                    pending.push(Node::Expr(callee));
                    pending.extend(args.iter().map(Node::Expr));
                }
                Expr::MethodCall { receiver, args, .. } => {
                    pending.push(Node::Expr(receiver));
                    pending.extend(args.iter().map(Node::Expr));
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
                } => pending.push(Node::Expr(receiver)),
                Expr::Binary { lhs, rhs, .. } => {
                    pending.push(Node::Expr(lhs));
                    pending.push(Node::Expr(rhs));
                }
                Expr::Index { base, index } => {
                    pending.push(Node::Expr(base));
                    match index {
                        IndexArg::Single(e) | IndexArg::RangeTo(e) | IndexArg::RangeFrom(e) => {
                            pending.push(Node::Expr(e))
                        }
                        IndexArg::Range(lo, hi) => {
                            pending.push(Node::Expr(lo));
                            pending.push(Node::Expr(hi));
                        }
                    }
                }
                Expr::Tuple(items) => pending.extend(items.iter().map(Node::Expr)),
                Expr::StructLit { fields, .. } => {
                    pending.extend(fields.iter().map(|(_, value)| Node::Expr(value)))
                }
                Expr::Quantifier { domain, body, .. } => {
                    pending.push(Node::Expr(domain));
                    pending.push(Node::Expr(body));
                }
                Expr::Match { scrutinee, arms } => {
                    pending.push(Node::Expr(scrutinee));
                    for arm in arms {
                        if let Some(guard) = &arm.guard {
                            pending.push(Node::Expr(guard));
                        }
                        pending.push(Node::Expr(&arm.body));
                    }
                }
                Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => {}
            },
        }
    }
    false
}
