//! Generated-boundary replay for RFC-3 certification metatheory AC-10.

use serde::Deserialize;

use crate::manifest::{
    CertificationBoundary, CertificationPosition, CertificationScope, ClassificationCertificate,
    ClassificationVerdict, RefutationChannel, ResidualTrust,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ReplayRow {
    pub id: String,
    pub scope: String,
    pub refutation: String,
    pub residual_trust: String,
    pub discharged_trust: Vec<String>,
    pub boundary_kind: String,
    pub boundary_value: String,
    pub model_family: String,
    pub model_version: String,
    pub frame_semantics: String,
    pub frame_version: u64,
    pub residual_context: String,
    pub classification_fragment: String,
    pub classification_verdict: String,
    pub policy_point: String,
    pub engineer_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormalReplay {
    pub position: CertificationPosition,
    pub model_family: String,
    pub model_version: String,
    pub frame_semantics: String,
    pub frame_version: u64,
    pub residual_context: String,
    pub classification: ClassificationCertificate,
    pub policy_point: String,
}

#[derive(Deserialize)]
struct ReplayMatrix {
    version: u64,
    case: Vec<ReplayRow>,
}

pub fn matrix() -> Vec<ReplayRow> {
    let parsed: ReplayMatrix =
        serde_json::from_str(include_str!("../../gates/rfc3-certification-replay.json"))
            .expect("checked RFC-3 replay matrix");
    assert_eq!(parsed.version, 1, "unsupported RFC-3 replay version");
    parsed.case
}

fn scope(value: &str) -> Option<CertificationScope> {
    match value {
        "all" => Some(CertificationScope::All),
        "per_execution" => Some(CertificationScope::PerExecution),
        "none" => Some(CertificationScope::None),
        value if value.starts_with("bounded:") => Some(CertificationScope::Bounded {
            bound: value["bounded:".len()..].to_owned(),
        }),
        _ => None,
    }
}

fn refutation(value: &str) -> Option<RefutationChannel> {
    match value {
        "complete" => Some(RefutationChannel::Complete),
        "incomplete" => Some(RefutationChannel::Incomplete),
        "empirical" => Some(RefutationChannel::Empirical),
        "abort" => Some(RefutationChannel::Abort),
        "none" => Some(RefutationChannel::None),
        value if value.starts_with("trace:") => Some(RefutationChannel::Trace {
            bound: value["trace:".len()..].to_owned(),
        }),
        _ => None,
    }
}

impl ReplayRow {
    pub fn formal(&self) -> Option<FormalReplay> {
        let residual_trust = match self.residual_trust.as_str() {
            "lean_checked" => ResidualTrust::LeanChecked,
            "solver" => ResidualTrust::Solver,
            "fiat" => ResidualTrust::Fiat,
            _ => return None,
        };
        let boundary = match self.boundary_kind.as_str() {
            "end_to_end" if self.boundary_value.is_empty() => CertificationBoundary::EndToEnd,
            "to_boundary" if !self.boundary_value.is_empty() => CertificationBoundary::ToBoundary {
                via: self.boundary_value.clone(),
            },
            "to_platform" if !self.boundary_value.is_empty() => CertificationBoundary::ToPlatform {
                platform: self.boundary_value.clone(),
            },
            _ => return None,
        };
        let position = CertificationPosition {
            scope: scope(&self.scope)?,
            refutation: refutation(&self.refutation)?,
            residual_trust,
            discharged_trust: self.discharged_trust.clone(),
            boundary,
        };
        position.validate().ok()?;
        let verdict = match self.classification_verdict.as_str() {
            "admitted" => ClassificationVerdict::Admitted,
            "rejected" => ClassificationVerdict::Rejected {
                reason: "matrix".into(),
            },
            "unknown" => ClassificationVerdict::Unknown {
                reason: "matrix".into(),
            },
            _ => return None,
        };
        Some(FormalReplay {
            position,
            model_family: self.model_family.clone(),
            model_version: self.model_version.clone(),
            frame_semantics: self.frame_semantics.clone(),
            frame_version: self.frame_version,
            residual_context: self.residual_context.clone(),
            classification: ClassificationCertificate {
                fragment: self.classification_fragment.clone(),
                verdict,
            },
            policy_point: self.policy_point.clone(),
        })
    }
}

/// Replay compares only authoritative formal data. Engineer labels are absent
/// from `FormalReplay` and therefore cannot drive admission.
pub fn replays_as(candidate: &ReplayRow, expected: &ReplayRow) -> bool {
    match (candidate.formal(), expected.formal()) {
        (Some(candidate), Some(expected)) => candidate == expected,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn generated_matrix_covers_authoritative_vocabulary() {
        let rows = matrix();
        assert_eq!(rows.len(), 7);
        assert!(rows.iter().all(|row| row.formal().is_some()));
        let values =
            |field: fn(&ReplayRow) -> &str| rows.iter().map(field).collect::<BTreeSet<_>>();
        assert_eq!(
            values(|r| &r.boundary_kind),
            BTreeSet::from(["end_to_end", "to_boundary", "to_platform"])
        );
        assert_eq!(
            values(|r| &r.classification_verdict),
            BTreeSet::from(["admitted", "rejected", "unknown"])
        );
        assert_eq!(
            values(|r| &r.policy_point),
            BTreeSet::from(["bounded", "lean_empirical", "runtime", "solver_complete"])
        );
    }

    #[test]
    fn every_authoritative_field_is_replay_load_bearing() {
        let expected = matrix()
            .into_iter()
            .find(|r| r.id == "complete_solver")
            .unwrap();
        let mut mutations: Vec<ReplayRow> = Vec::new();
        macro_rules! mutated {
            ($field:ident, $value:expr) => {{
                let mut r = expected.clone();
                r.$field = $value;
                mutations.push(r);
            }};
        }
        mutated!(scope, "none".into());
        mutated!(refutation, "incomplete".into());
        mutated!(residual_trust, "fiat".into());
        mutated!(discharged_trust, vec!["engineer-label".into()]);
        mutated!(boundary_kind, "to_boundary".into());
        mutated!(boundary_value, "engineer-label".into());
        mutated!(model_family, "engineer-label".into());
        mutated!(model_version, "1.96.0".into());
        mutated!(frame_semantics, "engineer-label".into());
        mutated!(frame_version, 2);
        mutated!(residual_context, "engineer-label".into());
        mutated!(classification_fragment, "engineer-label".into());
        mutated!(classification_verdict, "unknown".into());
        mutated!(policy_point, "lean_empirical".into());
        assert!(mutations.iter().all(|row| !replays_as(row, &expected)));
    }

    #[test]
    fn engineer_label_is_structurally_non_authoritative() {
        let expected = matrix()
            .into_iter()
            .find(|r| r.id == "complete_solver")
            .unwrap();
        let mut relabeled = expected.clone();
        relabeled.engineer_label = "anything presentational".into();
        assert!(replays_as(&relabeled, &expected));
        relabeled.scope = relabeled.engineer_label.clone();
        assert!(!replays_as(&relabeled, &expected));
    }
}
