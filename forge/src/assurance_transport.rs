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

    fn transport_family_key(&self) -> (&str, &str, &str, &str, &str, &str, &str, &str, &str, &str) {
        (
            &self.semantics,
            &self.implementation_model,
            &self.fragment_lineage,
            &self.procedure,
            &self.environment,
            &self.tool_version,
            &self.residual_context,
            &self.claim_identity,
            &self.evidence_identity,
            &self.reconstruction_identity,
        )
    }

    fn identity(&self) -> String {
        digest(&["coordinate", &canonical(self)])
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
    pub predecessor_receipts: Vec<String>,
    pub receipt: String,
}

impl TransportWitnessV2 {
    fn receipt_payload(&self) -> String {
        canonical(&(
            &self.source,
            &self.target,
            &self.program_translation,
            &self.evidence_translation,
            &self.observation_translation,
            &self.context_entailment,
            &self.boundary_preservation,
            &self.predecessor_receipts,
        ))
    }

    fn expected_receipt(&self) -> String {
        digest(&["transport", &self.receipt_payload()])
    }

    fn seal(mut self) -> Self {
        self.receipt = self.expected_receipt();
        self
    }

    pub fn validate(&self) -> Result<(), TransportError> {
        self.source.validate()?;
        self.target.validate()?;
        for (name, value) in [
            ("program_translation", &self.program_translation),
            ("evidence_translation", &self.evidence_translation),
            ("observation_translation", &self.observation_translation),
            ("context_entailment", &self.context_entailment),
            ("boundary_preservation", &self.boundary_preservation),
        ] {
            if value.is_empty() {
                return Err(TransportError::new(format!("{name} must not be empty")));
            }
        }
        if self.source.semantics != self.target.semantics
            || self.source.implementation_model != self.target.implementation_model
            || self.source.environment != self.target.environment
            || self.source.tool_version != self.target.tool_version
            || self.source.semantics_version > self.target.semantics_version
            || self.source.implementation_model_version > self.target.implementation_model_version
            || self.source.fragment_revision > self.target.fragment_revision
            || self.source.procedure_version > self.target.procedure_version
            || self.source.resource_budget > self.target.resource_budget
        {
            return Err(TransportError::new(
                "transport has an invalid version/model/resource direction",
            ));
        }
        if self.receipt != self.expected_receipt() {
            return Err(TransportError::new(
                "transport receipt does not bind the canonical witness",
            ));
        }
        Ok(())
    }

    pub fn identity(coordinate: AssuranceCoordinateV2) -> Self {
        Self {
            source: coordinate.clone(),
            target: coordinate,
            program_translation: "identity".into(),
            evidence_translation: "identity".into(),
            observation_translation: "identity".into(),
            context_entailment: "identity".into(),
            boundary_preservation: "identity".into(),
            predecessor_receipts: Vec::new(),
            receipt: String::new(),
        }
        .seal()
    }

    pub fn compose(first: &Self, second: &Self) -> Result<Self, TransportError> {
        first.validate()?;
        second.validate()?;
        if first.target != second.source {
            return Err(TransportError::new(
                "transport composition has mismatched middle fiber",
            ));
        }
        let composed = Self {
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
            predecessor_receipts: vec![first.receipt.clone(), second.receipt.clone()],
            receipt: String::new(),
        };
        Ok(composed.seal())
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
    fn receipt_payload(&self) -> String {
        canonical(&(
            &self.source,
            &self.target,
            &self.reason,
            &self.explanation,
            self.source_independent,
            self.target_independent,
            self.no_transport,
        ))
    }

    fn expected_receipt(&self) -> String {
        digest(&["incompatibility", &self.receipt_payload()])
    }

    pub fn new(
        source: String,
        target: String,
        reason: IncompatibilityReasonV2,
        explanation: String,
    ) -> Self {
        let mut witness = Self {
            source,
            target,
            reason,
            explanation,
            source_independent: true,
            target_independent: true,
            no_transport: true,
            receipt: String::new(),
        };
        witness.receipt = witness.expected_receipt();
        witness
    }

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
        if self.receipt != self.expected_receipt() {
            return Err(TransportError::new(
                "incompatibility receipt does not bind the canonical witness",
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
    fn endpoints(&self) -> (String, String) {
        match self {
            Self::Transported(witness) => (witness.source.identity(), witness.target.identity()),
            Self::Incompatible(witness) => (witness.source.clone(), witness.target.clone()),
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
            if !closure.contains(&endpoints) {
                return Err(TransportError::new(
                    "relation is outside the declared predecessor closure",
                ));
            }
            if !seen.insert(endpoints) {
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

#[cfg(test)]
impl ComparisonOutcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Strengthened => "strengthened",
            Self::Weakened => "weakened",
            Self::Incomparable => "incomparable",
            Self::ChangedFiber => "changed_fiber",
            Self::MissingClassification => "missing_classification",
        }
    }
}

pub fn compare_relation(
    source: &AssuranceCoordinateV2,
    target: &AssuranceCoordinateV2,
    relation: Option<&PredecessorRelationV2>,
) -> Result<ComparisonOutcome, TransportError> {
    let Some(relation) = relation else {
        return Ok(ComparisonOutcome::MissingClassification);
    };
    relation.validate()?;
    if relation.endpoints() != (source.identity(), target.identity()) {
        return Err(TransportError::new(
            "comparison relation does not bind the supplied coordinates",
        ));
    }
    Ok(match relation {
        PredecessorRelationV2::Incompatible(_) => ComparisonOutcome::Incomparable,
        PredecessorRelationV2::Transported(_)
            if source.transport_family_key() != target.transport_family_key() =>
        {
            ComparisonOutcome::ChangedFiber
        }
        PredecessorRelationV2::Transported(_) if source.boundary != target.boundary => {
            ComparisonOutcome::Weakened
        }
        PredecessorRelationV2::Transported(_) => ComparisonOutcome::Strengthened,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct ReplayMatrix {
        case: Vec<ReplayCase>,
    }

    #[derive(Deserialize)]
    struct ReplayCase {
        id: String,
        relation: String,
        fiber: String,
        boundary: String,
        outcome: String,
    }

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

    fn transport(
        source: AssuranceCoordinateV2,
        target: AssuranceCoordinateV2,
    ) -> TransportWitnessV2 {
        TransportWitnessV2 {
            source,
            target,
            program_translation: "checked-program-translation".into(),
            evidence_translation: "checked-evidence-translation".into(),
            observation_translation: "checked-observation-translation".into(),
            context_entailment: "checked-context-entailment".into(),
            boundary_preservation: "checked-boundary-preservation".into(),
            predecessor_receipts: Vec::new(),
            receipt: String::new(),
        }
        .seal()
    }

    #[test]
    fn identity_and_composition_preserve_endpoints() {
        let first = TransportWitnessV2::identity(coordinate(1, "a"));
        let second = TransportWitnessV2::identity(coordinate(1, "a"));
        let composed = TransportWitnessV2::compose(&first, &second).unwrap();
        assert_eq!(composed.source, first.source);
        assert_eq!(composed.target, second.target);
        assert_ne!(composed.receipt, first.receipt);
        assert_eq!(
            composed.predecessor_receipts,
            vec![first.receipt.clone(), second.receipt.clone()]
        );
        composed.validate().unwrap();

        let mut tampered = composed;
        tampered.evidence_translation = "counterfeit".into();
        assert!(tampered.validate().is_err());
    }

    #[test]
    fn predecessor_domain_requires_transitive_classification() {
        let source = coordinate(1, "a");
        let middle = coordinate(2, "b");
        let target = coordinate(3, "c");
        let source_id = source.identity();
        let middle_id = middle.identity();
        let target_id = target.identity();
        let first = transport(source, middle.clone());
        let second = transport(middle, target);
        let composed = TransportWitnessV2::compose(&first, &second).unwrap();
        let missing = PredecessorDomainV2 {
            identities: vec![source_id.clone(), middle_id.clone(), target_id.clone()],
            direct_predecessors: vec![(source_id, middle_id.clone()), (middle_id, target_id)],
            relations: vec![
                PredecessorRelationV2::Transported(Box::new(first.clone())),
                PredecessorRelationV2::Transported(Box::new(second)),
            ],
        };
        assert!(missing.validate().is_err());

        let mut complete = missing;
        complete
            .relations
            .push(PredecessorRelationV2::Transported(Box::new(composed)));
        complete.validate().unwrap();

        complete
            .relations
            .push(PredecessorRelationV2::Transported(Box::new(first)));
        assert!(complete.validate().is_err());
    }

    #[test]
    fn missing_and_incompatible_relations_are_distinct() {
        let source = coordinate(1, "a");
        let target = coordinate(1, "a");
        assert_eq!(
            compare_relation(&source, &target, None).unwrap(),
            ComparisonOutcome::MissingClassification,
        );
        let target = coordinate(1, "b");
        let witness = IncompatibilityWitnessV2::new(
            source.identity(),
            target.identity(),
            IncompatibilityReasonV2::ModelBreak,
            "model semantics fork".into(),
        );
        assert_eq!(
            compare_relation(
                &source,
                &target,
                Some(&PredecessorRelationV2::Incompatible(witness))
            ),
            Ok(ComparisonOutcome::Incomparable),
        );
    }

    #[test]
    fn invalid_or_misbound_relations_cannot_classify_authority() {
        let source = coordinate(1, "a");
        let target = coordinate(2, "a");
        let mut witness = transport(source.clone(), target.clone());
        witness.receipt = "0".repeat(64);
        assert!(compare_relation(
            &source,
            &target,
            Some(&PredecessorRelationV2::Transported(Box::new(witness)))
        )
        .is_err());

        let mut changed_environment = target.clone();
        changed_environment.environment = "unproved-environment".into();
        let witness = transport(source.clone(), changed_environment);
        assert!(witness.validate().is_err());

        let other = coordinate(3, "a");
        let witness = transport(source.clone(), other);
        assert!(compare_relation(
            &source,
            &target,
            Some(&PredecessorRelationV2::Transported(Box::new(witness)))
        )
        .is_err());
    }

    #[test]
    fn comparison_outcomes_preserve_typed_coordinate_distinctions() {
        let source = coordinate(1, "a");
        let target = coordinate(2, "a");
        let relation =
            PredecessorRelationV2::Transported(Box::new(transport(source.clone(), target.clone())));
        assert_eq!(
            compare_relation(&source, &target, Some(&relation)),
            Ok(ComparisonOutcome::Strengthened),
        );

        let mut weaker_boundary = target.clone();
        weaker_boundary.boundary = "compiler-only".into();
        let relation = PredecessorRelationV2::Transported(Box::new(transport(
            source.clone(),
            weaker_boundary.clone(),
        )));
        assert_eq!(
            compare_relation(&source, &weaker_boundary, Some(&relation)),
            Ok(ComparisonOutcome::Weakened),
        );

        let mut changed_fiber = target;
        changed_fiber.evidence_identity = "translated-evidence".into();
        let relation = PredecessorRelationV2::Transported(Box::new(transport(
            source.clone(),
            changed_fiber.clone(),
        )));
        assert_eq!(
            compare_relation(&source, &changed_fiber, Some(&relation)),
            Ok(ComparisonOutcome::ChangedFiber),
        );

        let mut backwards = transport(changed_fiber, source);
        backwards.receipt = backwards.expected_receipt();
        assert!(backwards.validate().is_err());
    }

    #[test]
    fn checked_replay_matrix_is_consumed_by_rust_comparison() {
        let matrix: ReplayMatrix =
            serde_json::from_str(include_str!("../../gates/assurance-transport-replay.json"))
                .unwrap();
        assert_eq!(matrix.case.len(), 7);

        for case in matrix.case {
            let source = coordinate(1, "claim");
            let mut target = coordinate(if case.id == "identity" { 1 } else { 2 }, "claim");
            if case.fiber != "same" {
                target.evidence_identity = format!("{}-evidence", case.id);
            }
            if case.boundary == "weaker" {
                target.boundary = "compiler-only".into();
            }
            let relation = match case.relation.as_str() {
                "transported" => Some(PredecessorRelationV2::Transported(Box::new(transport(
                    source.clone(),
                    target.clone(),
                )))),
                "incompatible" => Some(PredecessorRelationV2::Incompatible(
                    IncompatibilityWitnessV2::new(
                        source.identity(),
                        target.identity(),
                        IncompatibilityReasonV2::ModelBreak,
                        format!("{} is explicitly incompatible", case.id),
                    ),
                )),
                "missing" => None,
                other => panic!("unknown matrix relation {other}"),
            };
            assert_eq!(
                compare_relation(&source, &target, relation.as_ref())
                    .unwrap()
                    .as_str(),
                case.outcome,
                "matrix case {}",
                case.id,
            );
        }
    }
}
