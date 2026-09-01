//! RFC-10 checked-program construction boundary.
//!
//! Parsed-program APIs remain source-compatible, but semantic consumers build
//! this value first. Construction is all-or-nothing: canonical inventory,
//! region/lock resolution, and effect/holding analysis must all succeed.

use thermite_syntax::{
    is_lexically_shadowed, semantic_inventory, walk_semantic, NodeId, Program, RegionPath,
    SemanticEvent, SemanticFact, SemanticInventory, WorkBudget,
};

use crate::effects::{analyze_effects_unchecked, EffectAnalysis};
use crate::LowerError;

pub const DEFAULT_SEMANTIC_WORK_BUDGET: usize = 1_000_000;

#[derive(Debug, Clone)]
pub struct CheckedProgram {
    source: Program,
    inventory: SemanticInventory,
    regions: thermite_spec::RegionIndex,
    effects: EffectAnalysis,
    resource_flow: thermite_spec::ResourceFlowReport,
    holdings: Vec<CheckedHolding>,
    shared_places: Vec<CheckedSharedPlace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedHolding {
    pub node: NodeId,
    pub lock: String,
    pub guarded_region: RegionPath,
    pub capability: String,
    pub incoming_held: Vec<String>,
    pub outgoing_held: Vec<String>,
    pub close_edges: Vec<CheckedCloseEdge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    Fallthrough,
    Return,
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedCloseEdge {
    pub at: NodeId,
    pub reason: CloseReason,
    pub inner_to_outer: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    Read,
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedSharedPlace {
    pub node: NodeId,
    pub path: RegionPath,
    pub mode: AccessMode,
    pub authorizing_locks: Vec<String>,
}

impl CheckedProgram {
    pub fn build(source: &Program) -> Result<Self, Vec<LowerError>> {
        Self::build_with_budget(source, WorkBudget(DEFAULT_SEMANTIC_WORK_BUDGET))
    }

    pub fn build_with_budget(
        source: &Program,
        budget: WorkBudget,
    ) -> Result<Self, Vec<LowerError>> {
        let resources = thermite_spec::ResourceEnv::build(source).map_err(|errors| {
            errors
                .into_iter()
                .map(|error| LowerError::EffectAnalysis {
                    detail: format!("resource provenance failed: {error:?}"),
                    span: thermite_syntax::Span::new(0, 0),
                })
                .collect::<Vec<_>>()
        })?;
        let resource_flow =
            thermite_spec::check_resource_flow(source, &resources).map_err(|errors| {
                errors
                    .into_iter()
                    .map(|error| LowerError::EffectAnalysis {
                        detail: error.detail,
                        span: error.span,
                    })
                    .collect::<Vec<_>>()
            })?;
        let inventory = semantic_inventory(source, budget).map_err(|limit| {
            vec![LowerError::ResourceLimit {
                budget: limit.budget.0,
                required_at_least: limit.required_at_least,
            }]
        })?;
        let regions = thermite_spec::RegionIndex::build(source).map_err(|errors| {
            errors
                .into_iter()
                .map(|error| LowerError::EffectAnalysis {
                    detail: format!("region resolution failed: {error:?}"),
                    span: thermite_syntax::Span::new(0, 0),
                })
                .collect::<Vec<_>>()
        })?;
        let effects = analyze_effects_unchecked(source)?;
        let holdings = build_checked_holdings(&inventory, &regions).map_err(|error| vec![error])?;
        let shared_places =
            build_checked_shared_places(&inventory, &regions).map_err(|error| vec![error])?;
        Ok(Self {
            source: source.clone(),
            inventory,
            regions,
            effects,
            resource_flow,
            holdings,
            shared_places,
        })
    }

    pub fn source(&self) -> &Program {
        &self.source
    }

    pub fn inventory(&self) -> &SemanticInventory {
        &self.inventory
    }

    pub fn regions(&self) -> &thermite_spec::RegionIndex {
        &self.regions
    }

    pub fn effects(&self) -> &EffectAnalysis {
        &self.effects
    }

    pub fn resource_flow(&self) -> &thermite_spec::ResourceFlowReport {
        &self.resource_flow
    }

    pub fn holdings(&self) -> &[CheckedHolding] {
        &self.holdings
    }

    pub fn shared_places(&self) -> &[CheckedSharedPlace] {
        &self.shared_places
    }
}

pub(crate) fn first_rfc11_span(program: &Program) -> Option<thermite_syntax::Span> {
    program.items.iter().find_map(|item| match item {
        thermite_syntax::Item::Struct(item) if item.resource.is_some() => Some(item.span),
        thermite_syntax::Item::Enum(item) if item.resource.is_some() => Some(item.span),
        thermite_syntax::Item::Fn(item) => {
            let effect_span = matches!(
                &item.contract.effects,
                thermite_syntax::EffectRow::Set(effects)
                    if effects.iter().any(|effect| matches!(effect, thermite_syntax::Effect::Forgets(_)))
            )
            .then_some(item.span);
            effect_span.or_else(|| item.body.as_ref().and_then(first_forget_in_block))
        }
        _ => None,
    })
}

fn first_forget_in_block(block: &thermite_syntax::Block) -> Option<thermite_syntax::Span> {
    block.stmts.iter().find_map(|stmt| match stmt {
        thermite_syntax::Stmt::Forget { span, .. } => Some(*span),
        thermite_syntax::Stmt::If { then, else_, .. } => {
            first_forget_in_block(then).or_else(|| else_.as_ref().and_then(first_forget_in_block))
        }
        thermite_syntax::Stmt::Loop(loop_) => first_forget_in_block(&loop_.body),
        thermite_syntax::Stmt::Holding { body, .. } => first_forget_in_block(body),
        _ => None,
    })
}

#[derive(Clone, Copy)]
struct HeldScope {
    record: usize,
    loop_depth: usize,
}

fn build_checked_holdings(
    inventory: &SemanticInventory,
    regions: &thermite_spec::RegionIndex,
) -> Result<Vec<CheckedHolding>, LowerError> {
    let events = walk_semantic(
        inventory,
        WorkBudget(inventory.kinds.len().saturating_mul(2)),
    )
    .map_err(|limit| LowerError::ResourceLimit {
        budget: limit.budget.0,
        required_at_least: limit.required_at_least,
    })?;
    let mut records: Vec<CheckedHolding> = Vec::new();
    let mut held: Vec<HeldScope> = Vec::new();
    let mut loop_depth = 0usize;
    for event in events {
        let (id, entering) = match event {
            SemanticEvent::Enter { id, .. } => (id, true),
            SemanticEvent::Leave { id, .. } => (id, false),
        };
        let fact = &inventory.facts[id.0 as usize];
        if entering {
            match fact {
                SemanticFact::Loop => loop_depth += 1,
                SemanticFact::Holding { lock } => {
                    let guarded_region =
                        regions.guarded_region(lock).cloned().ok_or_else(|| {
                            LowerError::EffectAnalysis {
                                detail: format!("holding refers to unresolved lock `{lock}`"),
                                span: thermite_syntax::Span::new(0, 0),
                            }
                        })?;
                    let incoming_held = held
                        .iter()
                        .map(|scope| records[scope.record].lock.clone())
                        .collect::<Vec<_>>();
                    let record = records.len();
                    records.push(CheckedHolding {
                        node: id,
                        lock: lock.clone(),
                        guarded_region,
                        capability: format!("capability@{}", id.0),
                        incoming_held: incoming_held.clone(),
                        outgoing_held: incoming_held,
                        close_edges: Vec::new(),
                    });
                    held.push(HeldScope { record, loop_depth });
                }
                SemanticFact::Return => {
                    add_control_closes(&mut records, &held, id, CloseReason::Return, |_| true)
                }
                SemanticFact::Break => {
                    add_control_closes(&mut records, &held, id, CloseReason::Break, |scope| {
                        scope.loop_depth == loop_depth
                    })
                }
                SemanticFact::Continue => {
                    add_control_closes(&mut records, &held, id, CloseReason::Continue, |scope| {
                        scope.loop_depth == loop_depth
                    })
                }
                _ => {}
            }
        } else {
            match fact {
                SemanticFact::Holding { .. } => {
                    if let Some(scope) = held.pop() {
                        let lock = records[scope.record].lock.clone();
                        records[scope.record].close_edges.push(CheckedCloseEdge {
                            at: id,
                            reason: CloseReason::Fallthrough,
                            inner_to_outer: vec![lock],
                        });
                    }
                }
                SemanticFact::Loop => loop_depth = loop_depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    Ok(records)
}

fn add_control_closes(
    records: &mut [CheckedHolding],
    held: &[HeldScope],
    at: NodeId,
    reason: CloseReason,
    applies: impl Fn(&HeldScope) -> bool,
) {
    let affected = held.iter().copied().filter(applies).collect::<Vec<_>>();
    let sequence = affected
        .iter()
        .rev()
        .map(|scope| records[scope.record].lock.clone())
        .collect::<Vec<_>>();
    for scope in affected {
        records[scope.record].close_edges.push(CheckedCloseEdge {
            at,
            reason,
            inner_to_outer: sequence.clone(),
        });
    }
}

fn build_checked_shared_places(
    inventory: &SemanticInventory,
    regions: &thermite_spec::RegionIndex,
) -> Result<Vec<CheckedSharedPlace>, LowerError> {
    let events = walk_semantic(
        inventory,
        WorkBudget(inventory.kinds.len().saturating_mul(2)),
    )
    .map_err(|limit| LowerError::ResourceLimit {
        budget: limit.budget.0,
        required_at_least: limit.required_at_least,
    })?;
    let mut parents = vec![None; inventory.kinds.len()];
    for edge in &inventory.edges {
        parents[edge.child.0 as usize] = Some((edge.parent, edge.role));
    }
    let mut held = Vec::<String>::new();
    let mut clause_depth = 0usize;
    let mut places = Vec::new();
    for event in events {
        let (id, entering) = match event {
            SemanticEvent::Enter { id, .. } => (id, true),
            SemanticEvent::Leave { id, .. } => (id, false),
        };
        let fact = &inventory.facts[id.0 as usize];
        if entering {
            match fact {
                SemanticFact::Clause => clause_depth += 1,
                SemanticFact::Holding { lock } => held.push(lock.clone()),
                SemanticFact::Place(path) if clause_depth == 0 && regions.resolve(path).is_ok() => {
                    if path
                        .segments
                        .first()
                        .is_some_and(|root| is_lexically_shadowed(inventory, id, root))
                    {
                        continue;
                    }
                    let nested_in_place = parents[id.0 as usize].is_some_and(|(parent, _)| {
                        matches!(inventory.facts[parent.0 as usize], SemanticFact::Place(_))
                    });
                    if nested_in_place {
                        continue;
                    }
                    let mode = match parents[id.0 as usize] {
                        Some((_, thermite_syntax::ChildRole::Target)) => AccessMode::Write,
                        _ => AccessMode::Read,
                    };
                    let authorizing_locks = regions
                        .locks()
                        .filter(|(_, guarded)| regions.overlaps(guarded, path))
                        .filter(|(lock, _)| held.iter().any(|item| item == lock))
                        .map(|(lock, _)| lock.to_string())
                        .collect();
                    places.push(CheckedSharedPlace {
                        node: id,
                        path: path.clone(),
                        mode,
                        authorizing_locks,
                    });
                }
                _ => {}
            }
        } else {
            match fact {
                SemanticFact::Clause => clause_depth = clause_depth.saturating_sub(1),
                SemanticFact::Holding { .. } => {
                    held.pop();
                }
                _ => {}
            }
        }
    }
    Ok(places)
}

pub fn check_program(source: &Program) -> Result<CheckedProgram, Vec<LowerError>> {
    CheckedProgram::build(source)
}

pub(crate) fn require_checked(source: &Program) -> Result<CheckedProgram, LowerError> {
    CheckedProgram::build(source).map_err(|errors| {
        errors
            .into_iter()
            .next()
            .unwrap_or_else(|| LowerError::Unsupported {
                what: "checked-program construction failed without a diagnostic".to_string(),
                span: thermite_syntax::Span::new(0, 0),
            })
    })
}
