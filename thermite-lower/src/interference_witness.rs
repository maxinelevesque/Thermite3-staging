//! Versioned RFC-12 checked-relation witness and replay binding.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thermite_syntax::Program;

use crate::{CheckedProgram, WitnessError};

pub const INTERFERENCE_WITNESS_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterferenceWitness {
    pub version: u32,
    pub canonical_ast_sha256: String,
    pub checked_interference_sha256: String,
    pub functions: Vec<WitnessInterferenceFunction>,
    pub obligations: Vec<WitnessInterferenceObligation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalInterferenceProjection {
    pub canonical_ast_sha256: String,
    pub checked_interference_sha256: String,
    pub functions: Vec<WitnessInterferenceFunction>,
    pub obligations: Vec<WitnessInterferenceObligation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessInterferenceFunction {
    pub function: String,
    pub asks: Vec<WitnessMonotoneAtom>,
    pub promises: Vec<WitnessMonotoneAtom>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessMonotoneAtom {
    pub place: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessInterferenceObligation {
    pub composition: String,
    pub guarantor: String,
    pub relying: String,
}

impl InterferenceWitness {
    pub fn canonical_json(&self) -> Result<String, WitnessError> {
        serde_json::to_string(self).map_err(|error| WitnessError::Json(error.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, WitnessError> {
        serde_json::from_str(json).map_err(|error| WitnessError::Json(error.to_string()))
    }
}

pub fn emit_interference_witness(checked: &CheckedProgram) -> InterferenceWitness {
    let canonical_ast_sha256 = crate::witness::canonical_ast_sha256(checked.source());
    let functions = checked
        .interference()
        .functions
        .values()
        .map(|function| WitnessInterferenceFunction {
            function: function.function.clone(),
            asks: witness_atoms(&function.asks),
            promises: witness_atoms(&function.promises),
        })
        .collect::<Vec<_>>();
    let obligations = checked
        .interference()
        .obligations
        .iter()
        .map(|obligation| WitnessInterferenceObligation {
            composition: obligation.composition.clone(),
            guarantor: obligation.guarantor.clone(),
            relying: obligation.relying.clone(),
        })
        .collect::<Vec<_>>();
    let checked_interference_sha256 =
        checked_digest(&canonical_ast_sha256, &functions, &obligations);
    InterferenceWitness {
        version: INTERFERENCE_WITNESS_VERSION,
        canonical_ast_sha256,
        checked_interference_sha256,
        functions,
        obligations,
    }
}

pub fn canonical_interference_projection(
    source: &Program,
) -> Result<CanonicalInterferenceProjection, WitnessError> {
    let checked = CheckedProgram::build(source).map_err(WitnessError::Construction)?;
    let witness = emit_interference_witness(&checked);
    Ok(CanonicalInterferenceProjection {
        canonical_ast_sha256: witness.canonical_ast_sha256,
        checked_interference_sha256: witness.checked_interference_sha256,
        functions: witness.functions,
        obligations: witness.obligations,
    })
}

pub fn replay_interference_witness(
    source: &Program,
    witness: &InterferenceWitness,
) -> Result<CheckedProgram, WitnessError> {
    let checked = CheckedProgram::build(source).map_err(WitnessError::Construction)?;
    let expected = emit_interference_witness(&checked);
    if witness.version != INTERFERENCE_WITNESS_VERSION {
        return Err(WitnessError::Mismatch {
            field: "interference_version",
        });
    }
    if witness.canonical_ast_sha256 != expected.canonical_ast_sha256 {
        return Err(WitnessError::Mismatch {
            field: "interference_canonical_ast_sha256",
        });
    }
    if witness.checked_interference_sha256 != expected.checked_interference_sha256 {
        return Err(WitnessError::Mismatch {
            field: "checked_interference_sha256",
        });
    }
    if witness.functions != expected.functions {
        return Err(WitnessError::Mismatch {
            field: "interference_functions",
        });
    }
    if witness.obligations != expected.obligations {
        return Err(WitnessError::Mismatch {
            field: "interference_obligations",
        });
    }
    Ok(checked)
}

fn witness_atoms(relation: &thermite_spec::CheckedRelation) -> Vec<WitnessMonotoneAtom> {
    relation
        .atoms
        .iter()
        .map(|atom| WitnessMonotoneAtom {
            place: atom.place.clone(),
            kind: match atom.kind {
                thermite_spec::MonotoneKind::Ordered => "ordered",
                thermite_spec::MonotoneKind::BitSet => "bit_set",
                thermite_spec::MonotoneKind::Boolean => "boolean",
            }
            .to_string(),
        })
        .collect()
}

fn checked_digest(
    canonical_ast_sha256: &str,
    functions: &[WitnessInterferenceFunction],
    obligations: &[WitnessInterferenceObligation],
) -> String {
    let body = serde_json::to_string(&(functions, obligations))
        .expect("interference witness structures serialize");
    format!(
        "{:x}",
        Sha256::digest(
            format!("thermite-rfc12-checked-interference-v1\n{canonical_ast_sha256}\n{body}")
                .as_bytes()
        )
    )
}

pub fn lean_interference_replay_source(
    canonical: &CanonicalInterferenceProjection,
    witness: &InterferenceWitness,
) -> String {
    fn string(value: &str) -> String {
        serde_json::to_string(value).expect("serializing a string cannot fail")
    }
    fn atoms(values: &[WitnessMonotoneAtom]) -> String {
        values
            .iter()
            .map(|atom| format!("⟨{}, {}⟩", string(&atom.place), string(&atom.kind)))
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn functions(values: &[WitnessInterferenceFunction]) -> String {
        values
            .iter()
            .map(|function| {
                format!(
                    "⟨{}, [{}], [{}]⟩",
                    string(&function.function),
                    atoms(&function.asks),
                    atoms(&function.promises)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn obligations(values: &[WitnessInterferenceObligation]) -> String {
        values
            .iter()
            .map(|obligation| {
                format!(
                    "⟨{}, {}, {}⟩",
                    string(&obligation.composition),
                    string(&obligation.guarantor),
                    string(&obligation.relying)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    format!(
        "import Thermite.Interference\nopen Thermite.Interference\n\ndef canonical : Canonical := ⟨{}, {}, [{}], [{}]⟩\ndef witness : Witness := ⟨{}, {}, {}, [{}], [{}]⟩\ntheorem rfc12_interference_verified : verify canonical witness = true := by rfl\n#print axioms rfc12_interference_verified\n#eval IO.println \"THERMITE_RFC12_INTERFERENCE_REPLAY_ACCEPTED_V1\"\n",
        string(&canonical.canonical_ast_sha256),
        string(&canonical.checked_interference_sha256),
        functions(&canonical.functions),
        obligations(&canonical.obligations),
        witness.version,
        string(&witness.canonical_ast_sha256),
        string(&witness.checked_interference_sha256),
        functions(&witness.functions),
        obligations(&witness.obligations),
    )
}
