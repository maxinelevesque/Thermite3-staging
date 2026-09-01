//! RFC-11 executable ownership-flow checking.

use std::collections::{BTreeMap, BTreeSet};

use thermite_syntax::{
    Block, Effect, EffectRow, Expr, FnItem, Item, LoopKind, Pattern, Program, RegionPath, Span,
    Stmt, Type, VariantShape,
};

use crate::ResourceEnv;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceFlowReport {
    pub direct_forgets: BTreeMap<String, BTreeSet<RegionPath>>,
    pub functions: BTreeMap<String, ResourceFunctionFlow>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceFunctionFlow {
    pub entry_live: Vec<String>,
    pub returning_edges: Vec<ResourceReturningEdge>,
    pub joins: Vec<ResourceJoinFact>,
    pub loops: Vec<ResourceLoopFact>,
    pub forgets: Vec<ResourceForgetFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceReturningEdge {
    pub label: String,
    pub live: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceJoinFact {
    pub label: String,
    pub incoming: Vec<Vec<String>>,
    pub outgoing: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLoopFact {
    pub label: String,
    pub header: Vec<String>,
    pub back_edges: Vec<Vec<String>>,
    pub exit_edges: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceForgetFact {
    pub label: String,
    pub place: Option<String>,
    pub value_regions: Vec<RegionPath>,
    pub priced_regions: Vec<RegionPath>,
    pub declared_regions: Vec<RegionPath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceFlowErrorKind {
    Copy,
    UseAfterMove,
    ImplicitDrop,
    Unconsumed,
    NonResourceForget,
    MissingForgetEffect,
    BranchMismatch,
    LoopMismatch,
    UnsupportedProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceFlowError {
    pub kind: ResourceFlowErrorKind,
    pub place: Option<String>,
    pub detail: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
struct Binding {
    ty: Type,
    live: bool,
}

#[derive(Debug, Clone, Default)]
struct State {
    bindings: BTreeMap<String, Binding>,
}

impl State {
    fn live_resources(&self) -> BTreeSet<String> {
        self.bindings
            .iter()
            .filter(|(_, binding)| binding.live)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeKind {
    Next,
    Return,
    Break,
    Continue,
}

#[derive(Debug, Clone)]
struct Edge {
    kind: EdgeKind,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum UseMode {
    Move,
    Borrow,
    Observe,
}

#[derive(Clone)]
struct Signature {
    params: Vec<Type>,
    ret: Type,
}

struct Checker<'a> {
    resources: &'a ResourceEnv,
    signatures: BTreeMap<String, Signature>,
    structs: BTreeMap<String, Vec<(String, Type)>>,
    variants: BTreeMap<String, (String, VariantShape)>,
    errors: Vec<ResourceFlowError>,
    direct_forgets: BTreeMap<String, BTreeSet<RegionPath>>,
    functions: BTreeMap<String, ResourceFunctionFlow>,
    function: String,
    function_span: Span,
    declared_effects: BTreeSet<Effect>,
    return_index: usize,
    join_index: usize,
    loop_index: usize,
    forget_index: usize,
}

pub fn check_resource_flow(
    program: &Program,
    resources: &ResourceEnv,
) -> Result<ResourceFlowReport, Vec<ResourceFlowError>> {
    let signatures = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(function) => Some((
                function.name.clone(),
                Signature {
                    params: function
                        .params
                        .iter()
                        .map(|param| param.ty.clone())
                        .collect(),
                    ret: function.ret.clone(),
                },
            )),
            _ => None,
        })
        .collect();
    let structs = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(item) => Some((
                item.name.clone(),
                item.fields
                    .iter()
                    .map(|field| (field.name.clone(), field.ty.clone()))
                    .collect(),
            )),
            _ => None,
        })
        .collect();
    let variants = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Enum(item) => Some(item),
            _ => None,
        })
        .flat_map(|item| {
            item.variants.iter().map(|variant| {
                (
                    variant.name.clone(),
                    (item.name.clone(), variant.shape.clone()),
                )
            })
        })
        .collect();

    let mut checker = Checker {
        resources,
        signatures,
        structs,
        variants,
        errors: Vec::new(),
        direct_forgets: BTreeMap::new(),
        functions: BTreeMap::new(),
        function: String::new(),
        function_span: Span::new(0, 0),
        declared_effects: BTreeSet::new(),
        return_index: 0,
        join_index: 0,
        loop_index: 0,
        forget_index: 0,
    };
    for item in &program.items {
        if let Item::Fn(function) = item {
            checker.check_function(function);
        }
    }
    if checker.errors.is_empty() {
        Ok(ResourceFlowReport {
            direct_forgets: checker.direct_forgets,
            functions: checker.functions,
        })
    } else {
        Err(checker.errors)
    }
}

impl Checker<'_> {
    fn check_function(&mut self, function: &FnItem) {
        let Some(body) = &function.body else { return };
        self.function.clone_from(&function.name);
        self.function_span = function.span;
        self.return_index = 0;
        self.join_index = 0;
        self.loop_index = 0;
        self.forget_index = 0;
        self.declared_effects = match &function.contract.effects {
            EffectRow::Pure => BTreeSet::new(),
            EffectRow::Set(effects) => effects.iter().cloned().collect(),
        };
        let mut state = State::default();
        for param in &function.params {
            state.bindings.insert(
                param.name.clone(),
                Binding {
                    ty: param.ty.clone(),
                    live: self.is_resource(&param.ty),
                },
            );
        }
        self.functions.insert(
            function.name.clone(),
            ResourceFunctionFlow {
                entry_live: state.live_resources().into_iter().collect(),
                ..ResourceFunctionFlow::default()
            },
        );
        let edges = self.check_block(body, state);
        for edge in edges {
            match edge.kind {
                EdgeKind::Next => {
                    let mut state = edge.state;
                    if let Some(tail) = &body.tail {
                        self.use_expr(tail, UseMode::Move, &mut state);
                    }
                    self.require_empty(&state, "function fallthrough");
                }
                EdgeKind::Return => {}
                EdgeKind::Break | EdgeKind::Continue => self.error(
                    ResourceFlowErrorKind::LoopMismatch,
                    None,
                    "loop control escaped its enclosing loop".to_string(),
                    function.span,
                ),
            }
        }
    }

    fn check_block(&mut self, block: &Block, initial: State) -> Vec<Edge> {
        let mut edges = vec![Edge {
            kind: EdgeKind::Next,
            state: initial,
        }];
        for stmt in &block.stmts {
            let mut next = Vec::new();
            for edge in edges {
                if edge.kind == EdgeKind::Next {
                    next.extend(self.check_stmt(stmt, edge.state));
                } else {
                    next.push(edge);
                }
            }
            edges = next;
        }
        edges
    }

    fn check_stmt(&mut self, stmt: &Stmt, mut state: State) -> Vec<Edge> {
        match stmt {
            Stmt::Let { name, ty, init, .. } => {
                if state.bindings.get(name).is_some_and(|binding| binding.live) {
                    self.error(
                        ResourceFlowErrorKind::ImplicitDrop,
                        Some(name.clone()),
                        format!("shadowing overwrites live resource `{name}`"),
                        self.function_span,
                    );
                }
                let inferred = self.use_expr(init, UseMode::Move, &mut state);
                let binding_ty = ty.clone().or(inferred).unwrap_or(Type::Unit);
                state.bindings.insert(
                    name.clone(),
                    Binding {
                        live: self.is_resource(&binding_ty),
                        ty: binding_ty,
                    },
                );
                next(state)
            }
            Stmt::Assign { target, value } => {
                let Expr::Path(path) = target else {
                    // RFC-11 owns only resource-bearing assignment. Preserve
                    // existing field/index assignment for the RFC-10 and
                    // ordinary-value validators, while still walking the
                    // target so projecting through a resource is rejected and
                    // consuming a resource into an unsupported place fails
                    // closed.
                    self.use_expr(target, UseMode::Borrow, &mut state);
                    let inferred = self.use_expr(value, UseMode::Move, &mut state);
                    if inferred.as_ref().is_some_and(|ty| self.is_resource(ty)) {
                        self.error(
                            ResourceFlowErrorKind::UnsupportedProjection,
                            expr_place(value),
                            "moving a resource requires assignment to a semantic local place"
                                .to_string(),
                            self.function_span,
                        );
                    }
                    return next(state);
                };
                let Some(name) = path.first().filter(|_| path.len() == 1).cloned() else {
                    return next(state);
                };
                if state
                    .bindings
                    .get(&name)
                    .is_some_and(|binding| binding.live)
                {
                    self.error(
                        ResourceFlowErrorKind::ImplicitDrop,
                        Some(name.clone()),
                        format!("assignment overwrites live resource `{name}`"),
                        self.function_span,
                    );
                }
                let inferred = self.use_expr(value, UseMode::Move, &mut state);
                if let Some(binding) = state.bindings.get_mut(&name) {
                    if let Some(inferred) = inferred {
                        binding.ty = inferred;
                    }
                    binding.live = self.is_resource(&binding.ty);
                }
                next(state)
            }
            Stmt::Return(value) => {
                if let Some(value) = value {
                    self.use_expr(value, UseMode::Move, &mut state);
                }
                self.require_empty(&state, "return");
                vec![Edge {
                    kind: EdgeKind::Return,
                    state,
                }]
            }
            Stmt::Forget { value, span } => {
                let ty = self.use_expr(value, UseMode::Move, &mut state);
                let provenance = ty
                    .as_ref()
                    .map(|ty| self.resources.provenance_of_type(ty))
                    .unwrap_or_default();
                if provenance.is_empty() {
                    self.error(
                        ResourceFlowErrorKind::NonResourceForget,
                        expr_place(value),
                        "`forget` requires one live owned resource value".to_string(),
                        *span,
                    );
                } else {
                    let declared_regions: Vec<RegionPath> = self
                        .declared_effects
                        .iter()
                        .filter_map(|effect| match effect {
                            Effect::Forgets(region) => Some(region.clone()),
                            _ => None,
                        })
                        .collect();
                    let priced_regions: Vec<RegionPath> = provenance
                        .iter()
                        .filter(|region| {
                            self.declared_effects
                                .contains(&Effect::Forgets((*region).clone()))
                        })
                        .cloned()
                        .collect();
                    let label = format!("forget#{}", self.forget_index);
                    self.forget_index += 1;
                    self.current_flow_mut().forgets.push(ResourceForgetFact {
                        label,
                        place: expr_place(value),
                        value_regions: provenance.iter().cloned().collect(),
                        priced_regions,
                        declared_regions,
                    });
                    self.direct_forgets
                        .entry(self.function.clone())
                        .or_default()
                        .extend(provenance.iter().cloned());
                    for region in provenance {
                        if !self
                            .declared_effects
                            .contains(&Effect::Forgets(region.clone()))
                        {
                            self.error(
                                ResourceFlowErrorKind::MissingForgetEffect,
                                expr_place(value),
                                format!("`forget` requires `forgets({region})` in the declared effect row"),
                                *span,
                            );
                        }
                    }
                }
                next(state)
            }
            Stmt::Expr(expr) => {
                let ty = self.use_expr(expr, UseMode::Move, &mut state);
                if ty.as_ref().is_some_and(|ty| self.is_resource(ty)) {
                    self.error(
                        ResourceFlowErrorKind::ImplicitDrop,
                        expr_place(expr),
                        "resource-valued expression result is discarded".to_string(),
                        self.function_span,
                    );
                }
                next(state)
            }
            Stmt::If { cond, then, else_ } => {
                self.use_expr(cond, UseMode::Observe, &mut state);
                let then_edges = self.check_scoped_block(then, state.clone(), true);
                let else_edges = else_
                    .as_ref()
                    .map(|block| self.check_scoped_block(block, state.clone(), true))
                    .unwrap_or_else(|| next(state));
                self.join_branches(then_edges, else_edges)
            }
            Stmt::Loop(loop_) => {
                if let LoopKind::While(cond) = &loop_.kind {
                    self.use_expr(cond, UseMode::Observe, &mut state);
                }
                let header = state.live_resources();
                let body_edges = self.check_scoped_block(&loop_.body, state.clone(), true);
                let mut exits = Vec::new();
                let mut back_edges = Vec::new();
                let mut exit_edges = Vec::new();
                for edge in body_edges {
                    match edge.kind {
                        EdgeKind::Return => exits.push(edge),
                        EdgeKind::Next | EdgeKind::Continue => {
                            back_edges.push(edge.state.live_resources().into_iter().collect());
                            if edge.state.live_resources() != header {
                                self.error(
                                    ResourceFlowErrorKind::LoopMismatch,
                                    None,
                                    "loop back-edge changes the live resource set".to_string(),
                                    loop_.span,
                                );
                            }
                        }
                        EdgeKind::Break => {
                            exit_edges.push(edge.state.live_resources().into_iter().collect());
                            if edge.state.live_resources() != header {
                                self.error(
                                    ResourceFlowErrorKind::LoopMismatch,
                                    None,
                                    "loop break edge disagrees with the entry live resource set"
                                        .to_string(),
                                    loop_.span,
                                );
                            }
                            exits.push(Edge {
                                kind: EdgeKind::Next,
                                state: edge.state,
                            });
                        }
                    }
                }
                // A `while` may execute zero times, so its entry state always
                // contributes an ordinary fallthrough edge. A bare `loop`
                // has no such edge: without a reachable `break`, its back-edge
                // is a declared non-returning path and carries no resource
                // post-obligation (resource-types REQ-11). Returns collected
                // above remain checked returning edges.
                if matches!(loop_.kind, LoopKind::While(_)) {
                    exit_edges.push(header.iter().cloned().collect());
                    exits.push(Edge {
                        kind: EdgeKind::Next,
                        state,
                    });
                }
                let label = format!("loop#{}", self.loop_index);
                self.loop_index += 1;
                self.current_flow_mut().loops.push(ResourceLoopFact {
                    label,
                    header: header.into_iter().collect(),
                    back_edges,
                    exit_edges,
                });
                exits
            }
            Stmt::Holding { body, .. } => self.check_scoped_block(body, state, true),
            Stmt::Break => vec![Edge {
                kind: EdgeKind::Break,
                state,
            }],
            Stmt::Continue => vec![Edge {
                kind: EdgeKind::Continue,
                state,
            }],
        }
    }

    fn check_scoped_block(
        &mut self,
        block: &Block,
        initial: State,
        discard_tail: bool,
    ) -> Vec<Edge> {
        let outer: BTreeSet<String> = initial.bindings.keys().cloned().collect();
        let mut edges = self.check_block(block, initial);
        for edge in &mut edges {
            if edge.kind == EdgeKind::Next && discard_tail {
                if let Some(tail) = &block.tail {
                    let ty = self.use_expr(tail, UseMode::Move, &mut edge.state);
                    if ty.as_ref().is_some_and(|ty| self.is_resource(ty)) {
                        self.error(
                            ResourceFlowErrorKind::ImplicitDrop,
                            expr_place(tail),
                            "resource-valued block result is discarded".to_string(),
                            self.function_span,
                        );
                    }
                }
            }
            let locals: Vec<String> = edge
                .state
                .bindings
                .keys()
                .filter(|name| !outer.contains(*name))
                .cloned()
                .collect();
            for local in locals {
                if edge
                    .state
                    .bindings
                    .get(&local)
                    .is_some_and(|binding| binding.live)
                {
                    self.error(
                        ResourceFlowErrorKind::Unconsumed,
                        Some(local.clone()),
                        format!("resource local `{local}` remains live at block exit"),
                        self.function_span,
                    );
                }
                edge.state.bindings.remove(&local);
            }
        }
        edges
    }

    fn join_branches(&mut self, left: Vec<Edge>, right: Vec<Edge>) -> Vec<Edge> {
        let mut result = Vec::new();
        for kind in [
            EdgeKind::Next,
            EdgeKind::Return,
            EdgeKind::Break,
            EdgeKind::Continue,
        ] {
            let mut matching: Vec<Edge> = left
                .iter()
                .chain(&right)
                .filter(|edge| edge.kind == kind)
                .cloned()
                .collect();
            if let Some(first) = matching.first() {
                let live = first.state.live_resources();
                if matching
                    .iter()
                    .any(|edge| edge.state.live_resources() != live)
                {
                    self.error(
                        ResourceFlowErrorKind::BranchMismatch,
                        None,
                        format!("branch {kind:?} edges disagree on live resources"),
                        self.function_span,
                    );
                }
                let label = format!("join#{}:{kind:?}", self.join_index);
                self.join_index += 1;
                self.current_flow_mut().joins.push(ResourceJoinFact {
                    label,
                    incoming: matching
                        .iter()
                        .map(|edge| edge.state.live_resources().into_iter().collect())
                        .collect(),
                    outgoing: live.into_iter().collect(),
                });
            }
            result.append(&mut matching);
        }
        result
    }

    fn use_expr(&mut self, expr: &Expr, mode: UseMode, state: &mut State) -> Option<Type> {
        match expr {
            Expr::Path(path) if path.len() == 1 => {
                let name = &path[0];
                if let Some(binding) = state.bindings.get_mut(name) {
                    let ty = binding.ty.clone();
                    if self.is_resource(&ty) {
                        if !binding.live {
                            self.error(
                                ResourceFlowErrorKind::UseAfterMove,
                                Some(name.clone()),
                                format!("resource `{name}` was already consumed"),
                                self.function_span,
                            );
                        } else {
                            match mode {
                                UseMode::Move => binding.live = false,
                                UseMode::Borrow => {}
                                UseMode::Observe => self.error(
                                    ResourceFlowErrorKind::Copy,
                                    Some(name.clone()),
                                    format!(
                                        "resource `{name}` cannot be copied or observed by value"
                                    ),
                                    self.function_span,
                                ),
                            }
                        }
                    }
                    Some(ty)
                } else if let Some((enum_name, shape)) = self.variants.get(name) {
                    matches!(shape, VariantShape::Unit).then(|| Type::Named(enum_name.clone()))
                } else {
                    None
                }
            }
            Expr::Path(_) => None,
            Expr::Ref { expr, .. } => {
                let inner = self.use_expr(expr, UseMode::Borrow, state)?;
                Some(Type::Ref {
                    mutable: false,
                    inner: Box::new(inner),
                })
            }
            Expr::Call { callee, args } => {
                if let Expr::Path(path) = callee.as_ref() {
                    if let Some(name) = path.last() {
                        if let Some(signature) = self.signatures.get(name).cloned() {
                            for (index, arg) in args.iter().enumerate() {
                                let arg_mode = signature
                                    .params
                                    .get(index)
                                    .map(|ty| {
                                        if matches!(ty, Type::Ref { .. }) {
                                            UseMode::Borrow
                                        } else {
                                            UseMode::Move
                                        }
                                    })
                                    .unwrap_or(UseMode::Move);
                                self.use_expr(arg, arg_mode, state);
                            }
                            return Some(signature.ret);
                        }
                        if let Some((enum_name, shape)) = self.variants.get(name).cloned() {
                            let payloads = match shape {
                                VariantShape::Tuple(types) => types,
                                VariantShape::Unit => Vec::new(),
                                VariantShape::Struct(_) => Vec::new(),
                            };
                            for (arg, _) in args.iter().zip(payloads) {
                                self.use_expr(arg, UseMode::Move, state);
                            }
                            return Some(Type::Named(enum_name));
                        }
                    }
                }
                for arg in args {
                    self.use_expr(arg, UseMode::Move, state);
                }
                None
            }
            Expr::StructLit { path, fields } => {
                let name = path.last()?.clone();
                for (_, value) in fields {
                    self.use_expr(value, UseMode::Move, state);
                }
                let ty = self
                    .variants
                    .get(&name)
                    .map(|(owner, _)| Type::Named(owner.clone()))
                    .unwrap_or(Type::Named(name));
                Some(ty)
            }
            Expr::Tuple(elements) => Some(Type::Tuple(
                elements
                    .iter()
                    .map(|element| {
                        self.use_expr(element, UseMode::Move, state)
                            .unwrap_or(Type::Unit)
                    })
                    .collect(),
            )),
            Expr::Deref(inner) => match self.use_expr(inner, mode, state) {
                Some(Type::Box(inner)) => Some(*inner),
                other => other,
            },
            Expr::Cast { expr, ty } => {
                self.use_expr(expr, UseMode::Observe, state);
                Some(ty.clone())
            }
            Expr::If { cond, then, else_ } => {
                self.use_expr(cond, UseMode::Observe, state);
                let left = self.eval_value_block(then, state.clone(), mode);
                let right = self.eval_value_block(else_, state.clone(), mode);
                match (left, right) {
                    (Some((left_state, left_ty)), Some((right_state, right_ty))) => {
                        if left_state.live_resources() != right_state.live_resources() {
                            self.error(
                                ResourceFlowErrorKind::BranchMismatch,
                                None,
                                "if-expression arms disagree on live resources".to_string(),
                                self.function_span,
                            );
                        }
                        *state = left_state;
                        left_ty.or(right_ty)
                    }
                    _ => None,
                }
            }
            Expr::Match { scrutinee, arms } => {
                let scrutinee_ty = self.use_expr(scrutinee, UseMode::Move, state)?;
                let mut outcomes = Vec::new();
                for arm in arms {
                    let mut arm_state = state.clone();
                    let bound = self.bind_pattern(&arm.pattern, &scrutinee_ty, &mut arm_state);
                    let carried: BTreeSet<RegionPath> = bound
                        .iter()
                        .filter_map(|name| arm_state.bindings.get(name))
                        .flat_map(|binding| self.resources.provenance_of_type(&binding.ty))
                        .collect();
                    let provenance = self.pattern_provenance(&arm.pattern, &scrutinee_ty);
                    if !matches!(arm.pattern, Pattern::Binding(_))
                        && !provenance.is_subset(&carried)
                    {
                        self.error(
                            ResourceFlowErrorKind::ImplicitDrop,
                            expr_place(scrutinee),
                            format!(
                                "match pattern drops resource provenance {:?}",
                                provenance.difference(&carried).collect::<Vec<_>>()
                            ),
                            self.function_span,
                        );
                    }
                    if let Some(guard) = &arm.guard {
                        self.use_expr(guard, UseMode::Observe, &mut arm_state);
                    }
                    let ty = self.use_expr(&arm.body, mode, &mut arm_state);
                    for name in bound {
                        if arm_state
                            .bindings
                            .get(&name)
                            .is_some_and(|binding| binding.live)
                        {
                            self.error(
                                ResourceFlowErrorKind::Unconsumed,
                                Some(name.clone()),
                                format!("match binding `{name}` remains live at arm exit"),
                                self.function_span,
                            );
                        }
                        arm_state.bindings.remove(&name);
                    }
                    outcomes.push((arm_state, ty));
                }
                let (first_state, first_ty) = outcomes.first().cloned()?;
                let first_live = first_state.live_resources();
                if outcomes
                    .iter()
                    .any(|(arm_state, _)| arm_state.live_resources() != first_live)
                {
                    self.error(
                        ResourceFlowErrorKind::BranchMismatch,
                        None,
                        "match arms disagree on live resources".to_string(),
                        self.function_span,
                    );
                }
                *state = first_state;
                Some(first_ty.unwrap_or(Type::Unit))
            }
            Expr::Field { receiver, .. } | Expr::TupleProj { receiver, .. }
                if self
                    .infer_expr_type(receiver, state)
                    .as_ref()
                    .is_some_and(|ty| self.is_resource(ty)) =>
            {
                self.error(
                    ResourceFlowErrorKind::UnsupportedProjection,
                    expr_place(receiver),
                    "moving a resource component requires checked destructuring".to_string(),
                    self.function_span,
                );
                None
            }
            _ => {
                self.walk_observed(expr, state);
                None
            }
        }
    }

    fn eval_value_block(
        &mut self,
        block: &Block,
        initial: State,
        mode: UseMode,
    ) -> Option<(State, Option<Type>)> {
        let outer: BTreeSet<String> = initial.bindings.keys().cloned().collect();
        let mut edges = self.check_block(block, initial);
        let next_edges: Vec<_> = edges
            .drain(..)
            .filter(|edge| edge.kind == EdgeKind::Next)
            .collect();
        if next_edges.len() != 1 {
            return None;
        }
        let mut state = next_edges[0].state.clone();
        let ty = block
            .tail
            .as_ref()
            .and_then(|tail| self.use_expr(tail, mode, &mut state));
        let locals: Vec<String> = state
            .bindings
            .keys()
            .filter(|name| !outer.contains(*name))
            .cloned()
            .collect();
        for local in locals {
            if state
                .bindings
                .get(&local)
                .is_some_and(|binding| binding.live)
            {
                self.error(
                    ResourceFlowErrorKind::Unconsumed,
                    Some(local.clone()),
                    format!("resource local `{local}` remains live at value-block exit"),
                    self.function_span,
                );
            }
            state.bindings.remove(&local);
        }
        Some((state, ty))
    }

    fn bind_pattern(&mut self, pattern: &Pattern, ty: &Type, state: &mut State) -> Vec<String> {
        let mut bound = Vec::new();
        match pattern {
            Pattern::Binding(name) => {
                state.bindings.insert(
                    name.clone(),
                    Binding {
                        ty: ty.clone(),
                        live: self.is_resource(ty),
                    },
                );
                bound.push(name.clone());
            }
            Pattern::Wildcard if self.is_resource(ty) => self.error(
                ResourceFlowErrorKind::ImplicitDrop,
                None,
                "wildcard pattern discards a resource-bearing value".to_string(),
                self.function_span,
            ),
            Pattern::Enum { path, fields } => {
                if let Some(name) = path.last() {
                    if let Some((_, VariantShape::Tuple(types))) = self.variants.get(name).cloned()
                    {
                        for (pattern, ty) in fields.iter().zip(types) {
                            bound.extend(self.bind_pattern(pattern, &ty, state));
                        }
                    }
                }
            }
            Pattern::Struct { path, fields, rest } => {
                let name = path.last().cloned().unwrap_or_default();
                let definitions = self
                    .structs
                    .get(&name)
                    .cloned()
                    .or_else(|| match self.variants.get(&name) {
                        Some((_, VariantShape::Struct(fields))) => Some(
                            fields
                                .iter()
                                .map(|field| (field.name.clone(), field.ty.clone()))
                                .collect(),
                        ),
                        _ => None,
                    })
                    .unwrap_or_default();
                for (field_name, pattern) in fields {
                    if let Some((_, ty)) = definitions.iter().find(|(name, _)| name == field_name) {
                        bound.extend(self.bind_pattern(pattern, ty, state));
                    }
                }
                if *rest
                    && definitions.iter().any(|(name, ty)| {
                        !fields.iter().any(|(field, _)| field == name) && self.is_resource(ty)
                    })
                {
                    self.error(
                        ResourceFlowErrorKind::ImplicitDrop,
                        None,
                        "struct pattern `..` discards a resource-bearing field".to_string(),
                        self.function_span,
                    );
                }
            }
            Pattern::Or(alternatives) => {
                if let Some(first) = alternatives.first() {
                    bound.extend(self.bind_pattern(first, ty, state));
                }
            }
            Pattern::Literal(_) | Pattern::Slice(_) | Pattern::Wildcard => {}
        }
        bound
    }

    fn pattern_provenance(&self, pattern: &Pattern, scrutinee_ty: &Type) -> BTreeSet<RegionPath> {
        match pattern {
            Pattern::Binding(_) => self.resources.provenance_of_type(scrutinee_ty),
            Pattern::Enum { path, .. } => path
                .last()
                .and_then(|name| self.variants.get(name))
                .map(|(owner, shape)| {
                    let mut regions = self
                        .resources
                        .direct_declared(owner)
                        .cloned()
                        .unwrap_or_default();
                    match shape {
                        VariantShape::Unit => {}
                        VariantShape::Tuple(types) => {
                            for ty in types {
                                regions.extend(self.resources.provenance_of_type(ty));
                            }
                        }
                        VariantShape::Struct(fields) => {
                            for field in fields {
                                regions.extend(self.resources.provenance_of_type(&field.ty));
                            }
                        }
                    }
                    regions
                })
                .unwrap_or_else(|| self.resources.provenance_of_type(scrutinee_ty)),
            Pattern::Struct { path, .. } => path
                .last()
                .map(|name| {
                    if let Some((owner, shape)) = self.variants.get(name) {
                        let mut regions = self
                            .resources
                            .direct_declared(owner)
                            .cloned()
                            .unwrap_or_default();
                        if let VariantShape::Struct(fields) = shape {
                            for field in fields {
                                regions.extend(self.resources.provenance_of_type(&field.ty));
                            }
                        }
                        regions
                    } else {
                        let mut regions = self
                            .resources
                            .direct_declared(name)
                            .cloned()
                            .unwrap_or_default();
                        if let Some(fields) = self.structs.get(name) {
                            for (_, ty) in fields {
                                regions.extend(self.resources.provenance_of_type(ty));
                            }
                        }
                        regions
                    }
                })
                .unwrap_or_else(|| self.resources.provenance_of_type(scrutinee_ty)),
            Pattern::Or(alternatives) => alternatives
                .iter()
                .flat_map(|pattern| self.pattern_provenance(pattern, scrutinee_ty))
                .collect(),
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::Slice(_) => {
                self.resources.provenance_of_type(scrutinee_ty)
            }
        }
    }

    fn walk_observed(&mut self, expr: &Expr, state: &mut State) {
        match expr {
            Expr::Binary { lhs, rhs, .. } => {
                self.use_expr(lhs, UseMode::Observe, state);
                self.use_expr(rhs, UseMode::Observe, state);
            }
            Expr::Unary { expr, .. }
            | Expr::Field { receiver: expr, .. }
            | Expr::TupleProj { receiver: expr, .. }
            | Expr::Is {
                scrutinee: expr, ..
            } => {
                self.use_expr(expr, UseMode::Observe, state);
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.use_expr(receiver, UseMode::Borrow, state);
                for arg in args {
                    self.use_expr(arg, UseMode::Move, state);
                }
            }
            Expr::Index { base, index } => {
                self.use_expr(base, UseMode::Borrow, state);
                for expr in index_exprs(index) {
                    self.use_expr(expr, UseMode::Observe, state);
                }
            }
            Expr::Closure { body, .. } => {
                self.use_expr(body, UseMode::Observe, state);
            }
            Expr::Quantifier { domain, body, .. } => {
                self.use_expr(domain, UseMode::Borrow, state);
                self.use_expr(body, UseMode::Observe, state);
            }
            Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::StrLit(_) => {}
            _ => {}
        }
    }

    fn infer_expr_type(&self, expr: &Expr, state: &State) -> Option<Type> {
        match expr {
            Expr::Path(path) if path.len() == 1 => state
                .bindings
                .get(&path[0])
                .map(|binding| binding.ty.clone()),
            _ => None,
        }
    }

    fn is_resource(&self, ty: &Type) -> bool {
        !self.resources.provenance_of_type(ty).is_empty()
    }

    fn require_empty(&mut self, state: &State, edge: &str) {
        let live = state.live_resources();
        let label = format!("return#{}:{edge}", self.return_index);
        self.return_index += 1;
        self.current_flow_mut()
            .returning_edges
            .push(ResourceReturningEdge {
                label,
                live: live.iter().cloned().collect(),
            });
        for place in live {
            self.error(
                ResourceFlowErrorKind::Unconsumed,
                Some(place.clone()),
                format!("resource `{place}` remains live at {edge}"),
                self.function_span,
            );
        }
    }

    fn error(
        &mut self,
        kind: ResourceFlowErrorKind,
        place: Option<String>,
        detail: String,
        span: Span,
    ) {
        self.errors.push(ResourceFlowError {
            kind,
            place,
            detail,
            span,
        });
    }

    fn current_flow_mut(&mut self) -> &mut ResourceFunctionFlow {
        self.functions
            .get_mut(&self.function)
            .expect("function flow is initialized before body traversal")
    }
}

fn next(state: State) -> Vec<Edge> {
    vec![Edge {
        kind: EdgeKind::Next,
        state,
    }]
}

fn expr_place(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(path) if path.len() == 1 => Some(path[0].clone()),
        _ => None,
    }
}

fn index_exprs(index: &thermite_syntax::IndexArg) -> Vec<&Expr> {
    use thermite_syntax::IndexArg;
    match index {
        IndexArg::Single(expr) | IndexArg::RangeTo(expr) | IndexArg::RangeFrom(expr) => vec![expr],
        IndexArg::Range(start, end) => vec![start, end],
    }
}
