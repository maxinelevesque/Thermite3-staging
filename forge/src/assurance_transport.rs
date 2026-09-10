//! Checked transport and predecessor comparison for assurance authorities.
//!
//! This module is intentionally independent of `Level`.  It carries exact
//! coordinate identities and accepts a comparison only after a typed
//! transport or incompatibility relation has been validated.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AssuranceCoordinateV2 {
    pub semantics: String,
    pub semantics_version: u64,
    pub implementation_model: String,
    pub implementation_model_version: u64,
    pub fragment_lineage: String,
    pub fragment_revision: u64,
    pub procedure: String,
    pub procedure_version: u64,
    pub environment: String,
    pub tool_version: String,
    pub resource_budget: u64,
    pub residual_context: String,
    pub boundary: String,
    pub claim_identity: String,
    pub evidence_identity: String,
    pub reconstruction_identity: String,
}

impl AssuranceCoordinateV2 {
    fn validate(&self) -> Result<(), TransportError> {
        for (name, value) in [
            ("semantics", &self.semantics),
            ("implementation_model", &self.implementation_model),
            ("fragment_lineage", &self.fragment_lineage),
            ("procedure", &self.procedure),
            ("environment", &self.environment),
            ("tool_version", &self.tool_version),
            ("residual_context", &self.residual_context),
            ("boundary", &self.boundary),
            ("claim_identity", &self.claim_identity),
            ("evidence_identity", &self.evidence_identity),
            ("reconstruction_identity", &self.reconstruction_identity),
        ] {
            if value.is_empty() {
                return Err(TransportError::new(format!("{name} must not be empty")));
            }
        }
        Ok(())
    }

    fn fiber_key(&self) -> (&str, u64, &str, u64, &str, u64, &str, &str, &str, u64, &str) {
        (
            &self.semantics,
            self.semantics_version,
            &self.implementation_model,
            self.implementation_model_version,
            &self.fragment_lineage,
            self.fragment_revision,
            &self.procedure,
            &self.environment,
            &self.tool_version,
            self.resource_budget,
            &self.residual_context,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportError {
    pub reason: String,
}

impl TransportError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid assurance transport: {}", self.reason)
    }
}

impl std::error::Error for TransportError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TransportWitnessV2 {
    pub source: AssuranceCoordinateV2,
    pub target: AssuranceCoordinateV2,
    pub program_translation: String,
    pub evidence_translation: String,
    pub observation_translation: String,
    pub context_entailment: String,
    pub boundary_preservation: String,
    pub receipt: String,
}

impl TransportWitnessV2 {
    pub fn validate(&self) -> Result<(), TransportError> {
        self.source.validate()?;
        self.target.validate()?;
        for (name, value) in [
            ("program_translation", &self.program_translation),
            ("evidence_translation", &self.evidence_translation),
            ("observation_translation", &self.observation_translation),
            ("context_entailment", &self.context_entailment),
            ("boundary_preservation", &self.boundary_preservation),
            ("receipt", &self.receipt),
        ] {
            if value.is_empty() {
                return Err(TransportError::new(format!("{name} must not be empty")));
            }
        }
        if self.source.semantics != self.target.semantics
            || self.source.implementation_model != self.target.implementation_model
            || self.source.semantics_version > self.target.semantics_version
            || self.source.implementation_model_version > self.target.implementation_model_version
            || self.source.resource_budget > self.target.resource_budget
        {
            return Err(TransportError::new(
                "transport has an invalid version/model/resource direction",
            ));
        }
        Ok(())
    }

    pub fn identity(coordinate: AssuranceCoordinateV2) -> Self {
        let receipt = digest(&["identity", &canonical(&coordinate)]);
        Self {
            source: coordinate.clone(),
            target: coordinate,
            program_translation: "identity".into(),
            evidence_translation: "identity".into(),
            observation_translation: "identity".into(),
            context_entailment: "identity".into(),
            boundary_preservation: "identity".into(),
            receipt,
        }
    }

    pub fn compose(first: &Self, second: &Self) -> Result<Self, TransportError> {
        first.validate()?;
        second.validate()?;
        if first.target != second.source {
            return Err(TransportError::new(
                "transport composition has mismatched middle fiber",
            ));
        }
        let mut composed = Self {
            source: first.source.clone(),
            target: second.target.clone(),
            program_translation: compose_label(
                &first.program_translation,
                &second.program_translation,
            ),
            evidence_translation: compose_label(
                &first.evidence_translation,
                &second.evidence_translation,
            ),
            observation_translation: compose_label(
                &first.observation_translation,
                &second.observation_translation,
            ),
            context_entailment: compose_label(
                &first.context_entailment,
                &second.context_entailment,
            ),
            boundary_preservation: compose_label(
                &first.boundary_preservation,
                &second.boundary_preservation,
            ),
            receipt: String::new(),
        };
        composed.receipt = digest(&[
            "compose",
            &first.receipt,
            &second.receipt,
            &canonical(&composed.source),
            &canonical(&composed.target),
        ]);
        Ok(composed)
    }
}

fn compose_label(first: &str, second: &str) -> String {
    format!("{first}>{second}")
}

fn canonical<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("transport value serializes")
}

fn digest(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"thermite-assurance-transport-v1\0");
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum IncompatibilityReasonV2 {
    SemanticFork,
    ModelBreak,
    UnsupportedProcedure,
    BoundaryConflict,
    ContextConflict,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IncompatibilityWitnessV2 {
    pub source: String,
    pub target: String,
    pub reason: IncompatibilityReasonV2,
    pub explanation: String,
    pub source_independent: bool,
    pub target_independent: bool,
    pub no_transport: bool,
    pub receipt: String,
}

impl IncompatibilityWitnessV2 {
    fn validate(&self) -> Result<(), TransportError> {
        if self.source.is_empty() || self.target.is_empty() || self.explanation.is_empty() {
            return Err(TransportError::new(
                "incompatibility identity and explanation are required",
            ));
        }
        if !self.source_independent || !self.target_independent || !self.no_transport {
            return Err(TransportError::new(
                "incompatibility must preserve independent authority and prove no transport",
            ));
        }
        if self.receipt.len() != 64 || !self.receipt.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(TransportError::new(
                "incompatibility receipt must be a SHA-256 digest",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PredecessorRelationV2 {
    Transported(Box<TransportWitnessV2>),
    Incompatible(IncompatibilityWitnessV2),
}

impl PredecessorRelationV2 {
    fn endpoints(&self) -> (&str, &str) {
        match self {
            Self::Transported(witness) => (
                &witness.source.claim_identity,
                &witness.target.claim_identity,
            ),
            Self::Incompatible(witness) => (&witness.source, &witness.target),
        }
    }

    fn validate(&self) -> Result<(), TransportError> {
        match self {
            Self::Transported(witness) => witness.validate(),
            Self::Incompatible(witness) => witness.validate(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PredecessorDomainV2 {
    pub identities: Vec<String>,
    pub direct_predecessors: Vec<(String, String)>,
    pub relations: Vec<PredecessorRelationV2>,
}

impl PredecessorDomainV2 {
    pub fn validate(&self) -> Result<(), TransportError> {
        if self.identities.is_empty() {
            return Err(TransportError::new(
                "predecessor domain must contain an identity",
            ));
        }
        let identities = self.identities.iter().cloned().collect::<BTreeSet<_>>();
        if identities.len() != self.identities.len() || self.identities.iter().any(String::is_empty)
        {
            return Err(TransportError::new(
                "identities must be non-empty and duplicate-free",
            ));
        }
        let direct = self
            .direct_predecessors
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        if direct.len() != self.direct_predecessors.len()
            || direct.iter().any(|(source, target)| {
                source == target || !identities.contains(source) || !identities.contains(target)
            })
        {
            return Err(TransportError::new("direct predecessor edges are invalid"));
        }
        let closure = reachable_edges(&identities, &direct);
        let mut seen = BTreeSet::new();
        for relation in &self.relations {
            relation.validate()?;
            let endpoints = relation.endpoints();
            if !closure.contains(&(endpoints.0.to_owned(), endpoints.1.to_owned())) {
                return Err(TransportError::new(
                    "relation is outside the declared predecessor closure",
                ));
            }
            if !seen.insert((endpoints.0.to_owned(), endpoints.1.to_owned())) {
                return Err(TransportError::new("duplicate predecessor classification"));
            }
        }
        if seen.len() != closure.len() || closure.iter().any(|edge| !seen.contains(edge)) {
            return Err(TransportError::new(
                "predecessor closure has missing classification",
            ));
        }
        Ok(())
    }
}

fn reachable_edges(
    identities: &BTreeSet<String>,
    direct: &BTreeSet<(String, String)>,
) -> BTreeSet<(String, String)> {
    let mut adjacency = BTreeMap::<&str, Vec<&str>>::new();
    for (source, target) in direct {
        adjacency.entry(source).or_default().push(target);
    }
    let mut closure = BTreeSet::new();
    for source in identities {
        let mut queue = VecDeque::from([source.as_str()]);
        let mut visited = BTreeSet::new();
        while let Some(current) = queue.pop_front() {
            if !visited.insert(current) {
                continue;
            }
            for target in adjacency.get(current).into_iter().flatten() {
                if closure.insert((source.clone(), (*target).to_owned())) {
                    queue.push_back(target);
                }
            }
        }
    }
    closure
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonOutcome {
    Strengthened,
    Weakened,
    Incomparable,
    ChangedFiber,
    MissingClassification,
}

pub fn compare_relation(
    source: &AssuranceCoordinateV2,
    target: &AssuranceCoordinateV2,
    relation: Option<&PredecessorRelationV2>,
) -> ComparisonOutcome {
    let Some(relation) = relation else {
        return ComparisonOutcome::MissingClassification;
    };
    if relation.endpoints()
        != (
            source.claim_identity.as_str(),
            target.claim_identity.as_str(),
        )
    {
        return ComparisonOutcome::MissingClassification;
    }
    if source.fiber_key() != target.fiber_key() {
        return ComparisonOutcome::ChangedFiber;
    }
    match relation {
        PredecessorRelationV2::Incompatible(_) => ComparisonOutcome::Incomparable,
        PredecessorRelationV2::Transported(_) if source.boundary != target.boundary => {
            ComparisonOutcome::Weakened
        }
        PredecessorRelationV2::Transported(_) => ComparisonOutcome::Strengthened,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coordinate(version: u64, claim: &str) -> AssuranceCoordinateV2 {
        AssuranceCoordinateV2 {
            semantics: "thermite-language".into(),
            semantics_version: version,
            implementation_model: "rustc".into(),
            implementation_model_version: version,
            fragment_lineage: "thermite-emitted-rust".into(),
            fragment_revision: version,
            procedure: "lean".into(),
            procedure_version: version,
            environment: "linux".into(),
            tool_version: "tool-1".into(),
            resource_budget: version,
            residual_context: "none".into(),
            boundary: "end-to-end".into(),
            claim_identity: claim.into(),
            evidence_identity: format!("evidence-{claim}"),
            reconstruction_identity: format!("reconstruction-{claim}"),
        }
    }

    #[test]
    fn identity_and_composition_preserve_endpoints() {
        let first = TransportWitnessV2::identity(coordinate(1, "a"));
        let second = TransportWitnessV2::identity(coordinate(1, "a"));
        let composed = TransportWitnessV2::compose(&first, &second).unwrap();
        assert_eq!(composed.source, first.source);
        assert_eq!(composed.target, second.target);
        assert_ne!(composed.receipt, first.receipt);
    }

    #[test]
    fn predecessor_domain_requires_transitive_classification() {
        let source = coordinate(1, "a");
        let middle = coordinate(2, "b");
        let target = coordinate(3, "c");
        let first = TransportWitnessV2::identity(source.clone());
        let second = TransportWitnessV2::identity(middle.clone());
        let third = TransportWitnessV2::identity(target.clone());
        let missing = PredecessorDomainV2 {
            identities: vec!["a".into(), "b".into(), "c".into()],
            direct_predecessors: vec![("a".into(), "b".into()), ("b".into(), "c".into())],
            relations: vec![
                PredecessorRelationV2::Transported(Box::new(TransportWitnessV2 {
                    source,
                    target: middle,
                    ..first
                })),
                PredecessorRelationV2::Transported(Box::new(TransportWitnessV2 {
                    source: coordinate(2, "b"),
                    target,
                    ..second
                })),
            ],
        };
        assert!(missing.validate().is_err());
        let _ = third;
    }

    #[test]
    fn missing_and_incompatible_relations_are_distinct() {
        let source = coordinate(1, "a");
        let target = coordinate(1, "a");
        assert_eq!(
            compare_relation(&source, &target, None),
            ComparisonOutcome::MissingClassification
        );
        let witness = IncompatibilityWitnessV2 {
            source: "a".into(),
            target: "b".into(),
            reason: IncompatibilityReasonV2::ModelBreak,
            explanation: "model semantics fork".into(),
            source_independent: true,
            target_independent: true,
            no_transport: true,
            receipt: "a".repeat(64),
        };
        let target = coordinate(1, "b");
        assert_eq!(
            compare_relation(
                &source,
                &target,
                Some(&PredecessorRelationV2::Incompatible(witness))
            ),
            ComparisonOutcome::Incomparable
        );
    }
}
