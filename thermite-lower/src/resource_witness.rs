//! Versioned RFC-11 resource-flow witness and independent replay inputs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thermite_syntax::Program;

use crate::{CheckedProgram, WitnessError};

pub const RESOURCE_WITNESS_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceFlowWitness {
    pub version: u32,
    pub canonical_ast_sha256: String,
    pub checked_resource_sha256: String,
    pub functions: Vec<WitnessResourceFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalResourceProjection {
    pub canonical_ast_sha256: String,
    pub checked_resource_sha256: String,
    pub functions: Vec<WitnessResourceFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessResourceFunction {
    pub function: String,
    pub entry_live: Vec<String>,
    pub returning_edges: Vec<WitnessResourceReturningEdge>,
    pub joins: Vec<WitnessResourceJoin>,
    pub loops: Vec<WitnessResourceLoop>,
    pub forgets: Vec<WitnessResourceForget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessResourceReturningEdge {
    pub label: String,
    pub live: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessResourceJoin {
    pub label: String,
    pub incoming: Vec<Vec<String>>,
    pub outgoing: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessResourceLoop {
    pub label: String,
    pub header: Vec<String>,
    pub back_edges: Vec<Vec<String>>,
    pub exit_edges: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessResourceForget {
    pub label: String,
    pub place: Option<String>,
    pub value_regions: Vec<String>,
    pub priced_regions: Vec<String>,
    pub declared_regions: Vec<String>,
}

impl ResourceFlowWitness {
    pub fn canonical_json(&self) -> Result<String, WitnessError> {
        serde_json::to_string(self).map_err(|error| WitnessError::Json(error.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, WitnessError> {
        serde_json::from_str(json).map_err(|error| WitnessError::Json(error.to_string()))
    }
}

pub fn emit_resource_witness(checked: &CheckedProgram) -> ResourceFlowWitness {
    let canonical_ast_sha256 = crate::witness::canonical_ast_sha256(checked.source());
    let functions = witness_functions(&checked.resource_flow().functions);
    let checked_resource_sha256 = checked_resource_digest(&canonical_ast_sha256, &functions);
    ResourceFlowWitness {
        version: RESOURCE_WITNESS_VERSION,
        canonical_ast_sha256,
        checked_resource_sha256,
        functions,
    }
}

pub fn canonical_resource_projection(
    source: &Program,
) -> Result<CanonicalResourceProjection, WitnessError> {
    let checked = CheckedProgram::build(source).map_err(WitnessError::Construction)?;
    let witness = emit_resource_witness(&checked);
    Ok(CanonicalResourceProjection {
        canonical_ast_sha256: witness.canonical_ast_sha256,
        checked_resource_sha256: witness.checked_resource_sha256,
        functions: witness.functions,
    })
}

pub fn replay_resource_witness(
    source: &Program,
    witness: &ResourceFlowWitness,
) -> Result<CheckedProgram, WitnessError> {
    let checked = CheckedProgram::build(source).map_err(WitnessError::Construction)?;
    let expected = emit_resource_witness(&checked);
    if witness.version != RESOURCE_WITNESS_VERSION {
        return Err(WitnessError::Mismatch {
            field: "resource_version",
        });
    }
    if witness.canonical_ast_sha256 != expected.canonical_ast_sha256 {
        return Err(WitnessError::Mismatch {
            field: "resource_canonical_ast_sha256",
        });
    }
    if witness.checked_resource_sha256 != expected.checked_resource_sha256 {
        return Err(WitnessError::Mismatch {
            field: "checked_resource_sha256",
        });
    }
    if witness.functions != expected.functions {
        return Err(WitnessError::Mismatch {
            field: "resource_functions",
        });
    }
    Ok(checked)
}

fn witness_functions(
    functions: &BTreeMap<String, thermite_spec::ResourceFunctionFlow>,
) -> Vec<WitnessResourceFunction> {
    functions
        .iter()
        .map(|(function, flow)| WitnessResourceFunction {
            function: function.clone(),
            entry_live: flow.entry_live.clone(),
            returning_edges: flow
                .returning_edges
                .iter()
                .map(|edge| WitnessResourceReturningEdge {
                    label: edge.label.clone(),
                    live: edge.live.clone(),
                })
                .collect(),
            joins: flow
                .joins
                .iter()
                .map(|join| WitnessResourceJoin {
                    label: join.label.clone(),
                    incoming: join.incoming.clone(),
                    outgoing: join.outgoing.clone(),
                })
                .collect(),
            loops: flow
                .loops
                .iter()
                .map(|loop_| WitnessResourceLoop {
                    label: loop_.label.clone(),
                    header: loop_.header.clone(),
                    back_edges: loop_.back_edges.clone(),
                    exit_edges: loop_.exit_edges.clone(),
                })
                .collect(),
            forgets: flow
                .forgets
                .iter()
                .map(|forget| WitnessResourceForget {
                    label: forget.label.clone(),
                    place: forget.place.clone(),
                    value_regions: forget
                        .value_regions
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    priced_regions: forget
                        .priced_regions
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    declared_regions: forget
                        .declared_regions
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

fn checked_resource_digest(
    canonical_ast_sha256: &str,
    functions: &[WitnessResourceFunction],
) -> String {
    let body = serde_json::to_string(functions).expect("resource witness structures serialize");
    format!(
        "{:x}",
        Sha256::digest(
            format!("thermite-rfc11-checked-resource-v1\n{canonical_ast_sha256}\n{body}")
                .as_bytes()
        )
    )
}

pub fn lean_resource_replay_source(
    canonical: &CanonicalResourceProjection,
    witness: &ResourceFlowWitness,
) -> String {
    fn string(value: &str) -> String {
        serde_json::to_string(value).expect("serializing a string cannot fail")
    }
    fn strings(values: &[String]) -> String {
        format!(
            "[{}]",
            values
                .iter()
                .map(|value| string(value))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    fn nested(values: &[Vec<String>]) -> String {
        format!(
            "[{}]",
            values
                .iter()
                .map(|value| strings(value))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    fn functions(values: &[WitnessResourceFunction]) -> String {
        values
            .iter()
            .map(|function| {
                let returns = function
                    .returning_edges
                    .iter()
                    .map(|edge| format!("⟨{}, {}⟩", string(&edge.label), strings(&edge.live)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let joins = function
                    .joins
                    .iter()
                    .map(|join| {
                        format!(
                            "⟨{}, {}, {}⟩",
                            string(&join.label),
                            nested(&join.incoming),
                            strings(&join.outgoing)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let loops = function
                    .loops
                    .iter()
                    .map(|loop_| {
                        format!(
                            "⟨{}, {}, {}, {}⟩",
                            string(&loop_.label),
                            strings(&loop_.header),
                            nested(&loop_.back_edges),
                            nested(&loop_.exit_edges)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let forgets = function
                    .forgets
                    .iter()
                    .map(|forget| {
                        let place = forget
                            .place
                            .as_ref()
                            .map(|place| format!("some {}", string(place)))
                            .unwrap_or_else(|| "none".to_string());
                        format!(
                            "⟨{}, {}, {}, {}, {}⟩",
                            string(&forget.label),
                            place,
                            strings(&forget.value_regions),
                            strings(&forget.priced_regions),
                            strings(&forget.declared_regions)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "⟨{}, {}, [{}], [{}], [{}], [{}]⟩",
                    string(&function.function),
                    strings(&function.entry_live),
                    returns,
                    joins,
                    loops,
                    forgets
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    format!(
        "import Thermite.ResourceFlow\nopen Thermite.ResourceFlow\n\ndef canonical : Canonical := ⟨{}, {}, [{}]⟩\ndef witness : Witness := ⟨{}, {}, {}, [{}]⟩\ntheorem rfc11_resource_flow_verified : verify canonical witness = true := by decide\n#print axioms rfc11_resource_flow_verified\n#eval IO.println \"THERMITE_RFC11_RESOURCE_REPLAY_ACCEPTED_V1\"\n",
        string(&canonical.canonical_ast_sha256),
        string(&canonical.checked_resource_sha256),
        functions(&canonical.functions),
        witness.version,
        string(&witness.canonical_ast_sha256),
        string(&witness.checked_resource_sha256),
        functions(&witness.functions),
    )
}
