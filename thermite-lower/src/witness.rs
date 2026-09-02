//! Deterministic RFC-10 checked-traversal witness production and Rust replay.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thermite_syntax::ast::Effect;
use thermite_syntax::{
    is_lexically_shadowed, semantic_inventory, walk_semantic, ChildRole, Item, NodeId, Program,
    SemanticEvent, SemanticFact, SemanticInventory, WorkBudget,
};

use crate::{CheckedProgram, LowerError};

pub const WITNESS_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalAstProjection {
    pub digest: String,
    pub node_kinds: Vec<String>,
    pub node_facts: Vec<String>,
    pub edges: Vec<WitnessEdge>,
    pub lock_decls: Vec<CanonicalLockDecl>,
    pub events: Vec<CanonicalSemanticEvent>,
    pub direct_footprints: BTreeMap<String, Vec<String>>,
    pub calls: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalLockDecl {
    pub name: String,
    pub guarded_region: String,
    pub guarded_segments: Vec<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSemanticEvent {
    pub node: u32,
    pub entering: bool,
    pub kind: String,
    pub value: String,
    pub segments: Vec<String>,
    pub mode: String,
    pub eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraversalWitness {
    pub version: u32,
    pub canonical_ast_sha256: String,
    pub node_kinds: Vec<String>,
    pub node_facts: Vec<String>,
    pub edges: Vec<WitnessEdge>,
    pub direct_footprints: BTreeMap<String, Vec<String>>,
    pub calls: BTreeMap<String, Vec<String>>,
    pub footprints: BTreeMap<String, Vec<String>>,
    pub holdings: Vec<WitnessHolding>,
    pub shared_places: Vec<WitnessSharedPlace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessEdge {
    pub parent: u32,
    pub child: u32,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessHolding {
    pub node: u32,
    pub lock: String,
    pub guarded_region: String,
    pub capability: String,
    pub incoming_held: Vec<String>,
    pub outgoing_held: Vec<String>,
    pub close_edges: Vec<WitnessCloseEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessCloseEdge {
    pub at: u32,
    pub reason: String,
    pub inner_to_outer: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessSharedPlace {
    pub node: u32,
    pub path: String,
    pub mode: String,
    pub authorizing_locks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedObservation {
    pub path: String,
    pub ty: thermite_syntax::Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitnessError {
    Construction(Vec<LowerError>),
    Mismatch { field: &'static str },
    Json(String),
}

impl TraversalWitness {
    pub fn canonical_json(&self) -> Result<String, WitnessError> {
        serde_json::to_string(self).map_err(|error| WitnessError::Json(error.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, WitnessError> {
        serde_json::from_str(json).map_err(|error| WitnessError::Json(error.to_string()))
    }
}

/// Exact semantic-node budget required by the bounded Rust producer on the
/// currently claimed fragment. Computing it uses the repository ceiling; an
/// input beyond that ceiling remains an explicit resource-limit outcome.
pub fn required_witness_budget(source: &Program) -> Result<WorkBudget, WitnessError> {
    semantic_inventory(source, WorkBudget(crate::DEFAULT_SEMANTIC_WORK_BUDGET))
        .map(|inventory| WorkBudget(inventory.kinds.len()))
        .map_err(|limit| {
            WitnessError::Construction(vec![LowerError::ResourceLimit {
                budget: limit.budget.0,
                required_at_least: limit.required_at_least,
            }])
        })
}

/// Bounded producer with its resource premise in the public type signature.
pub fn emit_witness_with_budget(
    source: &Program,
    budget: WorkBudget,
) -> Result<TraversalWitness, WitnessError> {
    CheckedProgram::build_with_budget(source, budget)
        .map(|checked| emit_witness(&checked))
        .map_err(WitnessError::Construction)
}

pub fn emit_witness(checked: &CheckedProgram) -> TraversalWitness {
    let inventory = checked.inventory();
    let footprints = checked
        .effects()
        .footprints
        .iter()
        .map(|(function, effects)| {
            (
                function.clone(),
                effects.iter().map(|effect| format!("{effect:?}")).collect(),
            )
        })
        .collect();
    let direct_footprints = checked
        .effects()
        .direct_footprints
        .iter()
        .map(|(function, effects)| {
            (
                function.clone(),
                effects.iter().map(|effect| format!("{effect:?}")).collect(),
            )
        })
        .collect();
    let calls = checked
        .effects()
        .calls
        .iter()
        .map(|(function, callees)| (function.clone(), callees.iter().cloned().collect()))
        .collect();
    TraversalWitness {
        version: WITNESS_VERSION,
        canonical_ast_sha256: canonical_ast_sha256(checked.source()),
        node_kinds: inventory
            .kinds
            .iter()
            .map(|kind| format!("{kind:?}"))
            .collect(),
        node_facts: inventory
            .facts
            .iter()
            .map(|fact| format!("{fact:?}"))
            .collect(),
        edges: inventory
            .edges
            .iter()
            .map(|edge| WitnessEdge {
                parent: edge.parent.0,
                child: edge.child.0,
                role: format!("{:?}", edge.role),
            })
            .collect(),
        direct_footprints,
        calls,
        footprints,
        holdings: checked
            .holdings()
            .iter()
            .map(|holding| WitnessHolding {
                node: holding.node.0,
                lock: holding.lock.clone(),
                guarded_region: holding.guarded_region.to_string(),
                capability: holding.capability.clone(),
                incoming_held: holding.incoming_held.clone(),
                outgoing_held: holding.outgoing_held.clone(),
                close_edges: holding
                    .close_edges
                    .iter()
                    .map(|edge| WitnessCloseEdge {
                        at: edge.at.0,
                        reason: format!("{:?}", edge.reason),
                        inner_to_outer: edge.inner_to_outer.clone(),
                    })
                    .collect(),
            })
            .collect(),
        shared_places: checked
            .shared_places()
            .iter()
            .map(|place| WitnessSharedPlace {
                node: place.node.0,
                path: place.path.to_string(),
                mode: format!("{:?}", place.mode),
                authorizing_locks: place.authorizing_locks.clone(),
            })
            .collect(),
    }
}

fn canonical_shared_places(
    inventory: &SemanticInventory,
    regions: &thermite_spec::RegionIndex,
) -> Result<Vec<WitnessSharedPlace>, WitnessError> {
    let events =
        walk_semantic(inventory, WorkBudget(inventory.kinds.len() * 2)).map_err(|limit| {
            WitnessError::Construction(vec![LowerError::ResourceLimit {
                budget: limit.budget.0,
                required_at_least: limit.required_at_least,
            }])
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
                    let nested = parents[id.0 as usize].is_some_and(|(parent, _)| {
                        matches!(inventory.facts[parent.0 as usize], SemanticFact::Place(_))
                    });
                    if nested {
                        continue;
                    }
                    let mode = if parents[id.0 as usize]
                        .is_some_and(|(_, role)| role == ChildRole::Target)
                    {
                        "Write"
                    } else {
                        "Read"
                    };
                    let authorizing_locks = regions
                        .locks()
                        .filter(|(_, guarded)| regions.overlaps(guarded, path))
                        .filter(|(lock, _)| held.iter().any(|item| item == lock))
                        .map(|(lock, _)| lock.to_string())
                        .collect();
                    places.push(WitnessSharedPlace {
                        node: id.0,
                        path: path.to_string(),
                        mode: mode.to_string(),
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

pub fn canonical_ast_projection(source: &Program) -> Result<CanonicalAstProjection, WitnessError> {
    let inventory = semantic_inventory(source, WorkBudget(crate::DEFAULT_SEMANTIC_WORK_BUDGET))
        .map_err(|limit| {
            WitnessError::Construction(vec![LowerError::ResourceLimit {
                budget: limit.budget.0,
                required_at_least: limit.required_at_least,
            }])
        })?;
    let mut parents = vec![None; inventory.facts.len()];
    for edge in &inventory.edges {
        parents[edge.child.0 as usize] = Some((edge.parent.0 as usize, edge.role));
    }
    let functions = inventory
        .facts
        .iter()
        .filter_map(|fact| match fact {
            SemanticFact::Function { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut direct = functions
        .iter()
        .map(|name| (name.clone(), std::collections::BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let mut calls = functions
        .iter()
        .map(|name| (name.clone(), std::collections::BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let function_of = |mut node: usize| -> Option<String> {
        loop {
            if let SemanticFact::Function { name, .. } = &inventory.facts[node] {
                return Some(name.clone());
            }
            node = parents[node]?.0;
        }
    };
    let regions = thermite_spec::RegionIndex::build(source).map_err(|errors| {
        WitnessError::Construction(
            errors
                .into_iter()
                .map(|error| LowerError::EffectAnalysis {
                    detail: format!("canonical projection region resolution failed: {error:?}"),
                    span: thermite_syntax::Span::new(0, 0),
                })
                .collect(),
        )
    })?;
    let mut lock_decls = source
        .items
        .iter()
        .filter_map(|item| match item {
            Item::LockDecl(lock) => Some(CanonicalLockDecl {
                name: lock.name.clone(),
                guarded_region: lock.guards.to_string(),
                guarded_segments: lock.guards.segments.clone(),
                after: lock.after.clone(),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    lock_decls.sort_by(|left, right| left.name.cmp(&right.name));
    let semantic_events = walk_semantic(
        &inventory,
        WorkBudget(inventory.kinds.len().saturating_mul(2)),
    )
    .map_err(|limit| {
        WitnessError::Construction(vec![LowerError::ResourceLimit {
            budget: limit.budget.0,
            required_at_least: limit.required_at_least,
        }])
    })?;
    let events = semantic_events
        .into_iter()
        .map(|event| {
            let (id, entering) = match event {
                SemanticEvent::Enter { id, .. } => (id, true),
                SemanticEvent::Leave { id, .. } => (id, false),
            };
            let mut kind = "Other".to_string();
            let mut value = String::new();
            let mut segments = Vec::new();
            let mut mode = String::new();
            let mut eligible = false;
            match &inventory.facts[id.0 as usize] {
                SemanticFact::Holding { lock } => {
                    kind = "Holding".to_string();
                    value = lock.clone();
                }
                SemanticFact::Loop => kind = "Loop".to_string(),
                SemanticFact::Return => kind = "Return".to_string(),
                SemanticFact::Break => kind = "Break".to_string(),
                SemanticFact::Continue => kind = "Continue".to_string(),
                SemanticFact::Place(path) => {
                    kind = "Place".to_string();
                    value = path.to_string();
                    segments = path.segments.clone();
                    mode = if parents[id.0 as usize]
                        .is_some_and(|(_, role)| role == ChildRole::Target)
                    {
                        "Write".to_string()
                    } else {
                        "Read".to_string()
                    };
                    let shadowed = path
                        .segments
                        .first()
                        .is_some_and(|root| is_lexically_shadowed(&inventory, id, root));
                    let mut ancestor = parents[id.0 as usize];
                    let mut nested_place = false;
                    let mut in_clause = false;
                    while let Some((parent, _)) = ancestor {
                        nested_place |= matches!(inventory.facts[parent], SemanticFact::Place(_));
                        in_clause |= matches!(inventory.facts[parent], SemanticFact::Clause);
                        ancestor = parents[parent];
                    }
                    eligible =
                        !shadowed && !nested_place && !in_clause && regions.resolve(path).is_ok();
                }
                _ => {}
            }
            CanonicalSemanticEvent {
                node: id.0,
                entering,
                kind,
                value,
                segments,
                mode,
                eligible,
            }
        })
        // `deriveStep` is definitionally a no-op for `Other` events and for
        // ineligible places.  Keep the canonical replay stream semantically
        // complete without forcing Lean to normalize two inert records for
        // every node in large programs.
        .filter(|event| event.kind != "Other" && (event.kind != "Place" || event.eligible))
        .collect::<Vec<_>>();
    for (node, fact) in inventory.facts.iter().enumerate() {
        let Some(function) = function_of(node) else {
            continue;
        };
        match fact {
            SemanticFact::Holding { lock } => {
                direct
                    .get_mut(&function)
                    .unwrap()
                    .insert(Effect::Owns(lock.clone()));
            }
            SemanticFact::Call { path: Some(path) } => {
                if let Some(effect) = crate::effects::owned_constructor_effect(path) {
                    direct.get_mut(&function).unwrap().insert(effect);
                } else if let Some(callee) = path.last().filter(|name| functions.contains(*name)) {
                    calls.get_mut(&function).unwrap().insert(callee.clone());
                } else if let Some(effect) = crate::effects::call_path_effect(path) {
                    direct.get_mut(&function).unwrap().insert(effect);
                }
            }
            SemanticFact::MethodCall { method } => {
                if let Some(effect) = crate::effects::intrinsic_effect(method) {
                    direct.get_mut(&function).unwrap().insert(effect);
                } else if functions.contains(method) {
                    calls.get_mut(&function).unwrap().insert(method.clone());
                }
            }
            SemanticFact::StringLiteral => {
                direct.get_mut(&function).unwrap().insert(Effect::Alloc);
            }
            SemanticFact::Place(path)
                if path.segments.first().is_some_and(|root| {
                    !is_lexically_shadowed(&inventory, NodeId(node as u32), root)
                }) && regions.resolve(path).is_ok() =>
            {
                let mut ancestor = parents[node];
                let mut nested_place = false;
                let mut in_clause = false;
                while let Some((parent, _)) = ancestor {
                    nested_place |= matches!(inventory.facts[parent], SemanticFact::Place(_));
                    in_clause |= matches!(inventory.facts[parent], SemanticFact::Clause);
                    ancestor = parents[parent];
                }
                if !nested_place && !in_clause {
                    let write = parents[node].is_some_and(|(_, role)| role == ChildRole::Target);
                    direct.get_mut(&function).unwrap().insert(if write {
                        Effect::Write(path.clone())
                    } else {
                        Effect::Read(path.clone())
                    });
                }
            }
            _ => {}
        }
    }
    for item in &source.items {
        if let Item::Fn(function) = item {
            if function.body.is_none() {
                direct.insert(
                    function.name.clone(),
                    crate::effects::row_effects(&function.contract.effects),
                );
            }
        }
    }
    if let Ok(resources) = thermite_spec::ResourceEnv::build(source) {
        if let Ok(flow) = thermite_spec::check_resource_flow(source, &resources) {
            for (function, regions) in flow.direct_forgets {
                direct
                    .entry(function)
                    .or_default()
                    .extend(regions.into_iter().map(Effect::Forgets));
            }
        }
    }
    Ok(CanonicalAstProjection {
        digest: canonical_ast_sha256(source),
        node_kinds: inventory
            .kinds
            .iter()
            .map(|kind| format!("{kind:?}"))
            .collect(),
        node_facts: inventory
            .facts
            .iter()
            .map(|fact| format!("{fact:?}"))
            .collect(),
        edges: inventory
            .edges
            .iter()
            .map(|edge| WitnessEdge {
                parent: edge.parent.0,
                child: edge.child.0,
                role: format!("{:?}", edge.role),
            })
            .collect(),
        lock_decls,
        events,
        direct_footprints: direct
            .into_iter()
            .map(|(function, effects)| {
                (
                    function,
                    effects
                        .into_iter()
                        .map(|effect| format!("{effect:?}"))
                        .collect(),
                )
            })
            .collect(),
        calls: calls
            .into_iter()
            .map(|(function, callees)| (function, callees.into_iter().collect()))
            .collect(),
    })
}

/// Read-only scalar shared observations made by `function`, for the mutation
/// equivalence harness. Each observation becomes an explicit symbolic input so
/// both compared bodies see the same shared snapshot. Writes are rejected: a
/// value-only relation cannot soundly model their state transition.
pub fn equivalence_shared_observations(
    source: &Program,
    function: &str,
) -> Result<Vec<SharedObservation>, LowerError> {
    let inventory = semantic_inventory(source, WorkBudget(crate::DEFAULT_SEMANTIC_WORK_BUDGET))
        .map_err(|limit| LowerError::ResourceLimit {
            budget: limit.budget.0,
            required_at_least: limit.required_at_least,
        })?;
    let regions =
        thermite_spec::RegionIndex::build(source).map_err(|errors| LowerError::EffectAnalysis {
            detail: format!("equivalence shared-observation resolution failed: {errors:?}"),
            span: thermite_syntax::Span::new(0, 0),
        })?;
    let places = canonical_shared_places(&inventory, &regions).map_err(|error| match error {
        WitnessError::Construction(errors) => {
            errors
                .into_iter()
                .next()
                .unwrap_or_else(|| LowerError::EffectAnalysis {
                    detail:
                        "equivalence shared-observation construction failed without a diagnostic"
                            .to_string(),
                    span: thermite_syntax::Span::new(0, 0),
                })
        }
        other => LowerError::EffectAnalysis {
            detail: format!("equivalence shared-observation projection failed: {other:?}"),
            span: thermite_syntax::Span::new(0, 0),
        },
    })?;
    let mut parents = vec![None; inventory.facts.len()];
    for edge in &inventory.edges {
        parents[edge.child.0 as usize] = Some(edge.parent.0 as usize);
    }
    let function_of = |mut node: usize| -> Option<&str> {
        loop {
            if let SemanticFact::Function { name, .. } = &inventory.facts[node] {
                return Some(name.as_str());
            }
            node = parents[node]?;
        }
    };
    let mut out = Vec::new();
    for place in places {
        if function_of(place.node as usize) != Some(function) {
            continue;
        }
        if place.mode != "Read" {
            return Err(LowerError::Unsupported {
                what: format!(
                    "equivalence obligation does not model shared write `{}`; its state transition is outside scalar value equivalence",
                    place.path
                ),
                span: thermite_syntax::Span::new(0, 0),
            });
        }
        let SemanticFact::Place(path) = &inventory.facts[place.node as usize] else {
            continue;
        };
        let ty = regions
            .resolve(path)
            .map_err(|error| LowerError::EffectAnalysis {
                detail: format!(
                    "equivalence shared observation `{path}` failed to resolve: {error:?}"
                ),
                span: thermite_syntax::Span::new(0, 0),
            })?;
        out.push(SharedObservation {
            path: place.path,
            ty,
        });
    }
    out.sort_by(|left, right| left.path.cmp(&right.path));
    out.dedup_by(|left, right| left.path == right.path);
    Ok(out)
}

pub fn lean_replay_source(ast: &CanonicalAstProjection, witness: &TraversalWitness) -> String {
    fn string(value: &str) -> String {
        serde_json::to_string(value).expect("serializing a string cannot fail")
    }
    fn strings(values: &[String]) -> String {
        format!(
            "[{}]",
            values
                .iter()
                .map(|v| string(v))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    fn edges(values: &[WitnessEdge]) -> String {
        format!(
            "[{}]",
            values
                .iter()
                .map(|e| format!("⟨{}, {}, {}⟩", e.parent, e.child, string(&e.role)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    fn lock_decls(values: &[CanonicalLockDecl]) -> String {
        values
            .iter()
            .map(|lock| {
                let after = lock
                    .after
                    .as_ref()
                    .map(|name| format!("some {}", string(name)))
                    .unwrap_or_else(|| "none".to_string());
                format!(
                    "⟨{}, {}, {}, {}⟩",
                    string(&lock.name),
                    string(&lock.guarded_region),
                    strings(&lock.guarded_segments),
                    after
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn events(values: &[CanonicalSemanticEvent]) -> String {
        values
            .iter()
            .map(|event| {
                format!(
                    "⟨{}, {}, {}, {}, {}, {}, {}⟩",
                    event.node,
                    event.entering,
                    string(&event.kind),
                    string(&event.value),
                    strings(&event.segments),
                    string(&event.mode),
                    event.eligible
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn holdings(values: &[WitnessHolding]) -> String {
        values
            .iter()
            .map(|h| {
                let capability_node = h
                    .capability
                    .strip_prefix("capability@")
                    .and_then(|node| node.parse::<u32>().ok())
                    .unwrap_or(u32::MAX);
                let closes = h
                    .close_edges
                    .iter()
                    .map(|e| {
                        format!(
                            "⟨{}, {}, {}⟩",
                            e.at,
                            string(&e.reason),
                            strings(&e.inner_to_outer)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "⟨{}, {}, {}, {}, {}, {}, [{}]⟩",
                    h.node,
                    string(&h.lock),
                    string(&h.guarded_region),
                    capability_node,
                    strings(&h.incoming_held),
                    strings(&h.outgoing_held),
                    closes
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn places(values: &[WitnessSharedPlace]) -> String {
        values
            .iter()
            .map(|p| {
                format!(
                    "⟨{}, {}, {}, {}⟩",
                    p.node,
                    string(&p.path),
                    string(&p.mode),
                    strings(&p.authorizing_locks)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    let footprints = witness
        .footprints
        .iter()
        .map(|(f, es)| format!("⟨{}, {}⟩", string(f), strings(es)))
        .collect::<Vec<_>>()
        .join(", ");
    let direct_footprints = witness
        .direct_footprints
        .iter()
        .map(|(f, es)| format!("⟨{}, {}⟩", string(f), strings(es)))
        .collect::<Vec<_>>()
        .join(", ");
    let calls = witness
        .calls
        .iter()
        .map(|(f, cs)| format!("⟨{}, {}⟩", string(f), strings(cs)))
        .collect::<Vec<_>>()
        .join(", ");
    let ast_direct_footprints = ast
        .direct_footprints
        .iter()
        .map(|(f, es)| format!("⟨{}, {}⟩", string(f), strings(es)))
        .collect::<Vec<_>>()
        .join(", ");
    let ast_calls = ast
        .calls
        .iter()
        .map(|(f, cs)| format!("⟨{}, {}⟩", string(f), strings(cs)))
        .collect::<Vec<_>>()
        .join(", ");
    let ast_lock_decls = lock_decls(&ast.lock_decls);
    let ast_events = events(&ast.events);
    let witness_holdings = holdings(&witness.holdings);
    let witness_places = places(&witness.shared_places);
    format!(
        "import Thermite.CheckedTraversal\nopen Thermite.CheckedTraversal\nset_option maxRecDepth 100000\n\ndef ast : CanonicalAst := {{ digest := {}, nodeKinds := {}, nodeFacts := {}, edges := {}, lockDecls := [{}], events := [{}], directFootprints := [{}], calls := [{}] }}\ndef witness : Witness := ⟨{}, {}, {}, {}, {}, [{}], [{}], [{}], [{}], [{}]⟩\ntheorem rfc10_artifact_refines : producerRefines ast witness = true := by simp [producerRefines, produce, derivedHoldings, derivedSharedPlaces, derivedAuthorityRequiredNodes, deriveSemantics, deriveStep, closeAffected, addClose, updateHolding, orderValid, lockBefore, lookupLock, lockOfScope, regionsOverlap, isPrefix, ast, witness, closure, closureFuel, closureStep, unionEffects, lookupEffects, listSetEq]\ntheorem rfc10_artifact_verified : verify ast witness = true := verify_complete (by simp [SupportedRFC10, footprintClosureSound, holdingCoverageSound, derivedHoldings, derivedSharedPlaces, derivedAuthorityRequiredNodes, deriveSemantics, deriveStep, closeAffected, addClose, updateHolding, orderValid, lockBefore, lookupLock, lockOfScope, regionsOverlap, isPrefix, ast, witness, edgeWellFormed, footprintWellFormed, callsWellFormed, footprintsClosed, closure, closureFuel, closureStep, unionEffects, lookupEffects, listSetEq, holdingWellFormed, sharedPlaceWellFormed])\n#print axioms rfc10_artifact_refines\n#print axioms rfc10_artifact_verified\n#eval IO.println \"THERMITE_RFC10_REPLAY_ACCEPTED_V3\"\n",
        string(&ast.digest), strings(&ast.node_kinds), strings(&ast.node_facts), edges(&ast.edges), ast_lock_decls, ast_events, ast_direct_footprints, ast_calls,
        witness.version, string(&witness.canonical_ast_sha256), strings(&witness.node_kinds), strings(&witness.node_facts), edges(&witness.edges), direct_footprints, calls, footprints, witness_holdings, witness_places,
    )
}

pub fn replay_witness(
    source: &Program,
    witness: &TraversalWitness,
) -> Result<CheckedProgram, WitnessError> {
    let checked = CheckedProgram::build(source).map_err(WitnessError::Construction)?;
    let expected = emit_witness(&checked);
    if witness.version != WITNESS_VERSION {
        return Err(WitnessError::Mismatch { field: "version" });
    }
    if witness.canonical_ast_sha256 != expected.canonical_ast_sha256 {
        return Err(WitnessError::Mismatch {
            field: "canonical_ast_sha256",
        });
    }
    if witness.node_kinds != expected.node_kinds {
        return Err(WitnessError::Mismatch {
            field: "node_kinds",
        });
    }
    if witness.node_facts != expected.node_facts {
        return Err(WitnessError::Mismatch {
            field: "node_facts",
        });
    }
    if witness.edges != expected.edges {
        return Err(WitnessError::Mismatch { field: "edges" });
    }
    if witness.direct_footprints != expected.direct_footprints {
        return Err(WitnessError::Mismatch {
            field: "direct_footprints",
        });
    }
    if witness.calls != expected.calls {
        return Err(WitnessError::Mismatch { field: "calls" });
    }
    if witness.footprints != expected.footprints {
        return Err(WitnessError::Mismatch {
            field: "footprints",
        });
    }
    if witness.holdings != expected.holdings {
        return Err(WitnessError::Mismatch { field: "holdings" });
    }
    if witness.shared_places != expected.shared_places {
        return Err(WitnessError::Mismatch {
            field: "shared_places",
        });
    }
    Ok(checked)
}

pub(crate) fn canonical_ast_sha256(program: &Program) -> String {
    // Versioned by WITNESS_VERSION. Debug formatting is structural for these
    // repository-owned AST types and includes literal values, clauses, spans,
    // wrapper records, and source order; the digest binds the witness to that
    // complete canonical tree rather than only its traversal shape.
    let bytes = format!("thermite-rfc10-ast-v{WITNESS_VERSION}\n{program:#?}");
    format!("{:x}", Sha256::digest(bytes.as_bytes()))
}
