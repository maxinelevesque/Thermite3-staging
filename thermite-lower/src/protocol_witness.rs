//! Versioned RFC-13 protocol-projection and endpoint-flow witness.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thermite_syntax::Program;

use crate::{CheckedProgram, WitnessError};

pub const PROTOCOL_WITNESS_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolWitness {
    pub version: u32,
    pub canonical_ast_sha256: String,
    pub checked_protocol_sha256: String,
    pub definitions: Vec<WitnessProtocolDefinition>,
    pub functions: Vec<WitnessProtocolFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalProtocolProjection {
    pub canonical_ast_sha256: String,
    pub checked_protocol_sha256: String,
    pub definitions: Vec<WitnessProtocolDefinition>,
    pub functions: Vec<WitnessProtocolFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessProtocolDefinition {
    pub protocol: String,
    pub roles: Vec<String>,
    pub projections: Vec<WitnessRoleProjection>,
    pub repeat: bool,
    pub compatible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessRoleProjection {
    pub role: String,
    pub actions: Vec<WitnessProtocolAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessProtocolAction {
    pub kind: String,
    pub payload: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessProtocolFunction {
    pub function: String,
    pub endpoints: Vec<WitnessProtocolEndpoint>,
    pub transitions: Vec<WitnessProtocolTransition>,
    pub completed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessProtocolEndpoint {
    pub binding: String,
    pub protocol: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessProtocolTransition {
    pub endpoint: String,
    pub step: usize,
    pub action: String,
}

impl ProtocolWitness {
    pub fn canonical_json(&self) -> Result<String, WitnessError> {
        serde_json::to_string(self).map_err(|error| WitnessError::Json(error.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, WitnessError> {
        serde_json::from_str(json).map_err(|error| WitnessError::Json(error.to_string()))
    }
}

pub fn emit_protocol_witness(checked: &CheckedProgram) -> ProtocolWitness {
    let canonical_ast_sha256 = crate::witness::canonical_ast_sha256(checked.source());
    let definitions = checked
        .protocol_flow()
        .definitions
        .iter()
        .map(|(protocol, definition)| WitnessProtocolDefinition {
            protocol: protocol.clone(),
            roles: definition.roles.clone(),
            projections: definition
                .projections
                .iter()
                .map(|(role, actions)| WitnessRoleProjection {
                    role: role.clone(),
                    actions: actions.iter().map(witness_action).collect(),
                })
                .collect(),
            repeat: definition.repeat,
            compatible: definition.compatible,
        })
        .collect::<Vec<_>>();
    let functions = checked
        .protocol_flow()
        .functions
        .iter()
        .map(|(function, flow)| WitnessProtocolFunction {
            function: function.clone(),
            endpoints: flow
                .endpoints
                .iter()
                .map(|(binding, endpoint)| WitnessProtocolEndpoint {
                    binding: binding.clone(),
                    protocol: endpoint.protocol.clone(),
                    role: endpoint.role.clone(),
                })
                .collect(),
            transitions: flow
                .transitions
                .iter()
                .map(|transition| WitnessProtocolTransition {
                    endpoint: transition.endpoint.clone(),
                    step: transition.step,
                    action: transition.action.to_string(),
                })
                .collect(),
            completed: flow.completed.clone(),
        })
        .collect::<Vec<_>>();
    let checked_protocol_sha256 = checked_digest(&canonical_ast_sha256, &definitions, &functions);
    ProtocolWitness {
        version: PROTOCOL_WITNESS_VERSION,
        canonical_ast_sha256,
        checked_protocol_sha256,
        definitions,
        functions,
    }
}

pub fn canonical_protocol_projection(
    source: &Program,
) -> Result<CanonicalProtocolProjection, WitnessError> {
    let checked = CheckedProgram::build(source).map_err(WitnessError::Construction)?;
    let witness = emit_protocol_witness(&checked);
    Ok(CanonicalProtocolProjection {
        canonical_ast_sha256: witness.canonical_ast_sha256,
        checked_protocol_sha256: witness.checked_protocol_sha256,
        definitions: witness.definitions,
        functions: witness.functions,
    })
}

pub fn replay_protocol_witness(
    source: &Program,
    witness: &ProtocolWitness,
) -> Result<CheckedProgram, WitnessError> {
    let checked = CheckedProgram::build(source).map_err(WitnessError::Construction)?;
    let expected = emit_protocol_witness(&checked);
    if witness.version != PROTOCOL_WITNESS_VERSION {
        return Err(WitnessError::Mismatch {
            field: "protocol_version",
        });
    }
    if witness.canonical_ast_sha256 != expected.canonical_ast_sha256 {
        return Err(WitnessError::Mismatch {
            field: "protocol_canonical_ast_sha256",
        });
    }
    if witness.checked_protocol_sha256 != expected.checked_protocol_sha256 {
        return Err(WitnessError::Mismatch {
            field: "checked_protocol_sha256",
        });
    }
    if witness.definitions != expected.definitions {
        return Err(WitnessError::Mismatch {
            field: "protocol_definitions",
        });
    }
    if witness.functions != expected.functions {
        return Err(WitnessError::Mismatch {
            field: "protocol_functions",
        });
    }
    Ok(checked)
}

fn witness_action(action: &thermite_spec::ProtocolAction) -> WitnessProtocolAction {
    let (kind, payload): (&str, &[thermite_syntax::Type]) = match action {
        thermite_spec::ProtocolAction::Send(payload) => ("send", payload.as_slice()),
        thermite_spec::ProtocolAction::Receive(payload) => ("receive", payload.as_slice()),
        thermite_spec::ProtocolAction::ChooseRepeatOrEnd => ("choose_repeat_or_end", &[][..]),
        thermite_spec::ProtocolAction::AwaitRepeatOrEnd => ("await_repeat_or_end", &[][..]),
    };
    WitnessProtocolAction {
        kind: kind.to_string(),
        payload: payload.iter().map(|ty| format!("{ty:?}")).collect(),
    }
}

fn checked_digest(
    canonical_ast_sha256: &str,
    definitions: &[WitnessProtocolDefinition],
    functions: &[WitnessProtocolFunction],
) -> String {
    let body = serde_json::to_string(&(definitions, functions))
        .expect("protocol witness structures serialize");
    format!(
        "{:x}",
        Sha256::digest(
            format!("thermite-rfc13-checked-protocol-v1\n{canonical_ast_sha256}\n{body}")
                .as_bytes()
        )
    )
}
