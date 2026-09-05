//! Canonical semantic-node inventory and iterative traversal.
//!
//! This is the single structural child relation for executable syntax. Wrapper
//! records (`MatchArm`, `LoopKind`, `IndexArg`, and patterns) are nodes rather
//! than fields consumers must remember to inspect independently.

use crate::ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
    Program,
    Item,
    Clause,
    Block,
    Stmt,
    Loop,
    LoopKind,
    MatchArm,
    Expr,
    IndexArg,
    Pattern,
    SlicePattern,
    Inhabit,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticFact {
    None,
    NamedItem(String),
    Function { name: String, params: Vec<String> },
    LetBinding(String),
    ClosureBindings(Vec<String>),
    MatchBindings(Vec<String>),
    QuantifierBinding(String),
    Holding { lock: String },
    Loop,
    Return,
    Break,
    Continue,
    Clause,
    Place(RegionPath),
    Call { path: Option<Vec<String>> },
    MethodCall { method: String },
    StringLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChildRole {
    Item,
    Requires,
    Ensures,
    Asks,
    Promises,
    Measure,
    Keeps,
    Body,
    Statement,
    Tail,
    Initializer,
    Target,
    Value,
    ReturnValue,
    Condition,
    Then,
    Else,
    LoopKind,
    Invariant,
    HoldingBody,
    Callee,
    Argument,
    Receiver,
    ClosureBody,
    Scrutinee,
    MatchArm,
    Pattern,
    Guard,
    MatchBody,
    Left,
    Right,
    Operand,
    Index,
    RangeStart,
    RangeEnd,
    FieldValue,
    TupleElement,
    QuantifierDomain,
    QuantifierBody,
    PatternElement,
    WitnessArgument,
}

#[derive(Debug, Clone, Copy)]
pub enum SemanticNode<'a> {
    Program(&'a Program),
    Item(&'a Item),
    Clause(&'a Clause),
    Block(&'a Block),
    Stmt(&'a Stmt),
    Loop(&'a LoopNode),
    LoopKind(&'a LoopKind),
    MatchArm(&'a MatchArm),
    Expr(&'a Expr),
    IndexArg(&'a IndexArg),
    Pattern(&'a Pattern),
    SlicePattern(&'a SlicePat),
    Inhabit(&'a Inhabit),
}

impl<'a> SemanticNode<'a> {
    pub fn kind(self) -> NodeKind {
        match self {
            Self::Program(_) => NodeKind::Program,
            Self::Item(_) => NodeKind::Item,
            Self::Clause(_) => NodeKind::Clause,
            Self::Block(_) => NodeKind::Block,
            Self::Stmt(_) => NodeKind::Stmt,
            Self::Loop(_) => NodeKind::Loop,
            Self::LoopKind(_) => NodeKind::LoopKind,
            Self::MatchArm(_) => NodeKind::MatchArm,
            Self::Expr(_) => NodeKind::Expr,
            Self::IndexArg(_) => NodeKind::IndexArg,
            Self::Pattern(_) => NodeKind::Pattern,
            Self::SlicePattern(_) => NodeKind::SlicePattern,
            Self::Inhabit(_) => NodeKind::Inhabit,
        }
    }

    fn fact(self) -> SemanticFact {
        match self {
            Self::Item(Item::Fn(function)) => SemanticFact::Function {
                name: function.name.clone(),
                params: function
                    .params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect(),
            },
            Self::Item(item) => SemanticFact::NamedItem(item.name().to_string()),
            Self::Stmt(Stmt::Holding {
                lock,
                body: _,
                span: _,
            }) => SemanticFact::Holding { lock: lock.clone() },
            Self::Stmt(Stmt::Let {
                mutable: _,
                name,
                ty: _,
                init: _,
            }) => SemanticFact::LetBinding(name.clone()),
            Self::Stmt(Stmt::Return(_)) => SemanticFact::Return,
            Self::Stmt(Stmt::Break) => SemanticFact::Break,
            Self::Stmt(Stmt::Continue) => SemanticFact::Continue,
            Self::Loop(_) => SemanticFact::Loop,
            Self::Clause(_) => SemanticFact::Clause,
            Self::MatchArm(arm) => SemanticFact::MatchBindings(pattern_bindings(&arm.pattern)),
            Self::Expr(Expr::Call { callee, args: _ }) => SemanticFact::Call {
                path: match callee.as_ref() {
                    Expr::Path(path) => Some(path.clone()),
                    _ => None,
                },
            },
            Self::Expr(Expr::MethodCall {
                receiver: _,
                name,
                args: _,
            }) => SemanticFact::MethodCall {
                method: name.clone(),
            },
            Self::Expr(Expr::Closure { params, body: _ }) => {
                SemanticFact::ClosureBindings(params.clone())
            }
            Self::Expr(Expr::Quantifier {
                quant: _,
                var,
                sort: _,
                domain: _,
                body: _,
            }) => SemanticFact::QuantifierBinding(var.clone()),
            Self::Expr(Expr::StrLit(_)) => SemanticFact::StringLiteral,
            Self::Expr(expr) => semantic_place(expr)
                .map(SemanticFact::Place)
                .unwrap_or(SemanticFact::None),
            _ => SemanticFact::None,
        }
    }

    fn children(self) -> Vec<(ChildRole, SemanticNode<'a>)> {
        let mut out = Vec::new();
        match self {
            Self::Program(program) => {
                out.extend(
                    program
                        .items
                        .iter()
                        .map(|x| (ChildRole::Item, Self::Item(x))),
                );
            }
            Self::Item(item) => match item {
                Item::Fn(f) => {
                    out.push((ChildRole::Requires, Self::Clause(&f.contract.requires)));
                    out.extend(
                        f.contract
                            .ensures
                            .iter()
                            .map(|x| (ChildRole::Ensures, Self::Clause(x))),
                    );
                    if let Some(interference) = &f.contract.interference {
                        out.push((ChildRole::Asks, Self::Clause(&interference.asks)));
                        out.push((ChildRole::Promises, Self::Clause(&interference.promises)));
                    }
                    if let Some(x) = &f.measures {
                        out.push((ChildRole::Measure, Self::Clause(x)));
                    }
                    if let Some(x) = &f.body {
                        out.push((ChildRole::Body, Self::Block(x)));
                    }
                }
                Item::SpecFn(f) => {
                    out.push((ChildRole::Measure, Self::Clause(&f.measures)));
                    out.push((ChildRole::Body, Self::Block(&f.body)));
                }
                Item::Struct(s) => {
                    if let Some(x) = &s.keeps {
                        out.push((ChildRole::Keeps, Self::Clause(x)));
                    }
                }
                Item::Enum(_)
                | Item::EffectDecl(_)
                | Item::SharedDecl(_)
                | Item::Concurrent(_)
                | Item::LockDecl(_) => {}
                Item::Forge(forge) => match forge {
                    ForgeItem::PropFn(f) => {
                        if let Some(x) = &f.measures {
                            out.push((ChildRole::Measure, Self::Clause(x)));
                        }
                        out.push((ChildRole::Body, Self::Block(&f.body)));
                    }
                    ForgeItem::Lemma(l) => {
                        out.push((ChildRole::Requires, Self::Clause(&l.requires)));
                        out.extend(
                            l.ensures
                                .iter()
                                .map(|x| (ChildRole::Ensures, Self::Clause(x))),
                        );
                    }
                    ForgeItem::Proof(_) => {}
                    ForgeItem::Witness(w) => out.extend(
                        w.inhabits
                            .iter()
                            .map(|x| (ChildRole::Body, Self::Inhabit(x))),
                    ),
                },
            },
            Self::Clause(clause) => out.push((ChildRole::Body, Self::Expr(&clause.expr))),
            Self::Block(block) => {
                out.extend(
                    block
                        .stmts
                        .iter()
                        .map(|x| (ChildRole::Statement, Self::Stmt(x))),
                );
                if let Some(x) = &block.tail {
                    out.push((ChildRole::Tail, Self::Expr(x)));
                }
            }
            Self::Stmt(stmt) => match stmt {
                Stmt::Let {
                    mutable: _,
                    name: _,
                    ty: _,
                    init,
                } => out.push((ChildRole::Initializer, Self::Expr(init))),
                Stmt::Assign { target, value } => {
                    out.push((ChildRole::Target, Self::Expr(target)));
                    out.push((ChildRole::Value, Self::Expr(value)));
                }
                Stmt::Return(value) => {
                    if let Some(x) = value {
                        out.push((ChildRole::ReturnValue, Self::Expr(x)));
                    }
                }
                Stmt::If { cond, then, else_ } => {
                    out.push((ChildRole::Condition, Self::Expr(cond)));
                    out.push((ChildRole::Then, Self::Block(then)));
                    if let Some(x) = else_ {
                        out.push((ChildRole::Else, Self::Block(x)));
                    }
                }
                Stmt::Loop(loop_) => out.push((ChildRole::Body, Self::Loop(loop_))),
                Stmt::Holding {
                    lock: _,
                    body,
                    span: _,
                } => out.push((ChildRole::HoldingBody, Self::Block(body))),
                Stmt::Forget { value, .. } => out.push((ChildRole::Value, Self::Expr(value))),
                Stmt::Expr(expr) => out.push((ChildRole::Body, Self::Expr(expr))),
                Stmt::Break | Stmt::Continue => {}
            },
            Self::Loop(loop_) => {
                out.push((ChildRole::LoopKind, Self::LoopKind(&loop_.kind)));
                out.extend(
                    loop_
                        .invs
                        .iter()
                        .map(|x| (ChildRole::Invariant, Self::Clause(x))),
                );
                out.push((ChildRole::Measure, Self::Clause(&loop_.measures)));
                out.push((ChildRole::Body, Self::Block(&loop_.body)));
            }
            Self::LoopKind(kind) => match kind {
                LoopKind::Loop => {}
                LoopKind::While(cond) => out.push((ChildRole::Condition, Self::Expr(cond))),
            },
            Self::MatchArm(arm) => {
                out.push((ChildRole::Pattern, Self::Pattern(&arm.pattern)));
                if let Some(x) = &arm.guard {
                    out.push((ChildRole::Guard, Self::Expr(x)));
                }
                out.push((ChildRole::MatchBody, Self::Expr(&arm.body)));
            }
            Self::Expr(expr) => match expr {
                Expr::IntLit { value: _, raw: _ }
                | Expr::BoolLit(_)
                | Expr::Path(_)
                | Expr::StrLit(_) => {}
                Expr::Call { callee, args } => {
                    out.push((ChildRole::Callee, Self::Expr(callee)));
                    out.extend(args.iter().map(|x| (ChildRole::Argument, Self::Expr(x))));
                }
                Expr::MethodCall {
                    receiver,
                    name: _,
                    args,
                } => {
                    out.push((ChildRole::Receiver, Self::Expr(receiver)));
                    out.extend(args.iter().map(|x| (ChildRole::Argument, Self::Expr(x))));
                }
                Expr::Field { receiver, name: _ } | Expr::TupleProj { receiver, index: _ } => {
                    out.push((ChildRole::Receiver, Self::Expr(receiver)))
                }
                Expr::Closure { params: _, body } => {
                    out.push((ChildRole::ClosureBody, Self::Expr(body)))
                }
                Expr::Match { scrutinee, arms } => {
                    out.push((ChildRole::Scrutinee, Self::Expr(scrutinee)));
                    out.extend(
                        arms.iter()
                            .map(|x| (ChildRole::MatchArm, Self::MatchArm(x))),
                    );
                }
                Expr::If { cond, then, else_ } => {
                    out.push((ChildRole::Condition, Self::Expr(cond)));
                    out.push((ChildRole::Then, Self::Block(then)));
                    out.push((ChildRole::Else, Self::Block(else_)));
                }
                Expr::Binary { op: _, lhs, rhs } => {
                    out.push((ChildRole::Left, Self::Expr(lhs)));
                    out.push((ChildRole::Right, Self::Expr(rhs)));
                }
                Expr::Unary { op: _, expr }
                | Expr::Cast { expr, ty: _ }
                | Expr::Ref { mutable: _, expr }
                | Expr::Deref(expr)
                | Expr::Is {
                    scrutinee: expr,
                    variant: _,
                } => out.push((ChildRole::Operand, Self::Expr(expr))),
                Expr::Index { base, index } => {
                    out.push((ChildRole::Receiver, Self::Expr(base)));
                    out.push((ChildRole::Index, Self::IndexArg(index)));
                }
                Expr::StructLit { path: _, fields } => out.extend(
                    fields
                        .iter()
                        .map(|(_, x)| (ChildRole::FieldValue, Self::Expr(x))),
                ),
                Expr::Tuple(items) => out.extend(
                    items
                        .iter()
                        .map(|x| (ChildRole::TupleElement, Self::Expr(x))),
                ),
                Expr::Quantifier {
                    quant: _,
                    var: _,
                    sort: _,
                    domain,
                    body,
                } => {
                    out.push((ChildRole::QuantifierDomain, Self::Expr(domain)));
                    out.push((ChildRole::QuantifierBody, Self::Expr(body)));
                }
            },
            Self::IndexArg(index) => match index {
                IndexArg::Single(x) | IndexArg::RangeTo(x) => {
                    out.push((ChildRole::RangeEnd, Self::Expr(x)))
                }
                IndexArg::RangeFrom(x) => out.push((ChildRole::RangeStart, Self::Expr(x))),
                IndexArg::Range(a, b) => {
                    out.push((ChildRole::RangeStart, Self::Expr(a)));
                    out.push((ChildRole::RangeEnd, Self::Expr(b)));
                }
            },
            Self::Pattern(pattern) => match pattern {
                Pattern::Wildcard | Pattern::Binding(_) => {}
                Pattern::Literal(x) => out.push((ChildRole::Value, Self::Expr(x))),
                Pattern::Slice(items) => out.extend(
                    items
                        .iter()
                        .map(|x| (ChildRole::PatternElement, Self::SlicePattern(x))),
                ),
                Pattern::Enum { path: _, fields } | Pattern::Or(fields) => out.extend(
                    fields
                        .iter()
                        .map(|x| (ChildRole::PatternElement, Self::Pattern(x))),
                ),
                Pattern::Struct {
                    path: _,
                    fields,
                    rest: _,
                } => out.extend(
                    fields
                        .iter()
                        .map(|(_, x)| (ChildRole::PatternElement, Self::Pattern(x))),
                ),
            },
            Self::SlicePattern(pattern) => match pattern {
                SlicePat::Pat(x) => out.push((ChildRole::PatternElement, Self::Pattern(x))),
                SlicePat::Rest(_) => {}
            },
            Self::Inhabit(inhabit) => out.extend(
                inhabit
                    .args
                    .iter()
                    .map(|x| (ChildRole::WitnessArgument, Self::Expr(x))),
            ),
        }
        out
    }
}

fn pattern_bindings(pattern: &Pattern) -> Vec<String> {
    fn collect(pattern: &Pattern, bindings: &mut Vec<String>) {
        match pattern {
            Pattern::Binding(name) => bindings.push(name.clone()),
            Pattern::Slice(parts) => {
                for part in parts {
                    match part {
                        SlicePat::Pat(pattern) => collect(pattern, bindings),
                        SlicePat::Rest(name) => bindings.push(name.clone()),
                    }
                }
            }
            Pattern::Enum { path: _, fields } | Pattern::Or(fields) => {
                for field in fields {
                    collect(field, bindings);
                }
            }
            Pattern::Struct {
                path: _,
                fields,
                rest: _,
            } => {
                for (_, field) in fields {
                    collect(field, bindings);
                }
            }
            Pattern::Wildcard | Pattern::Literal(_) => {}
        }
    }
    let mut bindings = Vec::new();
    collect(pattern, &mut bindings);
    bindings.sort();
    bindings.dedup();
    bindings
}

fn semantic_place(expr: &Expr) -> Option<RegionPath> {
    match expr {
        Expr::Path(path) => Some(RegionPath {
            segments: path.clone(),
        }),
        Expr::Field { receiver, name } => {
            let mut path = semantic_place(receiver)?;
            path.segments.push(name.clone());
            Some(path)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticEdge {
    pub parent: NodeId,
    pub child: NodeId,
    pub role: ChildRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticInventory {
    pub kinds: Vec<NodeKind>,
    pub facts: Vec<SemanticFact>,
    pub edges: Vec<SemanticEdge>,
}

/// Return whether `root` is bound by a lexical scope visible at `node`.
///
/// This is the canonical scope query for semantic-inventory consumers. A let
/// binding becomes visible only after its statement; parameters, closure and
/// match bindings, and quantifier variables are visible throughout their
/// corresponding descendant scope.
pub fn is_lexically_shadowed(inventory: &SemanticInventory, node: NodeId, root: &str) -> bool {
    let mut parents = vec![None; inventory.facts.len()];
    let mut children = vec![Vec::new(); inventory.facts.len()];
    for edge in &inventory.edges {
        parents[edge.child.0 as usize] = Some(edge.parent);
        children[edge.parent.0 as usize].push(edge.child);
    }
    let mut cursor = Some(node);
    while let Some(current) = cursor {
        match &inventory.facts[current.0 as usize] {
            SemanticFact::Function { params, .. }
            | SemanticFact::ClosureBindings(params)
            | SemanticFact::MatchBindings(params)
                if params.iter().any(|name| name == root) =>
            {
                return true;
            }
            SemanticFact::QuantifierBinding(name) if name == root => return true,
            _ => {}
        }
        let Some(parent) = parents[current.0 as usize] else {
            break;
        };
        for sibling in &children[parent.0 as usize] {
            if sibling.0 >= current.0 {
                break;
            }
            if matches!(&inventory.facts[sibling.0 as usize], SemanticFact::LetBinding(name) if name == root)
            {
                return true;
            }
        }
        cursor = Some(parent);
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkBudget(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimit {
    pub budget: WorkBudget,
    pub required_at_least: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticEvent {
    Enter { id: NodeId, kind: NodeKind },
    Leave { id: NodeId, kind: NodeKind },
}

pub fn semantic_inventory(
    program: &Program,
    budget: WorkBudget,
) -> Result<SemanticInventory, ResourceLimit> {
    let mut kinds = Vec::new();
    let mut facts = Vec::new();
    let mut edges = Vec::new();
    let mut stack = vec![(None, SemanticNode::Program(program))];
    while let Some((parent, node)) = stack.pop() {
        if kinds.len() == budget.0 {
            return Err(ResourceLimit {
                budget,
                required_at_least: kinds.len() + 1,
            });
        }
        let id = NodeId(u32::try_from(kinds.len()).map_err(|_| ResourceLimit {
            budget,
            required_at_least: kinds.len() + 1,
        })?);
        kinds.push(node.kind());
        facts.push(node.fact());
        if let Some((parent, role)) = parent {
            edges.push(SemanticEdge {
                parent,
                child: id,
                role,
            });
        }
        let children = node.children();
        stack.extend(
            children
                .into_iter()
                .rev()
                .map(|(role, child)| (Some((id, role)), child)),
        );
    }
    Ok(SemanticInventory {
        kinds,
        facts,
        edges,
    })
}

pub fn walk_semantic(
    inventory: &SemanticInventory,
    budget: WorkBudget,
) -> Result<Vec<SemanticEvent>, ResourceLimit> {
    if inventory.kinds.is_empty() {
        return Ok(Vec::new());
    }
    let required = inventory.kinds.len().saturating_mul(2);
    if required > budget.0 {
        return Err(ResourceLimit {
            budget,
            required_at_least: required,
        });
    }
    let mut children = vec![Vec::new(); inventory.kinds.len()];
    for edge in &inventory.edges {
        children[edge.parent.0 as usize].push(edge.child);
    }
    let mut events = Vec::with_capacity(required);
    let mut stack = vec![(NodeId(0), false)];
    while let Some((id, leaving)) = stack.pop() {
        let kind = inventory.kinds[id.0 as usize];
        if leaving {
            events.push(SemanticEvent::Leave { id, kind });
        } else {
            events.push(SemanticEvent::Enter { id, kind });
            stack.push((id, true));
            stack.extend(
                children[id.0 as usize]
                    .iter()
                    .rev()
                    .map(|child| (*child, false)),
            );
        }
    }
    Ok(events)
}
