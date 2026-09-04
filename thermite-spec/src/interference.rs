//! RFC-12 relational-contract validation for the closed monotone v1 fragment.

use std::collections::{BTreeMap, BTreeSet};

use thermite_syntax::{
    BinOp, Clause, Effect, EffectRow, Expr, FnItem, Item, Program, RegionPath, Span, Type, UnaryOp,
};

use crate::RegionIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MonotoneKind {
    Ordered,
    BitSet,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonotoneAtom {
    pub place: String,
    pub kind: MonotoneKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedRelation {
    pub atoms: BTreeSet<MonotoneAtom>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedInterference {
    pub function: String,
    pub asks: CheckedRelation,
    pub promises: CheckedRelation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterferenceReport {
    pub functions: BTreeMap<String, CheckedInterference>,
    pub obligations: Vec<CompositionObligation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionObligation {
    pub composition: String,
    pub guarantor: String,
    pub relying: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterferenceErrorKind {
    UnsupportedRelation,
    UnstablePostcondition,
    DuplicateParticipant,
    MissingContract,
    IncompatiblePeer,
    InvalidHandlerPriority,
    UnresolvedStateIdentity,
    IncompleteConflictCoverage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterferenceError {
    pub kind: InterferenceErrorKind,
    pub function: Option<String>,
    pub detail: String,
    pub span: Span,
}

/// One unordered RFC-9 conflict that may be discharged by RFC-12. Multiple
/// conflicting effect pairs for the same roots are deliberately retained here:
/// every overlapping shared-place identity must be covered by the relations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InterferenceRequirement {
    pub composition: String,
    pub left_root: String,
    pub right_root: String,
    pub overlap: Option<RegionPath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Initial,
    Final,
}

pub fn check_interference(program: &Program) -> Result<InterferenceReport, Vec<InterferenceError>> {
    check_interference_inner(program, None, None)
}

/// Check all RFC-12 clauses while generating composition obligations only for
/// the supplied RFC-9 conflicts. This is the production seam: disjoint and
/// already-commuting compositions do not acquire a new mandatory contract.
pub fn check_interference_for_conflicts(
    program: &Program,
    requirements: &[InterferenceRequirement],
    regions: &RegionIndex,
) -> Result<InterferenceReport, Vec<InterferenceError>> {
    check_interference_inner(program, Some(requirements), Some(regions))
}

fn check_interference_inner(
    program: &Program,
    requirements: Option<&[InterferenceRequirement]>,
    regions: Option<&RegionIndex>,
) -> Result<InterferenceReport, Vec<InterferenceError>> {
    let mut functions = BTreeMap::new();
    let mut errors = Vec::new();

    for item in &program.items {
        let Item::Fn(function) = item else {
            continue;
        };
        let Some(contract) = &function.contract.interference else {
            continue;
        };
        let asks_raw = classify_relation(&function.name, "asks", &contract.asks, &mut errors);
        if let Some(asks) = &asks_raw {
            for (index, ensures) in function.contract.ensures.iter().enumerate() {
                if !postcondition_is_stable(&ensures.expr, asks) {
                    errors.push(InterferenceError {
                        kind: InterferenceErrorKind::UnstablePostcondition,
                        function: Some(function.name.clone()),
                        detail: format!(
                            "ensures#{index} is not upward-stable under `{}`'s asks relation",
                            function.name
                        ),
                        span: ensures.span,
                    });
                }
            }
        }
        let asks = asks_raw.and_then(|relation| {
            canonicalize_relation(function, relation, contract.asks.span, &mut errors)
        });
        let promises =
            classify_relation(&function.name, "promises", &contract.promises, &mut errors)
                .and_then(|relation| {
                    canonicalize_relation(function, relation, contract.promises.span, &mut errors)
                });
        if let (Some(asks), Some(promises)) = (asks, promises) {
            functions.insert(
                function.name.clone(),
                CheckedInterference {
                    function: function.name.clone(),
                    asks,
                    promises,
                },
            );
        }
    }

    let mut obligations = Vec::new();
    for item in &program.items {
        let Item::Concurrent(composition) = item else {
            continue;
        };
        let mut seen = BTreeSet::new();
        for root in &composition.roots {
            if !seen.insert(root) {
                errors.push(InterferenceError {
                    kind: InterferenceErrorKind::DuplicateParticipant,
                    function: Some(root.clone()),
                    detail: format!(
                        "composition `{}` repeats participant `{root}`",
                        composition.name
                    ),
                    span: composition.span,
                });
            }
        }

        if let Some(priorities) = &composition.handler_priorities {
            if priorities.len() != composition.roots.len()
                || priorities.contains(&0)
                || priorities.iter().copied().collect::<BTreeSet<_>>().len() != priorities.len()
            {
                errors.push(InterferenceError {
                    kind: InterferenceErrorKind::InvalidHandlerPriority,
                    function: None,
                    detail: "handler priorities must be non-zero, unique, and paired one-for-one with handlers"
                        .to_string(),
                    span: composition.span,
                });
                continue;
            }
            for high in 0..composition.roots.len() {
                for low in 0..composition.roots.len() {
                    if priorities[high] > priorities[low]
                        && pair_is_required(
                            requirements,
                            &composition.name,
                            &composition.roots[high],
                            &composition.roots[low],
                        )
                    {
                        add_obligation(
                            composition,
                            &composition.roots[high],
                            &composition.roots[low],
                            &functions,
                            &mut obligations,
                            &mut errors,
                            required_overlaps(
                                requirements,
                                &composition.name,
                                &composition.roots[high],
                                &composition.roots[low],
                            ),
                            regions,
                        );
                    }
                }
            }
        } else {
            for left in 0..composition.roots.len() {
                for right in left + 1..composition.roots.len() {
                    if !pair_is_required(
                        requirements,
                        &composition.name,
                        &composition.roots[left],
                        &composition.roots[right],
                    ) {
                        continue;
                    }
                    add_obligation(
                        composition,
                        &composition.roots[left],
                        &composition.roots[right],
                        &functions,
                        &mut obligations,
                        &mut errors,
                        required_overlaps(
                            requirements,
                            &composition.name,
                            &composition.roots[left],
                            &composition.roots[right],
                        ),
                        regions,
                    );
                    add_obligation(
                        composition,
                        &composition.roots[right],
                        &composition.roots[left],
                        &functions,
                        &mut obligations,
                        &mut errors,
                        required_overlaps(
                            requirements,
                            &composition.name,
                            &composition.roots[left],
                            &composition.roots[right],
                        ),
                        regions,
                    );
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(InterferenceReport {
            functions,
            obligations,
        })
    } else {
        Err(errors)
    }
}

fn pair_is_required(
    requirements: Option<&[InterferenceRequirement]>,
    composition: &str,
    left: &str,
    right: &str,
) -> bool {
    requirements.is_none_or(|requirements| {
        requirements.iter().any(|required| {
            required.composition == composition
                && ((required.left_root == left && required.right_root == right)
                    || (required.left_root == right && required.right_root == left))
        })
    })
}

fn required_overlaps<'a>(
    requirements: Option<&'a [InterferenceRequirement]>,
    composition: &str,
    left: &str,
    right: &str,
) -> Vec<&'a RegionPath> {
    requirements
        .into_iter()
        .flatten()
        .filter(|required| {
            required.composition == composition
                && ((required.left_root == left && required.right_root == right)
                    || (required.left_root == right && required.right_root == left))
        })
        .filter_map(|required| required.overlap.as_ref())
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn add_obligation(
    composition: &thermite_syntax::ConcurrentItem,
    guarantor: &str,
    relying: &str,
    functions: &BTreeMap<String, CheckedInterference>,
    obligations: &mut Vec<CompositionObligation>,
    errors: &mut Vec<InterferenceError>,
    overlaps: Vec<&RegionPath>,
    regions: Option<&RegionIndex>,
) {
    let (Some(guarantee), Some(rely)) = (functions.get(guarantor), functions.get(relying)) else {
        let missing = if !functions.contains_key(guarantor) {
            guarantor
        } else {
            relying
        };
        errors.push(InterferenceError {
            kind: InterferenceErrorKind::MissingContract,
            function: Some(missing.to_string()),
            detail: format!(
                "concurrent `{}` roots `{guarantor}` and `{relying}` have an overlapping conflict at {:?} and need an interference contract for `{missing}`",
                composition.name, overlaps
            ),
            span: composition.span,
        });
        return;
    };
    if !guarantee.promises.atoms.is_superset(&rely.asks.atoms) {
        let missing = rely
            .asks
            .atoms
            .difference(&guarantee.promises.atoms)
            .map(|atom| format!("{:?}({})", atom.kind, atom.place))
            .collect::<Vec<_>>();
        errors.push(InterferenceError {
            kind: InterferenceErrorKind::IncompatiblePeer,
            function: Some(relying.to_string()),
            detail: format!(
                "in `{}`, promises({guarantor}) does not imply asks({relying}); missing {missing:?}",
                composition.name
            ),
            span: composition.span,
        });
        return;
    }
    if let Some(regions) = regions {
        for overlap in overlaps {
            let guarantee_covers = relation_covers(&guarantee.promises, overlap, regions);
            let rely_covers = relation_covers(&rely.asks, overlap, regions);
            if !guarantee_covers || !rely_covers {
                errors.push(InterferenceError {
                    kind: InterferenceErrorKind::IncompleteConflictCoverage,
                    function: Some(relying.to_string()),
                    detail: format!(
                        "in `{}`, promises({guarantor}) and asks({relying}) do not both cover conflicting shared place `{overlap}`",
                        composition.name
                    ),
                    span: composition.span,
                });
                return;
            }
        }
    }
    obligations.push(CompositionObligation {
        composition: composition.name.clone(),
        guarantor: guarantor.to_string(),
        relying: relying.to_string(),
    });
}

fn relation_covers(
    relation: &CheckedRelation,
    overlap: &RegionPath,
    regions: &RegionIndex,
) -> bool {
    relation
        .atoms
        .iter()
        .any(|atom| regions.overlaps(&RegionPath::from(atom.place.as_str()), overlap))
}

fn classify_relation(
    function: &str,
    clause_name: &str,
    clause: &Clause,
    errors: &mut Vec<InterferenceError>,
) -> Option<CheckedRelation> {
    let mut expressions = Vec::new();
    flatten_conjunction(&clause.expr, &mut expressions);
    let mut atoms = BTreeSet::new();
    for expr in expressions {
        if matches!(expr, Expr::BoolLit(true)) {
            continue;
        }
        match classify_atom(expr) {
            Some(atom) => {
                atoms.insert(atom);
            }
            None => errors.push(InterferenceError {
                kind: InterferenceErrorKind::UnsupportedRelation,
                function: Some(function.to_string()),
                detail: format!(
                    "`{clause_name}` must be a conjunction of persistent ordered, bit-set, or boolean growth envelopes; exact steps and equality-only epochs belong elsewhere"
                ),
                span: clause.span,
            }),
        }
    }
    if errors.iter().any(|error| {
        error.function.as_deref() == Some(function)
            && error.kind == InterferenceErrorKind::UnsupportedRelation
            && error.span == clause.span
    }) {
        None
    } else {
        Some(CheckedRelation { atoms })
    }
}

fn flatten_conjunction<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
    if let Expr::Binary {
        op: BinOp::And,
        lhs,
        rhs,
    } = expr
    {
        flatten_conjunction(lhs, out);
        flatten_conjunction(rhs, out);
    } else {
        out.push(expr);
    }
}

fn canonicalize_relation(
    function: &FnItem,
    relation: CheckedRelation,
    span: Span,
    errors: &mut Vec<InterferenceError>,
) -> Option<CheckedRelation> {
    let roots: BTreeSet<String> = match &function.contract.effects {
        EffectRow::Pure => BTreeSet::new(),
        EffectRow::Set(effects) => effects
            .iter()
            .filter_map(|effect| match effect {
                Effect::Read(path) | Effect::Write(path) => path.segments.first().cloned(),
                _ => None,
            })
            .collect(),
    };
    if roots.is_empty() {
        errors.push(InterferenceError {
            kind: InterferenceErrorKind::UnresolvedStateIdentity,
            function: Some(function.name.clone()),
            detail: format!(
                "`{}` has interference clauses but no shared read/write effect identifying their state",
                function.name
            ),
            span,
        });
        return None;
    }
    let borrowed: BTreeSet<&str> = function
        .params
        .iter()
        .filter_map(|param| matches!(param.ty, Type::Ref { .. }).then_some(param.name.as_str()))
        .collect();
    let expected = relation.atoms.len();
    let mut canonical = BTreeSet::new();
    for atom in relation.atoms {
        let mut segments = atom.place.split('.');
        let head = segments.next().unwrap_or_default();
        let tail = segments.collect::<Vec<_>>();
        let root = if roots.contains(head) {
            head.to_string()
        } else if borrowed.contains(head) && roots.len() == 1 {
            roots.iter().next().cloned().unwrap_or_default()
        } else {
            errors.push(InterferenceError {
                kind: InterferenceErrorKind::UnresolvedStateIdentity,
                function: Some(function.name.clone()),
                detail: format!(
                    "relation place `{}` in `{}` cannot be mapped uniquely to shared effects {:?}",
                    atom.place, function.name, roots
                ),
                span,
            });
            continue;
        };
        let place = if tail.is_empty() {
            root
        } else {
            format!("{root}.{}", tail.join("."))
        };
        canonical.insert(MonotoneAtom {
            place,
            kind: atom.kind,
        });
    }
    (canonical.len() == expected).then_some(CheckedRelation { atoms: canonical })
}

fn classify_atom(expr: &Expr) -> Option<MonotoneAtom> {
    match expr {
        Expr::Binary {
            op: BinOp::Ge,
            lhs,
            rhs,
        }
        | Expr::Binary {
            op: BinOp::Le,
            lhs: rhs,
            rhs: lhs,
        } => matching_phase_place(lhs, rhs).map(|place| MonotoneAtom {
            place,
            kind: MonotoneKind::Ordered,
        }),
        Expr::Binary {
            op: BinOp::Eq,
            lhs,
            rhs,
        } => classify_bitset_growth(lhs, rhs).or_else(|| classify_bitset_growth(rhs, lhs)),
        Expr::Binary {
            op: BinOp::Or,
            lhs,
            rhs,
        } => classify_boolean_growth(lhs, rhs).or_else(|| classify_boolean_growth(rhs, lhs)),
        _ => None,
    }
}

fn matching_phase_place(final_expr: &Expr, initial_expr: &Expr) -> Option<String> {
    let (Phase::Final, final_place) = state_place(final_expr)? else {
        return None;
    };
    let (Phase::Initial, initial_place) = state_place(initial_expr)? else {
        return None;
    };
    (final_place == initial_place).then_some(final_place)
}

fn classify_bitset_growth(or_expr: &Expr, final_expr: &Expr) -> Option<MonotoneAtom> {
    let Expr::Binary {
        op: BinOp::BitOr,
        lhs,
        rhs,
    } = or_expr
    else {
        return None;
    };
    let place = matching_phase_place(lhs, rhs).or_else(|| matching_phase_place(rhs, lhs))?;
    let (Phase::Final, compared) = state_place(final_expr)? else {
        return None;
    };
    (place == compared).then_some(MonotoneAtom {
        place,
        kind: MonotoneKind::BitSet,
    })
}

fn classify_boolean_growth(not_initial: &Expr, final_expr: &Expr) -> Option<MonotoneAtom> {
    let Expr::Unary {
        op: UnaryOp::Not,
        expr: initial,
    } = not_initial
    else {
        return None;
    };
    matching_phase_place(final_expr, initial).map(|place| MonotoneAtom {
        place,
        kind: MonotoneKind::Boolean,
    })
}

fn state_place(expr: &Expr) -> Option<(Phase, String)> {
    match expr {
        Expr::Path(path) if !path.is_empty() => Some((Phase::Initial, path.join("."))),
        Expr::Call { callee, args }
            if args.len() == 1
                && matches!(callee.as_ref(), Expr::Path(path) if path.as_slice() == ["final"]) =>
        {
            let (_, place) = state_place(&args[0])?;
            Some((Phase::Final, place))
        }
        Expr::Field { receiver, name } => {
            let (phase, mut place) = state_place(receiver)?;
            place.push('.');
            place.push_str(name);
            Some((phase, place))
        }
        _ => None,
    }
}

fn postcondition_is_stable(expr: &Expr, asks: &CheckedRelation) -> bool {
    if !contains_final(expr) {
        return true;
    }
    stable_atom(expr, asks)
}

fn stable_atom(expr: &Expr, asks: &CheckedRelation) -> bool {
    match expr {
        Expr::BoolLit(true) => true,
        Expr::Binary {
            op: BinOp::And | BinOp::Or,
            lhs,
            rhs,
        } => stable_atom(lhs, asks) && stable_atom(rhs, asks),
        Expr::Binary {
            op: BinOp::Ge,
            lhs,
            rhs,
        }
        | Expr::Binary {
            op: BinOp::Le,
            lhs: rhs,
            rhs: lhs,
        } => upward_bound_place(lhs, rhs, asks),
        Expr::Binary {
            op: BinOp::Eq,
            lhs,
            rhs,
        } => bit_membership_place(lhs, rhs, asks)
            .or_else(|| bit_membership_place(rhs, lhs, asks))
            .is_some(),
        _ => false,
    }
}

fn upward_bound_place(lhs: &Expr, rhs: &Expr, asks: &CheckedRelation) -> bool {
    let Some((Phase::Final, place)) = state_place(lhs) else {
        return false;
    };
    !contains_final(rhs)
        && asks.atoms.contains(&MonotoneAtom {
            place,
            kind: MonotoneKind::Ordered,
        })
}

fn bit_membership_place(bit_expr: &Expr, one: &Expr, asks: &CheckedRelation) -> Option<()> {
    if !matches!(one, Expr::IntLit { value: 1, .. }) {
        return None;
    }
    let Expr::Binary {
        op: BinOp::BitAnd,
        lhs,
        rhs,
    } = bit_expr
    else {
        return None;
    };
    if !matches!(rhs.as_ref(), Expr::IntLit { value: 1, .. }) {
        return None;
    }
    let Expr::Binary {
        op: BinOp::Shr,
        lhs: value,
        ..
    } = lhs.as_ref()
    else {
        return None;
    };
    let (Phase::Final, place) = state_place(value)? else {
        return None;
    };
    asks.atoms
        .contains(&MonotoneAtom {
            place,
            kind: MonotoneKind::BitSet,
        })
        .then_some(())
}

fn contains_final(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, args } => {
            matches!(callee.as_ref(), Expr::Path(path) if path.as_slice() == ["final"])
                || contains_final(callee)
                || args.iter().any(contains_final)
        }
        Expr::MethodCall { receiver, args, .. } => {
            contains_final(receiver) || args.iter().any(contains_final)
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. } => contains_final(receiver),
        Expr::Binary { lhs, rhs, .. } => contains_final(lhs) || contains_final(rhs),
        Expr::Index { base, index } => {
            contains_final(base)
                || match index {
                    thermite_syntax::IndexArg::Single(value)
                    | thermite_syntax::IndexArg::RangeTo(value)
                    | thermite_syntax::IndexArg::RangeFrom(value) => contains_final(value),
                    thermite_syntax::IndexArg::Range(left, right) => {
                        contains_final(left) || contains_final(right)
                    }
                }
        }
        Expr::Closure { body, .. } => contains_final(body),
        Expr::Match { scrutinee, arms } => {
            contains_final(scrutinee)
                || arms.iter().any(|arm| {
                    arm.guard.as_ref().is_some_and(contains_final) || contains_final(&arm.body)
                })
        }
        Expr::If { cond, then, else_ } => {
            contains_final(cond) || block_contains_final(then) || block_contains_final(else_)
        }
        Expr::StructLit { fields, .. } => fields.iter().any(|(_, value)| contains_final(value)),
        Expr::Is { scrutinee, .. } => contains_final(scrutinee),
        Expr::Tuple(values) => values.iter().any(contains_final),
        Expr::Quantifier { domain, body, .. } => contains_final(domain) || contains_final(body),
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => false,
    }
}

fn block_contains_final(block: &thermite_syntax::Block) -> bool {
    block.tail.as_deref().is_some_and(contains_final)
        || block.stmts.iter().any(|statement| match statement {
            thermite_syntax::Stmt::Let { init, .. } => contains_final(init),
            thermite_syntax::Stmt::Assign { target, value } => {
                contains_final(target) || contains_final(value)
            }
            thermite_syntax::Stmt::Return(value) => value.as_ref().is_some_and(contains_final),
            thermite_syntax::Stmt::If { cond, then, else_ } => {
                contains_final(cond)
                    || block_contains_final(then)
                    || else_.as_ref().is_some_and(block_contains_final)
            }
            thermite_syntax::Stmt::Loop(loop_) => block_contains_final(&loop_.body),
            thermite_syntax::Stmt::Holding { body, .. } => block_contains_final(body),
            thermite_syntax::Stmt::Forget { value, .. } | thermite_syntax::Stmt::Expr(value) => {
                contains_final(value)
            }
            thermite_syntax::Stmt::Break | thermite_syntax::Stmt::Continue => false,
        })
}
