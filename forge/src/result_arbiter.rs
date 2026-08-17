//! Typed combination of proof-engine evidence and settled policy outcomes.
//!
//! A `Certificate` is the public rendering, not the transition policy.  This
//! module is the one place that decides whether later evidence may replace an
//! earlier result (`.design/forge-result-arbiter.md`).

use crate::engine::{Counterexample, Disagreement};
use crate::manifest::{Certificate, LiveResultDisposition, ObligationStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InconclusiveReason {
    VerusTimeout,
    TimeoutDegrade,
    EngineUnknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PolicyRejection {
    WeakContract,
    SemanticTautology,
    VacuousPrecondition,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BaseDisposition {
    Accepted,
    Inconclusive(InconclusiveReason),
    Refuted,
    PolicyRejected(PolicyRejection),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ItemOutcome {
    certificate: Certificate,
    disposition: BaseDisposition,
    engine: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PersistedOutcomeError {
    pub(crate) item: String,
    pub(crate) detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CombinationError {
    Disagreement(Disagreement),
    InvalidEvidence(PersistedOutcomeError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProofCandidate {
    Complete {
        engine: String,
        certificate: Certificate,
    },
    Refuted {
        engine: String,
        certificate: Certificate,
    },
    Inconclusive {
        engine: String,
        certificate: Certificate,
    },
    Partial,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PolicyDecision {
    Accepted(Certificate),
    Rejected {
        kind: PolicyRejection,
        certificate: Certificate,
    },
}

impl ItemOutcome {
    pub(crate) fn from_policy(decision: PolicyDecision, engine: impl Into<String>) -> Self {
        let engine = engine.into();
        match decision {
            PolicyDecision::Accepted(certificate) => Self::accepted(certificate, engine),
            PolicyDecision::Rejected { kind, certificate } => {
                Self::policy_rejected(certificate, engine, kind)
            }
        }
    }

    pub(crate) fn accepted(certificate: Certificate, engine: impl Into<String>) -> Self {
        Self {
            certificate: certificate.with_live_disposition(LiveResultDisposition::Accepted),
            disposition: BaseDisposition::Accepted,
            engine: engine.into(),
        }
    }

    pub(crate) fn inconclusive(
        certificate: Certificate,
        engine: impl Into<String>,
        reason: InconclusiveReason,
    ) -> Self {
        let live = match &reason {
            InconclusiveReason::VerusTimeout => LiveResultDisposition::VerusTimeout,
            InconclusiveReason::TimeoutDegrade => LiveResultDisposition::TimeoutDegrade,
            InconclusiveReason::EngineUnknown => LiveResultDisposition::EngineUnknown,
        };
        Self {
            certificate: certificate.with_live_disposition(live),
            disposition: BaseDisposition::Inconclusive(reason),
            engine: engine.into(),
        }
    }

    pub(crate) fn refuted(certificate: Certificate, engine: impl Into<String>) -> Self {
        Self {
            certificate: certificate.with_live_disposition(LiveResultDisposition::Refuted),
            disposition: BaseDisposition::Refuted,
            engine: engine.into(),
        }
    }

    pub(crate) fn policy_rejected(
        certificate: Certificate,
        engine: impl Into<String>,
        kind: PolicyRejection,
    ) -> Self {
        let live = match &kind {
            PolicyRejection::WeakContract => LiveResultDisposition::WeakContract,
            PolicyRejection::SemanticTautology => LiveResultDisposition::SemanticTautology,
            PolicyRejection::VacuousPrecondition => LiveResultDisposition::VacuousPrecondition,
            PolicyRejection::Other(detail) => LiveResultDisposition::SettledOther(detail.clone()),
        };
        Self {
            certificate: certificate.with_live_disposition(live),
            disposition: BaseDisposition::PolicyRejected(kind),
            engine: engine.into(),
        }
    }

    /// Recover a live producer result from its non-serialized typed stamp. Only
    /// genuinely deserialized/legacy values fall back to the structural adapter.
    pub(crate) fn from_certificate(
        certificate: Certificate,
    ) -> Result<Self, PersistedOutcomeError> {
        let Some(disposition) = certificate.live_disposition().cloned() else {
            return Self::from_persisted_certificate(certificate);
        };
        validate_live_shape(&certificate, &disposition)?;
        let engine = certificate
            .engine_attribution
            .as_ref()
            .map_or_else(|| "verus".to_string(), |a| a.engine.clone());
        Ok(match disposition {
            LiveResultDisposition::Accepted => Self::accepted(certificate, engine),
            LiveResultDisposition::VerusTimeout => {
                Self::inconclusive(certificate, engine, InconclusiveReason::VerusTimeout)
            }
            LiveResultDisposition::TimeoutDegrade => {
                Self::inconclusive(certificate, engine, InconclusiveReason::TimeoutDegrade)
            }
            LiveResultDisposition::EngineUnknown => {
                Self::inconclusive(certificate, engine, InconclusiveReason::EngineUnknown)
            }
            LiveResultDisposition::Refuted => Self::refuted(certificate, engine),
            LiveResultDisposition::WeakContract => {
                Self::policy_rejected(certificate, engine, PolicyRejection::WeakContract)
            }
            LiveResultDisposition::SemanticTautology => {
                Self::policy_rejected(certificate, engine, PolicyRejection::SemanticTautology)
            }
            LiveResultDisposition::VacuousPrecondition => {
                Self::policy_rejected(certificate, engine, PolicyRejection::VacuousPrecondition)
            }
            LiveResultDisposition::SettledOther(detail) => {
                Self::policy_rejected(certificate, engine, PolicyRejection::Other(detail))
            }
        })
    }

    /// Decode the unchanged public certificate shape at cache/legacy seams.
    /// Fresh producers use the typed constructors above.  Reject causes are
    /// interpreted only here, together with level, failed obligations, and the
    /// lowered-assurance marker.
    pub(crate) fn from_persisted_certificate(
        certificate: Certificate,
    ) -> Result<Self, PersistedOutcomeError> {
        let engine = certificate
            .engine_attribution
            .as_ref()
            .map_or_else(|| "verus".to_string(), |a| a.engine.clone());
        let has_failed = certificate
            .obligations
            .iter()
            .any(|obligation| obligation.status == ObligationStatus::Failed);
        if matches!(
            certificate.level,
            crate::manifest::Level::L3 | crate::manifest::Level::L4
        ) && has_failed
        {
            return Err(PersistedOutcomeError {
                item: certificate.item.clone(),
                detail: "certifying level carries a failed obligation".into(),
            });
        }
        if certificate.lowered_assurance {
            if certificate.reject.is_some() || certificate.degrade_reason.is_none() {
                return Err(PersistedOutcomeError {
                    item: certificate.item.clone(),
                    detail: "lowered assurance must carry a degrade reason and no terminal reject"
                        .into(),
                });
            }
            return Ok(Self::inconclusive(
                certificate,
                engine,
                InconclusiveReason::TimeoutDegrade,
            ));
        }
        if let Some(reject) = certificate.reject.as_ref() {
            if certificate.level != crate::manifest::Level::L0 {
                return Err(PersistedOutcomeError {
                    item: certificate.item.clone(),
                    detail: "terminal reject is not at L0".into(),
                });
            }
            if !has_failed {
                return Err(PersistedOutcomeError {
                    item: certificate.item.clone(),
                    detail: "terminal reject lacks a failed obligation".into(),
                });
            }
            let disposition = match reject.cause.as_str() {
                "VerusTimeout" => BaseDisposition::Inconclusive(InconclusiveReason::VerusTimeout),
                "VerusIncompleteUnknown" => {
                    BaseDisposition::Inconclusive(InconclusiveReason::EngineUnknown)
                }
                "WeakContract" => BaseDisposition::PolicyRejected(PolicyRejection::WeakContract),
                "SemanticTautology" => {
                    if !certificate.contract_quality.tautology {
                        return Err(PersistedOutcomeError {
                            item: certificate.item.clone(),
                            detail: "SemanticTautology lacks its contract-quality fact".into(),
                        });
                    }
                    BaseDisposition::PolicyRejected(PolicyRejection::SemanticTautology)
                }
                "VacuousPrecondition" => {
                    if !certificate.contract_quality.vacuous_precondition {
                        return Err(PersistedOutcomeError {
                            item: certificate.item.clone(),
                            detail: "VacuousPrecondition lacks its contract-quality fact".into(),
                        });
                    }
                    BaseDisposition::PolicyRejected(PolicyRejection::VacuousPrecondition)
                }
                other => BaseDisposition::PolicyRejected(PolicyRejection::Other(other.into())),
            };
            return Ok(Self {
                certificate,
                disposition,
                engine,
            });
        }
        if has_failed {
            return Ok(Self::refuted(certificate, engine));
        }
        if matches!(
            certificate.level,
            crate::manifest::Level::L3 | crate::manifest::Level::L4
        ) {
            if certificate.contract_quality.tautology
                || certificate.contract_quality.vacuous_precondition
            {
                return Err(PersistedOutcomeError {
                    item: certificate.item.clone(),
                    detail: "accepted proof carries a rejecting vacuity-policy fact".into(),
                });
            }
            if mutation_ratio_below_floor(&certificate.contract_quality.mutants_killed) {
                return Err(PersistedOutcomeError {
                    item: certificate.item.clone(),
                    detail: "accepted proof carries rejecting mutation-policy evidence".into(),
                });
            }
            return Ok(Self::accepted(certificate, engine));
        }
        // Non-upgradeable structural/legacy outcomes are retained as settled
        // rejections.  Only an explicit timeout/degrade is inconclusive.
        Ok(Self::policy_rejected(
            certificate,
            engine,
            PolicyRejection::Other("SettledNonProofOutcome".into()),
        ))
    }

    pub(crate) fn disposition(&self) -> &BaseDisposition {
        &self.disposition
    }

    pub(crate) fn needs_fallback(&self) -> bool {
        matches!(self.disposition, BaseDisposition::Inconclusive(_))
    }

    pub(crate) fn certificate(&self) -> &Certificate {
        &self.certificate
    }

    pub(crate) fn into_certificate(self) -> Certificate {
        self.certificate
    }

    pub(crate) fn render_settled_rejection(
        &self,
        kind: PolicyRejection,
        certificate: Certificate,
    ) -> Certificate {
        Self::policy_rejected(
            render_policy_result(&self.certificate, certificate),
            self.engine.clone(),
            kind,
        )
        .into_certificate()
    }

    /// Apply a policy producer without letting later proof evidence erase a
    /// rejection. Accepted policy evidence updates the accepted rendering;
    /// rejected policy evidence settles the item.
    pub(crate) fn apply_policy(
        self,
        decision: PolicyDecision,
    ) -> Result<Self, PersistedOutcomeError> {
        let result = match decision {
            PolicyDecision::Accepted(certificate) => {
                if matches!(self.disposition, BaseDisposition::PolicyRejected(_)) {
                    return Ok(self);
                }
                ensure_same_item(&self.certificate, &certificate)?;
                validate_live_shape(&certificate, &LiveResultDisposition::Accepted)?;
                let certificate = render_policy_result(&self.certificate, certificate);
                Self::accepted(certificate, self.engine)
            }
            PolicyDecision::Rejected { kind, certificate } => {
                ensure_same_item(&self.certificate, &certificate)?;
                validate_live_shape(&certificate, &policy_live_disposition(&kind))?;
                let certificate = render_policy_result(&self.certificate, certificate);
                Self::policy_rejected(certificate, self.engine, kind)
            }
        };
        Ok(result)
    }

    /// Combine optional/supplemental evidence (automatic Lean and EPR).
    pub(crate) fn combine(self, candidate: ProofCandidate) -> Result<Self, CombinationError> {
        match candidate {
            ProofCandidate::Partial | ProofCandidate::Unavailable => Ok(self),
            ProofCandidate::Inconclusive { .. } => Ok(self),
            ProofCandidate::Complete {
                engine,
                certificate,
            } => {
                ensure_same_item(&self.certificate, &certificate)
                    .map_err(CombinationError::InvalidEvidence)?;
                validate_live_shape(&certificate, &LiveResultDisposition::Accepted)
                    .map_err(CombinationError::InvalidEvidence)?;
                match &self.disposition {
                    BaseDisposition::Refuted => Err(CombinationError::Disagreement(
                        self.disagreement(engine, certificate, false),
                    )),
                    BaseDisposition::PolicyRejected(_) => Ok(self),
                    BaseDisposition::Accepted | BaseDisposition::Inconclusive(_) => {
                        let keep_policy = matches!(&self.disposition, BaseDisposition::Accepted);
                        let certificate =
                            render_replacement(&self.certificate, certificate, keep_policy);
                        Ok(Self::accepted(certificate, engine))
                    }
                }
            }
            ProofCandidate::Refuted {
                engine,
                certificate,
            } => {
                ensure_same_item(&self.certificate, &certificate)
                    .map_err(CombinationError::InvalidEvidence)?;
                validate_live_shape(&certificate, &LiveResultDisposition::Refuted)
                    .map_err(CombinationError::InvalidEvidence)?;
                match &self.disposition {
                    BaseDisposition::Accepted => Err(CombinationError::Disagreement(
                        self.disagreement(engine, certificate, true),
                    )),
                    BaseDisposition::PolicyRejected(_) | BaseDisposition::Refuted => Ok(self),
                    BaseDisposition::Inconclusive(_) => {
                        let certificate = render_replacement(&self.certificate, certificate, false);
                        Ok(Self::refuted(certificate, engine))
                    }
                }
            }
        }
    }

    /// Select a requested engine while retaining settled policy gates. Unknown
    /// selected-engine output replaces a non-policy base for honest diagnostics;
    /// proof/refutation contradictions still alarm because both engines ran.
    pub(crate) fn select(self, candidate: ProofCandidate) -> Result<Self, CombinationError> {
        if matches!(&self.disposition, BaseDisposition::PolicyRejected(_)) {
            return Ok(self);
        }
        match candidate {
            ProofCandidate::Inconclusive {
                engine,
                certificate,
            } => {
                ensure_same_item(&self.certificate, &certificate)
                    .map_err(CombinationError::InvalidEvidence)?;
                validate_live_shape(&certificate, &LiveResultDisposition::EngineUnknown)
                    .map_err(CombinationError::InvalidEvidence)?;
                Ok(Self::inconclusive(
                    render_replacement(&self.certificate, certificate, false),
                    engine,
                    InconclusiveReason::EngineUnknown,
                ))
            }
            other => self.combine(other),
        }
    }

    fn disagreement(
        &self,
        candidate_engine: String,
        candidate: Certificate,
        candidate_refuted: bool,
    ) -> Disagreement {
        let counterexample = if candidate_refuted {
            Counterexample {
                obligations: candidate.obligations,
            }
        } else {
            Counterexample {
                obligations: self.certificate.obligations.clone(),
            }
        };
        let (proven_engine, refuted_engine) = if candidate_refuted {
            (self.engine.clone(), candidate_engine)
        } else {
            (candidate_engine, self.engine.clone())
        };
        Disagreement {
            proven_engine,
            refuted_engine,
            item: self.certificate.item.clone(),
            counterexample,
        }
    }
}

fn policy_live_disposition(kind: &PolicyRejection) -> LiveResultDisposition {
    match kind {
        PolicyRejection::WeakContract => LiveResultDisposition::WeakContract,
        PolicyRejection::SemanticTautology => LiveResultDisposition::SemanticTautology,
        PolicyRejection::VacuousPrecondition => LiveResultDisposition::VacuousPrecondition,
        PolicyRejection::Other(detail) => LiveResultDisposition::SettledOther(detail.clone()),
    }
}

fn validate_live_shape(
    certificate: &Certificate,
    disposition: &LiveResultDisposition,
) -> Result<(), PersistedOutcomeError> {
    let failed = certificate
        .obligations
        .iter()
        .any(|obligation| obligation.status == ObligationStatus::Failed);
    let valid = match disposition {
        LiveResultDisposition::Accepted => {
            matches!(
                certificate.level,
                crate::manifest::Level::L3 | crate::manifest::Level::L4
            ) && certificate.reject.is_none()
                && !certificate.lowered_assurance
                && !failed
        }
        LiveResultDisposition::VerusTimeout | LiveResultDisposition::EngineUnknown => {
            certificate.level == crate::manifest::Level::L0
                && certificate.reject.is_some()
                && failed
        }
        LiveResultDisposition::TimeoutDegrade => {
            certificate.lowered_assurance
                && certificate.degrade_reason.is_some()
                && certificate.reject.is_none()
        }
        LiveResultDisposition::Refuted => certificate.level == crate::manifest::Level::L0 && failed,
        LiveResultDisposition::WeakContract
        | LiveResultDisposition::SemanticTautology
        | LiveResultDisposition::VacuousPrecondition => {
            certificate.level == crate::manifest::Level::L0
                && certificate.reject.is_some()
                && failed
        }
        LiveResultDisposition::SettledOther(_) => true,
    };
    if valid {
        Ok(())
    } else {
        Err(PersistedOutcomeError {
            item: certificate.item.clone(),
            detail: format!(
                "live typed disposition {disposition:?} contradicts its certificate rendering"
            ),
        })
    }
}

fn ensure_same_item(
    base: &Certificate,
    candidate: &Certificate,
) -> Result<(), PersistedOutcomeError> {
    if base.item == candidate.item && base.effects == candidate.effects {
        return Ok(());
    }
    Err(PersistedOutcomeError {
        item: base.item.clone(),
        detail: format!(
            "candidate identity mismatch: base `{}` {:?}, candidate `{}` {:?}",
            base.item, base.effects, candidate.item, candidate.effects
        ),
    })
}

fn mutation_ratio_below_floor(score: &str) -> bool {
    if score == "0/0" {
        return false;
    }
    let Some((killed, total)) = score.split_once('/') else {
        return true;
    };
    let (Ok(killed), Ok(total)) = (killed.parse::<usize>(), total.parse::<usize>()) else {
        return true;
    };
    total == 0 || killed > total || (killed as f64 / total as f64) < crate::mutation::MUTATION_FLOOR
}

fn render_replacement(
    base: &Certificate,
    mut replacement: Certificate,
    preserve_base_policy: bool,
) -> Certificate {
    if preserve_base_policy {
        replacement.contract_quality = base.contract_quality.clone();
        replacement.strengthening = base.strengthening.clone();
        if replacement.suggested_move.is_none() {
            replacement.suggested_move = base.suggested_move.clone();
        }
    }
    if let Some(scope) = base.assurance_scope.clone() {
        replacement = replacement.with_assurance_scope(scope);
    }
    // These blocks describe the checked source and the policy evidence admitted
    // before backend selection.  A later proof engine may replace the proof
    // payload, but it must not erase that independent evidence.
    replacement.covenant_evidence = base.covenant_evidence;
    replacement.meaning_audit = base.meaning_audit.clone();
    replacement
}

fn render_policy_result(base: &Certificate, mut replacement: Certificate) -> Certificate {
    if replacement.strengthening.is_empty() {
        replacement.strengthening = base.strengthening.clone();
    }
    if replacement.suggested_move.is_none() {
        replacement.suggested_move = base.suggested_move.clone();
    }
    if let Some(scope) = base.assurance_scope.clone() {
        replacement = replacement.with_assurance_scope(scope);
    }
    replacement.covenant_evidence = base.covenant_evidence;
    replacement.meaning_audit = base.meaning_audit.clone();
    replacement
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::covenant_engine::CovenantEvidence;
    use crate::manifest::{
        AssuranceScope, CertificationBoundary, Level, ObligationResult, RejectReason,
    };
    use crate::meaning::MeaningAudit;

    fn effects() -> Vec<String> {
        vec!["pure".into()]
    }

    fn proof(level: Level, engine: &str) -> ProofCandidate {
        ProofCandidate::Complete {
            engine: engine.into(),
            certificate: Certificate::new(
                "f",
                level,
                effects(),
                0,
                vec![ObligationResult::discharged("proved")],
            ),
        }
    }

    fn refutation(engine: &str) -> ProofCandidate {
        ProofCandidate::Refuted {
            engine: engine.into(),
            certificate: Certificate::new(
                "f",
                Level::L0,
                effects(),
                0,
                vec![ObligationResult::failed("counterexample", None, None)],
            ),
        }
    }

    #[test]
    fn total_combination_preserves_settled_policy_and_handles_both_disagreements() {
        let accepted = ItemOutcome::accepted(
            Certificate::new("f", Level::L3, effects(), 0, vec![]),
            "verus",
        );
        assert!(accepted.clone().combine(refutation("epr")).is_err());

        let refuted = ItemOutcome::refuted(
            Certificate::new(
                "f",
                Level::L0,
                effects(),
                0,
                vec![ObligationResult::failed("counterexample", None, None)],
            ),
            "verus",
        );
        assert!(refuted.clone().combine(proof(Level::L4, "epr")).is_err());
        assert_eq!(refuted.clone().combine(refutation("epr")).unwrap(), refuted);

        let weak =
            Certificate::rejected_weak_contract("f", effects(), "1/3".into(), "return 0".into());
        let weak =
            ItemOutcome::policy_rejected(weak.clone(), "mutation", PolicyRejection::WeakContract);
        assert_eq!(
            weak.clone()
                .combine(proof(Level::L4, "epr"))
                .unwrap()
                .into_certificate(),
            weak.into_certificate()
        );
    }

    #[test]
    fn inconclusive_and_degraded_results_upgrade_but_absence_preserves() {
        let timeout_cert = Certificate::rejected(
            "f",
            effects(),
            false,
            RejectReason {
                cause: "VerusTimeout".into(),
                detail: "budget".into(),
            },
        );
        let timeout = ItemOutcome::inconclusive(
            timeout_cert.clone(),
            "verus",
            InconclusiveReason::VerusTimeout,
        );
        let timeout_cert = timeout.certificate().clone();
        assert_eq!(
            timeout
                .clone()
                .combine(ProofCandidate::Unavailable)
                .unwrap()
                .into_certificate(),
            timeout_cert
        );
        for candidate in [
            ProofCandidate::Partial,
            ProofCandidate::Inconclusive {
                engine: "epr".into(),
                certificate: Certificate::rejected(
                    "f",
                    effects(),
                    false,
                    RejectReason {
                        cause: "EprUnknown".into(),
                        detail: "no terminal evidence".into(),
                    },
                ),
            },
        ] {
            assert_eq!(
                timeout
                    .clone()
                    .combine(candidate)
                    .unwrap()
                    .into_certificate(),
                timeout_cert
            );
        }
        assert_eq!(
            timeout
                .combine(proof(Level::L4, "epr"))
                .unwrap()
                .certificate()
                .level,
            Level::L4
        );

        let degraded_cert =
            Certificate::new("f", Level::L2, effects(), 0, vec![]).into_degraded(RejectReason {
                cause: "VerusTimeout".into(),
                detail: "budget".into(),
            });
        let degraded =
            ItemOutcome::inconclusive(degraded_cert, "verus", InconclusiveReason::TimeoutDegrade);
        assert_eq!(
            degraded
                .combine(proof(Level::L4, "epr"))
                .unwrap()
                .certificate()
                .level,
            Level::L4
        );
    }

    #[test]
    fn upgrade_preserves_proof_independent_context() {
        let scope = AssuranceScope::ToBoundary {
            via: "clock".into(),
        };
        let base = Certificate::new("f", Level::L3, effects(), 0, vec![])
            .with_mutation_score("3/3".into(), None)
            .with_strengthening(vec![crate::strengthen::Suggestion {
                clause: "result == x".into(),
                kills_survivor: None,
            }])
            .with_assurance_scope(scope.clone())
            .with_covenant_evidence(CovenantEvidence {
                witness_count: 1,
                falsify_generated: 10,
                falsify_refuted: 0,
                seed: 7,
            })
            .with_meaning_audit(MeaningAudit {
                tower_hash: "abc123".into(),
                depth: 2,
                definitions: 3,
            });
        let upgraded = ItemOutcome::accepted(base, "verus")
            .combine(proof(Level::L4, "epr"))
            .unwrap()
            .into_certificate();
        assert_eq!(upgraded.assurance_scope, Some(scope));
        assert_eq!(upgraded.contract_quality.mutants_killed, "3/3");
        assert_eq!(upgraded.strengthening.len(), 1);
        assert_eq!(upgraded.covenant_evidence.unwrap().witness_count, 1);
        assert_eq!(upgraded.meaning_audit.unwrap().tower_hash, "abc123");
        assert!(matches!(
            upgraded.certification.as_ref().map(|p| &p.boundary),
            Some(CertificationBoundary::ToBoundary { via }) if via == "clock"
        ));
    }

    #[test]
    fn exhaustive_current_base_by_candidate_matrix() {
        #[derive(Clone, Copy, Debug)]
        enum BaseCase {
            Accepted,
            Timeout,
            Degraded,
            Refuted,
            Weak,
            Tautology,
            Vacuous,
        }
        #[derive(Clone, Copy, Debug)]
        enum CandidateCase {
            Complete,
            Partial,
            Refuted,
            Unavailable,
            Unknown,
        }

        let base = |case| match case {
            BaseCase::Accepted => {
                ItemOutcome::accepted(Certificate::new("f", Level::L3, effects(), 0, vec![]), "v")
            }
            BaseCase::Timeout => ItemOutcome::inconclusive(
                Certificate::rejected(
                    "f",
                    effects(),
                    false,
                    RejectReason {
                        cause: "VerusTimeout".into(),
                        detail: "budget".into(),
                    },
                ),
                "v",
                InconclusiveReason::VerusTimeout,
            ),
            BaseCase::Degraded => ItemOutcome::inconclusive(
                Certificate::new("f", Level::L2, effects(), 0, vec![]).into_degraded(
                    RejectReason {
                        cause: "VerusTimeout".into(),
                        detail: "budget".into(),
                    },
                ),
                "kani",
                InconclusiveReason::TimeoutDegrade,
            ),
            BaseCase::Refuted => ItemOutcome::refuted(
                Certificate::new(
                    "f",
                    Level::L0,
                    effects(),
                    0,
                    vec![ObligationResult::failed("counterexample", None, None)],
                ),
                "v",
            ),
            BaseCase::Weak => ItemOutcome::policy_rejected(
                Certificate::rejected_weak_contract(
                    "f",
                    effects(),
                    "1/3".into(),
                    "return 0".into(),
                ),
                "mutation",
                PolicyRejection::WeakContract,
            ),
            BaseCase::Tautology | BaseCase::Vacuous => {
                let tautology = matches!(case, BaseCase::Tautology);
                let kind = if tautology {
                    PolicyRejection::SemanticTautology
                } else {
                    PolicyRejection::VacuousPrecondition
                };
                ItemOutcome::policy_rejected(
                    Certificate::rejected_vacuity(
                        "f",
                        effects(),
                        RejectReason {
                            cause: if tautology {
                                "SemanticTautology".into()
                            } else {
                                "VacuousPrecondition".into()
                            },
                            detail: "policy".into(),
                        },
                        tautology,
                        !tautology,
                    ),
                    "vacuity",
                    kind,
                )
            }
        };
        let candidate = |case| match case {
            CandidateCase::Complete => proof(Level::L4, "epr"),
            CandidateCase::Partial => ProofCandidate::Partial,
            CandidateCase::Refuted => refutation("epr"),
            CandidateCase::Unavailable => ProofCandidate::Unavailable,
            CandidateCase::Unknown => ProofCandidate::Inconclusive {
                engine: "epr".into(),
                certificate: Certificate::rejected(
                    "f",
                    effects(),
                    false,
                    RejectReason {
                        cause: "EprUnknown".into(),
                        detail: "undecided".into(),
                    },
                ),
            },
        };

        for base_case in [
            BaseCase::Accepted,
            BaseCase::Timeout,
            BaseCase::Degraded,
            BaseCase::Refuted,
            BaseCase::Weak,
            BaseCase::Tautology,
            BaseCase::Vacuous,
        ] {
            for candidate_case in [
                CandidateCase::Complete,
                CandidateCase::Partial,
                CandidateCase::Refuted,
                CandidateCase::Unavailable,
                CandidateCase::Unknown,
            ] {
                let initial = base(base_case);
                let result = initial.clone().combine(candidate(candidate_case));
                let should_alarm = matches!(
                    (base_case, candidate_case),
                    (BaseCase::Accepted, CandidateCase::Refuted)
                        | (BaseCase::Refuted, CandidateCase::Complete)
                );
                assert_eq!(
                    result.is_err(),
                    should_alarm,
                    "matrix cell {base_case:?} x {candidate_case:?}"
                );
                if should_alarm {
                    continue;
                }
                let settled = result.unwrap();
                match (base_case, candidate_case) {
                    (BaseCase::Timeout | BaseCase::Degraded, CandidateCase::Complete) => {
                        assert!(matches!(settled.disposition(), BaseDisposition::Accepted));
                    }
                    (BaseCase::Timeout | BaseCase::Degraded, CandidateCase::Refuted) => {
                        assert!(matches!(settled.disposition(), BaseDisposition::Refuted));
                    }
                    _ => assert_eq!(settled.disposition(), initial.disposition()),
                }
            }
        }
    }

    #[test]
    fn post_proof_policy_rendering_cannot_erase_context() {
        let scope = AssuranceScope::ToBoundary { via: "ffi".into() };
        let base = Certificate::new("f", Level::L3, effects(), 0, vec![])
            .with_assurance_scope(scope.clone())
            .with_covenant_evidence(CovenantEvidence {
                witness_count: 1,
                falsify_generated: 4,
                falsify_refuted: 0,
                seed: 9,
            })
            .with_meaning_audit(MeaningAudit {
                tower_hash: "tower".into(),
                depth: 1,
                definitions: 1,
            });
        let proved = ItemOutcome::accepted(base, "verus")
            .combine(proof(Level::L3, "lean"))
            .unwrap();
        let policy_certificate = proved
            .certificate()
            .clone()
            .with_mutation_score("3/3".into(), None);
        let settled = proved
            .apply_policy(PolicyDecision::Accepted(policy_certificate))
            .unwrap()
            .into_certificate();
        assert_eq!(settled.assurance_scope, Some(scope));
        assert_eq!(settled.covenant_evidence.unwrap().witness_count, 1);
        assert_eq!(settled.meaning_audit.unwrap().tower_hash, "tower");
    }

    #[test]
    fn candidate_and_policy_identity_mismatches_fail_closed() {
        let base = ItemOutcome::accepted(
            Certificate::new("f", Level::L3, effects(), 0, vec![]),
            "verus",
        );
        let wrong_proof = ProofCandidate::Complete {
            engine: "lean".into(),
            certificate: Certificate::new("g", Level::L3, effects(), 0, vec![]),
        };
        assert!(matches!(
            base.clone().combine(wrong_proof),
            Err(CombinationError::InvalidEvidence(_))
        ));
        let wrong_policy = PolicyDecision::Accepted(Certificate::new(
            "f",
            Level::L3,
            vec!["read(clock)".into()],
            0,
            vec![],
        ));
        assert!(base.apply_policy(wrong_policy).is_err());
    }

    #[test]
    fn malformed_typed_candidates_fail_before_acquiring_authority() {
        let base = ItemOutcome::inconclusive(
            Certificate::rejected(
                "f",
                effects(),
                false,
                RejectReason {
                    cause: "VerusTimeout".into(),
                    detail: "timeout".into(),
                },
            ),
            "verus",
            InconclusiveReason::VerusTimeout,
        );
        let malformed_complete = ProofCandidate::Complete {
            engine: "lean".into(),
            certificate: Certificate::new("f", Level::L0, effects(), 0, vec![]),
        };
        assert!(matches!(
            base.clone().combine(malformed_complete),
            Err(CombinationError::InvalidEvidence(_))
        ));

        let malformed_unknown = ProofCandidate::Inconclusive {
            engine: "lean".into(),
            certificate: Certificate::new("f", Level::L3, effects(), 0, vec![]),
        };
        assert!(matches!(
            base.select(malformed_unknown),
            Err(CombinationError::InvalidEvidence(_))
        ));
    }

    #[test]
    fn persisted_policy_shapes_are_checked_fail_closed() {
        let malformed = Certificate::rejected(
            "f",
            effects(),
            false,
            RejectReason {
                cause: "SemanticTautology".into(),
                detail: "missing quality bit".into(),
            },
        );
        let error = ItemOutcome::from_persisted_certificate(malformed).unwrap_err();
        assert!(error.detail.contains("contract-quality"));

        let contradictory = Certificate::new(
            "f",
            Level::L3,
            effects(),
            0,
            vec![ObligationResult::failed("impossible", None, None)],
        );
        let error = ItemOutcome::from_persisted_certificate(contradictory).unwrap_err();
        assert!(error.detail.contains("certifying level"));
    }
}
