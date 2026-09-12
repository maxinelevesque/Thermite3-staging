//! Exact, source-only Tier-A relational-frame witness production.
//!
//! The producer derives every authority-bearing field from a checked program.
//! It cannot emit `end_to_end`: that upgrade belongs to the separate semantic
//! transport theorem and receipt path.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thermite_syntax::{
    BinOp, Block, Effect, EffectRow, Expr, Item, PrimType, Program, Stmt, Type, UnaryOp,
};

use crate::{CheckedProgram, WitnessError};

pub const RELATIONAL_FRAME_WITNESS_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchGate {
    ProbabilisticDenotation,
    PeerProgress,
    ExternalCoupling,
    TerminationWitness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status", content = "gate")]
pub enum Support {
    Derived,
    Conditional(ResearchGate),
    Unavailable(ResearchGate),
    Structural,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionSupport {
    pub result: Support,
    pub write_frame: Support,
    pub outcome: Support,
    pub termination: Support,
    pub trace: Support,
    pub accumulator: Support,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Projection {
    Result,
    WriteFrame,
    Outcome,
    Termination,
    Trace,
    Accumulator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationalScope {
    SourceOnly,
    EndToEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundedIntTy {
    U8,
    U16,
    U32,
    U64,
    Usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationalArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationalCompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationalLogicOp {
    And,
    Or,
}

/// Canonical executable expression admitted by the current bounded relational
/// semantics. Region paths remain source strings in the receipt and are
/// injectively encoded for Lean replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CanonicalRelationalExpr {
    Int {
        ty: BoundedIntTy,
        value: u64,
    },
    Bool {
        value: bool,
    },
    Local {
        name: String,
    },
    Region {
        path: String,
    },
    Arith {
        op: RelationalArithOp,
        left: Box<Self>,
        right: Box<Self>,
    },
    Compare {
        op: RelationalCompareOp,
        left: Box<Self>,
        right: Box<Self>,
    },
    Logic {
        op: RelationalLogicOp,
        left: Box<Self>,
        right: Box<Self>,
    },
    Not {
        value: Box<Self>,
    },
    Cast {
        value: Box<Self>,
        ty: BoundedIntTy,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CanonicalRelationalProgram {
    Return {
        value: CanonicalRelationalExpr,
    },
    Write {
        region: String,
        value: CanonicalRelationalExpr,
        next: Box<Self>,
    },
    Branch {
        condition: CanonicalRelationalExpr,
        then_program: Box<Self>,
        else_program: Box<Self>,
    },
    Raise,
    Diverge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectSupport {
    pub effect: String,
    pub support: ProjectionSupport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalFunctionWitness {
    pub function: String,
    pub normalized_row: Vec<String>,
    pub read_footprint: Vec<String>,
    pub write_footprint: Vec<String>,
    pub effect_support: Vec<EffectSupport>,
    pub body: Option<CanonicalRelationalProgram>,
    pub unsupported_reason: Option<String>,
    pub semantic_fragment: String,
    pub projections: Vec<Projection>,
    pub scope: RelationalScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalFrameWitness {
    pub version: u32,
    pub canonical_ast_sha256: String,
    pub functions: Vec<RelationalFunctionWitness>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalTransportReceipt {
    pub version: u32,
    pub function: String,
    pub canonical_ast_sha256: String,
    pub source_body: CanonicalRelationalProgram,
    pub lowered_artifact_sha256: String,
    pub theorem: String,
    pub scope: RelationalScope,
}

#[derive(Debug)]
pub enum RelationalTransportError {
    Witness(WitnessError),
    Lowering(String),
    Mismatch(&'static str),
}

impl From<WitnessError> for RelationalTransportError {
    fn from(value: WitnessError) -> Self {
        Self::Witness(value)
    }
}

impl RelationalFrameWitness {
    pub fn canonical_json(&self) -> Result<String, WitnessError> {
        serde_json::to_string(self).map_err(|error| WitnessError::Json(error.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, WitnessError> {
        serde_json::from_str(json).map_err(|error| WitnessError::Json(error.to_string()))
    }
}

const STATE: ProjectionSupport = ProjectionSupport {
    result: Support::Derived,
    write_frame: Support::Derived,
    outcome: Support::Derived,
    termination: Support::Derived,
    trace: Support::NotApplicable,
    accumulator: Support::NotApplicable,
};

const IO: ProjectionSupport = ProjectionSupport {
    result: Support::Conditional(ResearchGate::ExternalCoupling),
    write_frame: Support::Derived,
    outcome: Support::Conditional(ResearchGate::ExternalCoupling),
    termination: Support::Derived,
    trace: Support::Conditional(ResearchGate::ExternalCoupling),
    accumulator: Support::NotApplicable,
};

const EXCEPTION: ProjectionSupport = ProjectionSupport {
    result: Support::NotApplicable,
    write_frame: Support::Derived,
    outcome: Support::Derived,
    termination: Support::NotApplicable,
    trace: Support::NotApplicable,
    accumulator: Support::NotApplicable,
};

const PARTIALITY: ProjectionSupport = ProjectionSupport {
    result: Support::Derived,
    write_frame: Support::Derived,
    outcome: Support::Derived,
    termination: Support::Unavailable(ResearchGate::TerminationWitness),
    trace: Support::NotApplicable,
    accumulator: Support::NotApplicable,
};

const STRUCTURAL: ProjectionSupport = ProjectionSupport {
    result: Support::Structural,
    write_frame: Support::Structural,
    outcome: Support::Structural,
    termination: Support::Structural,
    trace: Support::Structural,
    accumulator: Support::Structural,
};

fn combine_support(left: Support, right: Support) -> Support {
    match (left, right) {
        (Support::Unavailable(gate), _) | (_, Support::Unavailable(gate)) => {
            Support::Unavailable(gate)
        }
        (Support::Conditional(gate), _) | (_, Support::Conditional(gate)) => {
            Support::Conditional(gate)
        }
        (Support::NotApplicable, support) | (support, Support::NotApplicable) => support,
        (Support::Structural, Support::Structural) => Support::Structural,
        (Support::Structural, Support::Derived)
        | (Support::Derived, Support::Structural)
        | (Support::Derived, Support::Derived) => Support::Derived,
    }
}

fn combine(left: ProjectionSupport, right: ProjectionSupport) -> ProjectionSupport {
    ProjectionSupport {
        result: combine_support(left.result, right.result),
        write_frame: combine_support(left.write_frame, right.write_frame),
        outcome: combine_support(left.outcome, right.outcome),
        termination: combine_support(left.termination, right.termination),
        trace: combine_support(left.trace, right.trace),
        accumulator: combine_support(left.accumulator, right.accumulator),
    }
}

fn effect_support(effect: &Effect) -> ProjectionSupport {
    match effect {
        Effect::Read(_)
        | Effect::Write(_)
        | Effect::Alloc
        | Effect::Time
        | Effect::Rand
        | Effect::Term => STATE,
        Effect::Net(_) => combine(STATE, IO),
        Effect::Panic => EXCEPTION,
        Effect::Diverge => PARTIALITY,
        Effect::Owns(_) | Effect::Forgets(_) => STRUCTURAL,
        Effect::Blocks => ProjectionSupport {
            result: Support::Unavailable(ResearchGate::PeerProgress),
            write_frame: Support::Unavailable(ResearchGate::PeerProgress),
            outcome: Support::Unavailable(ResearchGate::PeerProgress),
            termination: Support::Unavailable(ResearchGate::PeerProgress),
            trace: Support::Unavailable(ResearchGate::PeerProgress),
            accumulator: Support::NotApplicable,
        },
    }
}

fn effect_name(effect: &Effect) -> String {
    match effect {
        Effect::Read(region) => format!("read({region})"),
        Effect::Write(region) => format!("write({region})"),
        Effect::Net(region) => format!("net({region})"),
        Effect::Forgets(region) => format!("forgets({region})"),
        Effect::Owns(lock) => format!("owns({lock})"),
        Effect::Alloc => "alloc".into(),
        Effect::Time => "time".into(),
        Effect::Rand => "rand".into(),
        Effect::Blocks => "blocks".into(),
        Effect::Panic => "panic".into(),
        Effect::Diverge => "diverge".into(),
        Effect::Term => "term".into(),
    }
}

fn row_effects(row: &EffectRow) -> Vec<Effect> {
    let mut effects = match row {
        EffectRow::Pure => Vec::new(),
        EffectRow::Set(effects) => effects.clone(),
    };
    effects.sort();
    effects.dedup();
    effects
}

fn support_at(support: ProjectionSupport, projection: Projection) -> Support {
    match projection {
        Projection::Result => support.result,
        Projection::WriteFrame => support.write_frame,
        Projection::Outcome => support.outcome,
        Projection::Termination => support.termination,
        Projection::Trace => support.trace,
        Projection::Accumulator => support.accumulator,
    }
}

fn claimable(support: Support) -> bool {
    matches!(support, Support::Derived | Support::Conditional(_))
}

fn projections(effects: &[Effect]) -> Vec<Projection> {
    let candidates = [
        Projection::Result,
        Projection::WriteFrame,
        Projection::Outcome,
        Projection::Termination,
        Projection::Trace,
        Projection::Accumulator,
    ];
    if effects.is_empty() {
        return candidates[..4].to_vec();
    }
    candidates
        .into_iter()
        .filter(|projection| {
            let support = effects
                .iter()
                .map(effect_support)
                .map(|support| support_at(support, *projection))
                .collect::<Vec<_>>();
            support.iter().any(|value| *value != Support::NotApplicable)
                && support.into_iter().all(claimable)
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScalarTy {
    Int(BoundedIntTy),
    Bool,
}

fn scalar_ty(ty: &Type) -> Option<ScalarTy> {
    match ty {
        Type::Prim(PrimType::U8) => Some(ScalarTy::Int(BoundedIntTy::U8)),
        Type::Prim(PrimType::U16) => Some(ScalarTy::Int(BoundedIntTy::U16)),
        Type::Prim(PrimType::U32) => Some(ScalarTy::Int(BoundedIntTy::U32)),
        Type::Prim(PrimType::U64) => Some(ScalarTy::Int(BoundedIntTy::U64)),
        Type::Prim(PrimType::Usize) => Some(ScalarTy::Int(BoundedIntTy::Usize)),
        Type::Prim(PrimType::Bool) => Some(ScalarTy::Bool),
        _ => None,
    }
}

struct TranslationEnv<'a> {
    locals: BTreeMap<&'a str, ScalarTy>,
    shared: BTreeMap<&'a str, &'a Type>,
    structs: BTreeMap<&'a str, BTreeMap<&'a str, &'a Type>>,
}

impl<'a> TranslationEnv<'a> {
    fn new(program: &'a Program, function: &'a thermite_syntax::FnItem) -> Self {
        let locals = function
            .params
            .iter()
            .filter_map(|param| scalar_ty(&param.ty).map(|ty| (param.name.as_str(), ty)))
            .collect();
        let shared = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::SharedDecl(decl) => Some((decl.name.as_str(), &decl.ty)),
                _ => None,
            })
            .collect();
        let structs = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Struct(definition) => Some((
                    definition.name.as_str(),
                    definition
                        .fields
                        .iter()
                        .map(|field| (field.name.as_str(), &field.ty))
                        .collect(),
                )),
                _ => None,
            })
            .collect();
        Self {
            locals,
            shared,
            structs,
        }
    }

    fn region_ty(&self, segments: &[String]) -> Option<ScalarTy> {
        let mut ty = *self.shared.get(segments.first()?.as_str())?;
        for segment in &segments[1..] {
            let Type::Named(name) = ty else { return None };
            ty = *self.structs.get(name.as_str())?.get(segment.as_str())?;
        }
        scalar_ty(ty)
    }
}

fn field_path(expr: &Expr) -> Option<Vec<String>> {
    match expr {
        Expr::Path(path) => Some(path.clone()),
        Expr::Field { receiver, name } => {
            let mut path = field_path(receiver)?;
            path.push(name.clone());
            Some(path)
        }
        _ => None,
    }
}

fn infer_expr_ty(expr: &Expr, env: &TranslationEnv<'_>) -> Option<ScalarTy> {
    match expr {
        Expr::BoolLit(_) => Some(ScalarTy::Bool),
        Expr::Path(path) if path.len() == 1 => env.locals.get(path[0].as_str()).copied(),
        Expr::Field { .. } => env.region_ty(&field_path(expr)?),
        Expr::Binary { op, lhs, rhs } => match op {
            BinOp::Eq
            | BinOp::Ne
            | BinOp::Lt
            | BinOp::Le
            | BinOp::Gt
            | BinOp::Ge
            | BinOp::And
            | BinOp::Or => Some(ScalarTy::Bool),
            _ => infer_expr_ty(lhs, env).or_else(|| infer_expr_ty(rhs, env)),
        },
        Expr::Unary { .. } => Some(ScalarTy::Bool),
        Expr::Cast { ty, .. } => scalar_ty(ty),
        Expr::IntLit { .. }
        | Expr::Call { .. }
        | Expr::MethodCall { .. }
        | Expr::Closure { .. }
        | Expr::Match { .. }
        | Expr::If { .. }
        | Expr::Index { .. }
        | Expr::Ref { .. }
        | Expr::StructLit { .. }
        | Expr::Is { .. }
        | Expr::Deref(_)
        | Expr::StrLit(_)
        | Expr::Tuple(_)
        | Expr::TupleProj { .. }
        | Expr::Quantifier { .. }
        | Expr::Path(_) => None,
    }
}

fn translate_expr(
    expr: &Expr,
    expected: Option<ScalarTy>,
    env: &TranslationEnv<'_>,
) -> Result<CanonicalRelationalExpr, String> {
    match expr {
        Expr::IntLit { value, .. } => {
            let ScalarTy::Int(ty) = expected.ok_or("integer literal has no scalar context")? else {
                return Err("integer literal appears in a boolean context".into());
            };
            let value = u64::try_from(*value)
                .map_err(|_| "integer literal exceeds the bounded u64 carrier")?;
            Ok(CanonicalRelationalExpr::Int { ty, value })
        }
        Expr::BoolLit(value) => {
            if matches!(expected, Some(ScalarTy::Int(_))) {
                return Err("boolean literal appears in an integer context".into());
            }
            Ok(CanonicalRelationalExpr::Bool { value: *value })
        }
        Expr::Path(path) if path.len() == 1 => {
            let actual = env
                .locals
                .get(path[0].as_str())
                .copied()
                .ok_or_else(|| format!("non-scalar or unbound local `{}`", path[0]))?;
            if expected.is_some_and(|expected| expected != actual) {
                return Err(format!("local `{}` has a different scalar type", path[0]));
            }
            Ok(CanonicalRelationalExpr::Local {
                name: path[0].clone(),
            })
        }
        Expr::Field { .. } => {
            let path = field_path(expr).ok_or("field expression is not a canonical path")?;
            let actual = env.region_ty(&path).ok_or_else(|| {
                format!("shared path `{}` is not a scalar region", path.join("."))
            })?;
            if expected.is_some_and(|expected| expected != actual) {
                return Err(format!(
                    "shared path `{}` has a different scalar type",
                    path.join(".")
                ));
            }
            Ok(CanonicalRelationalExpr::Region {
                path: path.join("."),
            })
        }
        Expr::Binary { op, lhs, rhs } => {
            let operand_ty = infer_expr_ty(lhs, env)
                .or_else(|| infer_expr_ty(rhs, env))
                .or(expected)
                .ok_or("binary expression has no scalar type context")?;
            let left = Box::new(translate_expr(lhs, Some(operand_ty), env)?);
            let right = Box::new(translate_expr(rhs, Some(operand_ty), env)?);
            match op {
                BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::Rem
                | BinOp::Shl
                | BinOp::Shr
                | BinOp::BitAnd
                | BinOp::BitOr
                | BinOp::BitXor => {
                    if !matches!(operand_ty, ScalarTy::Int(_)) {
                        return Err("arithmetic operator has non-integer operands".into());
                    }
                    let op = match op {
                        BinOp::Add => RelationalArithOp::Add,
                        BinOp::Sub => RelationalArithOp::Sub,
                        BinOp::Mul => RelationalArithOp::Mul,
                        BinOp::Div => RelationalArithOp::Div,
                        BinOp::Rem => RelationalArithOp::Rem,
                        BinOp::Shl => RelationalArithOp::Shl,
                        BinOp::Shr => RelationalArithOp::Shr,
                        BinOp::BitAnd => RelationalArithOp::BitAnd,
                        BinOp::BitOr => RelationalArithOp::BitOr,
                        BinOp::BitXor => RelationalArithOp::BitXor,
                        _ => unreachable!(),
                    };
                    Ok(CanonicalRelationalExpr::Arith { op, left, right })
                }
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    let op = match op {
                        BinOp::Eq => RelationalCompareOp::Eq,
                        BinOp::Ne => RelationalCompareOp::Ne,
                        BinOp::Lt => RelationalCompareOp::Lt,
                        BinOp::Le => RelationalCompareOp::Le,
                        BinOp::Gt => RelationalCompareOp::Gt,
                        BinOp::Ge => RelationalCompareOp::Ge,
                        _ => unreachable!(),
                    };
                    Ok(CanonicalRelationalExpr::Compare { op, left, right })
                }
                BinOp::And | BinOp::Or => {
                    if operand_ty != ScalarTy::Bool {
                        return Err("logical operator has non-boolean operands".into());
                    }
                    Ok(CanonicalRelationalExpr::Logic {
                        op: if *op == BinOp::And {
                            RelationalLogicOp::And
                        } else {
                            RelationalLogicOp::Or
                        },
                        left,
                        right,
                    })
                }
            }
        }
        Expr::Unary {
            op: UnaryOp::Not,
            expr,
        } => Ok(CanonicalRelationalExpr::Not {
            value: Box::new(translate_expr(expr, Some(ScalarTy::Bool), env)?),
        }),
        Expr::Cast { expr, ty } => {
            let ScalarTy::Int(ty) = scalar_ty(ty).ok_or("cast target is not a bounded integer")?
            else {
                return Err("cast target is not a bounded integer".into());
            };
            Ok(CanonicalRelationalExpr::Cast {
                value: Box::new(translate_expr(expr, infer_expr_ty(expr, env), env)?),
                ty,
            })
        }
        _ => Err("expression is outside the bounded relational fragment".into()),
    }
}

fn translate_block(
    block: &Block,
    continuation: Option<CanonicalRelationalProgram>,
    return_ty: ScalarTy,
    env: &TranslationEnv<'_>,
) -> Result<CanonicalRelationalProgram, String> {
    let mut current = match (&block.tail, continuation) {
        (Some(tail), None) => CanonicalRelationalProgram::Return {
            value: translate_expr(tail, Some(return_ty), env)?,
        },
        (None, Some(continuation)) => continuation,
        (Some(_), Some(_)) => return Err("statement block has a discarded tail value".into()),
        (None, None) => return Err("function has no relational return value".into()),
    };
    for statement in block.stmts.iter().rev() {
        current = match statement {
            Stmt::Assign { target, value } => {
                let path = field_path(target)
                    .filter(|path| env.region_ty(path).is_some())
                    .ok_or("assignment target is not a scalar shared region")?;
                let ty = env.region_ty(&path).expect("checked above");
                CanonicalRelationalProgram::Write {
                    region: path.join("."),
                    value: translate_expr(value, Some(ty), env)?,
                    next: Box::new(current),
                }
            }
            Stmt::Return(Some(value)) => CanonicalRelationalProgram::Return {
                value: translate_expr(value, Some(return_ty), env)?,
            },
            Stmt::If { cond, then, else_ } => CanonicalRelationalProgram::Branch {
                condition: translate_expr(cond, Some(ScalarTy::Bool), env)?,
                then_program: Box::new(translate_block(
                    then,
                    Some(current.clone()),
                    return_ty,
                    env,
                )?),
                else_program: Box::new(match else_ {
                    Some(else_block) => translate_block(else_block, Some(current), return_ty, env)?,
                    None => current,
                }),
            },
            Stmt::Holding { body, .. } => translate_block(body, Some(current), return_ty, env)?,
            Stmt::Let { .. }
            | Stmt::Return(None)
            | Stmt::Loop(_)
            | Stmt::Forget { .. }
            | Stmt::Break
            | Stmt::Continue
            | Stmt::Expr(_) => {
                return Err("statement is outside the bounded relational fragment".into());
            }
        };
    }
    Ok(current)
}

fn translate_function(
    program: &Program,
    function: &thermite_syntax::FnItem,
) -> Result<CanonicalRelationalProgram, String> {
    let body = function
        .body
        .as_ref()
        .ok_or("foreign boundary has no Thermite body")?;
    let return_ty = scalar_ty(&function.ret).ok_or("return type is not a bounded scalar")?;
    translate_block(
        body,
        None,
        return_ty,
        &TranslationEnv::new(program, function),
    )
}

fn body_regions(body: &CanonicalRelationalProgram) -> (Vec<String>, Vec<String>) {
    fn expr_reads(expr: &CanonicalRelationalExpr, reads: &mut BTreeSet<String>) {
        match expr {
            CanonicalRelationalExpr::Region { path } => {
                reads.insert(path.clone());
            }
            CanonicalRelationalExpr::Arith { left, right, .. }
            | CanonicalRelationalExpr::Compare { left, right, .. }
            | CanonicalRelationalExpr::Logic { left, right, .. } => {
                expr_reads(left, reads);
                expr_reads(right, reads);
            }
            CanonicalRelationalExpr::Not { value }
            | CanonicalRelationalExpr::Cast { value, .. } => expr_reads(value, reads),
            CanonicalRelationalExpr::Int { .. }
            | CanonicalRelationalExpr::Bool { .. }
            | CanonicalRelationalExpr::Local { .. } => {}
        }
    }
    fn walk(
        body: &CanonicalRelationalProgram,
        reads: &mut BTreeSet<String>,
        writes: &mut BTreeSet<String>,
    ) {
        match body {
            CanonicalRelationalProgram::Return { value } => expr_reads(value, reads),
            CanonicalRelationalProgram::Write {
                region,
                value,
                next,
            } => {
                writes.insert(region.clone());
                expr_reads(value, reads);
                walk(next, reads, writes);
            }
            CanonicalRelationalProgram::Branch {
                condition,
                then_program,
                else_program,
            } => {
                expr_reads(condition, reads);
                walk(then_program, reads, writes);
                walk(else_program, reads, writes);
            }
            CanonicalRelationalProgram::Raise | CanonicalRelationalProgram::Diverge => {}
        }
    }
    let mut reads = BTreeSet::new();
    let mut writes = BTreeSet::new();
    walk(body, &mut reads, &mut writes);
    (reads.into_iter().collect(), writes.into_iter().collect())
}

pub fn emit_relational_frame_witness(checked: &CheckedProgram) -> RelationalFrameWitness {
    let mut functions = Vec::new();
    for item in &checked.source().items {
        let Item::Fn(function) = item else { continue };
        let effects = row_effects(&function.contract.effects);
        let translated = translate_function(checked.source(), function);
        let (body, unsupported_reason) = match translated {
            Ok(body) => (Some(body), None),
            Err(reason) => (None, Some(reason)),
        };
        let (read_footprint, write_footprint) = body.as_ref().map(body_regions).unwrap_or_default();
        functions.push(RelationalFunctionWitness {
            function: function.name.clone(),
            normalized_row: effects.iter().map(effect_name).collect(),
            read_footprint,
            write_footprint,
            effect_support: effects
                .iter()
                .map(|effect| EffectSupport {
                    effect: effect_name(effect),
                    support: effect_support(effect),
                })
                .collect(),
            semantic_fragment: if body.is_some() {
                "tier-a-bounded-relational-v1".into()
            } else {
                "outside-tier-a-bounded-relational-v1".into()
            },
            projections: if body.is_some() {
                projections(&effects)
            } else {
                Vec::new()
            },
            body,
            unsupported_reason,
            scope: RelationalScope::SourceOnly,
        });
    }
    RelationalFrameWitness {
        version: RELATIONAL_FRAME_WITNESS_VERSION,
        canonical_ast_sha256: crate::witness::canonical_ast_sha256(checked.source()),
        functions,
    }
}

pub fn canonical_relational_frame_witness(
    source: &Program,
) -> Result<RelationalFrameWitness, WitnessError> {
    let checked = CheckedProgram::build(source).map_err(WitnessError::Construction)?;
    Ok(emit_relational_frame_witness(&checked))
}

pub fn replay_relational_frame_witness(
    source: &Program,
    witness: &RelationalFrameWitness,
) -> Result<CheckedProgram, WitnessError> {
    let checked = CheckedProgram::build(source).map_err(WitnessError::Construction)?;
    let expected = emit_relational_frame_witness(&checked);
    if witness.version != expected.version {
        return Err(WitnessError::Mismatch {
            field: "relational_frame_version",
        });
    }
    if witness.canonical_ast_sha256 != expected.canonical_ast_sha256 {
        return Err(WitnessError::Mismatch {
            field: "relational_frame_canonical_ast_sha256",
        });
    }
    if witness.functions != expected.functions {
        return Err(WitnessError::Mismatch {
            field: "relational_frame_functions",
        });
    }
    Ok(checked)
}

fn t2_expr_supported(expr: &CanonicalRelationalExpr) -> bool {
    match expr {
        CanonicalRelationalExpr::Bool { .. } | CanonicalRelationalExpr::Local { .. } => true,
        CanonicalRelationalExpr::Arith { left, right, .. }
        | CanonicalRelationalExpr::Compare { left, right, .. }
        | CanonicalRelationalExpr::Logic { left, right, .. } => {
            t2_expr_supported(left) && t2_expr_supported(right)
        }
        CanonicalRelationalExpr::Not { value } | CanonicalRelationalExpr::Cast { value, .. } => {
            t2_expr_supported(value)
        }
        // Integer literals need an in-range proof at the Rust→Lean boundary;
        // retain source-only scope until that proof is carried explicitly.
        CanonicalRelationalExpr::Int { .. } | CanonicalRelationalExpr::Region { .. } => false,
    }
}

/// Produce end-to-end receipts only for exact region-free return bodies covered
/// by `body_ref_sound`/T2. The source witness itself remains source-only; this
/// separately replayed receipt is the sole scope-upgrade authority.
pub fn emit_relational_transport_receipts(
    source: &Program,
    witness: &RelationalFrameWitness,
) -> Result<Vec<RelationalTransportReceipt>, RelationalTransportError> {
    replay_relational_frame_witness(source, witness)?;
    let lowered = crate::lower(source)
        .map_err(|error| RelationalTransportError::Lowering(format!("{error:?}")))?;
    let lowered_artifact_sha256 = format!("{:x}", Sha256::digest(lowered.as_bytes()));
    Ok(witness
        .functions
        .iter()
        .filter_map(|function| {
            let body = function.body.as_ref()?;
            let CanonicalRelationalProgram::Return { value } = body else {
                return None;
            };
            if !t2_expr_supported(value) {
                return None;
            }
            Some(RelationalTransportReceipt {
                version: 1,
                function: function.function.clone(),
                canonical_ast_sha256: witness.canonical_ast_sha256.clone(),
                source_body: body.clone(),
                lowered_artifact_sha256: lowered_artifact_sha256.clone(),
                theorem: "Thermite.RelationalFrameTransport.bounded_return_pair_end_to_end".into(),
                scope: RelationalScope::EndToEnd,
            })
        })
        .collect())
}

pub fn replay_relational_transport_receipts(
    source: &Program,
    witness: &RelationalFrameWitness,
    receipts: &[RelationalTransportReceipt],
) -> Result<(), RelationalTransportError> {
    let expected = emit_relational_transport_receipts(source, witness)?;
    if receipts != expected {
        return Err(RelationalTransportError::Mismatch(
            "relational_transport_receipts",
        ));
    }
    Ok(())
}

fn lean_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string cannot fail")
}

/// Injective, prefix-preserving encoding of a segmented source region into the
/// kernel's `List Nat` carrier. Each segment is length-prefixed UTF-8, so two
/// different segment sequences cannot alias and source ancestry is preserved.
fn lean_region(path: &str) -> String {
    let encoded = path
        .split('.')
        .flat_map(|segment| {
            std::iter::once(segment.len())
                .chain(segment.as_bytes().iter().copied().map(usize::from))
        })
        .map(|byte| byte.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{encoded}]")
}

fn lean_int_ty(ty: BoundedIntTy) -> &'static str {
    match ty {
        BoundedIntTy::U8 => ".u8",
        BoundedIntTy::U16 => ".u16",
        BoundedIntTy::U32 => ".u32",
        BoundedIntTy::U64 => ".u64",
        BoundedIntTy::Usize => ".usize",
    }
}

fn lean_rel_expr(expr: &CanonicalRelationalExpr) -> String {
    match expr {
        CanonicalRelationalExpr::Int { ty, value } => {
            format!(".literal (.int ⟨{}, {}⟩)", lean_int_ty(*ty), value)
        }
        CanonicalRelationalExpr::Bool { value } => format!(".literal (.bool {value})"),
        CanonicalRelationalExpr::Local { name } => {
            format!(".local (.var {})", lean_string(name))
        }
        CanonicalRelationalExpr::Region { path } => format!(".region {}", lean_region(path)),
        CanonicalRelationalExpr::Arith { op, left, right } => {
            let op = match op {
                RelationalArithOp::Add => ".add",
                RelationalArithOp::Sub => ".sub",
                RelationalArithOp::Mul => ".mul",
                RelationalArithOp::Div => ".div",
                RelationalArithOp::Rem => ".rem",
                RelationalArithOp::Shl => ".shl",
                RelationalArithOp::Shr => ".shr",
                RelationalArithOp::BitAnd => ".bitAnd",
                RelationalArithOp::BitOr => ".bitOr",
                RelationalArithOp::BitXor => ".bitXor",
            };
            format!(
                ".arith {op} ({}) ({})",
                lean_rel_expr(left),
                lean_rel_expr(right)
            )
        }
        CanonicalRelationalExpr::Compare { op, left, right } => {
            let op = match op {
                RelationalCompareOp::Eq => ".eq",
                RelationalCompareOp::Ne => ".ne",
                RelationalCompareOp::Lt => ".lt",
                RelationalCompareOp::Le => ".le",
                RelationalCompareOp::Gt => ".gt",
                RelationalCompareOp::Ge => ".ge",
            };
            format!(
                ".compare {op} ({}) ({})",
                lean_rel_expr(left),
                lean_rel_expr(right)
            )
        }
        CanonicalRelationalExpr::Logic { op, left, right } => {
            let op = match op {
                RelationalLogicOp::And => ".and",
                RelationalLogicOp::Or => ".or",
            };
            format!(
                ".logic {op} ({}) ({})",
                lean_rel_expr(left),
                lean_rel_expr(right)
            )
        }
        CanonicalRelationalExpr::Not { value } => {
            format!(".not ({})", lean_rel_expr(value))
        }
        CanonicalRelationalExpr::Cast { value, ty } => {
            format!(".cast ({}) {}", lean_rel_expr(value), lean_int_ty(*ty))
        }
    }
}

fn lean_exec_expr(expr: &CanonicalRelationalExpr) -> Option<String> {
    match expr {
        CanonicalRelationalExpr::Bool { value } => Some(format!(".boolLit {value}")),
        CanonicalRelationalExpr::Local { name } => Some(format!(".var {}", lean_string(name))),
        CanonicalRelationalExpr::Arith { op, left, right } => {
            let op = match op {
                RelationalArithOp::Add => ".add",
                RelationalArithOp::Sub => ".sub",
                RelationalArithOp::Mul => ".mul",
                RelationalArithOp::Div => ".div",
                RelationalArithOp::Rem => ".rem",
                RelationalArithOp::Shl => ".shl",
                RelationalArithOp::Shr => ".shr",
                RelationalArithOp::BitAnd => ".bitAnd",
                RelationalArithOp::BitOr => ".bitOr",
                RelationalArithOp::BitXor => ".bitXor",
            };
            Some(format!(
                ".arith {op} ({}) ({})",
                lean_exec_expr(left)?,
                lean_exec_expr(right)?
            ))
        }
        CanonicalRelationalExpr::Compare { op, left, right } => {
            let op = match op {
                RelationalCompareOp::Eq => ".eq",
                RelationalCompareOp::Ne => ".ne",
                RelationalCompareOp::Lt => ".lt",
                RelationalCompareOp::Le => ".le",
                RelationalCompareOp::Gt => ".gt",
                RelationalCompareOp::Ge => ".ge",
            };
            Some(format!(
                ".cmp {op} ({}) ({})",
                lean_exec_expr(left)?,
                lean_exec_expr(right)?
            ))
        }
        CanonicalRelationalExpr::Logic { op, left, right } => {
            let op = match op {
                RelationalLogicOp::And => ".and",
                RelationalLogicOp::Or => ".or",
            };
            Some(format!(
                ".logic {op} ({}) ({})",
                lean_exec_expr(left)?,
                lean_exec_expr(right)?
            ))
        }
        CanonicalRelationalExpr::Not { value } => {
            Some(format!(".not ({})", lean_exec_expr(value)?))
        }
        CanonicalRelationalExpr::Cast { value, ty } => Some(format!(
            ".cast ({}) {}",
            lean_exec_expr(value)?,
            lean_int_ty(*ty)
        )),
        CanonicalRelationalExpr::Int { .. } | CanonicalRelationalExpr::Region { .. } => None,
    }
}

fn lean_rel_program(program: &CanonicalRelationalProgram) -> String {
    match program {
        CanonicalRelationalProgram::Return { value } => {
            format!(".ret ({})", lean_rel_expr(value))
        }
        CanonicalRelationalProgram::Write {
            region,
            value,
            next,
        } => format!(
            ".write {} ({}) ({})",
            lean_region(region),
            lean_rel_expr(value),
            lean_rel_program(next)
        ),
        CanonicalRelationalProgram::Branch {
            condition,
            then_program,
            else_program,
        } => format!(
            ".branch ({}) ({}) ({})",
            lean_rel_expr(condition),
            lean_rel_program(then_program),
            lean_rel_program(else_program)
        ),
        CanonicalRelationalProgram::Raise => ".raise".into(),
        CanonicalRelationalProgram::Diverge => ".diverge".into(),
    }
}

fn lean_effect_kind(effect: &str) -> &'static str {
    if effect.starts_with("read(") {
        ".read"
    } else if effect.starts_with("write(") {
        ".write"
    } else if effect.starts_with("net(") {
        ".net"
    } else if effect.starts_with("owns(") {
        ".owns"
    } else if effect.starts_with("forgets(") {
        ".forgets"
    } else {
        match effect {
            "alloc" => ".alloc",
            "time" => ".time",
            "rand" => ".rand",
            "blocks" => ".blocks",
            "panic" => ".panic",
            "diverge" => ".diverge",
            "term" => ".term",
            _ => unreachable!("closed checked effect row: {effect}"),
        }
    }
}

fn lean_projection(projection: Projection) -> &'static str {
    match projection {
        Projection::Result => ".result",
        Projection::WriteFrame => ".writeFrame",
        Projection::Outcome => ".outcome",
        Projection::Termination => ".termination",
        Projection::Trace => ".trace",
        Projection::Accumulator => ".accumulator",
    }
}

fn lean_list(values: impl IntoIterator<Item = String>) -> String {
    format!("[{}]", values.into_iter().collect::<Vec<_>>().join(", "))
}

/// Emit a cold Lean replay for every authority-bearing function in an exact
/// checked witness. Unsupported functions are intentionally absent because
/// their canonical witness carries no projections.
pub fn lean_relational_frame_replay_source(
    source: &Program,
    witness: &RelationalFrameWitness,
) -> Result<String, WitnessError> {
    replay_relational_frame_witness(source, witness)?;
    let mut replay = String::from(
        "import Thermite.RelationalFrameWitness\n\
         open Thermite.RelationalFrame\n\
         open Thermite.RelationalFrameWitness\n\
         open Thermite.RelationalFrame.Bounded\n\n",
    );
    for (index, function) in witness.functions.iter().enumerate() {
        let Some(body) = &function.body else { continue };
        let row = lean_list(
            function
                .normalized_row
                .iter()
                .map(|effect| lean_effect_kind(effect).to_string()),
        );
        let row_text = lean_list(
            function
                .normalized_row
                .iter()
                .map(|value| lean_string(value)),
        );
        let reads = lean_list(function.read_footprint.iter().map(|path| lean_region(path)));
        let writes = lean_list(
            function
                .write_footprint
                .iter()
                .map(|path| lean_region(path)),
        );
        let requested = lean_list(
            function
                .projections
                .iter()
                .map(|projection| lean_projection(*projection).to_string()),
        );
        replay.push_str(&format!(
            "def relationalInput{index} : CanonicalInput :=\n  \
             {{ artifactDigest := {}, body := {}, normalizedRow := {row}, \
             normalizedRowText := {row_text}, readFootprint := {reads}, \
             writeFootprint := {writes}, semanticFragment := {}, requested := {requested} }}\n\
             def relationalWitness{index} : Witness := produce relationalInput{index}\n\
             theorem relationalReplay{index} : verify relationalInput{index} relationalWitness{index} = true := by decide\n\
             #print axioms relationalReplay{index}\n\n",
            lean_string(&witness.canonical_ast_sha256),
            lean_rel_program(body),
            lean_string(&function.semantic_fragment),
        ));
    }
    replay.push_str("#eval IO.println \"THERMITE_RELATIONAL_FRAME_REPLAY_ACCEPTED_V1\"\n");
    Ok(replay)
}

pub fn lean_relational_transport_replay_source(
    source: &Program,
    witness: &RelationalFrameWitness,
    receipts: &[RelationalTransportReceipt],
) -> Result<String, RelationalTransportError> {
    replay_relational_transport_receipts(source, witness, receipts)?;
    let mut replay = String::from(
        "import Thermite.RelationalFrameTransport\n\
         open Thermite.RelationalFrame\n\
         open Thermite.RelationalFrame.Bounded\n\
         open Thermite.RelationalFrameTransport\n\n",
    );
    for (index, receipt) in receipts.iter().enumerate() {
        let CanonicalRelationalProgram::Return { value } = &receipt.source_body else {
            return Err(RelationalTransportError::Mismatch(
                "relational_transport_source_body",
            ));
        };
        let encoded = lean_exec_expr(value).ok_or(RelationalTransportError::Mismatch(
            "relational_transport_exec_expression",
        ))?;
        replay.push_str(&format!(
            "def transportedSourceExpr{index} : Thermite.RelationalFrame.Bounded.Expr := {}\n\
             def transportedExecExpr{index} : Thermite.Exec.ExecExpr := {encoded}\n\
             theorem transportedMapping{index} : toExec transportedSourceExpr{index} = some transportedExecExpr{index} := by rfl\n\
             theorem transportedReceipt{index} (left right : World) (localsEq : left.locals = right.locals) :\n  \
             Thermite.Exec.bodyRefState (.mk [] (some transportedExecExpr{index})) left.locals =\n    \
             Thermite.Exec.bodyRefState (.mk [] (some transportedExecExpr{index})) right.locals := by\n  \
             exact bounded_return_pair_end_to_end transportedSourceExpr{index} transportedExecExpr{index} transportedMapping{index} left right localsEq\n\
             #print axioms transportedReceipt{index}\n\n",
            lean_rel_expr(value),
        ));
    }
    replay.push_str("#eval IO.println \"THERMITE_RELATIONAL_TRANSPORT_ACCEPTED_V1\"\n");
    Ok(replay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use thermite_syntax::{parse, RegionPath};

    const SOURCE: &str = "struct State { n: u64 } keeps n < 10
        shared state: State
        fn bump() -> u64 ! read(state.n), write(state.n)
          requires true ensures result < 10
        { state.n = state.n + 1; state.n }";

    fn fixture() -> (Program, RelationalFrameWitness) {
        let parsed = parse(SOURCE);
        assert!(parsed.is_clean(), "{:?}", parsed.errors);
        let witness = canonical_relational_frame_witness(&parsed.program).unwrap();
        (parsed.program, witness)
    }

    #[test]
    fn witness_is_canonical_source_only_and_round_trips() {
        let (program, witness) = fixture();
        assert_eq!(witness.functions[0].scope, RelationalScope::SourceOnly);
        assert_eq!(witness.functions[0].read_footprint, ["state.n"]);
        assert_eq!(witness.functions[0].write_footprint, ["state.n"]);
        assert!(witness.functions[0].body.is_some());
        assert!(witness.functions[0].unsupported_reason.is_none());
        assert!(witness.functions[0]
            .projections
            .contains(&Projection::Result));
        let json = witness.canonical_json().unwrap();
        assert_eq!(RelationalFrameWitness::from_json(&json).unwrap(), witness);
        replay_relational_frame_witness(&program, &witness).unwrap();
    }

    #[test]
    fn every_authority_bearing_mutation_is_rejected() {
        let (program, witness) = fixture();
        let mut mutants = Vec::new();
        let mut changed = witness.clone();
        changed.canonical_ast_sha256.push('0');
        mutants.push(changed);
        let mut changed = witness.clone();
        changed.functions[0].normalized_row.clear();
        mutants.push(changed);
        let mut changed = witness.clone();
        changed.functions[0].read_footprint.clear();
        mutants.push(changed);
        let mut changed = witness.clone();
        changed.functions[0].write_footprint.clear();
        mutants.push(changed);
        let mut changed = witness.clone();
        changed.functions[0].effect_support.clear();
        mutants.push(changed);
        let mut changed = witness.clone();
        changed.functions[0].body = None;
        mutants.push(changed);
        let mut changed = witness.clone();
        changed.functions[0].semantic_fragment = "other".into();
        mutants.push(changed);
        let mut changed = witness.clone();
        changed.functions[0].projections.clear();
        mutants.push(changed);
        let mut changed = witness.clone();
        changed.functions[0].scope = RelationalScope::EndToEnd;
        mutants.push(changed);
        for mutant in mutants {
            assert!(replay_relational_frame_witness(&program, &mutant).is_err());
        }
    }

    #[test]
    fn net_retains_frame_while_result_is_conditional() {
        let net = effect_support(&Effect::Net(RegionPath::from("socket")));
        assert_eq!(
            net.result,
            Support::Conditional(ResearchGate::ExternalCoupling)
        );
        assert_eq!(net.write_frame, Support::Derived);
    }

    #[test]
    fn structural_atoms_do_not_mint_semantic_projections() {
        let projections = projections(&[Effect::Owns("lock".into())]);
        assert!(projections.is_empty());
    }

    #[test]
    fn unsupported_body_fails_closed_without_semantic_projections() {
        let parsed =
            parse("fn f(x: u64) -> u64 ! pure requires true ensures result == x { let y = x; y }");
        assert!(parsed.is_clean(), "{:?}", parsed.errors);
        let witness = canonical_relational_frame_witness(&parsed.program).unwrap();
        assert!(witness.functions[0].body.is_none());
        assert!(witness.functions[0].unsupported_reason.is_some());
        assert!(witness.functions[0].projections.is_empty());
    }
}
