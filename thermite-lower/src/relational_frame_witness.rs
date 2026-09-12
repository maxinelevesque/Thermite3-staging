//! Exact, source-only Tier-A relational-frame witness production.
//!
//! The producer derives every authority-bearing field from a checked program.
//! It cannot emit `end_to_end`: that upgrade belongs to the separate semantic
//! transport theorem and receipt path.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thermite_syntax::{Effect, EffectRow, Item, Program};

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

fn ambient(name: &str) -> String {
    name.to_owned()
}

fn read_region(effect: &Effect) -> Option<String> {
    match effect {
        Effect::Read(region) | Effect::Write(region) | Effect::Net(region) => {
            Some(region.to_string())
        }
        Effect::Alloc => Some(ambient("heap")),
        Effect::Time => Some(ambient("clock")),
        Effect::Rand => Some(ambient("entropy")),
        Effect::Term => Some(ambient("termios")),
        Effect::Forgets(_) | Effect::Owns(_) | Effect::Blocks | Effect::Panic | Effect::Diverge => {
            None
        }
    }
}

fn write_region(effect: &Effect) -> Option<String> {
    match effect {
        Effect::Write(region) | Effect::Net(region) => Some(region.to_string()),
        Effect::Alloc => Some(ambient("heap")),
        Effect::Rand => Some(ambient("entropy")),
        Effect::Term => Some(ambient("termios")),
        Effect::Read(_)
        | Effect::Time
        | Effect::Forgets(_)
        | Effect::Owns(_)
        | Effect::Blocks
        | Effect::Panic
        | Effect::Diverge => None,
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

fn regions(effects: &BTreeSet<Effect>, projection: fn(&Effect) -> Option<String>) -> Vec<String> {
    effects
        .iter()
        .filter_map(projection)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn emit_relational_frame_witness(checked: &CheckedProgram) -> RelationalFrameWitness {
    let mut functions = Vec::new();
    for item in &checked.source().items {
        let Item::Fn(function) = item else { continue };
        let effects = row_effects(&function.contract.effects);
        let inferred = checked
            .effects()
            .footprints
            .get(&function.name)
            .cloned()
            .unwrap_or_default();
        functions.push(RelationalFunctionWitness {
            function: function.name.clone(),
            normalized_row: effects.iter().map(effect_name).collect(),
            read_footprint: regions(&inferred, read_region),
            write_footprint: regions(&inferred, write_region),
            effect_support: effects
                .iter()
                .map(|effect| EffectSupport {
                    effect: effect_name(effect),
                    support: effect_support(effect),
                })
                .collect(),
            semantic_fragment: "tier-a-source-v1".into(),
            projections: projections(&effects),
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
}
