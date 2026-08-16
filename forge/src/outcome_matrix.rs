//! Closed, stage-indexed operational classification for issue #48 AC-7.
//!
//! This is deliberately not an assurance ladder. It records why one attempted
//! stage stopped, without changing semantic-fragment membership or silently
//! selecting a lower certification claim.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Parser,
    Validator,
    CanonicalSemantics,
    CheckedIr,
    Lowering,
    ProofRoute,
    Policy,
    Certification,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeClass {
    Success,
    UnsupportedLanguage,
    InvalidSource,
    UnsupportedPolicy,
    ResourceExhausted,
    ToolUnavailable,
    ToolIncompatible,
    Counterexample,
    ProofFailure,
    SoundnessAlarm,
}

/// Facts returned by a concrete stage adapter. Exactly one terminal fact may
/// be set; contradictory adapters are classified as a soundness alarm.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AttemptFacts {
    #[serde(default)]
    pub unsupported_language: bool,
    #[serde(default)]
    pub invalid_source: bool,
    #[serde(default)]
    pub unsupported_policy: bool,
    #[serde(default)]
    pub resource_exhausted: bool,
    #[serde(default)]
    pub tool_unavailable: bool,
    #[serde(default)]
    pub tool_incompatible: bool,
    #[serde(default)]
    pub counterexample: bool,
    #[serde(default)]
    pub proof_failure: bool,
    #[serde(default)]
    pub soundness_alarm: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageAttempt {
    pub stage: Stage,
    pub program: String,
    pub facts: AttemptFacts,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageOutcome {
    pub stage: Stage,
    pub program: String,
    pub class: OutcomeClass,
}

/// Whether the representative program belongs to the semantic fragment being
/// attempted. Solver progress is recorded alongside this fact and never
/// rewrites it: an unavailable solver does not make a program unsupported.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentMembership {
    Admitted,
    Excluded,
}

/// Environment facts checked before a solver is invoked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SolverEnvironment {
    Available,
    Unavailable { tool: String },
    Incompatible { tool: String, version: String },
}

/// A solver route claims progress/classification, not solver completeness.
/// `Unknown` is intentionally distinct from resource exhaustion and from a
/// proof failure that the route can diagnose.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SolverObservation {
    Success,
    Timeout { resource: String },
    Unknown { detail: String },
    Counterexample { witness: String },
    ProofFailure { detail: String },
    SoundnessAlarm { detail: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SolverProgressClass {
    Success,
    Timeout,
    Unknown,
    ToolUnavailable,
    ToolIncompatible,
    Counterexample,
    ProofFailure,
    SoundnessAlarm,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SolverRouteOutcome {
    pub program: String,
    pub membership: FragmentMembership,
    pub class: SolverProgressClass,
}

/// Total solver-route classifier under explicit environment and resource
/// observations. The input membership is copied unchanged into every result.
pub fn classify_solver_route(
    program: String,
    membership: FragmentMembership,
    environment: SolverEnvironment,
    observation: Option<SolverObservation>,
) -> SolverRouteOutcome {
    let class = match (environment, observation) {
        (SolverEnvironment::Unavailable { .. }, None) => SolverProgressClass::ToolUnavailable,
        (SolverEnvironment::Incompatible { .. }, None) => SolverProgressClass::ToolIncompatible,
        (SolverEnvironment::Available, Some(SolverObservation::Success)) => {
            SolverProgressClass::Success
        }
        (SolverEnvironment::Available, Some(SolverObservation::Timeout { .. })) => {
            SolverProgressClass::Timeout
        }
        (SolverEnvironment::Available, Some(SolverObservation::Unknown { .. })) => {
            SolverProgressClass::Unknown
        }
        (SolverEnvironment::Available, Some(SolverObservation::Counterexample { .. })) => {
            SolverProgressClass::Counterexample
        }
        (SolverEnvironment::Available, Some(SolverObservation::ProofFailure { .. })) => {
            SolverProgressClass::ProofFailure
        }
        (SolverEnvironment::Available, Some(SolverObservation::SoundnessAlarm { .. }))
        | (SolverEnvironment::Available, None)
        | (SolverEnvironment::Unavailable { .. }, Some(_))
        | (SolverEnvironment::Incompatible { .. }, Some(_)) => SolverProgressClass::SoundnessAlarm,
    };
    SolverRouteOutcome {
        program,
        membership,
        class,
    }
}

/// Total classifier shared by every stage adapter. No branch degrades to a
/// different stage or assurance level. An all-false adapter result is success.
pub fn classify(facts: &AttemptFacts) -> OutcomeClass {
    let terminals = [
        facts.unsupported_language,
        facts.invalid_source,
        facts.unsupported_policy,
        facts.resource_exhausted,
        facts.tool_unavailable,
        facts.tool_incompatible,
        facts.counterexample,
        facts.proof_failure,
        facts.soundness_alarm,
    ];
    if terminals.iter().filter(|terminal| **terminal).count() > 1 || facts.soundness_alarm {
        return OutcomeClass::SoundnessAlarm;
    }
    if facts.unsupported_language {
        OutcomeClass::UnsupportedLanguage
    } else if facts.invalid_source {
        OutcomeClass::InvalidSource
    } else if facts.unsupported_policy {
        OutcomeClass::UnsupportedPolicy
    } else if facts.resource_exhausted {
        OutcomeClass::ResourceExhausted
    } else if facts.tool_unavailable {
        OutcomeClass::ToolUnavailable
    } else if facts.tool_incompatible {
        OutcomeClass::ToolIncompatible
    } else if facts.counterexample {
        OutcomeClass::Counterexample
    } else if facts.proof_failure {
        OutcomeClass::ProofFailure
    } else {
        OutcomeClass::Success
    }
}

/// Preserve the attempted stage and representative program alongside the
/// terminal class. Consumers cannot mistake a proof-route stop for a parser or
/// certification result merely because both share one outcome vocabulary.
pub fn classify_attempt(attempt: StageAttempt) -> StageOutcome {
    StageOutcome {
        stage: attempt.stage,
        program: attempt.program,
        class: classify(&attempt.facts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct Matrix {
        case: Vec<Case>,
    }

    #[derive(Debug, Deserialize)]
    struct Case {
        id: String,
        stage: Stage,
        program: String,
        expected: OutcomeClass,
        #[serde(default)]
        facts: AttemptFacts,
    }

    #[test]
    fn generated_matrix_drives_the_total_classifier() {
        let matrix: Matrix =
            serde_json::from_str(include_str!("../../gates/language-outcome-matrix.json"))
                .expect("checked outcome matrix must deserialize");
        for case in matrix.case {
            assert!(!case.id.is_empty() && !case.program.is_empty());
            let outcome = classify_attempt(StageAttempt {
                stage: case.stage,
                program: case.program,
                facts: case.facts,
            });
            assert_eq!(outcome.class, case.expected, "{}", case.id);
            assert_eq!(outcome.stage, case.stage, "{} changed stage", case.id);
        }
    }

    #[test]
    fn contradictory_adapter_facts_fail_closed() {
        let facts = AttemptFacts {
            resource_exhausted: true,
            counterexample: true,
            ..AttemptFacts::default()
        };
        assert_eq!(classify(&facts), OutcomeClass::SoundnessAlarm);
    }

    #[test]
    fn solver_progress_cases_do_not_relabel_fragment_membership() {
        let cases = [
            (
                SolverEnvironment::Available,
                Some(SolverObservation::Success),
                SolverProgressClass::Success,
            ),
            (
                SolverEnvironment::Available,
                Some(SolverObservation::Timeout {
                    resource: "rlimit".to_string(),
                }),
                SolverProgressClass::Timeout,
            ),
            (
                SolverEnvironment::Available,
                Some(SolverObservation::Unknown {
                    detail: "solver returned unknown".to_string(),
                }),
                SolverProgressClass::Unknown,
            ),
            (
                SolverEnvironment::Unavailable {
                    tool: "z3".to_string(),
                },
                None,
                SolverProgressClass::ToolUnavailable,
            ),
            (
                SolverEnvironment::Incompatible {
                    tool: "z3".to_string(),
                    version: "unsupported".to_string(),
                },
                None,
                SolverProgressClass::ToolIncompatible,
            ),
            (
                SolverEnvironment::Available,
                Some(SolverObservation::Counterexample {
                    witness: "x = 0".to_string(),
                }),
                SolverProgressClass::Counterexample,
            ),
            (
                SolverEnvironment::Available,
                Some(SolverObservation::ProofFailure {
                    detail: "residual goal".to_string(),
                }),
                SolverProgressClass::ProofFailure,
            ),
            (
                SolverEnvironment::Available,
                Some(SolverObservation::SoundnessAlarm {
                    detail: "adapter contradiction".to_string(),
                }),
                SolverProgressClass::SoundnessAlarm,
            ),
        ];

        for membership in [FragmentMembership::Admitted, FragmentMembership::Excluded] {
            for (environment, observation, expected) in &cases {
                let outcome = classify_solver_route(
                    "representative-program".to_string(),
                    membership,
                    environment.clone(),
                    observation.clone(),
                );
                assert_eq!(outcome.class, *expected);
                assert_eq!(outcome.membership, membership);
            }
        }

        assert_eq!(
            classify_solver_route(
                "contradictory-adapter".to_string(),
                FragmentMembership::Admitted,
                SolverEnvironment::Unavailable {
                    tool: "z3".to_string(),
                },
                Some(SolverObservation::Success),
            )
            .class,
            SolverProgressClass::SoundnessAlarm
        );
    }
}
