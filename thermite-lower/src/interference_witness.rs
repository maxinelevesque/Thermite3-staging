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
    pub requirements: Vec<WitnessCompositionRequirement>,
    pub obligations: Vec<WitnessInterferenceObligation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalInterferenceProjection {
    pub canonical_ast_sha256: String,
    pub checked_interference_sha256: String,
    pub functions: Vec<WitnessInterferenceFunction>,
    pub requirements: Vec<WitnessCompositionRequirement>,
    pub obligations: Vec<WitnessInterferenceObligation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessInterferenceFunction {
    pub function: String,
    pub asks: Vec<WitnessMonotoneAtom>,
    pub promises: Vec<WitnessMonotoneAtom>,
    /// Canonical promise-relevant shared-write regions in the checked
    /// transitive footprint.
    ///
    /// For an in-language body these are inferred from assignments and callees.
    /// For a foreign boundary they come from its declared effect row, which
    /// remains named residual trust.  The list is the bounded RFC-12 effect
    /// trace observable: it records *where* a step may write, not runtime values
    /// or a total order of concurrent events.
    pub observed_writes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessMonotoneAtom {
    pub place: String,
    pub kind: String,
}

pub const PROMISE_TRACE_OBSERVABLE_VERSION: &str =
    "rfc12-canonical-promised-shared-write-regions-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromiseTraceMutationKind {
    Weaken,
    Delete,
    Redirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromiseTraceMutationOutcome {
    Killed,
    Survived,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromiseTraceMutationCase {
    pub id: String,
    pub function: String,
    pub kind: PromiseTraceMutationKind,
    pub outcome: PromiseTraceMutationOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromiseTraceMutationScore {
    pub observable: String,
    pub killed: usize,
    pub survived: usize,
    pub unsupported: usize,
    pub cases: Vec<PromiseTraceMutationCase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessInterferenceObligation {
    pub composition: String,
    pub guarantor: String,
    pub relying: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessCompositionRequirement {
    pub composition: String,
    pub left_root: String,
    pub right_root: String,
    pub left_priority: Option<u64>,
    pub right_priority: Option<u64>,
    pub overlaps: Vec<String>,
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
        .map(|function| {
            let promises = witness_atoms(&function.promises);
            let observed_writes = checked
                .effects()
                .footprints
                .get(&function.function)
                .into_iter()
                .flatten()
                .filter_map(|effect| match effect {
                    thermite_syntax::Effect::Write(path) => Some(path.display()),
                    _ => None,
                })
                .filter(|write| {
                    promises
                        .iter()
                        .any(|atom| regions_overlap(&atom.place, write))
                })
                .collect();
            WitnessInterferenceFunction {
                function: function.function.clone(),
                asks: witness_atoms(&function.asks),
                promises,
                observed_writes,
            }
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
    let requirements = checked
        .interference()
        .requirements
        .iter()
        .map(|requirement| WitnessCompositionRequirement {
            composition: requirement.composition.clone(),
            left_root: requirement.left_root.clone(),
            right_root: requirement.right_root.clone(),
            left_priority: requirement.left_priority,
            right_priority: requirement.right_priority,
            overlaps: requirement.overlaps.clone(),
        })
        .collect::<Vec<_>>();
    let checked_interference_sha256 = checked_digest(
        &canonical_ast_sha256,
        &functions,
        &requirements,
        &obligations,
    );
    InterferenceWitness {
        version: INTERFERENCE_WITNESS_VERSION,
        canonical_ast_sha256,
        checked_interference_sha256,
        functions,
        requirements,
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
        requirements: witness.requirements,
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
    if !witness.functions.iter().all(promise_trace_sound) {
        return Err(WitnessError::Mismatch {
            field: "interference_promise_trace",
        });
    }
    if witness.requirements != expected.requirements {
        return Err(WitnessError::Mismatch {
            field: "interference_requirements",
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

fn region_contains(outer: &str, inner: &str) -> bool {
    let outer = outer.split('.').collect::<Vec<_>>();
    let inner = inner.split('.').collect::<Vec<_>>();
    outer.len() <= inner.len() && outer.iter().zip(inner).all(|(left, right)| left == &right)
}

fn regions_overlap(left: &str, right: &str) -> bool {
    region_contains(left, right) || region_contains(right, left)
}

/// Whether every checked shared-write observation is covered by the function's
/// promised monotone relation.
pub fn promise_trace_sound(function: &WitnessInterferenceFunction) -> bool {
    function.observed_writes.iter().all(|write| {
        !write.is_empty()
            && write.split('.').all(|segment| !segment.is_empty())
            && function
                .promises
                .iter()
                .any(|atom| regions_overlap(&atom.place, write))
    })
}

/// The deterministic RFC-12 mutation family over a checked promise trace.
///
/// Unsupported cases are retained explicitly rather than counted as kills or
/// survivors.  A supported mutant survives only when its changed promise still
/// covers every observed shared-write region.
pub fn promise_trace_mutants(
    function: &WitnessInterferenceFunction,
) -> Vec<(
    PromiseTraceMutationKind,
    Option<WitnessInterferenceFunction>,
)> {
    if function.observed_writes.is_empty() || function.promises.is_empty() {
        return [
            PromiseTraceMutationKind::Weaken,
            PromiseTraceMutationKind::Delete,
            PromiseTraceMutationKind::Redirect,
        ]
        .into_iter()
        .map(|kind| (kind, None))
        .collect();
    }

    let mut weakened = function.clone();
    weakened.promises.remove(0);

    let mut deleted = function.clone();
    deleted.promises.clear();

    let mut redirected = function.clone();
    redirected.promises[0].place = fresh_redirect_region(function);

    vec![
        (PromiseTraceMutationKind::Weaken, Some(weakened)),
        (PromiseTraceMutationKind::Delete, Some(deleted)),
        (PromiseTraceMutationKind::Redirect, Some(redirected)),
    ]
}

fn fresh_redirect_region(function: &WitnessInterferenceFunction) -> String {
    let occupied = function.promises.len() + function.observed_writes.len();
    (0..=occupied)
        .map(|index| format!("__thermite_promise_redirect_{index}"))
        .find(|candidate| {
            function
                .promises
                .iter()
                .all(|atom| !regions_overlap(&atom.place, candidate))
                && function
                    .observed_writes
                    .iter()
                    .all(|write| !regions_overlap(write, candidate))
        })
        .expect("one more deterministic root than occupied roots must be fresh")
}

pub fn score_promise_trace_mutations(witness: &InterferenceWitness) -> PromiseTraceMutationScore {
    let mut cases = Vec::new();
    for function in &witness.functions {
        for (kind, mutant) in promise_trace_mutants(function) {
            let outcome = match mutant {
                Some(mutant) if promise_trace_sound(&mutant) => {
                    PromiseTraceMutationOutcome::Survived
                }
                Some(_) => PromiseTraceMutationOutcome::Killed,
                None => PromiseTraceMutationOutcome::Unsupported,
            };
            cases.push(PromiseTraceMutationCase {
                id: format!("{}-{}", function.function, mutation_kind_name(kind)),
                function: function.function.clone(),
                kind,
                outcome,
            });
        }
    }
    let killed = cases
        .iter()
        .filter(|case| case.outcome == PromiseTraceMutationOutcome::Killed)
        .count();
    let survived = cases
        .iter()
        .filter(|case| case.outcome == PromiseTraceMutationOutcome::Survived)
        .count();
    let unsupported = cases
        .iter()
        .filter(|case| case.outcome == PromiseTraceMutationOutcome::Unsupported)
        .count();
    PromiseTraceMutationScore {
        observable: PROMISE_TRACE_OBSERVABLE_VERSION.to_string(),
        killed,
        survived,
        unsupported,
        cases,
    }
}

fn mutation_kind_name(kind: PromiseTraceMutationKind) -> &'static str {
    match kind {
        PromiseTraceMutationKind::Weaken => "weaken",
        PromiseTraceMutationKind::Delete => "delete",
        PromiseTraceMutationKind::Redirect => "redirect",
    }
}

fn checked_digest(
    canonical_ast_sha256: &str,
    functions: &[WitnessInterferenceFunction],
    requirements: &[WitnessCompositionRequirement],
    obligations: &[WitnessInterferenceObligation],
) -> String {
    let body = serde_json::to_string(&(functions, requirements, obligations))
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
    fn path(value: &str) -> String {
        format!(
            "[{}]",
            value.split('.').map(string).collect::<Vec<_>>().join(", ")
        )
    }
    fn atoms(values: &[WitnessMonotoneAtom]) -> String {
        values
            .iter()
            .map(|atom| format!("⟨{}, {}⟩", path(&atom.place), string(&atom.kind)))
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn functions(values: &[WitnessInterferenceFunction]) -> String {
        values
            .iter()
            .map(|function| {
                format!(
                    "⟨{}, [{}], [{}], [{}]⟩",
                    string(&function.function),
                    atoms(&function.asks),
                    atoms(&function.promises),
                    function
                        .observed_writes
                        .iter()
                        .map(|write| path(write))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn bools(values: &[bool]) -> String {
        values
            .iter()
            .map(|value| if *value { "true" } else { "false" })
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
    fn priority(value: Option<u64>) -> String {
        value.map_or_else(|| "none".to_string(), |value| format!("some {value}"))
    }
    fn requirements(values: &[WitnessCompositionRequirement]) -> String {
        values
            .iter()
            .map(|requirement| {
                let overlaps = requirement
                    .overlaps
                    .iter()
                    .map(|overlap| path(overlap))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "⟨{}, {}, {}, {}, {}, [{}]⟩",
                    string(&requirement.composition),
                    string(&requirement.left_root),
                    string(&requirement.right_root),
                    priority(requirement.left_priority),
                    priority(requirement.right_priority),
                    overlaps,
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    let promise_mutants = witness
        .functions
        .iter()
        .flat_map(promise_trace_mutants)
        .filter_map(|(_, mutant)| mutant)
        .collect::<Vec<_>>();
    let expected_survivors = promise_mutants
        .iter()
        .map(promise_trace_sound)
        .collect::<Vec<_>>();
    format!(
        "import Thermite.Interference\nopen Thermite.Interference\n\ndef canonical : Canonical := ⟨{}, {}, [{}], [{}], [{}]⟩\ndef witness : Witness := ⟨{}, {}, {}, [{}], [{}], [{}]⟩\ndef promiseMutants : List FunctionContract := [{}]\ndef expectedPromiseMutationSurvivors : List Bool := [{}]\ntheorem rfc12_interference_verified : verify canonical witness = true := by rfl\ntheorem rfc12_promise_mutation_replay_agrees : promiseMutants.map functionSound = expectedPromiseMutationSurvivors := by rfl\n#print axioms rfc12_interference_verified\n#print axioms rfc12_promise_mutation_replay_agrees\n#eval IO.println \"THERMITE_RFC12_INTERFERENCE_REPLAY_ACCEPTED_V1\"\n#eval IO.println \"THERMITE_RFC12_PROMISE_MUTATION_REPLAY_ACCEPTED_V1\"\n",
        string(&canonical.canonical_ast_sha256),
        string(&canonical.checked_interference_sha256),
        functions(&canonical.functions),
        requirements(&canonical.requirements),
        obligations(&canonical.obligations),
        witness.version,
        string(&witness.canonical_ast_sha256),
        string(&witness.checked_interference_sha256),
        functions(&witness.functions),
        requirements(&witness.requirements),
        obligations(&witness.obligations),
        functions(&promise_mutants),
        bools(&expected_survivors),
    )
}
