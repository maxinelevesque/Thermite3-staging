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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// The deliberately finite contract-quality policy fragment. Integer ratios
/// avoid importing floating-point edge cases into the completeness claim.
pub const POLICY_MUTANT_CAP: u16 = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyInput {
    pub mutants_killed: u16,
    pub mutants_scored: u16,
    pub floor_numerator: u16,
    pub floor_denominator: u16,
    pub tautology: bool,
    pub vacuous_precondition: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyDecision {
    Accepted,
    RejectedTautology,
    RejectedVacuousPrecondition,
    RejectedWeakContract,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyClass {
    Decided(PolicyDecision),
    UnsupportedPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyOutcome {
    pub class: PolicyClass,
    /// Policy incompleteness is not semantic exclusion.
    pub membership: FragmentMembership,
    /// An unsupported policy does not suppress non-policy proof analysis.
    pub proof_analysis_eligible: bool,
}

/// Membership in the finite policy domain. Every boundary is inclusive where
/// it represents a valid ratio: 1..=64 scored mutants, killed <= scored, and a
/// rational floor with 0 <= numerator <= denominator and denominator > 0.
pub fn in_finite_policy_fragment(input: PolicyInput) -> bool {
    (1..=POLICY_MUTANT_CAP).contains(&input.mutants_scored)
        && input.mutants_killed <= input.mutants_scored
        && input.floor_denominator > 0
        && input.floor_numerator <= input.floor_denominator
}

/// Total decision over the finite fragment. Cross multiplication is exact and
/// safe because every input is a bounded `u16` widened before multiplication.
pub fn classify_policy(input: PolicyInput, membership: FragmentMembership) -> PolicyOutcome {
    let class = if !in_finite_policy_fragment(input) {
        PolicyClass::UnsupportedPolicy
    } else if input.tautology {
        PolicyClass::Decided(PolicyDecision::RejectedTautology)
    } else if input.vacuous_precondition {
        PolicyClass::Decided(PolicyDecision::RejectedVacuousPrecondition)
    } else if u32::from(input.mutants_killed) * u32::from(input.floor_denominator)
        < u32::from(input.floor_numerator) * u32::from(input.mutants_scored)
    {
        PolicyClass::Decided(PolicyDecision::RejectedWeakContract)
    } else {
        PolicyClass::Decided(PolicyDecision::Accepted)
    };
    PolicyOutcome {
        class,
        membership,
        proof_analysis_eligible: true,
    }
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

    #[test]
    fn finite_policy_boundary_mutations_are_pinned() {
        let base = PolicyInput {
            mutants_killed: 3,
            mutants_scored: 5,
            floor_numerator: 3,
            floor_denominator: 5,
            tautology: false,
            vacuous_precondition: false,
        };
        let classify = |input| classify_policy(input, FragmentMembership::Admitted).class;

        assert_eq!(
            classify(base),
            PolicyClass::Decided(PolicyDecision::Accepted)
        );
        assert_eq!(
            classify(PolicyInput {
                mutants_killed: 2,
                ..base
            }),
            PolicyClass::Decided(PolicyDecision::RejectedWeakContract)
        );
        assert_eq!(
            classify(PolicyInput {
                tautology: true,
                ..base
            }),
            PolicyClass::Decided(PolicyDecision::RejectedTautology)
        );
        assert_eq!(
            classify(PolicyInput {
                vacuous_precondition: true,
                ..base
            }),
            PolicyClass::Decided(PolicyDecision::RejectedVacuousPrecondition)
        );

        for outside in [
            PolicyInput {
                mutants_scored: 0,
                mutants_killed: 0,
                ..base
            },
            PolicyInput {
                mutants_scored: POLICY_MUTANT_CAP + 1,
                ..base
            },
            PolicyInput {
                mutants_killed: 6,
                ..base
            },
            PolicyInput {
                floor_denominator: 0,
                ..base
            },
            PolicyInput {
                floor_numerator: 6,
                ..base
            },
        ] {
            assert_eq!(classify(outside), PolicyClass::UnsupportedPolicy);
        }

        assert_eq!(
            classify(PolicyInput {
                mutants_killed: POLICY_MUTANT_CAP,
                mutants_scored: POLICY_MUTANT_CAP,
                floor_numerator: 1,
                floor_denominator: 1,
                ..base
            }),
            PolicyClass::Decided(PolicyDecision::Accepted)
        );
    }

    #[test]
    fn unsupported_policy_preserves_non_policy_analysis() {
        for membership in [FragmentMembership::Admitted, FragmentMembership::Excluded] {
            let outcome = classify_policy(
                PolicyInput {
                    mutants_killed: 0,
                    mutants_scored: POLICY_MUTANT_CAP + 1,
                    floor_numerator: 3,
                    floor_denominator: 5,
                    tautology: false,
                    vacuous_precondition: false,
                },
                membership,
            );
            assert_eq!(outcome.class, PolicyClass::UnsupportedPolicy);
            assert_eq!(outcome.membership, membership);
            assert!(outcome.proof_analysis_eligible);
        }
    }
}
