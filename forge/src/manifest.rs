//! `forge/src/manifest.rs` — the certificate schema (`thermite-design.md` §5.1,
//! Appendix A). The `Certificate` is the deliverable's trust statement (§6): a
//! stable, versioned data contract that `forge check` emits. This module owns the
//! schema and its `serde_json` (de)serialization; it performs no I/O and runs no
//! verification. `check.rs` (`.design/forge/check.md`) produces the values.
//!
//! Governing design: `.design/forge/certificate-manifest.md`.
//!
//! The schema is fixed now at its full Appendix A shape; the producers arrive
//! over several issues (the "two-speed schema"). #5 fills `item`, `level`,
//! `effects`, `slag`, and `obligations` with real derived values; the
//! `contract_quality.*` battery fields are forward-declared (#5 values not
//! asserted against the golden cert, made live by #6/#12/#13) and
//! `suggested_move` is a reserved `None`. `solver_time_ms` is present but
//! non-deterministic and excluded from the cert-oracle comparison.
//!
//! ## REQ status
//!
//! <!-- generated:reqs view=forge-manifest-core-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-FORGE-MANIFEST-FORWARD-DECLARED | shipped | `forge/src/manifest.rs` | Forward-declared quality fields |  |
//! | REQ-FORGE-MANIFEST-OBLIGATIONS | shipped | `forge/src/manifest.rs` | Per-obligation certificate results |  |
//! | REQ-FORGE-MANIFEST-PRODUCED-FIELDS | shipped | `forge/src/manifest.rs` | Produced certificate fields |  |
//! | REQ-FORGE-MANIFEST-SCHEMA | shipped | `forge/src/manifest.rs` | Stable certificate schema |  |
//! | REQ-FORGE-MANIFEST-SERDE | shipped | `forge/src/manifest.rs` | Certificate serde serialization |  |
//! | REQ-FORGE-MANIFEST-SOLVER-TIME-ORACLE | shipped | `forge/src/manifest.rs` | solver_time_ms oracle exclusion |  |
//! | REQ-FORGE-MANIFEST-SUGGESTED-MOVE-RESERVED | shipped | `forge/src/manifest.rs` | Reserved suggested move slot |  |
//! <!-- /generated:reqs -->
//!
//! ## #6 additive schema (slag-triage, this iteration)
//!
//! <!-- generated:reqs view=forge-manifest-slag-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-FORGE-MANIFEST-REJECT-REASON | shipped | `forge/src/manifest.rs` | Verdict-in-certificate reject reason |  |
//! | REQ-FORGE-MANIFEST-SLAG-META | shipped | `forge/src/manifest.rs` | Slag certificate metadata |  |
//! <!-- /generated:reqs -->
//!
//! ## #8 additive schema (proof-cache provenance, this iteration)
//!
//! <!-- generated:reqs view=forge-manifest-cache-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-FORGE-MANIFEST-CACHE-PROVENANCE | shipped | `forge/src/manifest.rs` | Proof cache provenance flag |  |
//! <!-- /generated:reqs -->
//!
//! ## #11 additive schema (solver-profile timeout slot, this iteration)
//!
//! <!-- generated:reqs view=forge-manifest-solver-profile-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-FORGE-MANIFEST-SOLVER-PROFILE | shipped | `forge/src/manifest.rs` | Solver profile timeout slot |  |
//! | REQ-FORGE-MANIFEST-SUGGESTED-MOVE-PROFILE | shipped | `forge/src/manifest.rs` | Profile-populated suggested move |  |
//! <!-- /generated:reqs -->
//!
//! ## #13 producer (SOLVER-vacuity reject sets a `contract_quality` bool true)
//!
//! <!-- generated:reqs view=forge-manifest-solver-vacuity-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-FORGE-MANIFEST-SOLVER-VACUITY-REJECT | shipped | `forge/src/manifest.rs` | Solver-vacuity reject certificate |  |
//! <!-- /generated:reqs -->
//!
//! ## #16 additive schema (boundary-fn FFI cert — `.design/boundary/ffi-boundary.md`)
//!
//! <!-- generated:reqs view=forge-manifest-boundary-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-FORGE-MANIFEST-BOUNDARY-FLAG | shipped | `forge/src/manifest.rs` | Boundary verdict flag |  |
//! | REQ-FORGE-MANIFEST-BOUNDARY-L1 | shipped | `forge/src/manifest.rs` | Boundary L1 certificate constructor |  |
//! | REQ-FORGE-MANIFEST-BOUNDARY-TARGET | shipped | `forge/src/manifest.rs` | Boundary target diagnostic field |  |
//! <!-- /generated:reqs -->
//!
//! ## #10 additive schema (the degrade ladder + assurance aggregate, this iteration)
//!
//! <!-- generated:reqs view=forge-manifest-degrade-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-FORGE-MANIFEST-ASSURANCE-AGGREGATE | shipped | `forge/src/manifest.rs` | Assurance manifest aggregate |  |
//! | REQ-FORGE-MANIFEST-DEGRADE-FLAG | shipped | `forge/src/manifest.rs` | Lowered assurance flag |  |
//! | REQ-FORGE-MANIFEST-DEGRADE-REASON | shipped | `forge/src/manifest.rs` | Degrade reason field |  |
//! | REQ-FORGE-MANIFEST-LEVEL-ORD | shipped | `forge/src/manifest.rs` | Level ladder ordering |  |
//! <!-- /generated:reqs -->
//!
//! ## #17 additive schema (the §9 end-to-end vs to-the-boundary scope, this iteration)
//!
//! <!-- generated:reqs view=forge-manifest-e2e-status -->
//! Source: `.design/reqs/registry.toml`
//!
//! | ID | Status | Owner | Title | Follow-up |
//! |---|---|---|---|---|
//! | REQ-FORGE-MANIFEST-ASSURANCE-SCOPE | shipped | `forge/src/manifest.rs` | Per-function assurance scope |  |
//! | REQ-FORGE-MANIFEST-PROJECT-SCOPE | shipped | `forge/src/manifest.rs` | Project assurance scope |  |
//! <!-- /generated:reqs -->

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thermite_syntax::{Effect, EffectRow};

use crate::profile::SolverProfile;
use crate::strengthen::Suggestion;

/// RFC-3 certification scope: the population quantified over by the claim.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CertificationScope {
    All,
    Bounded { bound: String },
    PerExecution,
    None,
}

/// RFC-3 falsification channel. A false clause is not silently collapsed into
/// an assurance level: the certificate states what kind of witness can exist.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RefutationChannel {
    Complete,
    Incomplete,
    Empirical,
    Trace { bound: String },
    Abort,
    None,
}

/// Residual trust after discharged proof obligations have been removed. The
/// relax route and an unreconstructed cage differ in discharged evidence but
/// intentionally occupy the same residual-trust order position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResidualTrust {
    LeanChecked,
    Solver,
    Fiat,
}

/// RFC-3 boundary qualification: how far the certified claim closes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CertificationBoundary {
    EndToEnd,
    ToBoundary { via: String },
    ToPlatform { platform: String },
}

/// The authoritative RFC-3 formal certification position. `discharged_trust`
/// records bridge/reconstruction facts without treating them as residual risk.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CertificationPosition {
    pub scope: CertificationScope,
    pub refutation: RefutationChannel,
    pub residual_trust: ResidualTrust,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discharged_trust: Vec<String>,
    pub boundary: CertificationBoundary,
}

/// Pre-discharge RFC-3 classification certificate. Classification determines
/// which refutation fiber is available; discharge later fills the trust
/// position without erasing the routing prognosis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClassificationCertificate {
    /// Stable, versioned fragment/classifier identity.
    pub fragment: String,
    pub verdict: ClassificationVerdict,
}

/// Stable identity of a contract clause in certificate evidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClauseAddress {
    pub item: String,
    pub family: ClauseFamily,
    pub index: u32,
}

impl std::fmt::Display for ClauseAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.family {
            ClauseFamily::Ensures => write!(f, "{}::ens#{}", self.item, self.index),
        }
    }
}

impl ClauseAddress {
    pub fn from_selector(
        proof_target: &str,
        selector: &thermite_syntax::ClauseSelector,
        expected_item: &str,
        expected_count: usize,
    ) -> Result<Self, ClausePortfolioError> {
        if proof_target != expected_item {
            return Err(ClausePortfolioError::new(
                "proof target does not match certificate item",
            ));
        }
        if selector.keyword != "ensures" {
            return Err(ClausePortfolioError::new(
                "only indexed ensures clauses are certifiable",
            ));
        }
        let index = selector
            .index
            .ok_or_else(|| ClausePortfolioError::new("ensures selector requires an ordinal"))?;
        if index as usize >= expected_count {
            return Err(ClausePortfolioError::new(
                "clause ordinal is outside the contract",
            ));
        }
        Ok(Self {
            item: expected_item.to_string(),
            family: ClauseFamily::Ensures,
            index,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClauseFamily {
    Ensures,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ClauseProcedure {
    BitVector { frame: String },
    Epr { frame: String },
    Nlsat { frame: String },
    AuthorLean { frame: String },
}

impl ClauseProcedure {
    fn expected_fragment(&self) -> &'static str {
        match self {
            Self::BitVector { .. } => "thermite-bv-clause-v1",
            Self::Epr { .. } => "thermite-epr-clause-v1",
            Self::Nlsat { .. } => "thermite-nlsat-clause-v1",
            Self::AuthorLean { .. } => "thermite-author-lean-clause-v1",
        }
    }

    fn expected_engine(&self) -> &'static str {
        match self {
            Self::BitVector { .. } => crate::engine::EngineName::BitVector.tag(),
            Self::Epr { .. } => crate::engine::EngineName::Epr.tag(),
            Self::Nlsat { .. } => crate::engine::EngineName::Nlsat.tag(),
            Self::AuthorLean { .. } => crate::engine::EngineName::LeanInteractive.tag(),
        }
    }

    fn has_governed_frame(&self) -> bool {
        match self {
            Self::BitVector { frame } => matches!(
                frame.as_str(),
                "qf-bv8-v1" | "qf-bv16-v1" | "qf-bv32-v1" | "qf-bv64-v1"
            ),
            Self::Epr { frame } => frame == "s2-epr-reconstruction-v1",
            Self::Nlsat { frame } => frame == "real-relax-v1",
            Self::AuthorLean { frame } => frame == "author-lean-body-grounded-v1",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ClauseRouteEvidence {
    BitVector {
        query_sha256: String,
        shadow: BvShadow,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reconstruction: Option<crate::lean_smt_export::ReconstructionEvidence>,
    },
    Epr {
        query_sha256: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reconstruction: Option<crate::lean_smt_export::ReconstructionEvidence>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        witness: Option<String>,
    },
    Nlsat {
        query_sha256: String,
        result: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reconstruction: Option<crate::lean_smt_export::ReconstructionEvidence>,
    },
    AuthorLean {
        query_sha256: String,
        proof_sha256: String,
        burn: crate::burn::BurnReceipt,
        checker: String,
        evidence_key_sha256: String,
        axioms: Vec<String>,
    },
    BitVectorAttempted {
        query_sha256: String,
        outcome: crate::outcome_matrix::SolverProgressClass,
        detail: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        witness_sha256: Option<String>,
    },
    EprAttempted {
        query_sha256: String,
        outcome: crate::outcome_matrix::SolverProgressClass,
        detail: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        witness_sha256: Option<String>,
    },
    NlsatAttempted {
        query_sha256: String,
        outcome: crate::outcome_matrix::SolverProgressClass,
        detail: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        witness_sha256: Option<String>,
    },
    AuthorLeanAttempted {
        query_sha256: String,
        outcome: crate::outcome_matrix::SolverProgressClass,
        detail: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        witness_sha256: Option<String>,
    },
    NotAttempted,
}

impl ClauseRouteEvidence {
    fn attempted_fields(
        &self,
    ) -> Option<(
        &String,
        &crate::outcome_matrix::SolverProgressClass,
        &String,
        &Option<String>,
    )> {
        match self {
            Self::BitVectorAttempted {
                query_sha256,
                outcome,
                detail,
                witness_sha256,
            }
            | Self::EprAttempted {
                query_sha256,
                outcome,
                detail,
                witness_sha256,
            }
            | Self::NlsatAttempted {
                query_sha256,
                outcome,
                detail,
                witness_sha256,
            }
            | Self::AuthorLeanAttempted {
                query_sha256,
                outcome,
                detail,
                witness_sha256,
            } => Some((query_sha256, outcome, detail, witness_sha256)),
            _ => None,
        }
    }

    fn is_attempt_for(&self, procedure: &ClauseProcedure) -> bool {
        matches!(
            (procedure, self),
            (
                ClauseProcedure::BitVector { .. },
                Self::BitVectorAttempted { .. }
            ) | (ClauseProcedure::Epr { .. }, Self::EprAttempted { .. })
                | (ClauseProcedure::Nlsat { .. }, Self::NlsatAttempted { .. })
                | (
                    ClauseProcedure::AuthorLean { .. },
                    Self::AuthorLeanAttempted { .. }
                )
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PortfolioStopCause {
    ClauseTerminal {
        address: ClauseAddress,
        terminal: ClauseTerminalKind,
    },
    ItemGate {
        gate: ItemGateKind,
        class: crate::outcome_matrix::OutcomeClass,
        detail: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClauseTerminalKind {
    Refuted,
    Undecided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemGateKind {
    Covenant,
    MeaningTower,
    Vacuity,
    Body,
    Prerequisite,
    MutationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ClauseTerminalState {
    Discharged,
    Refuted {
        witness_sha256: String,
    },
    Undecided {
        outcome: crate::outcome_matrix::SolverProgressClass,
    },
    NotAttempted {
        cause: PortfolioStopCause,
    },
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ClauseAuthorityStamp(Option<String>);

impl PartialEq for ClauseAuthorityStamp {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for ClauseAuthorityStamp {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClauseCertification {
    pub address: ClauseAddress,
    pub artifact_sha256: String,
    pub query_sha256: String,
    pub expected_count: u32,
    pub classification: ClassificationCertificate,
    pub procedure: ClauseProcedure,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<CertificationPosition>,
    pub evidence: ClauseRouteEvidence,
    pub terminal: ClauseTerminalState,
    #[serde(skip)]
    authority: ClauseAuthorityStamp,
}

impl ClauseCertification {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn issued(
        address: ClauseAddress,
        artifact_sha256: String,
        query_sha256: String,
        expected_count: u32,
        classification: ClassificationCertificate,
        procedure: ClauseProcedure,
        position: Option<CertificationPosition>,
        evidence: ClauseRouteEvidence,
        terminal: ClauseTerminalState,
        authority: crate::check::ClauseAuthority,
    ) -> Self {
        Self {
            address,
            artifact_sha256,
            query_sha256,
            expected_count,
            classification,
            procedure,
            position,
            evidence,
            terminal,
            authority: ClauseAuthorityStamp(None),
        }
        .authoritative(authority)
    }

    pub(crate) fn authoritative(mut self, _authority: crate::check::ClauseAuthority) -> Self {
        self.authority = ClauseAuthorityStamp(Some("pending-clause-seal-v1".into()));
        self
    }

    fn authority_digest(&self, obligation: &ObligationResult) -> String {
        let bytes = serde_json::to_vec(&(
            (
                &self.address,
                &self.artifact_sha256,
                &self.query_sha256,
                self.expected_count,
                &self.classification,
                &self.procedure,
                &self.position,
                &self.evidence,
                &self.terminal,
            ),
            (
                &obligation.name,
                &obligation.status,
                &obligation.location,
                &obligation.diagnostic,
                &obligation.engine,
                &obligation.trust,
                &obligation.verdict,
                &obligation.bv_shadow,
                &obligation.reconstruction,
            ),
        ))
        .expect("clause authority projection is serializable");
        let mut hash = Sha256::new();
        hash.update(b"thermite-live-clause-authority-v1");
        hash.update(bytes);
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn seal_for(&mut self, obligation: &ObligationResult) {
        assert_eq!(
            self.authority.0.as_deref(),
            Some("pending-clause-seal-v1"),
            "only freshly issued clause evidence can acquire a content seal"
        );
        self.authority = ClauseAuthorityStamp(Some(self.authority_digest(obligation)));
    }

    fn has_valid_seal(&self, obligation: &ObligationResult) -> bool {
        self.authority.0.as_deref() == Some(self.authority_digest(obligation).as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClausePortfolioKind {
    AcceptedHomogeneous,
    Heterogeneous,
    Incomplete,
    PolicyRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClausePortfolio {
    pub kind: ClausePortfolioKind,
    pub clauses: Vec<ClauseCertification>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClauseOracleEntry {
    pub certification: ClauseCertification,
    pub status: ObligationStatus,
    pub engine: Option<String>,
    pub trust: Vec<String>,
    pub verdict: Option<crate::verdict::CertVerdict>,
    pub bv_shadow: Option<BvShadow>,
    pub reconstruction: Option<crate::lean_smt_export::ReconstructionEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClauseOraclePortfolio {
    pub clauses: Vec<ClauseOracleEntry>,
    pub classification: Option<ClassificationCertificate>,
    pub certification: Option<CertificationPosition>,
    pub engine_attribution: Option<crate::engine::EngineAttribution>,
    pub compatibility_burn: Option<crate::burn::BurnReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClausePortfolioError {
    pub reason: String,
}
impl ClausePortfolioError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}
impl std::fmt::Display for ClausePortfolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid clause portfolio: {}", self.reason)
    }
}
impl std::error::Error for ClausePortfolioError {}

pub(crate) fn clause_terminal_witness_digest(
    obligation: &ObligationResult,
    reject: &Option<RejectReason>,
) -> String {
    let bytes = serde_json::to_vec(&(
        &obligation.name,
        &obligation.status,
        &obligation.location,
        &obligation.diagnostic,
        &obligation.engine,
        &obligation.trust,
        &obligation.verdict,
        &obligation.bv_shadow,
        &obligation.reconstruction,
        reject,
    ))
    .expect("terminal witness projection is serializable");
    let mut hash = Sha256::new();
    hash.update(b"thermite-clause-terminal-witness-v1");
    hash.update(bytes);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ClassificationVerdict {
    Admitted,
    Rejected { reason: String },
    Unknown { reason: String },
}

/// A schema-level failure: the coordinate tuple lands outside RFC-3's eight
/// coherent cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncoherentCertificationPosition {
    pub reason: &'static str,
}

impl std::fmt::Display for IncoherentCertificationPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "incoherent RFC-3 certification position: {}",
            self.reason
        )
    }
}

impl std::error::Error for IncoherentCertificationPosition {}

/// The seven order-elements in RFC-3 §3.2. Relax and unreconstructed cage
/// routes are deliberately order-equivalent at `CompleteSolver`.
// ASSURANCE_V2_PREDECESSOR forge::manifest::AssuranceElement compatibility_only
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AssuranceElement {
    NoClaim,
    Runtime,
    Bounded,
    IncompleteSolver,
    CompleteSolver,
    EmpiricalLean,
    CompleteLean,
}

impl CertificationPosition {
    /// Reject every tuple outside RFC-3 §3.1's eight coherent cells.
    pub fn validate(&self) -> Result<(), IncoherentCertificationPosition> {
        // Self-dominance exercises the same comparison path used by declared
        // floors; a newly-added cell cannot validate while being absent from
        // the order relation.
        let _ = self.dominates(self)?;
        Ok(())
    }

    fn element(&self) -> Result<AssuranceElement, IncoherentCertificationPosition> {
        use AssuranceElement::*;
        use CertificationScope as S;
        use RefutationChannel as R;
        use ResidualTrust as T;

        match (&self.scope, &self.refutation, self.residual_trust) {
            (S::None, R::None, T::Fiat) => Ok(NoClaim),
            (S::PerExecution, R::Abort, T::Fiat) => Ok(Runtime),
            (S::Bounded { bound: a }, R::Trace { bound: b }, T::Solver) if a == b => Ok(Bounded),
            (S::All, R::Incomplete, T::Solver) => Ok(IncompleteSolver),
            (S::All, R::Complete, T::Solver) => Ok(CompleteSolver),
            (S::All, R::Empirical, T::LeanChecked) => Ok(EmpiricalLean),
            (S::All, R::Complete, T::LeanChecked) => Ok(CompleteLean),
            (S::Bounded { .. }, R::Trace { .. }, T::Solver) => {
                Err(IncoherentCertificationPosition {
                    reason: "bounded scope and trace refutation must carry the same bound",
                })
            }
            _ => Err(IncoherentCertificationPosition {
                reason: "scope, refutation, and residual trust do not form an RFC-3 cell",
            }),
        }
    }

    /// Product-order comparison. `Some(Ordering)` means comparable; `None`
    /// preserves RFC-3's intentionally incomparable solver/forge positions.
    pub fn partial_cmp_assurance(
        &self,
        other: &Self,
    ) -> Result<Option<std::cmp::Ordering>, IncoherentCertificationPosition> {
        use std::cmp::Ordering;
        use AssuranceElement::*;

        let a = self.element()?;
        let b = other.element()?;
        if a == b {
            return Ok(Some(Ordering::Equal));
        }
        let rank = |e| match e {
            NoClaim => 0,
            Runtime => 1,
            Bounded => 2,
            IncompleteSolver | EmpiricalLean => 3,
            CompleteSolver => 4,
            CompleteLean => 5,
        };
        let incomparable = matches!(
            (a, b),
            (EmpiricalLean, IncompleteSolver | CompleteSolver)
                | (IncompleteSolver | CompleteSolver, EmpiricalLean)
        );
        if incomparable {
            Ok(None)
        } else {
            Ok(Some(rank(a).cmp(&rank(b))))
        }
    }

    /// True when this position meets a declared formal floor. Boundary values
    /// must match exactly; platform/boundary assumptions are never silently
    /// erased by the assurance order.
    pub fn dominates(&self, floor: &Self) -> Result<bool, IncoherentCertificationPosition> {
        if self.boundary != floor.boundary {
            return Ok(false);
        }
        Ok(matches!(
            self.partial_cmp_assurance(floor)?,
            Some(std::cmp::Ordering::Equal | std::cmp::Ordering::Greater)
        ))
    }
}

/// The §9 assurance scope of a function (issue #17,
/// `.design/forge/e2e-vs-boundary.md` REQ-2/REQ-3; `thermite-design.md` §9). The
/// manifest distinction between "verified to the boundary" and "verified, period":
///
/// - [`AssuranceScope::EndToEnd`] — the fn's transitive intra-file call closure
///   reaches no `#[boundary]` (foreign body) and no `#[slag]` (fiat-trusted body)
///   fn; the whole-program guarantee rests only on the toolchain ("verified,
///   period").
/// - [`AssuranceScope::ToBoundary`] — the closure transitively reaches a crossing;
///   `via` names the first reached `#[boundary]`/`#[slag]` fn. The fn's own
///   contract is verified, while the end-to-end guarantee crosses a foreign/unproven
///   body (`goal.md` R-DEFER-9 — mark such a guarantee).
///
/// Orthogonal to [`Level`] (REQ-5): a `ToBoundary` fn may be `Level::L3` (its own
/// body fully SMT-proved against the crossing's contract). Produced by
/// [`crate::closure::classify`] (the structural call-closure analysis); recorded
/// as the additive [`Certificate::assurance_scope`] field.
///
/// The serialized form is a tagged enum (`{"kind": "end_to_end"}` /
/// `{"kind": "to_boundary", "via": "<fn>"}`), mirroring [`ProjectAssurance`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum AssuranceScope {
    /// "Verified, period": no `#[boundary]`/`#[slag]` fn in the transitive closure.
    EndToEnd,
    /// "Verified to the boundary": the closure reaches a crossing; `via` is the
    /// first reached `#[boundary]`/`#[slag]` fn (the deterministic crossing).
    ToBoundary {
        /// The name of the first `#[boundary]`/`#[slag]` fn the closure reaches.
        via: String,
    },
}

impl AssuranceScope {
    /// `true` iff this scope is end-to-end ("verified, period"). The verdict-
    /// relevant bit the cert-oracle compares (see [`Certificate::oracle_subset`]):
    /// a `None` `assurance_scope` (the frozen golden `sum.cert.json`, which omits
    /// the field) and `Some(EndToEnd)` (a freshly-classified pure fn) both read
    /// `true` here, so the golden subset stays stable (R-SPEC-2) while a
    /// `ToBoundary` verdict is oracle-visible.
    pub fn is_end_to_end(&self) -> bool {
        matches!(self, AssuranceScope::EndToEnd)
    }
}

/// `true` iff `scope` is end-to-end for the oracle (REQ-3): `None` (field absent,
/// the golden default) or `Some(EndToEnd)`. A `Some(ToBoundary)` reads `false`.
/// This is the normalization that keeps the golden `sum.cert.json` (no
/// `assurance_scope` key) oracle-equal to a freshly-classified `Some(EndToEnd)`
/// `sum` cert (`.design/forge/e2e-vs-boundary.md` Verification).
fn scope_is_end_to_end(scope: &Option<AssuranceScope>) -> bool {
    match scope {
        None => true,
        Some(s) => s.is_end_to_end(),
    }
}

/// The assurance level (`thermite-design.md` §6). Serializes to the string form
/// `"L0".."L4"` to match the golden cert's `"level": "L3"` (REQ-1, REQ-7).
///
/// The declaration order `L0 < L1 < L2 < L3 < L4` is the ladder ordering
/// (`.design/forge/degrade-ladder.md` REQ-6, reconciled to RFC-1 / GH #2 in
/// increment 2f): `#[derive(PartialOrd, Ord)]` makes it the `Ord` the
/// assurance-manifest aggregate uses for the min-over-functions project headline.
/// The aggregate depends on this discriminant order.
///
/// L4 is the **caged** rung — RFC-1 §2: a clause decidable by an admission test,
/// push-button, *every failure a concrete countermodel* (finite structure, integer
/// point, or bit pattern), never degraded. RUNG and TRUST BASE are orthogonal axes:
/// the rung records refutation quality; what is trusted is recorded separately per
/// clause in the engine attribution. Three L4 route families exist today, same rung,
/// different
/// trust bases:
///
/// - the **nlsat real-relaxation** discharge (Stage-1 REQ-8, `--engine nlsat`):
///   trust `solver(nlsat) + spine-lemma(kernel)`, the ℝ→ℤ bridge sound by the
///   kernel-checked `r_relax_sound` + `rencode_sound` (`lean/Thermite/Relax.lean`);
/// - the **`@bv` machine-width** discharge (Stage-3 REQ-2, `--engine bv`): QF_BV
///   solving followed by checked reconstruction of the actual clause theorem;
/// - admitted **finite relation/sequence** clauses (Stage 4): grounded SAT/LRAT
///   reconstruction and Lean replay, with false clauses returning finite models.
///
/// (The general Verus/Z3 cage still certifies at L3 in code pending its own promotion
/// — a follow-up; the RFC ladder eventually places the whole decidable cage at L4.)
/// Automatic CLI routing overlays eligible BV and EPR reconstruction on the base
/// engine path; explicit engine flags remain diagnostic overrides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Level {
    /// L0 — unverified / `#[slag]` escape hatch (§6, §8).
    L0,
    /// L1 — executable runtime check compiled in (§6).
    L1,
    /// L2 — bounded model check (Kani; issue #9) (§6).
    L2,
    /// L3 — SMT proof: the contract holds for all inputs (§6). The Verus/Z3 SOLVER
    /// rung.
    L3,
    /// L4 — CAGED (RFC-1 §2): decidable, push-button, every failure a concrete
    /// countermodel; never degraded. The rung is refutation quality; the trust base is
    /// recorded separately per clause. Current routes cover nlsat real-relaxation,
    /// reconstructed fixed-width BV, and reconstructed finite relation/sequence
    /// clauses. Above L3 on the ladder.
    L4,
}

/// Temporary beta-line bridge for producers not yet converted to explicit
/// classification certificates. L2 intentionally returns `None`: the scalar
/// never stored its bound, so inventing one would repeat the lossy migration
/// RFC-3 is removing.
fn legacy_position(level: Level) -> Option<CertificationPosition> {
    let (scope, refutation, residual_trust) = match level {
        Level::L0 => (
            CertificationScope::None,
            RefutationChannel::None,
            ResidualTrust::Fiat,
        ),
        Level::L1 => (
            CertificationScope::PerExecution,
            RefutationChannel::Abort,
            ResidualTrust::Fiat,
        ),
        Level::L2 => return None,
        Level::L3 => (
            CertificationScope::All,
            RefutationChannel::Incomplete,
            ResidualTrust::Solver,
        ),
        Level::L4 => (
            CertificationScope::All,
            RefutationChannel::Complete,
            ResidualTrust::Solver,
        ),
    };
    Some(CertificationPosition {
        scope,
        refutation,
        residual_trust,
        discharged_trust: Vec::new(),
        boundary: CertificationBoundary::EndToEnd,
    })
}

/// The status of a single proof obligation (REQ-5). v0.1 records discharged or
/// failed; the failure carries a source-located diagnostic (the §5.1
/// "counterexamples, not adjectives" payload) rather than a bare boolean.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationStatus {
    /// The obligation was discharged by the solver.
    Discharged,
    /// The obligation failed; see the `diagnostic` and `location` on the result.
    Failed,
}

/// One per-obligation verification result (REQ-5, `.design/forge/check.md`
/// REQ-4). For a clean proof, `check.rs` records the verified item(s) as
/// `Discharged`; for a failure it records the failed obligation with verus's
/// `error: <clause>` description and its `--> file:line:col` source span — the
/// §5.1 "counterexample, not adjective".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObligationResult {
    /// The obligation identity (the verus function name for a discharged item,
    /// or the failed-clause description for a failure).
    pub name: String,
    /// Discharged or failed.
    pub status: ObligationStatus,
    /// `file:line:col` source span of the obligation, when verus reports one.
    /// `None` for a summary-only discharged result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// The concrete failure diagnostic from verus's stderr (`error: <clause>`),
    /// present only on a failure, rather than a bare "verification failed".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<String>,
    /// (Schema v2, REQ-1/AC-4 — `.design/stage1-forge-tier.md`) The proof engine that
    /// discharged this clause (e.g. `"lean-auto"`, `"verus"`). Additive: `#[serde(default,
    /// skip_serializing_if = "Option::is_none")]` (the `engine_attribution` precedent), so
    /// the v1 golden certs — which omit it — deserialize unchanged and re-serialize
    /// byte-identically. Populated only on the forge-tier (Lean) cert paths; absent on the
    /// v1 Verus corpus, whose clauses keep the v1 `status`/`level` representation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// (Schema v2) The named trust base the clause's engine added (the `TrustProfile`
    /// items — e.g. `{Lean kernel, propext, …, EXP}`). Additive, `skip_serializing_if`
    /// `Vec::is_empty`, so the v1 goldens (which omit it) stay byte-identical.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trust: Vec<String>,
    /// (Schema v2) The cert-level seven-verdict for this clause (the closed forge-tier
    /// vocabulary [`crate::verdict::CertVerdict`]). Additive, skip-if-none; absent on the
    /// v1 corpus (whose clauses are recorded by `status`/`level`, the v1 representation),
    /// so the golden oracle is unperturbed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict: Option<crate::verdict::CertVerdict>,
    /// (Schema v2, REQ-3 / AC-4 — `.design/stage3-bv-reconstruction.md`) Lock 1, the
    /// shadow FLAG (RFC §9 shape): present on every `@bv`-tagged clause's obligation,
    /// absent on every untagged / v1 clause. The machine-semantics fork is then
    /// impossible to hide — `grep bv_shadow` over the certificates ≡ exactly the set of
    /// tagged clauses, the same "`grep slag` is the complete inventory" discipline the
    /// `#[slag]` block follows. Additive, `#[serde(default, skip_serializing_if =
    /// "Option::is_none")]` (the `engine`/`trust`/`verdict` precedent), so the v1 golden
    /// certs — which omit it — deserialize unchanged and re-serialize byte-identically.
    /// Oracle-INCLUDED via [`Certificate::oracle_subset`] (Q-ORACLE: deterministic +
    /// verdict-relevant → included), unlike the provenance-only `engine`/`trust`: a
    /// semantic fork changes what the clause means, so the oracle pins it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bv_shadow: Option<BvShadow>,
    /// Evidence that Lean accepted this clause's `req → clause` theorem and
    /// its axiom report passed. Failed or unavailable replay leaves this absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconstruction: Option<crate::lean_smt_export::ReconstructionEvidence>,
    /// Source-bound formal coordinates for migrated mixed-route clauses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clause_certification: Option<ClauseCertification>,
}

/// Lock 1, the bv shadow FLAG (`.design/stage3-bv-reconstruction.md` REQ-3 / AC-4 — the
/// RFC §9 shape). Every `@bv`-tagged clause's certificate carries this block, so a
/// fixed-width machine-semantics clause is loud and greppable at every layer `#[slag]`
/// is (the certificate JSON, `forge review`, `forge audit`). The first of the three
/// locks that ship inside gate G3 with the `@bv` feature itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BvShadow {
    /// Always `true` for a tagged clause — the flag that says "this clause is
    /// interpreted over a fixed-width machine-semantics fork, not the default unbounded
    /// integers". The grep anchor: a present `bv_shadow` (with `flagged: true`) marks a
    /// tagged clause.
    pub flagged: bool,
    /// The fixed-width semantics the clause was interpreted over, e.g.
    /// `"bv64 (wraparound)"` / `"bv32 (nowrap)"` (the `check::bv_semantics_label`
    /// shape) — the named fork, so a reader sees WHICH machine semantics this clause
    /// committed to.
    pub semantics: String,
    /// The `@bvN(nowrap)` no-overflow side obligation's verdict (REQ-5 / AC-6, lock 3).
    /// `Some(verdict)` for a `@bvN(nowrap)` clause whose side obligation ran — the in-cage
    /// no-overflow discharge result ("discharged: no operation overflows…" when it holds,
    /// "FAILED: a bvN operation overflows on input […]" with the witnessing bit pattern
    /// when a concrete input overflows, or an "undecided …" label when Z3 could not
    /// decide it). `None` for a bare `@bvN` (no side obligation requested). Filled by
    /// `check::bv_nowrap_verdict`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nowrap_obligation: Option<String>,
    /// A human note naming the fork + the lock (the auditor's one-line "why this block
    /// is here"). Deterministic (R-CODE-5).
    pub note: String,
}

impl ObligationResult {
    /// A discharged obligation (a verified verus function), summary-only. The schema-v2
    /// per-clause block is absent (the v1 representation) — attach it with
    /// [`ObligationResult::with_clause_attribution`] on the forge-tier paths.
    pub fn discharged(name: impl Into<String>) -> Self {
        ObligationResult {
            name: name.into(),
            status: ObligationStatus::Discharged,
            location: None,
            diagnostic: None,
            engine: None,
            trust: Vec::new(),
            verdict: None,
            bv_shadow: None,
            reconstruction: None,
            clause_certification: None,
        }
    }

    /// A failed obligation carrying its source location + diagnostic witness
    /// (§5.1 "counterexamples, not adjectives").
    pub fn failed(
        name: impl Into<String>,
        location: Option<String>,
        diagnostic: Option<String>,
    ) -> Self {
        ObligationResult {
            name: name.into(),
            status: ObligationStatus::Failed,
            location,
            diagnostic,
            engine: None,
            trust: Vec::new(),
            verdict: None,
            bv_shadow: None,
            reconstruction: None,
            clause_certification: None,
        }
    }

    pub(crate) fn with_clause_certification(mut self, mut clause: ClauseCertification) -> Self {
        clause.seal_for(&self);
        self.clause_certification = Some(clause);
        self
    }

    /// Attach the schema-v2 per-clause attribution block (REQ-1/AC-4): the engine that
    /// discharged this clause, its named trust base, and the cert-level verdict. Used by
    /// the forge-tier (Lean) cert paths ([`crate::check`]); the v1 Verus corpus leaves the
    /// block absent, so its golden certs stay byte-identical.
    #[must_use]
    pub fn with_clause_attribution(
        mut self,
        engine: impl Into<String>,
        trust: Vec<String>,
        verdict: crate::verdict::CertVerdict,
    ) -> Self {
        self.engine = Some(engine.into());
        self.trust = trust;
        self.verdict = Some(verdict);
        self
    }

    /// Attach Lock 1, the bv shadow flag (REQ-3 / AC-4) — the RFC §9 visibility lock
    /// every `@bv`-tagged clause's obligation carries. Orthogonal to
    /// [`ObligationResult::with_clause_attribution`] (engine/trust/verdict are
    /// provenance; the shadow flag is the semantic-fork marker), so the two compose on a
    /// single tagged-clause obligation.
    #[must_use]
    pub fn with_bv_shadow(mut self, shadow: BvShadow) -> Self {
        self.bv_shadow = Some(shadow);
        self
    }

    /// Attach the checked theorem evidence that authorizes kernel trust.
    #[must_use]
    pub fn with_reconstruction(
        mut self,
        evidence: crate::lean_smt_export::ReconstructionEvidence,
    ) -> Self {
        self.reconstruction = Some(evidence);
        self
    }
}

/// The contract-quality block (`thermite-design.md` §7, Appendix A) — REQ-3.
/// Forward-declared in #5: the vacuity battery (`tautology`/
/// `vacuous_precondition`, #6/#13) and the mutation scorer
/// (`mutants_killed`/`survivor`, #12) are not yet built, so these carry
/// non-asserted values and are excluded from the cert-oracle comparison
/// (`Certificate::oracle_subset`). The schema reserves the slot; its producer
/// fills the value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractQuality {
    /// Is the contract a tautology? (issue #6/#13) — `false` placeholder in #5.
    pub tautology: bool,
    /// Is the precondition vacuous? (issue #6/#13) — `false` placeholder in #5.
    pub vacuous_precondition: bool,
    /// Mutation kill ratio `"killed/total"` (issue #12) — `"0/0"` (unscored) in
    /// #5; typed `String` to match the Appendix A `"17/18"` shape (OQ-1).
    pub mutants_killed: String,
    /// Survivors removed from the denominator only after a proof of observable
    /// equivalence. Omitted when zero to preserve pre-exclusion certificate JSON.
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub equivalent_mutants_excluded: usize,
    /// The surviving-mutant description (issue #12) — `None` (unscored) in #5.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survivor: Option<String>,
    /// Every addressed G1 mutation replay contribution used by the item policy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clause_mutation_replays: Vec<ClauseMutationReplay>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClauseMutationReplay {
    pub mutant_sha256: String,
    pub mutant: String,
    pub address: ClauseAddress,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_sha256: Option<String>,
    pub outcome: crate::engine::MutationReplayOutcome,
}

fn is_zero_usize(value: &usize) -> bool {
    *value == 0
}

impl ContractQuality {
    /// The #5 value: the battery has not run, so nothing is asserted.
    /// `mutants_killed` is the unscored `"0/0"`, not the golden `"17/18"` (REQ-3;
    /// `conformance/README.md` forward-declaration).
    pub fn forward_declared() -> Self {
        ContractQuality {
            tautology: false,
            vacuous_precondition: false,
            mutants_killed: "0/0".to_string(),
            equivalent_mutants_excluded: 0,
            survivor: None,
            clause_mutation_replays: Vec::new(),
        }
    }
}

/// A reserved `suggested_move` heuristic hint (`thermite-design.md` §5.1) —
/// REQ-4. The slot exists so populating it later (missing-invariant patterns,
/// overflow-guard templates, trigger hints) is not a breaking schema change. In
/// #5 the `Certificate`'s `suggested_move` is always `None` (a reserved
/// absence, carrying no placeholder string).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedMove {
    /// A short kind tag for the heuristic (e.g. `"missing-invariant"`).
    pub kind: String,
    /// The suggested edit text.
    pub detail: String,
}

/// The validated `#[slag]` metadata carried into a certificate (§8;
/// `.design/forge/slag.md` REQ-4). Produced by `slag::validate` once all three
/// mandatory fields are confirmed present + non-empty; recorded on the cert so a
/// reviewer can audit the fiat-trusted block (`slag: true` is the inventory flag,
/// these are the justification).
///
/// Additive schema field (`slag.md` OQ-1, ratified): Appendix A's certificate has
/// `slag: bool` only; `slag_meta` is a faithful superset, serialized only when
/// present (`#[serde(skip_serializing_if)]`), so the golden `sum.cert.json`
/// (which omits it) still deserializes (R-SPEC-2 — no frozen field renamed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlagMeta {
    /// Why the body is fiat-trusted (non-empty after trim — `slag.rs` REQ-1).
    pub reason: String,
    /// The accountable owner (non-empty after trim).
    pub owner: String,
    /// The review status / requirement (non-empty after trim).
    pub review: String,
}

/// The structured reason a certificate is not certified (`.design/forge/vacuity-triage.md`
/// REQ-5; `slag.md` REQ-5). A triage / slag-validation failure is a contract-
/// certification failure surfaced inside the certificate (§7 "a function does not
/// certify until its contract certifies"), not a `ForgeError`: the cert is a
/// valid document describing why the item did not certify. `check.rs` records
/// this on a non-certified (`Level::L0`) cert and exits non-zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectReason {
    /// A short machine-readable cause tag (the §7.1 verdict variant name, e.g.
    /// `"EnsIsTrivial"`, or a slag cause `"SlagFieldMissing"`).
    pub cause: String,
    /// A human-readable detail naming the offending clause / field.
    pub detail: String,
}

/// RFC-11 resource-flow evidence attached only after the independent Lean replay
/// accepts the checked witness. This is deterministic, verdict-relevant evidence:
/// changing the source, checked flow, or abandonment footprint changes the block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceFlowEvidence {
    pub verdict: ResourceFlowVerdict,
    pub forgets: Vec<ResourceForgetFootprint>,
    pub formal_replay: ResourceFormalReplay,
    pub residual_trust: Vec<ResourceResidualTrust>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFlowVerdict {
    Accepted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceForgetFootprint {
    pub function: String,
    pub disposition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<String>,
    pub regions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceFormalReplay {
    pub checker: String,
    pub checker_sha256: String,
    pub witness_version: u32,
    pub canonical_ast_sha256: String,
    pub checked_resource_sha256: String,
    pub verdict: ResourceFormalReplayVerdict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFormalReplayVerdict {
    KernelAccepted,
}

/// The exact boundary left outside RFC-11's finite-graph replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceResidualTrust {
    Parser,
    TypeProvenanceResolution,
    ResourceFlowComputation,
    WitnessExtraction,
    ExecutableTargetBehavior,
}

/// The certificate `forge check` emits for one item (`thermite-design.md` §5.1,
/// Appendix A). Field declaration order is the deterministic serialization order
/// (REQ-7) and mirrors Appendix A: `item`, `level`, `solver_time_ms`,
/// `contract_quality`, `effects`, `slag`; the #5 additive schema surface
/// (`obligations` — REQ-5; `suggested_move` — REQ-4) follows.
#[derive(Debug, Clone, Default)]
struct AuditAdmission {
    live: bool,
    verus: Option<VerusAuditAuthority>,
    resource: Option<ResourceAuditAuthority>,
    clause_policy_digest: Option<String>,
}

#[derive(Debug, Clone)]
struct VerusAuditAuthority {
    item: String,
    effects: Vec<String>,
    query_identity: String,
    succeeded: bool,
}

#[derive(Debug, Clone)]
struct ResourceAuditAuthority {
    evidence: ResourceFlowEvidence,
}

impl AuditAdmission {
    fn live() -> Self {
        Self {
            live: true,
            verus: None,
            resource: None,
            clause_policy_digest: None,
        }
    }
}

// Admission is deliberately outside the stable data contract. Equality of the
// serialized certificate remains equality of its public evidence; the in-memory
// capability only answers whether audit may treat that evidence as producer output.
impl PartialEq for AuditAdmission {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for AuditAdmission {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certificate {
    /// The checked item's name.
    pub item: String,
    /// The assurance level (REQ-2: L3 iff verus reports 0 errors).
    pub level: Level,
    /// RFC-3's authoritative certification tuple. During the beta migration the
    /// historical `level` remains readable, but new producers persist this
    /// independently inspectable surface and audit copies it verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) certification: Option<CertificationPosition>,
    /// The pre-discharge fragment prognosis (RFC-3 R2-8). `None` only for
    /// historical certificates and producers not yet migrated to the classifier
    /// seam; never reconstructed from the post-discharge result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) classification: Option<ClassificationCertificate>,
    /// Non-serializable admission capability. Constructors mint it for live
    /// producer output; deserialization defaults to unadmitted. Consequently a
    /// persisted row is data to inspect, not authority that audit can project.
    #[serde(skip)]
    audit_admission: AuditAdmission,
    /// Wall-clock solver time in ms — non-deterministic, excluded from the
    /// oracle comparison (REQ-6; `conformance/README.md`). `#[serde(default)]`
    /// so the golden deterministic-subset cert (which omits this non-det field)
    /// still deserializes into a full `Certificate` (certificate-manifest.md
    /// AC-2 — the schema is a faithful superset of the golden subset).
    #[serde(default)]
    pub solver_time_ms: u64,
    /// The contract-quality battery block — forward-declared in #5 (REQ-3).
    pub contract_quality: ContractQuality,
    /// The item's effect row (REQ-2: `["pure"]` for the corpus).
    pub effects: Vec<String>,
    /// Whether the item is `#[slag]` — `true` for a valid `#[slag]` item (#6/§8),
    /// `false` otherwise. Set by `check.rs` after `slag::validate` succeeds.
    pub slag: bool,
    /// The validated `#[slag]` metadata (#6 additive field; `slag.md` REQ-4).
    /// `Some` only on a valid slag item; `#[serde(default)]` + skip-if-none so
    /// the frozen golden cert (which omits it) still deserializes (R-SPEC-2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slag_meta: Option<SlagMeta>,
    /// The structured reason this item did not certify (#6 additive field;
    /// vacuity-triage.md REQ-5 / slag.md REQ-5). `Some` only on a triage / slag
    /// reject; `#[serde(default)]` + skip-if-none so a clean golden cert
    /// deserializes unchanged (R-SPEC-2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject: Option<RejectReason>,
    /// Per-obligation results parsed from verus (REQ-5; #5 additive field).
    /// `#[serde(default)]` so a golden cert that does not enumerate the
    /// per-obligation array (the golden asserts only the item-level summary,
    /// certificate-manifest.md OQ-2) deserializes into a `Certificate`.
    #[serde(default)]
    pub obligations: Vec<ObligationResult>,
    /// Whether this certificate was served from the proof cache (#8 additive
    /// field; `.design/forge/proof-cache.md` REQ-7). `true` on a cache hit (verus
    /// skipped), `false` on a fresh verify. `#[serde(default)]` so the frozen
    /// golden `conformance/sum.cert.json` (which omits it) still deserializes,
    /// mirroring the #6 `slag_meta`/`reject` additive precedent (R-SPEC-2). It is
    /// provenance rather than verdict: excluded from `oracle_subset` so a cache hit
    /// and a fresh verify compare oracle-equal (REQ-2, the soundness invariant).
    #[serde(default)]
    pub cached: bool,
    /// The structured Z3 quantifier-instantiation report attached on a verus
    /// timeout / rlimit-hit (issue #11 additive field;
    /// `.design/forge/solver-profiles.md` REQ-6). `Some` only on a timeout cert
    /// (`Certificate::timeout`); `None` on a proved (`L3`) cert and on a
    /// counterexample-L0 cert (AC-4). `#[serde(default)]` + skip-if-none so the
    /// frozen golden `conformance/sum.cert.json` (which omits it) still
    /// deserializes (R-SPEC-2, additive only), mirroring the `slag_meta`/`reject`
    /// and `cached` additive precedents. Diagnostic and non-deterministic (§5.3):
    /// excluded from `oracle_subset` (a timeout cert with a profile is
    /// oracle-equal to the same cert with the profile stripped).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solver_profile: Option<SolverProfile>,
    /// Reserved heuristic-hint slot — `None` in #5 (REQ-4); populated on a
    /// timeout cert from `profile::suggested_move` (#11; the profile-derived
    /// proof-repair hint).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_move: Option<SuggestedMove>,
    /// Whether this certificate was achieved by an automatic degrade below L3
    /// (issue #10 additive field; `.design/forge/degrade-ladder.md` REQ-4). `true`
    /// on a cert the L3→L2→L1 ladder produced after a verus timeout degraded the
    /// item to L2 (kani bounded check) or L1 (runtime checks); `false` on every
    /// directly-achieved cert (an L3 proof, an explicit `--level l2`/`--level l1`
    /// choice, a `#[slag]` L1-by-fiat, a reject). `#[serde(default)]` so the frozen
    /// golden `conformance/sum.cert.json` (which omits it) still deserializes,
    /// mirroring the `cached` additive precedent (R-SPEC-2). It is verdict-relevant
    /// (it qualifies the achieved level as "lowered, not proved") so it is not
    /// oracle-excluded; the corpus at the default budget never degrades, so the
    /// golden cert keeps the default `false` (AC-1/AC-6).
    #[serde(default)]
    pub lowered_assurance: bool,
    /// The structured reason this item was degraded below L3 (issue #10 additive
    /// field; `.design/forge/degrade-ladder.md` REQ-4). `Some` only on a
    /// `lowered_assurance` cert — the `VerusTimeout` reason carried from the L3
    /// attempt that timed out. `#[serde(default,
    /// skip_serializing_if)]` so a non-degraded cert (the golden) deserializes
    /// unchanged (R-SPEC-2). Diagnostic and non-deterministic in content (it carries
    /// the same kind of material as the §5.3 `solver_profile`): excluded from
    /// `oracle_subset` (a degraded cert is oracle-compared on its `level` +
    /// `lowered_assurance` flag, not on the prose reason).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degrade_reason: Option<RejectReason>,
    /// The §7 step-5 strengthening suggestions surfaced for this item (issue #14
    /// additive field; `.design/forge/strengthening-probes.md` REQ-4). Each
    /// [`Suggestion`] is an adoptable stronger-`ens` clause that verifies against
    /// the body and is strictly stronger than the current `ens` (it would
    /// kill a #12 survivor / adds an equality the `ens` lacks). Advisory: a probe
    /// only adds these; it does not change the verdict (`level`/`reject`/the oracle
    /// subset). `#[serde(default, skip_serializing_if = Vec::is_empty)]` so the
    /// frozen golden `conformance/sum.cert.json` (which omits it, and for which the
    /// probe emits nothing) still deserializes (R-SPEC-2, additive only), mirroring
    /// the `solver_profile` additive precedent. Diagnostic and verus-version-
    /// sensitive (a future verus might prove a candidate today's cannot), so it is
    /// excluded from `oracle_subset` (parallel to `solver_profile`/`mutants_killed`,
    /// OQ-3). An item with no surviving candidate carries an empty list (an
    /// absence, mirroring the `suggested_move: None` precedent).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strengthening: Vec<Suggestion>,
    /// Whether this item is a foreign-crossing boundary fn (issue #16 additive
    /// field; `.design/boundary/ffi-boundary.md` REQ-5). `true` only on a
    /// boundary-fn cert (`Certificate::boundary_l1`): a `#[boundary("crate::path")]`
    /// fn whose foreign body is unproven and whose contract is enforced at L1 (the
    /// FFI analog of `slag: true`). `#[serde(default)]` so the frozen golden
    /// `conformance/sum.cert.json` (which omits it) still deserializes, defaulting
    /// `false`, mirroring the `slag`/`cached`/`lowered_assurance` additive
    /// precedents (R-SPEC-2). Verdict-relevant (it qualifies the achieved L1 as
    /// "to-the-boundary, foreign body unproven", the #15 TCB-enumeration + #17
    /// verified-to-the-boundary input), so it joins `slag` in `oracle_subset`.
    #[serde(default)]
    pub boundary: bool,
    /// The foreign `crate::path` target a boundary fn's L1 wrapper calls (issue #16
    /// additive field; `.design/boundary/ffi-boundary.md` REQ-5). `Some` only on a
    /// boundary cert (`Certificate::boundary_l1`); `None` otherwise. `#[serde(default,
    /// skip_serializing_if = "Option::is_none")]` so a non-boundary cert (the golden)
    /// deserializes unchanged (R-SPEC-2). Diagnostic — the prose half of the #15
    /// audit hook (the `boundary` flag is the verdict-relevant half): excluded from
    /// `oracle_subset` (parallel to `slag_meta`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary_target: Option<String>,
    /// The §9 assurance scope of this fn (issue #17 additive field;
    /// `.design/forge/e2e-vs-boundary.md` REQ-3). `Some(EndToEnd)` when the fn's
    /// transitive intra-file call closure reaches no `#[boundary]`/`#[slag]` fn;
    /// `Some(ToBoundary { via })` when it does (the first reached crossing). `None`
    /// only on a cert built before classification ran (the constructors below set
    /// `None`; `check::check_file_with_options` attaches the real scope via
    /// `Certificate::with_assurance_scope`). `#[serde(default, skip_serializing_if =
    /// "Option::is_none")]` so the frozen golden `conformance/sum.cert.json` (which
    /// omits this field) still deserializes, defaulting `None`, mirroring the
    /// `slag_meta`/`solver_profile`/`boundary_target` additive precedents (R-SPEC-2).
    ///
    /// Verdict-relevant (§9 / R-DEFER-9 — a guarantee depending on an unproven
    /// foreign body must be marked), so it joins the `oracle_subset`, normalized to
    /// a bool (`scope_is_end_to_end`): `None` (the golden default) and
    /// `Some(EndToEnd)` (a freshly-classified pure fn) are oracle-equal, so the
    /// golden `sum.cert.json` stays stable while a `ToBoundary` verdict is
    /// oracle-visible (the design's stability requirement). Orthogonal to `level`
    /// (REQ-5): recorded alongside, never merged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assurance_scope: Option<AssuranceScope>,
    /// The per-obligation engine attribution (`.design/verified/proof-backends.md`
    /// REQ-4, increment (iii), #247): the `{engine, trust_profile}` pair recorded when
    /// a non-default engine (Lean) discharged this item's certification obligation —
    /// so an auditor sees that L3-via-Lean enumerates a smaller trusted base ({Lean
    /// kernel + 3 axioms, EXP}) than L3-via-Verus ({Z3, Verus VC-gen, lowering
    /// theorem}). `Some` only when a non-default engine discharged (the default Verus
    /// path leaves it `None`); set by `Certificate::with_engine_attribution`, consumed
    /// by `cli::run_check`'s `--engine lean` path. `#[serde(default,
    /// skip_serializing_if = "Option::is_none")]` so the frozen golden
    /// `conformance/sum.cert.json` (which omits it — the default Verus path never
    /// populates it) still deserializes, defaulting `None`, mirroring the
    /// `slag_meta`/`solver_profile`/`assurance_scope` additive precedents (R-SPEC-2 —
    /// the cert oracle stays byte-identical because `serde(default)` keeps the golden
    /// green: a Verus cert never gains the field). Diagnostic and verdict-orthogonal (the
    /// `Level` is unchanged — L3 still means "proven for all inputs"; the trust base is
    /// the auditor-visible refinement): excluded from `oracle_subset` (OQ-2 decided
    /// diagnostic-only so the golden stays stable; the project-min aggregate is
    /// unchanged — REQ-4 "minimum aggregation unchanged").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_attribution: Option<crate::engine::EngineAttribution>,
    /// The covenant evidence block (`.design/stage1-forge-tier.md` REQ-4, increment
    /// 2b; Q-ORACLE). `Some` only on a forge-routed item carrying a `witness` block —
    /// the author witness count, the deterministic `falsify` generated/refuted counts,
    /// and the fixed seed (all reproducible, so the block joins the cert oracle and
    /// cannot drift silently: weakening a falsify budget or dropping a witness changes
    /// these numbers). `None` on every v1 item (no covenant declared), and
    /// `#[serde(default, skip_serializing_if = "Option::is_none")]` so the 7 frozen v1
    /// golden certs (which omit it) serialize BYTE-IDENTICALLY (R-SPEC-2, additive only),
    /// mirroring the `engine_attribution`/`assurance_scope` additive precedents. Per
    /// Q-ORACLE the covenant record is verdict-relevant evidence (a refuted covenant is
    /// a hard fail; a validated one is the proof-search precondition), so it joins the
    /// `oracle_subset` — `None` for both a fresh v1 cert and the golden, so the v1
    /// oracle stays byte-identical while a forge-tier covenant block is oracle-visible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub covenant_evidence: Option<crate::covenant_engine::CovenantEvidence>,
    /// The meaning-audit block: the definition-tower hash + depth + definition count
    /// (`.design/stage1-forge-tier.md` REQ-6c, increment 2d; Q-ORACLE). `Some` only on
    /// a forge-tier item whose contract was tower-audited at certify time (the
    /// `--engine` discharge path): the certify-time gate refuses an over-budget tower
    /// (a `DefinitionTowerBudget` reject) and pins the unfolded-tower hash on a
    /// within-budget cert, so a reader can confirm the certified meaning is the one in
    /// front of them — a changed definition anywhere in the tower changes the hash.
    /// Per Q-ORACLE the meaning-audit hash is verdict-relevant evidence (it cannot
    /// drift silently), so it joins the `oracle_subset`. `None` for every v1 item (the
    /// default Verus corpus never tower-audits), and `#[serde(default,
    /// skip_serializing_if = "Option::is_none")]` so the 7 frozen v1 golden certs
    /// (which omit it) serialize BYTE-IDENTICALLY (R-SPEC-2, additive only), mirroring
    /// the `covenant_evidence`/`engine_attribution` additive precedents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meaning_audit: Option<crate::meaning::MeaningAudit>,
    /// The L3 burn receipt (`.design/stage1-forge-tier.md` REQ-7, increment 2e; RFC-1
    /// §9; Q-burn). `Some` only on a forge-tier item whose proof closed a goal (the
    /// proof-view discharge path): the committed-proof lexer-token count, the optional
    /// authoring spend, and the lemmas the proof cited. Per Q-ORACLE / Q-burn the burn
    /// receipt is verdict-IRRELEVANT (re-authoring a proof legitimately changes the
    /// token count and authoring spend without changing what was proven), so it is
    /// excluded from [`Certificate::oracle_subset`] — like `solver_time_ms`.
    /// `None` for every v1 item (no forge-tier burn), and `#[serde(default,
    /// skip_serializing_if = "Option::is_none")]` so the 7 frozen v1 golden certs
    /// (which omit it) serialize BYTE-IDENTICALLY (R-SPEC-2, additive only), mirroring
    /// the `meaning_audit`/`covenant_evidence` additive precedents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burn: Option<crate::burn::BurnReceipt>,
    /// RFC-11 checked resource-flow and independent formal-replay disclosure.
    /// Absent for programs outside the resource fragment. A resource-bearing
    /// current producer must carry this block; historical rows cannot acquire
    /// authority merely by deserializing a compatible-looking JSON value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_flow: Option<ResourceFlowEvidence>,
    /// Process-local typed result provenance. This is deliberately absent from
    /// the public certificate wire format: live proof/policy producers stamp it
    /// so later backend arbitration never has to rediscover authority from
    /// display strings, while deserialized cache rows must pass the stricter
    /// persisted adapter.
    #[serde(skip)]
    pub(crate) live_disposition: LiveDispositionStamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LiveResultDisposition {
    Accepted,
    VerusTimeout,
    TimeoutDegrade,
    EngineUnknown,
    Refuted,
    WeakContract,
    SemanticTautology,
    VacuousPrecondition,
    SettledOther(String),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LiveDispositionStamp(Option<LiveResultDisposition>);

impl PartialEq for LiveDispositionStamp {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for LiveDispositionStamp {}

impl Certificate {
    /// Read the RFC-3 pair through one structural seam. A classification without
    /// a position is rejected rather than projected as a partial claim. Legacy
    /// position-only certificates remain readable during migration.
    pub fn rfc3_coordinates(
        &self,
    ) -> Result<
        Option<(&CertificationPosition, Option<&ClassificationCertificate>)>,
        IncoherentCertificationPosition,
    > {
        match (&self.certification, &self.classification) {
            (None, Some(_)) => Err(IncoherentCertificationPosition {
                reason: "classification cannot exist without a certification position",
            }),
            (Some(_), None) if self.level == Level::L2 => Err(IncoherentCertificationPosition {
                reason: "migrated L2 certification requires its classification pair",
            }),
            (Some(position), None)
                if position
                    .discharged_trust
                    .iter()
                    .any(|fact| fact.starts_with("thermite-l1-wrapper-v1:")) =>
            {
                Err(IncoherentCertificationPosition {
                    reason: "migrated L1 certification requires its classification pair",
                })
            }
            (Some(position), None)
                if position
                    .discharged_trust
                    .iter()
                    .any(|fact| fact.starts_with("thermite-verus-query-v1:")) =>
            {
                Err(IncoherentCertificationPosition {
                    reason: "migrated Verus certification requires its classification pair",
                })
            }
            (Some(position), Some(classification))
                if self.level != Level::L1
                    && (position
                        .discharged_trust
                        .iter()
                        .any(|fact| fact.starts_with("thermite-l1-wrapper-v1:"))
                        || classification.fragment.starts_with("thermite-l1-")) =>
            {
                Err(IncoherentCertificationPosition {
                    reason: "migrated L1 evidence requires the legacy Level::L1 projection",
                })
            }
            (Some(position), Some(classification))
                if position
                    .discharged_trust
                    .iter()
                    .any(|fact| fact.starts_with("thermite-verus-query-v1:"))
                    != classification.fragment.starts_with("thermite-verus-") =>
            {
                Err(IncoherentCertificationPosition {
                    reason: "migrated Verus query evidence and classification must remain paired",
                })
            }
            (position, classification) => Ok(position
                .as_ref()
                .map(|position| (position, classification.as_ref()))),
        }
    }

    /// Whether this row carries any structural sign of the migrated runtime-L1
    /// producer. Audit uses this independently of the mutable legacy `level`, so
    /// changing that projection cannot bypass checked-artifact validation.
    pub(crate) fn requires_l1_artifact_validation(&self) -> bool {
        self.level == Level::L1
            || self
                .classification
                .as_ref()
                .is_some_and(|classification| classification.fragment.starts_with("thermite-l1-"))
            || self.certification.as_ref().is_some_and(|position| {
                position
                    .discharged_trust
                    .iter()
                    .any(|fact| fact.starts_with("thermite-l1-wrapper-v1:"))
                    || (position.scope == CertificationScope::PerExecution
                        && position.refutation == RefutationChannel::Abort
                        && position.residual_trust == ResidualTrust::Fiat)
            })
    }

    /// Whether this value came from a live producer (or the separately validated
    /// proof-cache boundary), rather than directly from attacker-mutable JSON.
    pub(crate) fn is_audit_admitted(&self) -> bool {
        self.audit_admission.live
    }

    fn clause_policy_digest(&self) -> String {
        let clause_coordinates: Vec<_> = self
            .obligations
            .iter()
            .filter_map(|obligation| {
                obligation
                    .clause_certification
                    .as_ref()
                    .map(|clause| (&clause.address, &clause.procedure, &clause.terminal))
            })
            .collect();
        let bytes = serde_json::to_vec(&(
            &self.item,
            &self.contract_quality,
            &self.burn,
            &self.reject,
            clause_coordinates,
        ))
        .expect("clause policy authority projection is serializable");
        let mut hash = Sha256::new();
        hash.update(b"thermite-live-clause-policy-authority-v1");
        hash.update(bytes);
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    /// Validate and expose the authoritative mixed-route clause portfolio.
    pub fn clause_portfolio(
        &self,
        final_accepted: bool,
    ) -> Result<Option<ClausePortfolio>, ClausePortfolioError> {
        let clauses: Vec<(&ObligationResult, &ClauseCertification)> = self
            .obligations
            .iter()
            .filter_map(|obligation| {
                obligation
                    .clause_certification
                    .as_ref()
                    .map(|c| (obligation, c))
            })
            .collect();
        if clauses.is_empty() {
            return Ok(None);
        }
        if self.audit_admission.clause_policy_digest.as_deref()
            != Some(self.clause_policy_digest().as_str())
        {
            return Err(ClausePortfolioError::new(
                "clause mutation policy lacks an intact live producer content seal",
            ));
        }
        if clauses
            .iter()
            .any(|(obligation, clause)| !clause.has_valid_seal(obligation))
        {
            return Err(ClausePortfolioError::new(
                "clause data lacks an intact live producer content seal",
            ));
        }
        let expected = clauses[0].1.expected_count as usize;
        if expected == 0 || clauses.len() != expected {
            return Err(ClausePortfolioError::new(
                "clause inventory is not an exact non-empty range",
            ));
        }
        let item = &clauses[0].1.address.item;
        if item != &self.item {
            return Err(ClausePortfolioError::new(
                "clause portfolio item does not match its certificate",
            ));
        }
        let artifact = &clauses[0].1.artifact_sha256;
        let mut prior_terminal: std::collections::HashMap<ClauseAddress, ClauseTerminalKind> =
            std::collections::HashMap::new();
        for (ordinal, (obligation, clause)) in clauses.iter().enumerate() {
            if clause.address.item != *item
                || clause.address.family != ClauseFamily::Ensures
                || clause.address.index as usize != ordinal
                || clause.expected_count as usize != expected
                || clause.artifact_sha256 != *artifact
                || clause.artifact_sha256.is_empty()
                || clause.query_sha256.is_empty()
                || !clause.procedure.has_governed_frame()
                || clause.classification.fragment != clause.procedure.expected_fragment()
            {
                return Err(ClausePortfolioError::new(
                    "clause addresses or fingerprints do not form one exact portfolio",
                ));
            }
            clause
                .position
                .as_ref()
                .map(CertificationPosition::validate)
                .transpose()
                .map_err(|e| ClausePortfolioError::new(e.to_string()))?;
            let route_coherent = match (&clause.procedure, &clause.evidence) {
                (
                    ClauseProcedure::BitVector { .. },
                    ClauseRouteEvidence::BitVector {
                        query_sha256,
                        shadow,
                        reconstruction,
                    },
                ) => {
                    query_sha256 == &clause.query_sha256
                        && obligation.bv_shadow.as_ref() == Some(shadow)
                        && obligation.reconstruction.as_ref() == reconstruction.as_ref()
                }
                (
                    ClauseProcedure::Epr { .. },
                    ClauseRouteEvidence::Epr {
                        query_sha256,
                        reconstruction,
                        ..
                    },
                ) => {
                    query_sha256 == &clause.query_sha256
                        && obligation.reconstruction.as_ref() == reconstruction.as_ref()
                        && obligation.bv_shadow.is_none()
                }
                (
                    ClauseProcedure::Nlsat { .. },
                    ClauseRouteEvidence::Nlsat {
                        query_sha256,
                        result,
                        reconstruction,
                    },
                ) => {
                    query_sha256 == &clause.query_sha256
                        && !result.trim().is_empty()
                        && obligation.bv_shadow.is_none()
                        && obligation.reconstruction.as_ref() == reconstruction.as_ref()
                }
                (
                    ClauseProcedure::AuthorLean { .. },
                    ClauseRouteEvidence::AuthorLean {
                        query_sha256,
                        proof_sha256,
                        checker,
                        evidence_key_sha256,
                        axioms,
                        ..
                    },
                ) => {
                    query_sha256 == &clause.query_sha256
                        && !proof_sha256.is_empty()
                        && checker == "lean-interactive axiom-gated replay v1"
                        && !evidence_key_sha256.is_empty()
                        && axioms.iter().map(String::as_str).eq([
                            "propext",
                            "Classical.choice",
                            "Quot.sound",
                        ])
                        && obligation.bv_shadow.is_none()
                        && obligation.reconstruction.is_none()
                }
                (procedure, evidence) if evidence.is_attempt_for(procedure) => evidence
                    .attempted_fields()
                    .is_some_and(|(query, _, detail, _)| {
                        query == &clause.query_sha256 && !detail.trim().is_empty()
                    }),
                (_, ClauseRouteEvidence::NotAttempted) => {
                    matches!(clause.terminal, ClauseTerminalState::NotAttempted { .. })
                }
                _ => false,
            };
            if !route_coherent {
                return Err(ClausePortfolioError::new(
                    "clause route evidence contradicts its procedure or obligation evidence",
                ));
            }
            match (&clause.terminal, &obligation.status) {
                (ClauseTerminalState::Discharged, ObligationStatus::Discharged) => {
                    let (fragment, expected_engine, expected_trust, expected_position) =
                        match (&clause.procedure, &clause.evidence) {
                            (
                                ClauseProcedure::BitVector { .. },
                                ClauseRouteEvidence::BitVector { reconstruction, .. },
                            ) => {
                                let reconstructed = reconstruction.is_some();
                                (
                                    "thermite-bv-clause-v1",
                                    crate::engine::EngineName::BitVector.tag(),
                                    if reconstructed {
                                        crate::engine::bv_kernel_checked_trust_profile().items
                                    } else {
                                        crate::engine::bv_trust_profile().items
                                    },
                                    CertificationPosition {
                                        scope: CertificationScope::All,
                                        refutation: RefutationChannel::Complete,
                                        residual_trust: if reconstructed {
                                            ResidualTrust::LeanChecked
                                        } else {
                                            ResidualTrust::Solver
                                        },
                                        discharged_trust: Vec::new(),
                                        boundary: CertificationBoundary::EndToEnd,
                                    },
                                )
                            }
                            (
                                ClauseProcedure::Epr { .. },
                                ClauseRouteEvidence::Epr {
                                    reconstruction: Some(_),
                                    ..
                                },
                            ) => (
                                "thermite-epr-clause-v1",
                                crate::engine::EngineName::Epr.tag(),
                                crate::engine::epr_kernel_checked_trust_profile().items,
                                CertificationPosition {
                                    scope: CertificationScope::All,
                                    refutation: RefutationChannel::Complete,
                                    residual_trust: ResidualTrust::LeanChecked,
                                    discharged_trust: Vec::new(),
                                    boundary: CertificationBoundary::EndToEnd,
                                },
                            ),
                            (
                                ClauseProcedure::Nlsat { .. },
                                ClauseRouteEvidence::Nlsat { reconstruction, .. },
                            ) => {
                                let reconstructed = reconstruction.is_some();
                                (
                                    "thermite-nlsat-clause-v1",
                                    crate::engine::EngineName::Nlsat.tag(),
                                    if reconstructed {
                                        crate::engine::lia_kernel_checked_trust_profile().items
                                    } else {
                                        crate::engine::nlsat_trust_profile().items
                                    },
                                    CertificationPosition {
                                        scope: CertificationScope::All,
                                        refutation: RefutationChannel::Complete,
                                        residual_trust: if reconstructed {
                                            ResidualTrust::LeanChecked
                                        } else {
                                            ResidualTrust::Solver
                                        },
                                        discharged_trust: Vec::new(),
                                        boundary: CertificationBoundary::EndToEnd,
                                    },
                                )
                            }
                            (
                                ClauseProcedure::AuthorLean { .. },
                                ClauseRouteEvidence::AuthorLean { .. },
                            ) => (
                                "thermite-author-lean-clause-v1",
                                crate::engine::EngineName::LeanInteractive.tag(),
                                crate::engine::trust_profile_interactive().items,
                                CertificationPosition {
                                    scope: CertificationScope::All,
                                    refutation: RefutationChannel::Empirical,
                                    residual_trust: ResidualTrust::LeanChecked,
                                    discharged_trust: Vec::new(),
                                    boundary: CertificationBoundary::EndToEnd,
                                },
                            ),
                            _ => {
                                return Err(ClausePortfolioError::new(
                                    "discharged clause lacks procedure-specific evidence",
                                ));
                            }
                        };
                    if clause.position.is_none()
                        || clause.classification.verdict != ClassificationVerdict::Admitted
                        || clause.classification.fragment != fragment
                        || clause.position.as_ref() != Some(&expected_position)
                        || obligation.engine.as_deref() != Some(expected_engine)
                        || obligation.trust != expected_trust
                        || obligation.verdict != Some(crate::verdict::CertVerdict::Proved)
                        || matches!(
                            clause.evidence,
                            ClauseRouteEvidence::BitVectorAttempted { .. }
                                | ClauseRouteEvidence::EprAttempted { .. }
                                | ClauseRouteEvidence::NlsatAttempted { .. }
                                | ClauseRouteEvidence::AuthorLeanAttempted { .. }
                                | ClauseRouteEvidence::NotAttempted
                        )
                    {
                        return Err(ClausePortfolioError::new(
                            "discharged clause lacks admitted coherent coordinates",
                        ));
                    }
                }
                (ClauseTerminalState::Refuted { witness_sha256 }, ObligationStatus::Failed)
                    if !witness_sha256.is_empty()
                        && witness_sha256
                            == &clause_terminal_witness_digest(obligation, &self.reject)
                        && clause.position.is_none()
                        && clause.classification.verdict == ClassificationVerdict::Admitted
                        && obligation.engine.as_deref()
                            == Some(clause.procedure.expected_engine())
                        && matches!(
                            obligation.verdict,
                            Some(crate::verdict::CertVerdict::Counterexample { .. })
                        )
                        && clause.evidence.attempted_fields().is_some_and(
                            |(_, outcome, _, evidence_witness)| {
                                *outcome
                                    == crate::outcome_matrix::SolverProgressClass::Counterexample
                                    && evidence_witness.as_ref() == Some(witness_sha256)
                            },
                        ) =>
                {
                    prior_terminal.insert(clause.address.clone(), ClauseTerminalKind::Refuted);
                }
                (ClauseTerminalState::Undecided { outcome }, ObligationStatus::Failed)
                    if clause.position.is_none()
                        && matches!(
                            clause.classification.verdict,
                            ClassificationVerdict::Unknown { .. }
                        )
                        && obligation
                            .engine
                            .as_deref()
                            .is_none_or(|engine| engine == clause.procedure.expected_engine())
                        && !matches!(
                            obligation.verdict,
                            Some(crate::verdict::CertVerdict::Proved)
                                | Some(crate::verdict::CertVerdict::Counterexample { .. })
                        )
                        && clause.evidence.attempted_fields().is_some_and(
                            |(_, evidence_outcome, _, witness)| {
                                evidence_outcome == outcome && witness.is_none()
                            },
                        ) =>
                {
                    prior_terminal.insert(clause.address.clone(), ClauseTerminalKind::Undecided);
                }
                (ClauseTerminalState::NotAttempted { cause }, ObligationStatus::Failed) => {
                    if clause.position.is_some()
                        || !matches!(
                            clause.classification.verdict,
                            ClassificationVerdict::Unknown { .. }
                        )
                        || obligation.engine.is_some()
                        || !obligation.trust.is_empty()
                        || obligation.verdict.is_some()
                        || obligation.bv_shadow.is_some()
                        || obligation.reconstruction.is_some()
                    {
                        return Err(ClausePortfolioError::new(
                            "not-attempted clause carries attempted-route authority",
                        ));
                    }
                    match cause {
                        PortfolioStopCause::ClauseTerminal { address, terminal } => {
                            if address.index >= clause.address.index
                                || prior_terminal.get(address) != Some(terminal)
                            {
                                return Err(ClausePortfolioError::new("not-attempted cause is not rooted in an earlier terminal clause"));
                            }
                        }
                        PortfolioStopCause::ItemGate { detail, .. } if detail.trim().is_empty() => {
                            return Err(ClausePortfolioError::new(
                                "item-gate cause requires an outcome",
                            ));
                        }
                        PortfolioStopCause::ItemGate { gate, class, .. } => {
                            let cause = self.reject.as_ref().map(|reason| reason.cause.as_str());
                            let (matches_gate, expected_class) = match gate {
                                ItemGateKind::Covenant => (
                                    cause.is_some_and(|c| c.starts_with("Covenant")),
                                    if cause == Some("CovenantRefuted") {
                                        crate::outcome_matrix::OutcomeClass::Counterexample
                                    } else {
                                        crate::outcome_matrix::OutcomeClass::InvalidSource
                                    },
                                ),
                                ItemGateKind::MeaningTower => (
                                    cause == Some("DefinitionTowerBudget"),
                                    crate::outcome_matrix::OutcomeClass::ResourceExhausted,
                                ),
                                ItemGateKind::Vacuity => (
                                    cause == Some("VacuousPrecondition"),
                                    crate::outcome_matrix::OutcomeClass::UnsupportedPolicy,
                                ),
                                ItemGateKind::Body => (
                                    cause == Some("ForgeGateNoBody"),
                                    crate::outcome_matrix::OutcomeClass::UnsupportedLanguage,
                                ),
                                ItemGateKind::Prerequisite => (
                                    cause.is_some_and(|c| {
                                        matches!(c, "ForgeGateMissingProof" | "Prerequisite")
                                    }),
                                    crate::outcome_matrix::OutcomeClass::InvalidSource,
                                ),
                                ItemGateKind::MutationPolicy => (
                                    cause.is_some_and(|c| {
                                        matches!(c, "WeakContract" | "MutationPolicy")
                                    }),
                                    crate::outcome_matrix::OutcomeClass::UnsupportedPolicy,
                                ),
                            };
                            if !matches_gate || *class != expected_class {
                                return Err(ClausePortfolioError::new(
                                    "item-gate stop cause contradicts certificate policy evidence",
                                ));
                            }
                        }
                    }
                }
                _ => {
                    return Err(ClausePortfolioError::new(
                        "terminal state contradicts obligation status",
                    ))
                }
            }
        }
        let author_burns: Vec<&crate::burn::BurnReceipt> = clauses
            .iter()
            .filter_map(|(_, clause)| match &clause.evidence {
                ClauseRouteEvidence::AuthorLean { burn, .. } => Some(burn),
                _ => None,
            })
            .collect();
        let burn_compatible = match author_burns.as_slice() {
            [only] => self.burn.as_ref() == Some(*only),
            [] | [_, _, ..] => self.burn.is_none(),
        };
        if !burn_compatible {
            return Err(ClausePortfolioError::new(
                "certificate-level compatibility burn contradicts clause-local evidence",
            ));
        }
        let all_discharged = clauses
            .iter()
            .all(|(_, c)| matches!(c.terminal, ClauseTerminalState::Discharged));
        let author_addresses: std::collections::HashSet<_> = clauses
            .iter()
            .filter_map(|(_, clause)| {
                matches!(clause.procedure, ClauseProcedure::AuthorLean { .. })
                    .then_some(clause.address.clone())
            })
            .collect();
        let replays = &self.contract_quality.clause_mutation_replays;
        if all_discharged && !author_addresses.is_empty() && replays.is_empty() {
            return Err(ClausePortfolioError::new(
                "an author-Lean portfolio lacks its complete mutation replay matrix",
            ));
        }
        if !replays.is_empty() {
            let expected_addresses: std::collections::HashSet<_> = clauses
                .iter()
                .map(|(_, clause)| clause.address.clone())
                .collect();
            let procedures_by_address: std::collections::HashMap<_, _> = clauses
                .iter()
                .map(|(_, clause)| (clause.address.clone(), &clause.procedure))
                .collect();
            let mut by_mutant: std::collections::BTreeMap<&str, Vec<&ClauseMutationReplay>> =
                std::collections::BTreeMap::new();
            for replay in replays {
                if replay.mutant_sha256.is_empty() || replay.mutant.trim().is_empty() {
                    return Err(ClausePortfolioError::new(
                        "mutation replay lacks a bound mutant identity",
                    ));
                }
                by_mutant
                    .entry(replay.mutant_sha256.as_str())
                    .or_default()
                    .push(replay);
            }
            let mut killed = 0usize;
            let mut scored = 0usize;
            let mut surviving_mutants = Vec::new();
            for group in by_mutant.values() {
                let seen: std::collections::HashSet<_> =
                    group.iter().map(|replay| replay.address.clone()).collect();
                if group.len() != expected_addresses.len() || seen != expected_addresses {
                    return Err(ClausePortfolioError::new(
                        "mutation replay does not contribute exactly once at every clause address",
                    ));
                }
                if group.iter().any(|replay| replay.mutant != group[0].mutant) {
                    return Err(ClausePortfolioError::new(
                        "one mutant digest names contradictory mutation descriptions",
                    ));
                }
                let mut applicable = false;
                let mut all_applicable_discharged = true;
                let mut mutant_killed = false;
                for replay in group {
                    let Some(procedure) = procedures_by_address.get(&replay.address) else {
                        return Err(ClausePortfolioError::new(
                            "mutation replay address is outside the clause portfolio",
                        ));
                    };
                    if replay.query_sha256.as_ref().is_some_and(String::is_empty) {
                        return Err(ClausePortfolioError::new(
                            "mutation replay carries an empty query identity",
                        ));
                    }
                    if matches!(
                        replay.outcome,
                        crate::engine::MutationReplayOutcome::ProofRejected
                            | crate::engine::MutationReplayOutcome::Counterexample
                            | crate::engine::MutationReplayOutcome::Discharged
                    ) && replay.query_sha256.is_none()
                    {
                        return Err(ClausePortfolioError::new(
                            "decisive mutation replay lacks its exact query identity",
                        ));
                    }
                    match (procedure, &replay.outcome) {
                        (
                            ClauseProcedure::Nlsat { .. },
                            crate::engine::MutationReplayOutcome::Inapplicable,
                        ) => {}
                        (
                            ClauseProcedure::AuthorLean { .. },
                            crate::engine::MutationReplayOutcome::Inapplicable,
                        )
                        | (
                            ClauseProcedure::Nlsat { .. },
                            crate::engine::MutationReplayOutcome::Discharged
                            | crate::engine::MutationReplayOutcome::ProofRejected
                            | crate::engine::MutationReplayOutcome::Counterexample
                            | crate::engine::MutationReplayOutcome::Unavailable
                            | crate::engine::MutationReplayOutcome::Undecided,
                        ) => {
                            return Err(ClausePortfolioError::new(
                                "mutation replay outcome contradicts its clause procedure",
                            ));
                        }
                        (_, crate::engine::MutationReplayOutcome::ProofRejected)
                        | (_, crate::engine::MutationReplayOutcome::Counterexample) => {
                            applicable = true;
                            mutant_killed = true;
                        }
                        (_, crate::engine::MutationReplayOutcome::Discharged) => {
                            applicable = true;
                        }
                        (_, crate::engine::MutationReplayOutcome::Unavailable)
                        | (_, crate::engine::MutationReplayOutcome::Undecided) => {
                            applicable = true;
                            all_applicable_discharged = false;
                        }
                        (_, crate::engine::MutationReplayOutcome::Inapplicable) => {}
                    }
                }
                if mutant_killed {
                    killed += 1;
                    scored += 1;
                } else if applicable && all_applicable_discharged {
                    scored += 1;
                    surviving_mutants.push(group[0].mutant.as_str());
                }
            }
            if self.contract_quality.mutants_killed != format!("{killed}/{scored}")
                || self.contract_quality.equivalent_mutants_excluded != 0
                || match (
                    &self.contract_quality.survivor,
                    surviving_mutants.as_slice(),
                ) {
                    (None, []) => false,
                    (Some(recorded), survivors) => {
                        !survivors.iter().any(|survivor| *survivor == recorded)
                    }
                    _ => true,
                }
            {
                return Err(ClausePortfolioError::new(
                    "published mutation score does not equal the addressed replay fold",
                ));
            }
        }
        let first = clauses[0];
        let homogeneous = all_discharged
            && clauses.iter().all(|(obligation, c)| {
                c.classification == first.1.classification
                    && c.position == first.1.position
                    && c.procedure == first.1.procedure
                    && obligation.engine == first.0.engine
                    && obligation.trust == first.0.trust
            });
        let kind = if !all_discharged {
            ClausePortfolioKind::Incomplete
        } else if !final_accepted {
            ClausePortfolioKind::PolicyRejected
        } else if homogeneous {
            ClausePortfolioKind::AcceptedHomogeneous
        } else {
            ClausePortfolioKind::Heterogeneous
        };
        if kind == ClausePortfolioKind::AcceptedHomogeneous {
            let expected_attr =
                first
                    .0
                    .engine
                    .as_ref()
                    .map(|engine| crate::engine::EngineAttribution {
                        engine: engine.clone(),
                        trust_profile: first.0.trust.clone(),
                    });
            if self.classification.as_ref() != Some(&first.1.classification)
                || self.certification.as_ref() != first.1.position.as_ref()
                || self.engine_attribution != expected_attr
            {
                return Err(ClausePortfolioError::new(
                    "homogeneous portfolio disagrees with singular authority",
                ));
            }
        } else if self.classification.is_some()
            || self.certification.is_some()
            || self.engine_attribution.is_some()
        {
            return Err(ClausePortfolioError::new(
                "non-homogeneous or rejected portfolio carries singular authority",
            ));
        }
        Ok(Some(ClausePortfolio {
            kind,
            clauses: clauses.into_iter().map(|(_, c)| c.clone()).collect(),
        }))
    }

    pub(crate) fn with_clause_portfolio(
        mut self,
        obligations: Vec<ObligationResult>,
        final_accepted: bool,
    ) -> Result<Self, ClausePortfolioError> {
        self.obligations = obligations;
        self.certification = None;
        self.classification = None;
        self.engine_attribution = None;
        let clauses: Vec<&ObligationResult> = self
            .obligations
            .iter()
            .filter(|o| o.clause_certification.is_some())
            .collect();
        let all_discharged = !clauses.is_empty()
            && clauses.iter().all(|o| {
                matches!(
                    o.clause_certification.as_ref().unwrap().terminal,
                    ClauseTerminalState::Discharged
                )
            });
        let homogeneous = all_discharged
            && clauses.iter().skip(1).all(|o| {
                let a = clauses[0];
                let x = a.clause_certification.as_ref().unwrap();
                let y = o.clause_certification.as_ref().unwrap();
                x.classification == y.classification
                    && x.position == y.position
                    && x.procedure == y.procedure
                    && a.engine == o.engine
                    && a.trust == o.trust
            });
        if final_accepted && homogeneous {
            let first = clauses[0];
            let clause = first.clause_certification.as_ref().unwrap();
            self.classification = Some(clause.classification.clone());
            self.certification = clause.position.clone();
            self.engine_attribution =
                first
                    .engine
                    .as_ref()
                    .map(|engine| crate::engine::EngineAttribution {
                        engine: engine.clone(),
                        trust_profile: first.trust.clone(),
                    });
        }
        self.audit_admission.clause_policy_digest = Some(self.clause_policy_digest());
        self.clause_portfolio(final_accepted)?;
        if final_accepted {
            // Mixed-route producers return early from the ordinary arbiter path,
            // so this sealed assembler is the point at which their accepted live
            // disposition becomes known. Audit must not reinterpret an unstamped,
            // all-discharged portfolio as policy-rejected.
            self.live_disposition = LiveDispositionStamp(Some(LiveResultDisposition::Accepted));
        }
        Ok(self)
    }

    pub(crate) fn requires_verus_artifact_validation(&self) -> bool {
        self.audit_admission.verus.is_some()
            || self.classification.as_ref().is_some_and(|classification| {
                classification.fragment.starts_with("thermite-verus-")
            })
            || self.certification.as_ref().is_some_and(|position| {
                position
                    .discharged_trust
                    .iter()
                    .any(|fact| fact.starts_with("thermite-verus-query-v1:"))
            })
    }

    /// Assemble a #5 certificate from the pipeline data (REQ-2). `check.rs`
    /// derives `level`/`obligations` from verus and `effects` from the item's
    /// `fx` row; the forward-declared and reserved fields take their #5
    /// values here.
    pub fn new(
        item: impl Into<String>,
        level: Level,
        effects: Vec<String>,
        solver_time_ms: u64,
        obligations: Vec<ObligationResult>,
    ) -> Self {
        Certificate {
            item: item.into(),
            level,
            certification: legacy_position(level),
            classification: None,
            audit_admission: AuditAdmission::live(),
            solver_time_ms,
            contract_quality: ContractQuality::forward_declared(),
            effects,
            slag: false,
            slag_meta: None,
            reject: None,
            obligations,
            cached: false,
            solver_profile: None,
            suggested_move: None,
            lowered_assurance: false,
            degrade_reason: None,
            strengthening: Vec::new(),
            boundary: false,
            boundary_target: None,
            assurance_scope: None,
            engine_attribution: None,
            covenant_evidence: None,
            meaning_audit: None,
            burn: None,
            resource_flow: None,
            live_disposition: LiveDispositionStamp::default(),
        }
    }

    /// Build a timeout certificate for a verus run that exhausted its resource
    /// budget (`.design/forge/solver-profiles.md` REQ-6/REQ-7). Distinct from a
    /// counterexample-L0: a timeout records `Level::L0` with a structured
    /// `RejectReason { cause: "VerusTimeout" }` (not a `postcondition not
    /// satisfied` witness), carries the parsed `SolverProfile`, and populates
    /// `suggested_move` with the profile-derived proof-repair hint. v0.1 has no
    /// automatic degrade (#10), so the level is the un-discharged `L0` with the
    /// timeout reason — a timeout is reported, not treated as success
    /// (R-CODE-4). The profile + `suggested_move` are oracle-excluded (§5.3).
    pub fn timeout(
        item: impl Into<String>,
        effects: Vec<String>,
        solver_time_ms: u64,
        profile: SolverProfile,
        suggested_move: Option<SuggestedMove>,
        detail: String,
    ) -> Self {
        let reason = RejectReason {
            cause: "VerusTimeout".to_string(),
            detail,
        };
        let obligation =
            ObligationResult::failed(reason.cause.clone(), None, Some(reason.detail.clone()));
        Certificate {
            item: item.into(),
            level: Level::L0,
            certification: legacy_position(Level::L0),
            classification: None,
            audit_admission: AuditAdmission::live(),
            solver_time_ms,
            contract_quality: ContractQuality::forward_declared(),
            effects,
            slag: false,
            slag_meta: None,
            reject: Some(reason),
            obligations: vec![obligation],
            cached: false,
            solver_profile: Some(profile),
            suggested_move,
            lowered_assurance: false,
            degrade_reason: None,
            strengthening: Vec::new(),
            boundary: false,
            boundary_target: None,
            assurance_scope: None,
            engine_attribution: None,
            covenant_evidence: None,
            meaning_audit: None,
            burn: None,
            resource_flow: None,
            live_disposition: LiveDispositionStamp::default(),
        }
    }

    /// Set this certificate's cache-provenance flag (#8;
    /// `.design/forge/proof-cache.md` REQ-7). Returns the certificate with
    /// `cached` set: `true` when served from a hit (verus skipped), `false` on a
    /// fresh verify. Only the provenance bit changes — every deterministic
    /// (oracle) field is untouched, so a hit stays oracle-equal to the fresh
    /// verify it was stored from (REQ-2, the soundness invariant). Consumed by
    /// `check::check_file` and `cache::store` (which clears it before persisting).
    pub fn with_cached(mut self, cached: bool) -> Self {
        self.cached = cached;
        self
    }

    pub(crate) fn with_live_disposition(mut self, disposition: LiveResultDisposition) -> Self {
        self.live_disposition = LiveDispositionStamp(Some(disposition));
        self
    }

    pub(crate) fn live_disposition(&self) -> Option<&LiveResultDisposition> {
        self.live_disposition.0.as_ref()
    }

    /// Graduate the two §7.1 structural-triage `contract_quality` bools to their
    /// #6-live `false` values on an item that passed triage
    /// (`.design/forge/vacuity-triage.md` REQ-6 / AC-7). The syntactic triage has
    /// confirmed the contract is not a syntactic tautology and its precondition is
    /// not syntactically vacuous, so these are asserted `false` (no longer
    /// forward-declared placeholders). The solver-derived truth of these fields
    /// (a non-syntactic tautology / unsat precondition) stays
    /// forward-declared for #13; `mutants_killed`/`survivor` stay #12.
    pub fn graduate_triage_clean(mut self) -> Self {
        self.contract_quality.tautology = false;
        self.contract_quality.vacuous_precondition = false;
        self
    }

    /// Build a valid-`#[slag]` certificate (`.design/forge/slag.md` REQ-2/REQ-4):
    /// `Level::L1` (contract runtime-enforced; body fiat-trusted), `slag: true`,
    /// the validated metadata, and a single discharged obligation recording the
    /// proof-exempt-by-fiat fact (not a verus obligation, since no proof was run). The
    /// triage bools graduate to live-`false` (a slag item still passes (a)/(b)/(c)
    /// triage before this is built).
    pub fn slag_l1(item: impl Into<String>, effects: Vec<String>, meta: SlagMeta) -> Self {
        Certificate {
            item: item.into(),
            level: Level::L1,
            certification: legacy_position(Level::L1),
            classification: None,
            audit_admission: AuditAdmission::live(),
            solver_time_ms: 0,
            contract_quality: ContractQuality::forward_declared(),
            effects,
            slag: true,
            slag_meta: Some(meta),
            reject: None,
            obligations: vec![ObligationResult::discharged(
                "contract enforced at L1 (slag); proof exempt by fiat",
            )],
            cached: false,
            solver_profile: None,
            suggested_move: None,
            lowered_assurance: false,
            degrade_reason: None,
            strengthening: Vec::new(),
            boundary: false,
            boundary_target: None,
            assurance_scope: None,
            engine_attribution: None,
            covenant_evidence: None,
            meaning_audit: None,
            burn: None,
            resource_flow: None,
            live_disposition: LiveDispositionStamp::default(),
        }
        .graduate_triage_clean()
    }

    /// Build a boundary-fn certificate (`.design/boundary/ffi-boundary.md` REQ-5,
    /// §9). The FFI analog of [`Certificate::slag_l1`]: a `#[boundary("crate::path")]`
    /// fn whose foreign body is unproven, so it certifies at `Level::L1` (the
    /// contract enforced at the crossing — `req` before, `ens` after — by
    /// `thermite_lower::l1`'s wrapper) with `boundary: true` and the foreign
    /// `target` recorded for the #15 TCB enumeration, not L3 (no verus run on a
    /// foreign body). A single discharged obligation records the trusted-by-fiat
    /// fact (not a verus obligation). The §7.1 (a)/(b)/(c) triage still applies (a
    /// boundary fn with a vacuous contract is rejected — slag-adjacent: it exempts
    /// proving, not stating), so the triage bools graduate to live-`false`
    /// (`graduate_triage_clean`, the slag precedent). `slag` stays `false`: a
    /// boundary fn is a distinct TCB category from a `#[slag]` block.
    pub fn boundary_l1(item: impl Into<String>, effects: Vec<String>, target: String) -> Self {
        Certificate {
            item: item.into(),
            level: Level::L1,
            certification: legacy_position(Level::L1),
            classification: None,
            audit_admission: AuditAdmission::live(),
            solver_time_ms: 0,
            contract_quality: ContractQuality::forward_declared(),
            effects,
            slag: false,
            slag_meta: None,
            reject: None,
            obligations: vec![ObligationResult::discharged(
                "contract enforced at L1 (boundary); foreign body trusted by fiat",
            )],
            cached: false,
            solver_profile: None,
            suggested_move: None,
            lowered_assurance: false,
            degrade_reason: None,
            strengthening: Vec::new(),
            boundary: true,
            boundary_target: Some(target),
            assurance_scope: None,
            engine_attribution: None,
            covenant_evidence: None,
            meaning_audit: None,
            burn: None,
            resource_flow: None,
            live_disposition: LiveDispositionStamp::default(),
        }
        .graduate_triage_clean()
    }

    /// Build a non-certified certificate for a triage / slag-validation reject
    /// (`.design/forge/vacuity-triage.md` REQ-5 / `slag.md` REQ-5). The item did
    /// not certify (`Level::L0`); the cert is a valid document carrying the
    /// structured `reject` cause + a single failed obligation naming it. `slag`
    /// records whether the rejected item carried a `#[slag]` attribute (its
    /// metadata is not carried, since the item did not certify).
    pub fn rejected(
        item: impl Into<String>,
        effects: Vec<String>,
        slag: bool,
        reason: RejectReason,
    ) -> Self {
        let obligation =
            ObligationResult::failed(reason.cause.clone(), None, Some(reason.detail.clone()));
        Certificate {
            item: item.into(),
            level: Level::L0,
            certification: legacy_position(Level::L0),
            classification: None,
            audit_admission: AuditAdmission::live(),
            solver_time_ms: 0,
            contract_quality: ContractQuality::forward_declared(),
            effects,
            slag,
            slag_meta: None,
            reject: Some(reason),
            obligations: vec![obligation],
            cached: false,
            solver_profile: None,
            suggested_move: None,
            lowered_assurance: false,
            degrade_reason: None,
            strengthening: Vec::new(),
            boundary: false,
            boundary_target: None,
            assurance_scope: None,
            engine_attribution: None,
            covenant_evidence: None,
            meaning_audit: None,
            burn: None,
            resource_flow: None,
            live_disposition: LiveDispositionStamp::default(),
        }
    }

    /// Build a non-certified certificate for a solver-vacuity reject (#13;
    /// `.design/forge/solver-vacuity.md` REQ-5/REQ-6). Like [`Certificate::rejected`]
    /// (`Level::L0`, the structured `reject` cause, one failed obligation naming
    /// it), but it also sets the solver-confirmed `contract_quality` bool that the
    /// detected degeneracy corresponds to (REQ-6, OQ-1): a `"SemanticTautology"`
    /// reject sets `contract_quality.tautology = true`; a `"VacuousPrecondition"`
    /// reject sets `contract_quality.vacuous_precondition = true`. `set_tautology` /
    /// `set_vacuous_precondition` are the two existing Appendix A bools — no schema
    /// change (R-SPEC-2); #13 only makes the `true` detection real (solver-confirmed)
    /// rather than the #6-syntactic `false`. Consumed by `check::gate_fn`.
    pub fn rejected_vacuity(
        item: impl Into<String>,
        effects: Vec<String>,
        reason: RejectReason,
        set_tautology: bool,
        set_vacuous_precondition: bool,
    ) -> Self {
        let mut cert = Certificate::rejected(item, effects, false, reason);
        cert.contract_quality.tautology = set_tautology;
        cert.contract_quality.vacuous_precondition = set_vacuous_precondition;
        cert
    }

    /// Stamp this certificate as an automatic degrade below L3 (issue #10;
    /// `.design/forge/degrade-ladder.md` REQ-4). Called by the ladder
    /// (`degrade::run_ladder`) on a cert achieved at L2 (kani) or L1 after a verus
    /// L3 timeout: sets the `lowered_assurance` flag `true` and records the
    /// `degrade_reason` (the `VerusTimeout` reason).
    /// Only the two degrade fields change — `level`, `effects`, `obligations`, and
    /// the rest are the underlying rung's verdict, untouched. This is not applied
    /// to a hard-failed cert (a counterexample): the ladder short-circuits a
    /// counterexample to a hard fail without degrading (REQ-2 anti-cheat).
    pub fn into_degraded(mut self, reason: RejectReason) -> Self {
        self.lowered_assurance = true;
        self.degrade_reason = Some(reason);
        self
    }

    /// Graduate the mutation-scoring `contract_quality` fields on a certified
    /// (kill-ratio-met) item (#12; `.design/forge/mutation-scoring.md` REQ-6). The
    /// item proved L3 and its frozen mutant set met the floor, so the cert records
    /// the real `"<killed>/<scored>"` kill ratio (graduated from the forward-
    /// declared `"0/0"`) and a representative `survivor` (the first surviving
    /// mutant's description, or `None` when every scored mutant was killed). No
    /// schema field is added or renamed (R-SPEC-2); this only makes the two
    /// existing Appendix A `contract_quality` fields live. Consumed by
    /// `check::check_file_with_options`'s post-L3 mutation stage.
    pub fn with_mutation_score(mut self, mutants_killed: String, survivor: Option<String>) -> Self {
        self.contract_quality.mutants_killed = mutants_killed;
        self.contract_quality.survivor = survivor;
        self
    }

    pub fn with_mutation_score_and_equivalents(
        self,
        mutants_killed: String,
        survivor: Option<String>,
        equivalent_mutants_excluded: usize,
    ) -> Self {
        let mut scored = self.with_mutation_score(mutants_killed, survivor);
        scored.contract_quality.equivalent_mutants_excluded = equivalent_mutants_excluded;
        scored
    }

    pub(crate) fn with_clause_mutation_replays(
        mut self,
        replays: Vec<ClauseMutationReplay>,
    ) -> Self {
        self.contract_quality.clause_mutation_replays = replays;
        self
    }

    /// Attach the §7 step-5 strengthening suggestions to this certificate (#14;
    /// `.design/forge/strengthening-probes.md` REQ-4). Advisory: only the additive
    /// `strengthening` field and the reserved `suggested_move` headline change —
    /// `level`, `reject`, and the `oracle_subset` are untouched, so a probe does not
    /// change the verdict (a `fn` that certified L3 still certifies L3 with the
    /// same oracle subset, now carrying suggestions). The top suggestion (the first
    /// in the deterministic family order) becomes the `suggested_move` headline
    /// (§5.1 "every message is a prompt"); the full ordered list lives in
    /// `strengthening`. An empty `suggestions` is a no-op (the
    /// `suggested_move` stays whatever it was, the list stays empty). Consumed by
    /// `check::strengthen_certificate`.
    pub fn with_strengthening(mut self, suggestions: Vec<Suggestion>) -> Self {
        if let Some(top) = suggestions.first() {
            // The headline hint (the §5.1 reserved `suggested_move` slot): the
            // top adoptable tightening. A probe does not overwrite a non-`None`
            // `suggested_move` (e.g. a timeout cert's profile hint); a probe
            // only runs on a certified L3 item whose `suggested_move` is `None`,
            // so this is the first writer in that path.
            self.suggested_move = Some(SuggestedMove {
                kind: "strengthen-ens".to_string(),
                detail: match &top.kills_survivor {
                    Some(survivor) => format!(
                        "consider strengthening `ens` with `{}` — it holds for your body and \
                         would kill survivor `{survivor}`",
                        top.clause
                    ),
                    None => format!(
                        "consider strengthening `ens` with `{}` — it holds for your body and \
                         pins the result more tightly than the current `ens`",
                        top.clause
                    ),
                },
            });
        }
        self.strengthening = suggestions;
        self
    }

    /// Build a non-certified certificate for a weak-contract reject (#12;
    /// `.design/forge/mutation-scoring.md` REQ-5/REQ-6). The item's body
    /// proved L3, but its frozen mutant set scored below the floor — the contract
    /// under-constrains the body (mutants survive). Like [`Certificate::rejected`]
    /// (`Level::L0`, the structured `reject` cause, one failed obligation naming
    /// it), but it also records the real `mutants_killed` ratio and the surviving-
    /// mutant `survivor` — the §7 "precise strengthening prompt". The `cause` is
    /// `"WeakContract"` (a distinct tag namespace from #6/#13's vacuity causes), so
    /// a cert reader can tell an under-constraining contract from a degenerate one.
    /// Consumed by `check::check_file_with_options`.
    pub fn rejected_weak_contract(
        item: impl Into<String>,
        effects: Vec<String>,
        mutants_killed: String,
        survivor: String,
    ) -> Self {
        let reason = RejectReason {
            cause: "WeakContract".to_string(),
            detail: format!(
                "§7 step 4: the contract under-constrains the body — mutation kill ratio \
                 {mutants_killed} is below the floor; mutant `{survivor}` survived (verus \
                 proved the deliberately-wrong body against this contract), so the contract \
                 does not distinguish it from the real body — strengthen the `ens` to pin \
                 the behavior `{survivor}` changes"
            ),
        };
        // Reuse the triage-clean reject shape (the item passed #6 + #13 + L3; the
        // only defect is contract strength), then record the mutation fields.
        Certificate::rejected(item, effects, false, reason)
            .with_mutation_score(mutants_killed, Some(survivor))
    }

    /// Attach the §9 assurance scope to this certificate (#17;
    /// `.design/forge/e2e-vs-boundary.md` REQ-3). Returns the cert with
    /// `assurance_scope` set to the classified value. Orthogonal to the verdict
    /// (REQ-5): only this field changes — `level`, `reject`, `boundary`, `slag`
    /// are untouched, so a fn keeps its achieved level and records its scope (an
    /// L3 fn whose closure crosses a boundary stays `Level::L3` + `ToBoundary`).
    /// Set by `check::check_file_with_options` after `closure::classify`.
    pub fn with_assurance_scope(mut self, scope: AssuranceScope) -> Self {
        if let Some(position) = &mut self.certification {
            position.boundary = match &scope {
                AssuranceScope::EndToEnd => CertificationBoundary::EndToEnd,
                AssuranceScope::ToBoundary { via } => {
                    CertificationBoundary::ToBoundary { via: via.clone() }
                }
            };
        }
        self.assurance_scope = Some(scope);
        self
    }

    /// Atomically attach the RFC-3 post-classification position and the
    /// pre-discharge classifier prognosis. This is the migration seam for new
    /// producers: neither half is persisted when the position is incoherent or
    /// the classifier identity is empty.
    pub fn with_rfc3_coordinates(
        mut self,
        position: CertificationPosition,
        classification: ClassificationCertificate,
    ) -> Result<Self, IncoherentCertificationPosition> {
        position.validate()?;
        if classification.fragment.trim().is_empty() {
            return Err(IncoherentCertificationPosition {
                reason: "classification fragment identity must be non-empty",
            });
        }
        self.certification = Some(position);
        self.classification = Some(classification);
        Ok(self)
    }

    /// Attach the runtime-enforced L1 RFC-3 pair from one opaque checked-lowering
    /// artifact. The wrapper identity is retained as a discharged bridge fact,
    /// which both binds the certificate to emitted code and marks migrated L1
    /// documents for fail-closed partial-pair validation. Historical unmarked L1
    /// position-only documents remain readable.
    pub fn with_l1_artifact(
        self,
        artifact: &thermite_lower::L1Artifact,
    ) -> Result<Self, IncoherentCertificationPosition> {
        if self.level != Level::L1 {
            return Err(IncoherentCertificationPosition {
                reason: "an L1 artifact can only certify Level::L1",
            });
        }
        if self.item != artifact.item() {
            return Err(IncoherentCertificationPosition {
                reason: "the L1 artifact item must match the certificate item",
            });
        }
        if self.effects != effects_of(artifact.effect_row()) {
            return Err(IncoherentCertificationPosition {
                reason: "the L1 artifact effect row must match the certificate effect row",
            });
        }
        let boundary = match artifact.route() {
            thermite_lower::L1Route::Runtime | thermite_lower::L1Route::Diverge => {
                if self.slag || self.boundary {
                    return Err(IncoherentCertificationPosition {
                        reason: "runtime/diverge L1 artifacts cannot certify slag or FFI rows",
                    });
                }
                CertificationBoundary::EndToEnd
            }
            thermite_lower::L1Route::Slag => {
                if !self.slag || self.boundary {
                    return Err(IncoherentCertificationPosition {
                        reason: "a slag L1 artifact requires a slag certificate row",
                    });
                }
                CertificationBoundary::ToBoundary {
                    via: artifact.item().to_string(),
                }
            }
            thermite_lower::L1Route::Boundary { target } => {
                if !self.boundary || self.slag || self.boundary_target.as_deref() != Some(target) {
                    return Err(IncoherentCertificationPosition {
                        reason: "an FFI L1 artifact requires the same certificate boundary target",
                    });
                }
                CertificationBoundary::ToBoundary {
                    via: artifact.item().to_string(),
                }
            }
        };
        let attached = self.with_rfc3_coordinates(
            CertificationPosition {
                scope: CertificationScope::PerExecution,
                refutation: RefutationChannel::Abort,
                residual_trust: ResidualTrust::Fiat,
                discharged_trust: vec![artifact.wrapper_identity().to_string()],
                boundary,
            },
            ClassificationCertificate {
                fragment: artifact.classifier_fragment().to_string(),
                verdict: ClassificationVerdict::Admitted,
            },
        )?;
        attached.validate_l1_artifact(artifact, None)?;
        Ok(attached)
    }

    /// Attach the homogeneous general-Verus route from checked lowering. The
    /// classifier and query identity exist before solver execution and therefore
    /// remain identical on proof success and every non-success outcome.
    pub fn with_verus_artifact(
        self,
        artifact: &thermite_lower::L3Artifact,
        succeeded: bool,
    ) -> Result<Self, IncoherentCertificationPosition> {
        let expected_level = if succeeded { Level::L3 } else { Level::L0 };
        if self.level != expected_level || self.item != artifact.item() {
            return Err(IncoherentCertificationPosition {
                reason: "the Verus artifact item/outcome must match the certificate",
            });
        }
        let expected_effects = artifact
            .effect_row()
            .map_or_else(|| vec!["pure".to_string()], effects_of);
        if self.effects != expected_effects {
            return Err(IncoherentCertificationPosition {
                reason: "the Verus artifact effect row must match the certificate",
            });
        }
        let (scope, refutation, residual_trust) = if succeeded {
            (
                CertificationScope::All,
                RefutationChannel::Incomplete,
                ResidualTrust::Solver,
            )
        } else {
            (
                CertificationScope::None,
                RefutationChannel::None,
                ResidualTrust::Fiat,
            )
        };
        let mut attached = self.with_rfc3_coordinates(
            CertificationPosition {
                scope,
                refutation,
                residual_trust,
                discharged_trust: vec![artifact.query_identity().to_string()],
                boundary: CertificationBoundary::EndToEnd,
            },
            ClassificationCertificate {
                fragment: artifact.classifier_fragment().to_string(),
                verdict: ClassificationVerdict::Admitted,
            },
        )?;
        attached.audit_admission.verus = Some(VerusAuditAuthority {
            item: artifact.item().to_string(),
            effects: expected_effects,
            query_identity: artifact.query_identity().to_string(),
            succeeded,
        });
        attached.validate_verus_artifact_authority()?;
        Ok(attached)
    }

    /// Attach RFC-11 evidence produced by the typed Lean-replay path. The
    /// public block is checked against the pre-execution L3 artifact before a
    /// private audit capability is minted, so display text cannot manufacture
    /// resource authority.
    pub(crate) fn with_resource_flow_evidence_for_witness(
        mut self,
        witness: &thermite_lower::ResourceFlowWitness,
        evidence: ResourceFlowEvidence,
    ) -> Result<Self, IncoherentCertificationPosition> {
        self.validate_resource_evidence_against(witness, &evidence)?;
        self.resource_flow = Some(evidence.clone());
        self.audit_admission.resource = Some(ResourceAuditAuthority { evidence });
        Ok(self)
    }

    pub(crate) fn validate_resource_flow_authority(
        &self,
    ) -> Result<(), IncoherentCertificationPosition> {
        match (&self.resource_flow, &self.audit_admission.resource) {
            (None, None) => Ok(()),
            (Some(public), Some(authority)) if public == &authority.evidence => Ok(()),
            (Some(_), None) => Err(IncoherentCertificationPosition {
                reason: "RFC-11 evidence requires live formal-replay authority",
            }),
            (None, Some(_)) => Err(IncoherentCertificationPosition {
                reason: "RFC-11 live authority requires a public evidence block",
            }),
            _ => Err(IncoherentCertificationPosition {
                reason: "RFC-11 public evidence differs from live replay authority",
            }),
        }
    }

    pub(crate) fn inherit_resource_flow_from(mut self, source: &Certificate) -> Self {
        self.resource_flow = source.resource_flow.clone();
        self.audit_admission.resource = source.audit_admission.resource.clone();
        self
    }

    pub(crate) fn persisted_resource_evidence_matches(
        &self,
        artifact: &thermite_lower::L3Artifact,
        expected: Option<&ResourceFlowEvidence>,
    ) -> bool {
        match (&self.resource_flow, expected) {
            (None, None) => artifact.resource_witness().is_none(),
            (Some(actual), Some(expected)) => {
                artifact.resource_witness().is_some() && actual == expected
            }
            _ => false,
        }
    }

    fn validate_resource_evidence_against(
        &self,
        witness: &thermite_lower::ResourceFlowWitness,
        evidence: &ResourceFlowEvidence,
    ) -> Result<(), IncoherentCertificationPosition> {
        let expected_forgets: Vec<_> = witness
            .functions
            .iter()
            .flat_map(|function| {
                function
                    .forgets
                    .iter()
                    .map(|forget| ResourceForgetFootprint {
                        function: function.function.clone(),
                        disposition: forget.label.clone(),
                        place: forget.place.clone(),
                        regions: forget.priced_regions.clone(),
                    })
            })
            .collect();
        let replay = &evidence.formal_replay;
        let expected_residual = vec![
            ResourceResidualTrust::Parser,
            ResourceResidualTrust::TypeProvenanceResolution,
            ResourceResidualTrust::ResourceFlowComputation,
            ResourceResidualTrust::WitnessExtraction,
            ResourceResidualTrust::ExecutableTargetBehavior,
        ];
        if evidence.verdict != ResourceFlowVerdict::Accepted
            || evidence.forgets != expected_forgets
            || replay.checker != "Thermite.ResourceFlow/v1"
            || replay.witness_version != witness.version
            || replay.canonical_ast_sha256 != witness.canonical_ast_sha256
            || replay.checked_resource_sha256 != witness.checked_resource_sha256
            || replay.verdict != ResourceFormalReplayVerdict::KernelAccepted
            || evidence.residual_trust != expected_residual
        {
            return Err(IncoherentCertificationPosition {
                reason: "RFC-11 evidence does not match the checked resource artifact",
            });
        }
        Ok(())
    }

    /// Revalidate the persisted general-Verus coordinates against the private
    /// live-producer facts retained across certificate transformations.
    pub(crate) fn validate_verus_artifact_authority(
        &self,
    ) -> Result<(), IncoherentCertificationPosition> {
        let authority =
            self.audit_admission
                .verus
                .as_ref()
                .ok_or(IncoherentCertificationPosition {
                    reason: "migrated Verus evidence requires live artifact authority",
                })?;
        self.validate_verus_evidence_against(authority)
    }

    /// Validate a deserialized main-item cache candidate against the artifact
    /// freshly constructed for this invocation. This deliberately does not mint
    /// or restore audit authority; audit continues to reject serialized values.
    pub(crate) fn persisted_verus_artifact_matches(
        &self,
        artifact: &thermite_lower::L3Artifact,
    ) -> bool {
        let succeeded = match self.level {
            Level::L3 => true,
            Level::L0 => false,
            _ => return false,
        };
        let effects = artifact
            .effect_row()
            .map_or_else(|| vec!["pure".to_string()], effects_of);
        let authority = VerusAuditAuthority {
            item: artifact.item().to_string(),
            effects,
            query_identity: artifact.query_identity().to_string(),
            succeeded,
        };
        self.validate_verus_evidence_against(&authority).is_ok()
    }

    fn validate_verus_evidence_against(
        &self,
        authority: &VerusAuditAuthority,
    ) -> Result<(), IncoherentCertificationPosition> {
        let expected_level = if authority.succeeded {
            Level::L3
        } else {
            Level::L0
        };
        if self.item != authority.item
            || self.effects != authority.effects
            || self.level != expected_level
        {
            return Err(IncoherentCertificationPosition {
                reason: "persisted Verus item/effects/outcome do not match live authority",
            });
        }
        let classification =
            self.classification
                .as_ref()
                .ok_or(IncoherentCertificationPosition {
                    reason: "migrated Verus evidence requires its classification",
                })?;
        if classification.fragment != "thermite-verus-v1"
            || classification.verdict != ClassificationVerdict::Admitted
        {
            return Err(IncoherentCertificationPosition {
                reason: "persisted Verus classification does not match live authority",
            });
        }
        let position = self
            .certification
            .as_ref()
            .ok_or(IncoherentCertificationPosition {
                reason: "migrated Verus evidence requires its certification position",
            })?;
        let (scope, refutation, residual_trust) = if authority.succeeded {
            (
                CertificationScope::All,
                RefutationChannel::Incomplete,
                ResidualTrust::Solver,
            )
        } else {
            (
                CertificationScope::None,
                RefutationChannel::None,
                ResidualTrust::Fiat,
            )
        };
        let boundary = match self.assurance_scope.as_ref() {
            Some(AssuranceScope::ToBoundary { via }) => {
                CertificationBoundary::ToBoundary { via: via.clone() }
            }
            None | Some(AssuranceScope::EndToEnd) => CertificationBoundary::EndToEnd,
        };
        if position.scope != scope
            || position.refutation != refutation
            || position.residual_trust != residual_trust
            || position.discharged_trust != [authority.query_identity.as_str()]
            || position.boundary != boundary
        {
            return Err(IncoherentCertificationPosition {
                reason: "persisted Verus coordinates do not match live artifact authority",
            });
        }
        Ok(())
    }

    /// Revalidate a persisted migrated L1 row against checked lowering from the
    /// source program. This is deliberately stronger than pair-shape validation:
    /// it binds every mutable manifest field that determines route provenance to
    /// the opaque pre-execution artifact before audit projection.
    pub(crate) fn validate_l1_artifact(
        &self,
        artifact: &thermite_lower::L1Artifact,
        expected_scope: Option<&AssuranceScope>,
    ) -> Result<(), IncoherentCertificationPosition> {
        if self.level != Level::L1 || self.item != artifact.item() {
            return Err(IncoherentCertificationPosition {
                reason: "persisted L1 item/level does not match its checked artifact",
            });
        }
        if self.effects != effects_of(artifact.effect_row()) {
            return Err(IncoherentCertificationPosition {
                reason: "persisted L1 effects do not match the checked artifact",
            });
        }
        let expected_fragment = artifact.classifier_fragment();
        let classification =
            self.classification
                .as_ref()
                .ok_or(IncoherentCertificationPosition {
                    reason: "a checked L1 artifact requires its classification pair",
                })?;
        if classification.fragment != expected_fragment
            || classification.verdict != ClassificationVerdict::Admitted
        {
            return Err(IncoherentCertificationPosition {
                reason: "persisted L1 classification does not match the checked route",
            });
        }
        let position = self
            .certification
            .as_ref()
            .ok_or(IncoherentCertificationPosition {
                reason: "a checked L1 artifact requires its certification position",
            })?;
        if position.scope != CertificationScope::PerExecution
            || position.refutation != RefutationChannel::Abort
            || position.residual_trust != ResidualTrust::Fiat
            || position.discharged_trust != [artifact.wrapper_identity()]
        {
            return Err(IncoherentCertificationPosition {
                reason: "persisted L1 position does not match the checked wrapper",
            });
        }
        let route_boundary = match artifact.route() {
            thermite_lower::L1Route::Runtime | thermite_lower::L1Route::Diverge => {
                if self.slag || self.boundary || self.boundary_target.is_some() {
                    return Err(IncoherentCertificationPosition {
                        reason: "runtime/diverge L1 route flags contradict the checked artifact",
                    });
                }
                CertificationBoundary::EndToEnd
            }
            thermite_lower::L1Route::Slag => {
                let expected_meta =
                    artifact
                        .slag_metadata()
                        .ok_or(IncoherentCertificationPosition {
                            reason: "checked slag artifact is missing its source metadata",
                        })?;
                let actual_meta =
                    self.slag_meta
                        .as_ref()
                        .ok_or(IncoherentCertificationPosition {
                            reason: "persisted slag L1 row is missing its source metadata",
                        })?;
                if !self.slag
                    || self.boundary
                    || self.boundary_target.is_some()
                    || (
                        actual_meta.reason.as_str(),
                        actual_meta.owner.as_str(),
                        actual_meta.review.as_str(),
                    ) != expected_meta
                {
                    return Err(IncoherentCertificationPosition {
                        reason: "slag L1 route fields contradict the checked artifact",
                    });
                }
                CertificationBoundary::ToBoundary {
                    via: artifact.item().to_string(),
                }
            }
            thermite_lower::L1Route::Boundary { target } => {
                if self.slag
                    || self.slag_meta.is_some()
                    || !self.boundary
                    || self.boundary_target.as_deref() != Some(target)
                {
                    return Err(IncoherentCertificationPosition {
                        reason: "FFI L1 route fields contradict the checked artifact",
                    });
                }
                CertificationBoundary::ToBoundary {
                    via: artifact.item().to_string(),
                }
            }
        };
        let expected_boundary = match expected_scope {
            Some(scope) => {
                if self.assurance_scope.as_ref() != Some(scope) {
                    return Err(IncoherentCertificationPosition {
                        reason: "persisted L1 assurance scope does not match program closure",
                    });
                }
                match scope {
                    AssuranceScope::EndToEnd => CertificationBoundary::EndToEnd,
                    AssuranceScope::ToBoundary { via } => {
                        CertificationBoundary::ToBoundary { via: via.clone() }
                    }
                }
            }
            None => {
                if self.assurance_scope.is_some() {
                    return Err(IncoherentCertificationPosition {
                        reason: "pre-closure L1 artifact attachment cannot carry authored scope",
                    });
                }
                route_boundary
            }
        };
        if position.boundary != expected_boundary {
            return Err(IncoherentCertificationPosition {
                reason: "persisted L1 boundary does not match its route/closure classification",
            });
        }
        Ok(())
    }

    /// Attach the per-obligation engine attribution (`.design/verified/
    /// proof-backends.md` REQ-4, increment (iii), #247). Returns the cert with
    /// `engine_attribution` set to the discharging engine's `{engine, trust_profile}`
    /// pair — recorded only when a non-default engine (Lean) proved the item, so an
    /// auditor sees the smaller trusted base. Orthogonal to the verdict (REQ-4 — the
    /// `Level` is unchanged; the trust base is the auditor-visible refinement): only
    /// this field changes. The default Verus path does not call this (the field stays
    /// `None`), so the cert oracle is byte-identical (the `serde(default)` keeps the
    /// golden green). Set by `cli::run_check`'s `--engine lean` path.
    #[must_use]
    pub fn with_engine_attribution(
        mut self,
        attribution: crate::engine::EngineAttribution,
    ) -> Self {
        if attribution.engine.starts_with("lean") {
            if let Some(position) = &mut self.certification {
                position.refutation = RefutationChannel::Empirical;
                position.residual_trust = ResidualTrust::LeanChecked;
            }
        }
        self.engine_attribution = Some(attribution);
        self
    }

    /// Attach the covenant evidence block to a validated forge-tier certificate
    /// (`.design/stage1-forge-tier.md` REQ-4, increment 2b; Q-ORACLE). Set on a
    /// covenant-routed item whose covenant validated (the L3 burn ran behind the
    /// covenant-before-burn gate); the evidence joins the cert oracle. A v1 item never
    /// calls this, so its `covenant_evidence` stays `None` and its cert is byte-stable.
    #[must_use]
    pub fn with_covenant_evidence(
        mut self,
        evidence: crate::covenant_engine::CovenantEvidence,
    ) -> Self {
        self.covenant_evidence = Some(evidence);
        self
    }

    /// Pin the definition-tower meaning audit on a forge-tier certificate
    /// (`.design/stage1-forge-tier.md` REQ-6c, increment 2d; Q-ORACLE). Set at certify
    /// time on a within-budget forge/Lean-discharged item: the unfolded-tower hash +
    /// depth + count join the cert oracle, so a reader can confirm the certified
    /// meaning. A v1 item never calls this, so its `meaning_audit` stays `None` and its
    /// cert is byte-stable. (An over-budget tower never reaches here — it is refused by
    /// [`Certificate::rejected_over_budget_tower`].)
    #[must_use]
    pub fn with_meaning_audit(mut self, audit: crate::meaning::MeaningAudit) -> Self {
        self.meaning_audit = Some(audit);
        self
    }

    /// Attach the L3 burn receipt to a forge-tier certificate whose proof closed a
    /// goal (`.design/stage1-forge-tier.md` REQ-7, increment 2e; RFC-1 §9). Set on the
    /// proof-view discharge path: the committed-proof token count + cited lemmas (and
    /// the optional authoring spend) join the cert as auditable burn evidence. Per
    /// Q-burn the receipt is oracle-excluded (re-authoring a proof changes it without
    /// changing the claim), so only this field changes — the verdict and the
    /// `oracle_subset` are untouched. A v1 item never calls this, so its `burn` stays
    /// `None` and its cert is byte-stable.
    #[must_use]
    pub fn with_burn(mut self, receipt: crate::burn::BurnReceipt) -> Self {
        self.burn = Some(receipt);
        self
    }

    /// Build a non-certified certificate for an over-budget definition tower
    /// (`.design/stage1-forge-tier.md` REQ-6c, increment 2d; AC-10). The forge-tier
    /// item's contract unfolds a definition tower deeper or wider than the Q2 default
    /// budget (depth 4 / 40 definitions) — a Goodhart move (hiding the claim
    /// behind an unreadable tower), refused at certify time. Like
    /// [`Certificate::rejected`] (`Level::L0`, the structured `DefinitionTowerBudget`
    /// cause, one failed obligation naming it), but it ALSO pins the unfolded-tower
    /// hash (`meaning_audit`) so the refusal carries the same auditable meaning record
    /// the cert would have pinned had it certified (AC-10: the certificate pins the
    /// unfolded tower hash even on the refusal).
    #[must_use]
    pub fn rejected_over_budget_tower(
        item: impl Into<String>,
        effects: Vec<String>,
        detail: String,
        audit: crate::meaning::MeaningAudit,
    ) -> Self {
        let reason = RejectReason {
            cause: "DefinitionTowerBudget".to_string(),
            detail,
        };
        let mut cert = Certificate::rejected(item, effects, false, reason);
        cert.meaning_audit = Some(audit);
        cert
    }

    /// Build a non-certified certificate for a covenant `falsify` refutation
    /// (`.design/stage1-forge-tier.md` REQ-4, increment 2b; AC-8). The covenant's
    /// executable semantics found a `req`-satisfying input whose body violates `ens` —
    /// a [`crate::verdict::CertVerdict::CovenantRefuted`] hard fail, the same
    /// never-degrades treatment as a `Counterexample` (`Level::L0`, never a lowered
    /// rung). Carries the structured `CovenantRefuted` reason naming the concrete
    /// counterexample + the deterministic seed, a single failed obligation, and the
    /// covenant evidence block (`falsify_refuted == 1`). The burn never ran (the gate
    /// short-circuited before proof search), so there is no proof/profile material.
    pub fn covenant_refuted(
        item: impl Into<String>,
        effects: Vec<String>,
        counterexample: &crate::verdict::CovenantCounterexample,
        evidence: crate::covenant_engine::CovenantEvidence,
    ) -> Self {
        let item = item.into();
        let reason = RejectReason {
            cause: "CovenantRefuted".to_string(),
            detail: format!(
                "the covenant `falsify` run refuted `{item}`: the input {} satisfies `req` \
                 but the executable body violates `ens` (deterministic, seed {:#x}) — a hard \
                 fail, never degraded (REQ-4 / AC-8)",
                counterexample.input, counterexample.seed
            ),
        };
        let obligation = ObligationResult::failed(
            "covenant refuted: ens violated on a req-satisfying input",
            None,
            Some(format!("counterexample: {}", counterexample.input)),
        );
        Certificate {
            item,
            level: Level::L0,
            certification: legacy_position(Level::L0),
            classification: None,
            audit_admission: AuditAdmission::live(),
            solver_time_ms: 0,
            contract_quality: ContractQuality::forward_declared(),
            effects,
            slag: false,
            slag_meta: None,
            reject: Some(reason),
            obligations: vec![obligation],
            cached: false,
            solver_profile: None,
            suggested_move: None,
            lowered_assurance: false,
            degrade_reason: None,
            strengthening: Vec::new(),
            boundary: false,
            boundary_target: None,
            assurance_scope: None,
            engine_attribution: None,
            covenant_evidence: Some(evidence),
            meaning_audit: None,
            burn: None,
            resource_flow: None,
            live_disposition: LiveDispositionStamp::default(),
        }
    }

    /// The deterministic, currently-producible oracle subset (REQ-3/REQ-6,
    /// `.design/forge/check.md` AC-1; ffi-boundary.md REQ-5/AC-2; e2e-vs-boundary.md
    /// REQ-3; stage1-forge-tier.md REQ-4): `(item, level, effects, slag, boundary,
    /// end_to_end, covenant_evidence)`. The forward-declared `contract_quality.*` and
    /// the non-deterministic `solver_time_ms` are excluded by being absent from this
    /// tuple. `boundary` joins because it is verdict-relevant (an L1 "to-the-boundary"
    /// is distinct from a proved/runtime L1); `boundary_target` is diagnostic and
    /// stays excluded (parallel to `slag_meta`).
    ///
    /// `end_to_end` is the §9 assurance-scope bit (#17), normalized via
    /// `scope_is_end_to_end`: `None` (the frozen golden `sum.cert.json`, which omits
    /// `assurance_scope`) and `Some(EndToEnd)` (a freshly-classified pure fn) are
    /// both `true`, so the golden subset stays oracle-stable (R-SPEC-2) while a
    /// `Some(ToBoundary)` verdict reads `false` and is oracle-visible (§9 / R-DEFER-9
    /// — a to-the-boundary guarantee is distinguished). The `via`
    /// crossing name is diagnostic detail and stays excluded (parallel to
    /// `boundary_target`).
    ///
    /// `covenant_evidence` is the REQ-4 forge-tier covenant block (Q-ORACLE): `None`
    /// for every v1 item (the golden default — no covenant declared) and `None` for a
    /// fresh v1 cert, so the v1 oracle stays byte-identical; a forge-routed item's
    /// deterministic witness/falsify counts + seed are oracle-visible (REQ-4: the
    /// covenant cannot drift silently).
    ///
    /// `meaning_audit` is the REQ-6c definition-tower hash + depth + count (Q-ORACLE:
    /// the meaning-audit hash joins the oracle subset): `None` for every v1 item (the
    /// golden default — no tower audit on the Verus path) and `None` for a fresh v1
    /// cert, so the v1 oracle stays byte-identical; a forge-tier item's unfolded-tower
    /// hash is oracle-visible (REQ-6c: the certified meaning cannot drift silently).
    ///
    /// The REQ-7 `burn` receipt (increment 2e) is ABSENT from this tuple
    /// (Q-burn): re-authoring a proof legitimately changes its committed-token count +
    /// authoring spend without changing what was proven, so it is oracle-excluded like
    /// `solver_time_ms` — a forge-tier cert and the same cert with its `burn` stripped
    /// compare oracle-equal.
    ///
    /// `bv_shadows` is the Lock 1 shadow flag (stage-3 REQ-3 / AC-4, Q-ORACLE:
    /// deterministic + verdict-relevant → included): the present `bv_shadow` blocks, in
    /// obligation source order. It is FILTERED to the tagged clauses (not one slot per
    /// obligation) so a v1 / untagged cert — whatever its obligation count, including a
    /// hand-authored golden with no `obligations` key — contributes an empty vec and stays
    /// oracle-byte-identical (the obligation count was never an oracle field and must not
    /// become one). A tagged clause's machine-semantics fork is then oracle-visible (a
    /// semantic fork CHANGES what the clause means — it cannot drift silently, unlike the
    /// provenance-only `engine`/`trust`, which stay oracle-excluded).
    #[allow(
        clippy::type_complexity,
        reason = "the oracle subset is deliberately a flat positional tuple of the \
                  verdict-relevant fields (item/level/effects/slag/boundary/end_to_end/\
                  covenant/meaning/bv_shadows) — a named struct would obscure that this IS \
                  the exact comparison surface the cert oracle compares; the tuple is the \
                  contract."
    )]
    pub fn oracle_subset(
        &self,
    ) -> (
        &str,
        Level,
        &[String],
        bool,
        bool,
        bool,
        Option<crate::covenant_engine::CovenantEvidence>,
        Option<crate::meaning::MeaningAudit>,
        Vec<BvShadow>,
        Option<ClauseOraclePortfolio>,
        Option<ResourceFlowEvidence>,
    ) {
        let clause_entries: Vec<_> = self
            .obligations
            .iter()
            .filter_map(|obligation| {
                obligation
                    .clause_certification
                    .clone()
                    .map(|certification| ClauseOracleEntry {
                        certification,
                        status: obligation.status.clone(),
                        engine: obligation.engine.clone(),
                        trust: obligation.trust.clone(),
                        verdict: obligation.verdict.clone(),
                        bv_shadow: obligation.bv_shadow.clone(),
                        reconstruction: obligation.reconstruction.clone(),
                    })
            })
            .collect();
        let clause_portfolio = (!clause_entries.is_empty()).then(|| ClauseOraclePortfolio {
            clauses: clause_entries,
            classification: self.classification.clone(),
            certification: self.certification.clone(),
            engine_attribution: self.engine_attribution.clone(),
            compatibility_burn: self.burn.clone(),
        });
        (
            &self.item,
            self.level,
            &self.effects,
            self.slag,
            self.boundary,
            scope_is_end_to_end(&self.assurance_scope),
            self.covenant_evidence,
            self.meaning_audit.clone(),
            self.obligations
                .iter()
                .filter_map(|o| o.bv_shadow.clone())
                .collect(),
            clause_portfolio,
            self.resource_flow.clone(),
        )
    }
}

/// The project-level assurance headline an aggregate of the per-fn certificates
/// resolves to (`.design/forge/degrade-ladder.md` REQ-6). Distinct from a per-fn
/// `Level`: a single hard-failed (non-certifying) function makes the whole project
/// `Failed` — a rejected item is not a rung the min ranges over (REQ-2/REQ-6 — "a
/// non-certifying item is not a rung"). When every function certifies, the
/// headline is the min over their achieved levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "level")]
pub enum ProjectAssurance {
    /// Every function certifies; the headline is the min over their levels (§5.2
    /// "the whole-project assurance level is the min over functions"). The carried
    /// `Level` is the weakest function's rung.
    Certified(Level),
    /// At least one function did not certify (a counterexample / reject /
    /// un-discharged proof). The project does not certify at any rung — it is a
    /// failure rather than a lowered level (REQ-2 anti-cheat: falsity never becomes a rung).
    Failed,
}

/// The project-level assurance manifest: an aggregate over the per-fn certificate
/// collection `forge check` returns (`.design/forge/degrade-ladder.md` REQ-5,
/// OQ-4 reading (b) — a render-time aggregate, not a separately-materialized
/// schema object). It is computed from `&[Certificate]` and carries the
/// project headline (the min-over-functions, REQ-6) plus the per-fn degrade view
/// (each fn's name, achieved level, and whether it was a lowered-assurance
/// degrade). Consumed by `cli::run_check` to display the project assurance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssuranceManifest {
    /// The project headline: the min over functions when all certify, else
    /// `Failed` (REQ-6).
    pub project: ProjectAssurance,
    /// The §9 project assurance scope (issue #17;
    /// `.design/forge/e2e-vs-boundary.md` REQ-4): end-to-end iff every fn is
    /// end-to-end, else to-the-boundary listing the crossings. A render-time
    /// aggregate of the per-fn `Certificate::assurance_scope`, orthogonal to the
    /// `project` level headline (a project can be `Certified(L3)` and
    /// `ToBoundary` — every fn proved its own contract while the closure crosses a
    /// foreign body).
    pub scope: ProjectScope,
    /// The per-fn degrade view in cert order: `(item, level, lowered_assurance)`.
    pub functions: Vec<FunctionAssurance>,
}

/// The project-level §9 assurance-scope claim (issue #17;
/// `.design/forge/e2e-vs-boundary.md` REQ-4). The aggregate of the per-fn
/// [`AssuranceScope`]s, orthogonal to [`ProjectAssurance`] (the level headline):
///
/// - [`ProjectScope::EndToEnd`] — every fn is end-to-end (no fn's closure reaches a
///   `#[boundary]`/`#[slag]`); the whole project is "verified, period".
/// - [`ProjectScope::ToBoundary`] — at least one fn is to-the-boundary; `crossings`
///   lists the reached `#[boundary]`/`#[slag]` fns (deduplicated, sorted —
///   deterministic, R-CODE-5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ProjectScope {
    /// Every fn is end-to-end: the project is "verified, period".
    EndToEnd,
    /// At least one fn reaches a crossing; `crossings` are the reached
    /// `#[boundary]`/`#[slag]` fns (sorted, deduplicated).
    ToBoundary {
        /// The `#[boundary]`/`#[slag]` fns the project's closures reach.
        crossings: Vec<String>,
    },
}

/// One function's row in the [`AssuranceManifest`] (REQ-5): its achieved level,
/// whether it certifies, and whether it was an automatic degrade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionAssurance {
    /// The function name.
    pub item: String,
    /// The achieved assurance level.
    pub level: Level,
    /// `true` iff this item certifies (a certified rung with no `reject`).
    pub certified: bool,
    /// `true` iff this level was reached by an automatic degrade below L3 (#10).
    pub lowered_assurance: bool,
}

impl AssuranceManifest {
    /// Aggregate a per-fn certificate collection into the project-level manifest
    /// (`.design/forge/degrade-ladder.md` REQ-5/REQ-6). The headline is the min
    /// over functions (`Level`'s `Ord`, `L0 < L1 < L2 < L3`) when every function
    /// certifies; if any function does not certify (a counterexample / reject /
    /// un-discharged proof — `cert_certifies` is `false`), the project is `Failed`
    /// (REQ-2/REQ-6: a non-certifying item is a project failure, not a lowered
    /// rung). An empty collection certifies vacuously at the top rung (`L3`) — a
    /// file with no `fn` items has nothing un-proved. Deterministic (REQ-7): a pure
    /// function of the cert collection, no wall-clock / ordering nondeterminism.
    pub fn aggregate(certs: &[Certificate]) -> Self {
        let functions: Vec<FunctionAssurance> = certs
            .iter()
            .map(|c| FunctionAssurance {
                item: c.item.clone(),
                level: c.level,
                certified: cert_certifies(c),
                lowered_assurance: c.lowered_assurance,
            })
            .collect();
        let project = if functions.iter().any(|f| !f.certified) {
            // REQ-2/REQ-6: any non-certifying function caps the project at failure,
            // not a lowered level — falsity is not a rung.
            ProjectAssurance::Failed
        } else {
            // Min over the certified functions' levels (REQ-6). Empty → vacuous L3.
            let min = functions.iter().map(|f| f.level).min().unwrap_or(Level::L3);
            ProjectAssurance::Certified(min)
        };
        let scope = project_scope(certs);
        AssuranceManifest {
            project,
            scope,
            functions,
        }
    }
}

/// Aggregate the per-fn [`AssuranceScope`]s into the §9 project scope claim (issue
/// #17; `.design/forge/e2e-vs-boundary.md` REQ-4): [`ProjectScope::EndToEnd`] iff
/// every cert is end-to-end (a `None` scope reads end-to-end — the golden default),
/// else [`ProjectScope::ToBoundary`] listing the reached crossings (the `via` fns,
/// deduplicated + sorted — deterministic, R-CODE-5). An empty collection is
/// vacuously end-to-end (nothing crosses a boundary). Orthogonal to the level
/// headline: a project can be `Certified(L3)` and `ToBoundary`.
fn project_scope(certs: &[Certificate]) -> ProjectScope {
    // BTreeSet → sorted + deduplicated crossings (deterministic).
    let mut crossings: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for cert in certs {
        if let Some(AssuranceScope::ToBoundary { via }) = &cert.assurance_scope {
            crossings.insert(via.clone());
        }
    }
    if crossings.is_empty() {
        ProjectScope::EndToEnd
    } else {
        ProjectScope::ToBoundary {
            crossings: crossings.into_iter().collect(),
        }
    }
}

/// `true` iff a certificate represents a certified item: no reject cause and a
/// certified assurance rung (`L4` kernel-grounded, `L3` proved, `L2` bounded, or
/// `L1` runtime/slag). `L0` (a triage / counterexample / timeout reject, or an
/// un-discharged proof) is not certified. Shared by the assurance aggregate (REQ-6)
/// and `cli`'s exit-code path (so the project headline and the exit code agree on
/// what "certifies"). L4 (the relax route's kernel-grounded rung, 2f) certifies like
/// any proven rung.
pub fn cert_certifies(cert: &Certificate) -> bool {
    cert.reject.is_none() && matches!(cert.level, Level::L4 | Level::L3 | Level::L2 | Level::L1)
}

/// Map a parsed `EffectRow` to the certificate's `effects` string vector
/// (REQ-2). `Pure` → `["pure"]`; a non-pure row maps each `Effect` to its
/// canonical lowercase token in declaration order (deterministic, R-CODE-5).
/// Covers every `Effect` variant (the whole closed enum), beyond the corpus's
/// `pure`.
pub fn effects_of(fx: &EffectRow) -> Vec<String> {
    match fx {
        EffectRow::Pure => vec!["pure".to_string()],
        EffectRow::Set(effects) => effects.iter().map(effect_token).collect(),
    }
}

/// The canonical lowercase token for one `Effect` (e.g. `read(x)`, `alloc`).
fn effect_token(effect: &Effect) -> String {
    match effect {
        Effect::Read(name) => format!("read({name})"),
        Effect::Write(name) => format!("write({name})"),
        Effect::Net(name) => format!("net({name})"),
        Effect::Alloc => "alloc".to_string(),
        Effect::Time => "time".to_string(),
        Effect::Rand => "rand".to_string(),
        Effect::Panic => "panic".to_string(),
        Effect::Diverge => "diverge".to_string(),
        // The #106 terminal-control atom (`fx term` → the `ioctl` seccomp grant,
        // runtime-sandbox.md REQ-7). A bare atom like `alloc`/`time`.
        Effect::Term => "term".to_string(),
        Effect::Owns(lock) => format!("owns({lock})"),
        Effect::Forgets(region) => format!("forgets({region})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod rfc3_coordinates {
        use super::*;

        fn position(
            scope: CertificationScope,
            refutation: RefutationChannel,
            residual_trust: ResidualTrust,
        ) -> CertificationPosition {
            CertificationPosition {
                scope,
                refutation,
                residual_trust,
                discharged_trust: Vec::new(),
                boundary: CertificationBoundary::EndToEnd,
            }
        }

        #[test]
        fn coherent_cells_validate() {
            let cells = [
                position(
                    CertificationScope::None,
                    RefutationChannel::None,
                    ResidualTrust::Fiat,
                ),
                position(
                    CertificationScope::PerExecution,
                    RefutationChannel::Abort,
                    ResidualTrust::Fiat,
                ),
                position(
                    CertificationScope::Bounded {
                        bound: "8".to_string(),
                    },
                    RefutationChannel::Trace {
                        bound: "8".to_string(),
                    },
                    ResidualTrust::Solver,
                ),
                position(
                    CertificationScope::All,
                    RefutationChannel::Incomplete,
                    ResidualTrust::Solver,
                ),
                position(
                    CertificationScope::All,
                    RefutationChannel::Complete,
                    ResidualTrust::Solver,
                ),
                position(
                    CertificationScope::All,
                    RefutationChannel::Empirical,
                    ResidualTrust::LeanChecked,
                ),
                position(
                    CertificationScope::All,
                    RefutationChannel::Complete,
                    ResidualTrust::LeanChecked,
                ),
            ];
            for cell in cells {
                cell.validate().expect("an RFC-3 cell must validate");
            }
        }

        #[test]
        fn incoherent_cells_fail_closed() {
            let cases = [
                position(
                    CertificationScope::All,
                    RefutationChannel::Complete,
                    ResidualTrust::Fiat,
                ),
                position(
                    CertificationScope::None,
                    RefutationChannel::Incomplete,
                    ResidualTrust::Solver,
                ),
                position(
                    CertificationScope::Bounded {
                        bound: "8".to_string(),
                    },
                    RefutationChannel::Trace {
                        bound: "7".to_string(),
                    },
                    ResidualTrust::Solver,
                ),
                position(
                    CertificationScope::All,
                    RefutationChannel::Incomplete,
                    ResidualTrust::LeanChecked,
                ),
                position(
                    CertificationScope::All,
                    RefutationChannel::Empirical,
                    ResidualTrust::Solver,
                ),
            ];
            for case in cases {
                assert!(
                    case.validate().is_err(),
                    "incoherent tuple accepted: {case:?}"
                );
            }
        }

        #[test]
        fn product_order_preserves_solver_forge_incomparability() {
            let bounded = position(
                CertificationScope::Bounded {
                    bound: "8".to_string(),
                },
                RefutationChannel::Trace {
                    bound: "8".to_string(),
                },
                ResidualTrust::Solver,
            );
            let solver = position(
                CertificationScope::All,
                RefutationChannel::Incomplete,
                ResidualTrust::Solver,
            );
            let forge = position(
                CertificationScope::All,
                RefutationChannel::Empirical,
                ResidualTrust::LeanChecked,
            );
            let top = position(
                CertificationScope::All,
                RefutationChannel::Complete,
                ResidualTrust::LeanChecked,
            );

            assert_eq!(solver.partial_cmp_assurance(&forge).unwrap(), None);
            assert_eq!(forge.partial_cmp_assurance(&solver).unwrap(), None);
            assert!(solver.dominates(&bounded).unwrap());
            assert!(forge.dominates(&bounded).unwrap());
            assert!(top.dominates(&solver).unwrap());
            assert!(top.dominates(&forge).unwrap());
        }

        #[test]
        fn explicit_position_is_validated_before_storage() {
            let bad = position(
                CertificationScope::PerExecution,
                RefutationChannel::Complete,
                ResidualTrust::Fiat,
            );
            assert!(Certificate::new("f", Level::L3, vec![], 0, vec![])
                .with_rfc3_coordinates(
                    bad,
                    ClassificationCertificate {
                        fragment: "thermite-test-v1".to_string(),
                        verdict: ClassificationVerdict::Admitted,
                    },
                )
                .is_err());
        }

        #[test]
        fn empty_classifier_identity_fails_closed() {
            let valid = position(
                CertificationScope::All,
                RefutationChannel::Incomplete,
                ResidualTrust::Solver,
            );
            assert!(Certificate::new("f", Level::L3, vec![], 0, vec![])
                .with_rfc3_coordinates(
                    valid,
                    ClassificationCertificate {
                        fragment: "  ".to_string(),
                        verdict: ClassificationVerdict::Admitted,
                    },
                )
                .is_err());
        }

        #[test]
        fn classification_without_position_is_rejected_by_public_reader() {
            let mut cert = Certificate::new("f", Level::L2, vec![], 0, vec![]);
            cert.classification = Some(ClassificationCertificate {
                fragment: "thermite-kani-v1".to_string(),
                verdict: ClassificationVerdict::Admitted,
            });
            assert!(cert.rfc3_coordinates().is_err());
        }

        #[test]
        fn serialized_position_only_l2_is_rejected_by_public_reader() {
            let mut source = Certificate::new("f", Level::L2, vec![], 0, vec![]);
            source.certification = Some(position(
                CertificationScope::Bounded {
                    bound: "unwind 5".to_string(),
                },
                RefutationChannel::Trace {
                    bound: "unwind 5".to_string(),
                },
                ResidualTrust::Solver,
            ));
            let json = serde_json::to_string(&source).expect("position-only JSON");
            let decoded: Certificate = serde_json::from_str(&json).expect("compatibility parse");
            assert!(decoded.rfc3_coordinates().is_err());
        }

        #[test]
        fn migrated_l1_pair_is_bound_and_serialized_half_pair_fails_closed() {
            let parsed = thermite_syntax::parse(
                "fn f(x: u32) -> u32 ! pure requires x < 100 ensures result == x { x }",
            );
            assert!(parsed.is_clean());
            let artifact = thermite_lower::lower_l1_artifact(&parsed.program, "f")
                .expect("checked runtime artifact");
            let cert = Certificate::new("f", Level::L1, vec!["pure".into()], 0, vec![])
                .with_l1_artifact(&artifact)
                .expect("atomic L1 coordinates");
            let (position, classification) = cert
                .rfc3_coordinates()
                .expect("valid pair")
                .expect("position exists");
            assert_eq!(position.scope, CertificationScope::PerExecution);
            assert_eq!(position.refutation, RefutationChannel::Abort);
            assert_eq!(position.residual_trust, ResidualTrust::Fiat);
            assert_eq!(
                classification.expect("classification").fragment,
                "thermite-l1-runtime-v1"
            );
            assert_eq!(
                position.discharged_trust,
                vec![artifact.wrapper_identity().to_string()]
            );

            let mut hostile = serde_json::to_value(&cert).expect("serialize");
            hostile
                .as_object_mut()
                .expect("certificate object")
                .remove("classification");
            let decoded: Certificate = serde_json::from_value(hostile).expect("compat parse");
            assert!(decoded.rfc3_coordinates().is_err());
        }

        #[test]
        fn migrated_l1_pair_rejects_serialized_legacy_level_substitution() {
            let parsed = thermite_syntax::parse(
                "fn f(x: u32) -> u32 ! pure requires x < 100 ensures result == x { x }",
            );
            let artifact = thermite_lower::lower_l1_artifact(&parsed.program, "f").unwrap();
            let cert = Certificate::new("f", Level::L1, vec!["pure".into()], 0, vec![])
                .with_l1_artifact(&artifact)
                .unwrap();
            let mut hostile = serde_json::to_value(cert).unwrap();
            hostile["level"] = serde_json::json!("L3");
            let decoded: Certificate = serde_json::from_value(hostile).unwrap();
            assert!(decoded.rfc3_coordinates().is_err());
            assert!(decoded.requires_l1_artifact_validation());
        }

        #[test]
        fn verus_artifact_preserves_classification_across_success_and_failure() {
            let parsed = thermite_syntax::parse(
                "fn f(x: u32) -> u32 ! pure requires x < 100 ensures result == x { x }",
            );
            let artifact = thermite_lower::lower_l3_artifact(&parsed.program, "f").unwrap();
            let success = Certificate::new("f", Level::L3, vec!["pure".into()], 0, vec![])
                .with_verus_artifact(&artifact, true)
                .unwrap();
            let failure = Certificate::new("f", Level::L0, vec!["pure".into()], 0, vec![])
                .with_verus_artifact(&artifact, false)
                .unwrap();
            for cert in [&success, &failure] {
                assert_eq!(
                    cert.classification.as_ref().unwrap().fragment,
                    "thermite-verus-v1"
                );
                assert_eq!(
                    cert.certification.as_ref().unwrap().discharged_trust,
                    [artifact.query_identity()]
                );
                cert.validate_verus_artifact_authority().unwrap();
            }
            assert_eq!(
                success.certification.as_ref().unwrap().refutation,
                RefutationChannel::Incomplete
            );
            assert_eq!(
                failure.certification.as_ref().unwrap().scope,
                CertificationScope::None
            );
        }

        #[test]
        fn verus_live_authority_rejects_provenance_and_outcome_substitution() {
            let parsed = thermite_syntax::parse(
                "fn f(x: u32) -> u32 ! pure requires x < 100 ensures result == x { x }",
            );
            let artifact = thermite_lower::lower_l3_artifact(&parsed.program, "f").unwrap();
            let cert = Certificate::new("f", Level::L3, vec!["pure".into()], 0, vec![])
                .with_verus_artifact(&artifact, true)
                .unwrap();

            let mut level = cert.clone();
            level.level = Level::L4;
            assert!(level.validate_verus_artifact_authority().is_err());

            let mut effects = cert.clone();
            effects.effects = vec!["time".into()];
            assert!(effects.validate_verus_artifact_authority().is_err());

            let mut query = cert.clone();
            query.certification.as_mut().unwrap().discharged_trust =
                vec!["thermite-verus-query-v1:f:sha256:invented".into()];
            assert!(query.validate_verus_artifact_authority().is_err());

            let mut classifier = cert;
            classifier.classification.as_mut().unwrap().fragment = "thermite-kani-v1".into();
            assert!(classifier.validate_verus_artifact_authority().is_err());
        }

        #[test]
        fn serialized_verus_half_pair_fails_the_public_reader() {
            let parsed = thermite_syntax::parse(
                "fn f(x: u32) -> u32 ! pure requires x < 100 ensures result == x { x }",
            );
            let artifact = thermite_lower::lower_l3_artifact(&parsed.program, "f").unwrap();
            let cert = Certificate::new("f", Level::L3, vec!["pure".into()], 0, vec![])
                .with_verus_artifact(&artifact, true)
                .unwrap();
            let mut hostile = serde_json::to_value(cert).unwrap();
            hostile.as_object_mut().unwrap().remove("classification");
            let decoded: Certificate = serde_json::from_value(hostile).unwrap();
            assert!(decoded.rfc3_coordinates().is_err());
        }

        #[test]
        fn main_cache_replay_requires_the_fresh_artifact_without_restoring_authority() {
            let parsed = thermite_syntax::parse(
                "fn f(x: u32) -> u32 ! pure requires x < 100 ensures result == x { x }",
            );
            let artifact = thermite_lower::lower_l3_artifact(&parsed.program, "f").unwrap();
            let cert = Certificate::new("f", Level::L3, vec!["pure".into()], 0, vec![])
                .with_verus_artifact(&artifact, true)
                .unwrap();
            let decoded: Certificate =
                serde_json::from_value(serde_json::to_value(cert).unwrap()).unwrap();
            assert!(decoded.persisted_verus_artifact_matches(&artifact));
            assert!(!decoded.is_audit_admitted());

            let bare: Certificate = serde_json::from_value(
                serde_json::to_value(Certificate::new(
                    "f",
                    Level::L3,
                    vec!["pure".into()],
                    0,
                    vec![],
                ))
                .unwrap(),
            )
            .unwrap();
            assert!(!bare.persisted_verus_artifact_matches(&artifact));

            let changed = thermite_syntax::parse(
                "fn f(x: u32) -> u32 ! pure requires x < 99 ensures result == x { x }",
            );
            let changed = thermite_lower::lower_l3_artifact(&changed.program, "f").unwrap();
            assert!(!decoded.persisted_verus_artifact_matches(&changed));
        }

        #[test]
        fn historical_unmarked_l1_position_remains_readable() {
            let historical = Certificate::new("f", Level::L1, vec![], 0, vec![]);
            let (_, classification) = historical
                .rfc3_coordinates()
                .expect("legacy L1 remains readable")
                .expect("legacy position exists");
            assert!(classification.is_none());
        }

        #[test]
        fn deserialized_certificate_is_readable_but_not_audit_authority() {
            let source = Certificate::new("f", Level::L1, vec![], 0, vec![]);
            assert!(source.is_audit_admitted());
            let json = serde_json::to_string(&source).unwrap();
            let decoded: Certificate = serde_json::from_str(&json).unwrap();
            assert!(decoded.rfc3_coordinates().is_ok());
            assert!(!decoded.is_audit_admitted());
            assert_eq!(source, decoded, "admission is outside document equality");
        }

        #[test]
        fn l1_artifact_cannot_be_attached_to_another_item() {
            let parsed = thermite_syntax::parse(
                "fn f(x: u32) -> u32 ! pure requires x < 100 ensures result == x { x }",
            );
            let artifact = thermite_lower::lower_l1_artifact(&parsed.program, "f").unwrap();
            assert!(Certificate::new("g", Level::L1, vec![], 0, vec![])
                .with_l1_artifact(&artifact)
                .is_err());
        }

        #[test]
        fn ffi_artifact_rejects_boundary_target_substitution() {
            let parsed = thermite_syntax::parse(
                "#[boundary(\"ext::read\")] \
                 fn f(x: u32) -> u32 ! pure requires x < 100 ensures result == x ;",
            );
            let artifact = thermite_lower::lower_l1_artifact(&parsed.program, "f").unwrap();
            assert!(
                Certificate::boundary_l1("f", vec!["pure".into()], "ext::write".into())
                    .with_l1_artifact(&artifact)
                    .is_err()
            );
        }

        #[test]
        fn diverge_artifact_rejects_runtime_effect_substitution() {
            let parsed = thermite_syntax::parse(
                "fn spin(n: u64) -> u64 ! diverge requires n <= 100 ensures result == 0 \
                 { if n == 0 { 0 } else { spin(n - 1) } }",
            );
            let artifact = thermite_lower::lower_l1_artifact(&parsed.program, "spin").unwrap();
            assert!(
                Certificate::new("spin", Level::L1, vec!["pure".into()], 0, vec![])
                    .with_l1_artifact(&artifact)
                    .is_err()
            );
        }
    }

    /// Test-local serializer (mirrors `cli::run_check`'s
    /// `serde_json::to_string_pretty`).
    fn serialize(cert: &Certificate) -> String {
        serde_json::to_string_pretty(cert).expect("serialize cert")
    }

    /// The deterministic-subset equality the cert-oracle uses, expressed via the
    /// production `oracle_subset` accessor (so the test exercises the real schema
    /// property, not a re-implementation).
    fn oracle_eq(a: &Certificate, b: &Certificate) -> bool {
        a.oracle_subset() == b.oracle_subset()
    }

    // AC-1: schema matches Appendix A — every documented key present, Level::L3
    // serializes to "L3". Expected keys/values trace to `thermite-design.md`
    // Appendix A (R-CHAR-3), not to forge's own output.
    #[test]
    fn schema_matches_appendix_a() {
        let cert = Certificate::new(
            "sum",
            Level::L3,
            vec!["pure".to_string()],
            612,
            vec![ObligationResult::discharged("sum_check::sum")],
        );
        let json = serialize(&cert);
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse");
        // Appendix A keys.
        for key in [
            "item",
            "level",
            "solver_time_ms",
            "contract_quality",
            "effects",
            "slag",
        ] {
            assert!(value.get(key).is_some(), "missing Appendix A key `{key}`");
        }
        // contract_quality sub-keys (Appendix A).
        let cq = value.get("contract_quality").expect("contract_quality");
        for key in ["tautology", "vacuous_precondition", "mutants_killed"] {
            assert!(cq.get(key).is_some(), "missing contract_quality.{key}");
        }
        // Level::L3 serializes to the string "L3".
        assert_eq!(value.get("level").and_then(|v| v.as_str()), Some("L3"));
    }

    // AC-2: the golden cert's deterministic subset deserializes into a
    // Certificate and re-serializes equal on those fields. Anchors to the golden
    // `conformance/sum.cert.json` (R-CHAR-3), not forge's output.
    #[test]
    fn golden_deterministic_subset_round_trips() {
        let golden_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("conformance")
            .join("sum.cert.json");
        let golden_src = std::fs::read_to_string(&golden_path).expect("read golden cert");
        let golden: Certificate = serde_json::from_str(&golden_src).expect("deserialize golden");
        assert_eq!(golden.item, "sum");
        assert_eq!(golden.level, Level::L3);
        assert_eq!(golden.effects, vec!["pure".to_string()]);
        assert!(!golden.slag);
        // A freshly assembled #5 cert with the same deterministic fields is
        // oracle-equal to the golden, despite differing battery / time fields.
        let ours = Certificate::new(
            "sum",
            Level::L3,
            vec!["pure".to_string()],
            42,
            vec![ObligationResult::discharged("sum_check::sum")],
        );
        assert!(
            oracle_eq(&golden, &ours),
            "the golden subset must oracle-match a #5 cert"
        );
    }

    // REQ-1 / AC-4 (schema v2 additive — `.design/stage1-forge-tier.md`): the per-clause
    // `engine`/`trust`/`verdict` fields are additive (`#[serde(default,
    // skip_serializing_if)]`), so the v1 oracle subset is unperturbed for all seven
    // conformance goldens. The goldens are HAND-AUTHORED stable-subset oracles (R-CHAR-3
    // — item/level/contract_quality/effects/slag, not full forge-emitted certs), so the
    // byte-identity claim is verified two ways: (1) NONE of the seven golden FILES carries
    // any schema-v2 key (`engine`/`trust`/`verdict`) anywhere — they are pristine v1
    // oracles, untouched by this increment; (2) a v1-shape cert (a Verus-corpus discharged
    // clause, no per-clause block) SERIALIZES with no schema-v2 keys (the `skip_serializing_if`
    // omits the absent fields), so forge's v1 output still matches those goldens byte-for-byte.
    // A populated block (the forge-tier Lean path) does serialize the keys — the field works.
    #[test]
    fn schema_v2_additive_leaves_all_seven_goldens_byte_identical() {
        // Recursively assert no JSON key named `engine`/`trust`/`verdict` appears.
        fn has_no_schema_v2_key(v: &serde_json::Value) -> bool {
            match v {
                serde_json::Value::Object(map) => {
                    map.keys()
                        .all(|k| !matches!(k.as_str(), "engine" | "trust" | "verdict"))
                        && map.values().all(has_no_schema_v2_key)
                }
                serde_json::Value::Array(items) => items.iter().all(has_no_schema_v2_key),
                _ => true,
            }
        }
        let conformance = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("conformance");
        let goldens = [
            "bank_account",
            "bytes_eq_demo",
            "map_kv",
            "option_result",
            "parse_u64",
            "shape",
            "sum",
        ];
        for name in goldens {
            let src = std::fs::read_to_string(conformance.join(format!("{name}.cert.json")))
                .unwrap_or_else(|e| panic!("read golden {name}: {e}"));
            let v: serde_json::Value =
                serde_json::from_str(&src).unwrap_or_else(|e| panic!("parse golden {name}: {e}"));
            assert!(
                has_no_schema_v2_key(&v),
                "golden `{name}` must carry NO schema-v2 per-clause key — it is a pristine \
                 v1 oracle, unperturbed by the additive fields (AC-4)"
            );
        }

        // (2) A v1-shape clause (no per-clause block) serializes with no schema-v2 keys.
        let v1 = Certificate::new(
            "sum",
            Level::L3,
            vec!["pure".to_string()],
            42,
            vec![ObligationResult::discharged("sum_check::sum")],
        );
        let v1_json = serialize(&v1);
        assert!(
            !v1_json.contains("\"engine\"")
                && !v1_json.contains("\"trust\"")
                && !v1_json.contains("\"verdict\""),
            "a v1-shape cert must serialize WITHOUT the additive per-clause keys \
             (skip_serializing_if), so it stays byte-identical to the v1 goldens: {v1_json}"
        );

        // (3) A populated forge-tier clause does serialize the keys (the field is live).
        let forge_tier = Certificate::new(
            "isqrt",
            Level::L3,
            vec!["pure".to_string()],
            0,
            vec![
                ObligationResult::discharged("isqrt::ens#0").with_clause_attribution(
                    "lean-auto",
                    vec!["Lean kernel".to_string()],
                    crate::verdict::CertVerdict::Proved,
                ),
            ],
        );
        let ft_json = serialize(&forge_tier);
        assert!(
            ft_json.contains("\"engine\"")
                && ft_json.contains("\"verdict\"")
                && ft_json.contains("Proved"),
            "a populated forge-tier clause must serialize the per-clause block: {ft_json}"
        );
    }

    // Stage-3 REQ-3 / AC-4 (Lock 1, the shadow flag): the `bv_shadow` field is additive —
    // a v1 / untagged clause omits it (byte-identical goldens), a tagged clause serializes
    // the RFC §9 block; `nowrap_obligation` is omitted for a bare tag and filled (REQ-5)
    // for the `nowrap` spelling.
    #[test]
    fn bv_shadow_is_additive_and_serializes_the_rfc9_shape() {
        // A v1-shape clause (no shadow) must not serialize a `bv_shadow` key — so the v1
        // goldens stay byte-identical (the `engine`/`trust`/`verdict` discipline).
        let v1 = Certificate::new(
            "sum",
            Level::L3,
            vec!["pure".to_string()],
            0,
            vec![ObligationResult::discharged("sum_check::sum")],
        );
        assert!(
            !serialize(&v1).contains("bv_shadow"),
            "a clause with no shadow flag must omit `bv_shadow` (skip_serializing_if)"
        );

        // A tagged clause serializes the RFC §9 block; `nowrap_obligation` is the reserved
        // slot (None → omitted, never a placeholder — the `suggested_move` precedent).
        let tagged = Certificate::new(
            "mix64",
            Level::L4,
            vec!["pure".to_string()],
            0,
            vec![
                ObligationResult::discharged("mix64::ens#0").with_bv_shadow(BvShadow {
                    flagged: true,
                    semantics: "bv64 (wraparound)".to_string(),
                    nowrap_obligation: None,
                    note: "machine-semantics fork".to_string(),
                }),
            ],
        );
        let json = serialize(&tagged);
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse");
        let shadow = &v["obligations"][0]["bv_shadow"];
        assert_eq!(shadow["flagged"], serde_json::Value::Bool(true));
        assert_eq!(
            shadow["semantics"],
            serde_json::Value::from("bv64 (wraparound)")
        );
        assert!(shadow.get("note").is_some(), "the §9 note is present");
        assert!(
            shadow.get("nowrap_obligation").is_none(),
            "a bare/wraparound clause runs no side obligation, so `nowrap_obligation` is \
             omitted (None → skipped, never a placeholder): {json}"
        );
        // Round-trips: deserialize → re-serialize is byte-stable.
        let back: Certificate = serde_json::from_str(&json).expect("round-trip");
        assert_eq!(serialize(&back), json, "the shadow block round-trips");

        // Lock 3 (REQ-5 / AC-6): a `@bvN(nowrap)` clause whose obligation ran serializes
        // the verdict — the filled slot is present and greppable, round-tripping stably.
        let nowrap = Certificate::new(
            "add_nowrap",
            Level::L4,
            vec!["pure".to_string()],
            0,
            vec![
                ObligationResult::discharged("add_nowrap::ens#0").with_bv_shadow(BvShadow {
                    flagged: true,
                    semantics: "bv64 (nowrap)".to_string(),
                    nowrap_obligation: Some(
                        "discharged: no operation in the clause overflows at bv64".to_string(),
                    ),
                    note: "machine-semantics fork".to_string(),
                }),
            ],
        );
        let njson = serialize(&nowrap);
        let nv: serde_json::Value = serde_json::from_str(&njson).expect("parse");
        assert_eq!(
            nv["obligations"][0]["bv_shadow"]["nowrap_obligation"],
            serde_json::Value::from("discharged: no operation in the clause overflows at bv64"),
            "the filled nowrap obligation verdict serializes: {njson}"
        );
        let nback: Certificate = serde_json::from_str(&njson).expect("round-trip");
        assert_eq!(serialize(&nback), njson, "the filled slot round-trips");
    }

    // Stage-3 REQ-3 / AC-4: the shadow flag is ORACLE-INCLUDED (Q-ORACLE: deterministic +
    // verdict-relevant → included) — two certs differing only in an obligation's
    // `bv_shadow` compare oracle-UNEQUAL, unlike the provenance-only engine/trust (which
    // are oracle-excluded). A semantic fork cannot drift silently.
    #[test]
    fn bv_shadow_is_oracle_included() {
        let bare = Certificate::new(
            "mix64",
            Level::L4,
            vec!["pure".to_string()],
            0,
            vec![ObligationResult::discharged("mix64::ens#0")],
        );
        let mut shadowed = bare.clone();
        shadowed.obligations[0] =
            ObligationResult::discharged("mix64::ens#0").with_bv_shadow(BvShadow {
                flagged: true,
                semantics: "bv64 (wraparound)".to_string(),
                nowrap_obligation: None,
                note: "machine-semantics fork".to_string(),
            });
        assert!(
            !oracle_eq(&bare, &shadowed),
            "the oracle must DISTINGUISH a tagged clause from an untagged one (the fork is \
             verdict-relevant — REQ-3 / AC-4)"
        );
        // But engine/trust attribution differences alone stay oracle-excluded (provenance).
        let mut attributed = bare.clone();
        attributed.obligations[0] = ObligationResult::discharged("mix64::ens#0")
            .with_clause_attribution(
                "bitvector",
                vec!["solver Z3 QF_BV".to_string()],
                crate::verdict::CertVerdict::Proved,
            );
        assert!(
            oracle_eq(&bare, &attributed),
            "engine/trust attribution is provenance — oracle-excluded (the existing \
             discipline; only the shadow flag joins the oracle)"
        );
    }

    // AC-3: forward-declared fields excluded from the live oracle — two certs
    // differing only in contract_quality / solver_time_ms compare equal.
    #[test]
    fn oracle_ignores_forward_declared_and_time() {
        let mut a = Certificate::new("f", Level::L3, vec!["pure".to_string()], 1, vec![]);
        let mut b = a.clone();
        b.solver_time_ms = 99_999;
        b.contract_quality.mutants_killed = "17/18".to_string();
        b.contract_quality.tautology = true;
        assert!(
            oracle_eq(&a, &b),
            "oracle must ignore time + battery fields"
        );
        // But a differing deterministic field is caught.
        a.level = Level::L1;
        assert!(!oracle_eq(&a, &b), "oracle must catch a level mismatch");
    }

    // AC-4: suggested_move is a reserved absence — serializes as omitted (its
    // Option is None), never a placeholder.
    #[test]
    fn suggested_move_is_reserved_absence() {
        let cert = Certificate::new("f", Level::L3, vec!["pure".to_string()], 0, vec![]);
        assert!(cert.suggested_move.is_none());
        let json = serialize(&cert);
        assert!(
            !json.contains("suggested_move"),
            "None suggested_move must be omitted, not a placeholder:\n{json}"
        );
    }

    // AC-5: per-obligation list present for pass and fail; a failure carries a
    // source-located diagnostic.
    #[test]
    fn obligation_results_present() {
        let pass = ObligationResult::discharged("sum_check::sum");
        assert_eq!(pass.status, ObligationStatus::Discharged);
        let fail = ObligationResult::failed(
            "postcondition not satisfied",
            Some("broken_check.rs:5:13".to_string()),
            Some("error: postcondition not satisfied".to_string()),
        );
        assert_eq!(fail.status, ObligationStatus::Failed);
        assert!(fail.location.is_some(), "failure carries a source location");
        assert!(fail.diagnostic.is_some(), "failure carries a diagnostic");
    }

    // AC-6: determinism — serializing the same Certificate twice is
    // byte-identical (R-CODE-5).
    #[test]
    fn serialization_is_deterministic() {
        let cert = Certificate::new(
            "sum",
            Level::L3,
            vec!["pure".to_string()],
            612,
            vec![ObligationResult::discharged("sum_check::sum")],
        );
        let a = serialize(&cert);
        let b = serialize(&cert);
        assert_eq!(a, b);
    }

    // #6 AC: the additive `slag_meta`/`reject` fields are absent on a plain #5
    // cert and on the golden — so the golden `sum.cert.json` still deserializes
    // (R-SPEC-2). A None `slag_meta`/`reject` must not serialize.
    #[test]
    fn slag_and_reject_fields_are_additive_and_skipped_when_none() {
        let cert = Certificate::new("f", Level::L3, vec!["pure".to_string()], 0, vec![]);
        assert!(cert.slag_meta.is_none());
        assert!(cert.reject.is_none());
        let json = serialize(&cert);
        assert!(
            !json.contains("slag_meta"),
            "None slag_meta omitted:\n{json}"
        );
        assert!(!json.contains("reject"), "None reject omitted:\n{json}");
        // The frozen golden cert (no slag_meta/reject) deserializes unchanged.
        let golden_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("conformance")
            .join("sum.cert.json");
        let golden_src = std::fs::read_to_string(&golden_path);
        assert!(golden_src.is_ok(), "read golden: {golden_src:?}");
        if let Ok(src) = golden_src {
            let golden: Result<Certificate, _> = serde_json::from_str(&src);
            assert!(golden.is_ok(), "golden deserializes: {golden:?}");
            if let Ok(g) = golden {
                assert!(g.slag_meta.is_none());
                assert!(g.reject.is_none());
            }
        }
    }

    // #8 (proof-cache REQ-7 / AC-5): `cached` is an additive field that defaults
    // `false` (the golden `sum.cert.json` omits it) and is excluded from the
    // oracle subset -- a hit (`cached: true`) is oracle-equal to the fresh verify
    // it was stored from. Expected behavior traces to `proof-cache.md` REQ-7/REQ-2
    // (R-CHAR-3), not forge's output.
    #[test]
    fn cached_field_is_additive_and_oracle_excluded() {
        let fresh = Certificate::new("f", Level::L3, vec!["pure".to_string()], 1, vec![]);
        assert!(!fresh.cached, "a fresh cert is not cached by default");

        // `with_cached(true)` flips only the provenance bit; the oracle subset is
        // unchanged, so a hit is oracle-equal to the fresh verify (REQ-2).
        let hit = fresh.clone().with_cached(true);
        assert!(hit.cached);
        assert!(
            oracle_eq(&fresh, &hit),
            "a cache hit must be oracle-equal to the fresh verify it was stored from"
        );

        // The golden `conformance/sum.cert.json` (which omits `cached`) still
        // deserializes, defaulting `cached` to `false` (additive, R-SPEC-2).
        let golden_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("conformance")
            .join("sum.cert.json");
        let golden_src = std::fs::read_to_string(&golden_path);
        assert!(golden_src.is_ok(), "read golden cert: {golden_src:?}");
        if let Ok(src) = golden_src {
            let golden: Result<Certificate, _> = serde_json::from_str(&src);
            assert!(golden.is_ok(), "golden deserializes: {golden:?}");
            if let Ok(g) = golden {
                assert!(
                    !g.cached,
                    "golden omits `cached`, defaults false (additive)"
                );
            }
        }
    }

    // #6 AC-1/AC-4 (slag.md): a valid slag cert is L1, slag:true, carries the
    // metadata, and is not a verus obligation. Expected level/flag trace to
    // `slag.md` REQ-2/REQ-4 (R-CHAR-3), not forge's output.
    #[test]
    fn slag_l1_cert_shape() {
        let meta = SlagMeta {
            reason: "vendored".to_string(),
            owner: "agent:forge-7".to_string(),
            review: "required".to_string(),
        };
        let cert = Certificate::slag_l1("simd_sum", vec!["pure".to_string()], meta.clone());
        assert_eq!(cert.level, Level::L1);
        assert!(cert.slag);
        assert_eq!(cert.slag_meta, Some(meta));
        // The triage bools graduated to live-false even on the slag path.
        assert!(!cert.contract_quality.tautology);
        assert!(!cert.contract_quality.vacuous_precondition);
        let json = serialize(&cert);
        assert!(json.contains("slag_meta"), "slag cert carries metadata");
    }

    // #6 (vacuity-triage REQ-5): a triage reject is a non-certified (L0) cert
    // carrying the structured cause, not a ForgeError.
    #[test]
    fn rejected_cert_carries_cause_and_is_not_l3() {
        let reason = RejectReason {
            cause: "EnsIsTrivial".to_string(),
            detail: "ens#0 is the literal `true`".to_string(),
        };
        let cert = Certificate::rejected("f", vec!["pure".to_string()], false, reason);
        assert_eq!(cert.level, Level::L0);
        assert_ne!(cert.level, Level::L3);
        assert_eq!(
            cert.reject.as_ref().map(|r| r.cause.as_str()),
            Some("EnsIsTrivial")
        );
        assert_eq!(cert.obligations.len(), 1);
        assert_eq!(cert.obligations[0].status, ObligationStatus::Failed);
    }

    // #6 (vacuity-triage AC-7): a triage-passing item graduates the two bools to
    // asserted live-false.
    #[test]
    fn graduate_triage_clean_sets_live_false() {
        let cert = Certificate::new("f", Level::L3, vec!["pure".to_string()], 0, vec![])
            .graduate_triage_clean();
        assert!(!cert.contract_quality.tautology);
        assert!(!cert.contract_quality.vacuous_precondition);
    }

    // #12 (mutation-scoring REQ-6): `with_mutation_score` graduates the two
    // forward-declared fields to live on a certified item; the oracle subset is
    // unchanged (the fields stay oracle-excluded). Expected behavior traces to
    // `mutation-scoring.md` REQ-6 (R-CHAR-3), not forge's output.
    #[test]
    fn with_mutation_score_graduates_fields_and_stays_oracle_excluded() {
        let base = Certificate::new("sum", Level::L3, vec!["pure".to_string()], 0, vec![]);
        // Forward-declared default before scoring.
        assert_eq!(base.contract_quality.mutants_killed, "0/0");
        assert_eq!(base.contract_quality.equivalent_mutants_excluded, 0);
        assert!(base.contract_quality.survivor.is_none());

        let scored = base.clone().with_mutation_score("17/18".to_string(), None);
        assert_eq!(scored.contract_quality.mutants_killed, "17/18");
        assert!(scored.contract_quality.survivor.is_none());
        let narrowed =
            base.clone()
                .with_mutation_score_and_equivalents("17/17".to_string(), None, 2);
        assert_eq!(narrowed.contract_quality.equivalent_mutants_excluded, 2);
        assert_eq!(
            serde_json::to_value(&narrowed).unwrap()["contract_quality"]
                ["equivalent_mutants_excluded"],
            2
        );
        // The kill ratio is oracle-excluded: a graduated cert is oracle-equal to the
        // forward-declared one (OQ-1 — the ratio is verus-version-sensitive).
        assert!(oracle_eq(&base, &scored));
        assert_eq!(base.level, scored.level);
    }

    // #12 (mutation-scoring REQ-5/REQ-6): a `WeakContract` reject is a non-certified
    // (L0) cert carrying the `"WeakContract"` cause, the real kill ratio, and a
    // surviving-mutant `survivor` (the §7 strengthening prompt). Expected cause/level
    // trace to `mutation-scoring.md` REQ-5 (R-CHAR-3), not forge's output.
    #[test]
    fn rejected_weak_contract_carries_cause_ratio_and_survivor() {
        let cert = Certificate::rejected_weak_contract(
            "f",
            vec!["pure".to_string()],
            "1/3".to_string(),
            "insert early `return 0` at body head".to_string(),
        );
        assert_eq!(cert.level, Level::L0);
        assert_ne!(cert.level, Level::L3);
        assert_eq!(
            cert.reject.as_ref().map(|r| r.cause.as_str()),
            Some("WeakContract")
        );
        assert_eq!(cert.contract_quality.mutants_killed, "1/3");
        assert_eq!(
            cert.contract_quality.survivor.as_deref(),
            Some("insert early `return 0` at body head")
        );
        // The detail names the surviving mutant (the precise prompt §7 describes).
        let detail = cert
            .reject
            .as_ref()
            .map(|r| r.detail.clone())
            .unwrap_or_default();
        assert!(
            detail.contains("insert early `return 0` at body head"),
            "detail names the survivor: {detail}"
        );
    }

    // #10 (degrade-ladder REQ-6): Level's derived Ord is the ladder ordering
    // L0 < L1 < L2 < L3. Expected from the design doc's REQ-6 (R-CHAR-3).
    #[test]
    fn level_ord_is_the_ladder_ordering() {
        assert!(Level::L0 < Level::L1);
        assert!(Level::L1 < Level::L2);
        assert!(Level::L2 < Level::L3);
        // min over a mixed set is the weakest rung.
        let levels = [Level::L3, Level::L1, Level::L2];
        assert_eq!(levels.iter().min().copied(), Some(Level::L1));
    }

    // #10 (degrade-ladder REQ-4): into_degraded stamps the lowered_assurance flag +
    // the degrade reason, leaving the level + obligations untouched.
    #[test]
    fn into_degraded_stamps_flag_and_reason() {
        let base = Certificate::new("g", Level::L2, vec!["pure".to_string()], 0, vec![]);
        assert!(!base.lowered_assurance);
        let reason = RejectReason {
            cause: "VerusTimeout".to_string(),
            detail: "rlimit exhausted".to_string(),
        };
        let degraded = base.clone().into_degraded(reason);
        assert!(degraded.lowered_assurance);
        assert_eq!(
            degraded.degrade_reason.as_ref().map(|r| r.cause.as_str()),
            Some("VerusTimeout")
        );
        // The achieved level is untouched — into_degraded qualifies it.
        assert_eq!(degraded.level, Level::L2);
    }

    // #10 (degrade-ladder AC-6, R-SPEC-2): the lowered_assurance / degrade_reason
    // fields are additive — absent on a plain cert and on the golden, so the frozen
    // golden `sum.cert.json` still deserializes. A non-degraded cert serializes
    // lowered_assurance:false and omits degrade_reason.
    #[test]
    fn degrade_fields_are_additive() {
        let cert = Certificate::new("f", Level::L3, vec!["pure".to_string()], 0, vec![]);
        assert!(!cert.lowered_assurance);
        assert!(cert.degrade_reason.is_none());
        let json = serialize(&cert);
        assert!(
            !json.contains("degrade_reason"),
            "None degrade_reason is omitted:\n{json}"
        );
        // The frozen golden cert (no #10 fields) deserializes, defaulting the flag.
        let golden_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("conformance")
            .join("sum.cert.json");
        if let Ok(src) = std::fs::read_to_string(&golden_path) {
            let golden: Result<Certificate, _> = serde_json::from_str(&src);
            assert!(
                golden.is_ok(),
                "golden deserializes with #10 additive fields: {golden:?}"
            );
            if let Ok(g) = golden {
                assert!(
                    !g.lowered_assurance,
                    "golden defaults lowered_assurance false"
                );
                assert!(g.degrade_reason.is_none());
            }
        }
    }

    // #10 (degrade-ladder REQ-5/REQ-6 / AC-5): the assurance aggregate headline is
    // the min over functions. {L3,L2,L1} → Certified(L1). Expected: Level's Ord
    // (REQ-6), not forge's output (R-CHAR-3).
    #[test]
    fn aggregate_headline_is_min_over_functions() {
        let certs = vec![
            Certificate::new("f", Level::L3, vec!["pure".to_string()], 0, vec![]),
            Certificate::new("g", Level::L2, vec!["pure".to_string()], 0, vec![]),
            Certificate::new("h", Level::L1, vec!["pure".to_string()], 0, vec![]),
        ];
        let m = AssuranceManifest::aggregate(&certs);
        assert_eq!(m.project, ProjectAssurance::Certified(Level::L1));
        assert_eq!(m.functions.len(), 3);
    }

    // #10 (REQ-2/REQ-6 / AC-5): a single non-certifying (counterexample / reject)
    // fn caps the whole project at failure — not a lowered rung (falsity is not a
    // rung). Expected from REQ-6 (R-CHAR-3).
    #[test]
    fn aggregate_hard_fail_is_project_failure() {
        let reason = RejectReason {
            cause: "EnsIsTrivial".to_string(),
            detail: "ens#0 is true".to_string(),
        };
        let certs = vec![
            Certificate::new("f", Level::L3, vec!["pure".to_string()], 0, vec![]),
            Certificate::rejected("bad", vec!["pure".to_string()], false, reason),
        ];
        let m = AssuranceManifest::aggregate(&certs);
        assert_eq!(m.project, ProjectAssurance::Failed);
        // The rejected fn is recorded as non-certified in its row.
        let bad = m.functions.iter().find(|r| r.item == "bad");
        assert_eq!(bad.map(|r| r.certified), Some(false));
    }

    // #10 (REQ-6): an empty cert collection certifies vacuously at the top rung —
    // a file with no fn has nothing un-proved.
    #[test]
    fn aggregate_empty_is_vacuous_l3() {
        let m = AssuranceManifest::aggregate(&[]);
        assert_eq!(m.project, ProjectAssurance::Certified(Level::L3));
        assert!(m.functions.is_empty());
    }

    // #10: cert_certifies treats L3/L2/L1 (no reject) as certified and L0 / a
    // reject as not — the shared predicate the aggregate + cli exit code use.
    #[test]
    fn cert_certifies_recognizes_the_certified_rungs() {
        assert!(cert_certifies(&Certificate::new(
            "a",
            Level::L3,
            vec![],
            0,
            vec![]
        )));
        assert!(cert_certifies(&Certificate::new(
            "b",
            Level::L2,
            vec![],
            0,
            vec![]
        )));
        assert!(cert_certifies(&Certificate::new(
            "c",
            Level::L1,
            vec![],
            0,
            vec![]
        )));
        assert!(!cert_certifies(&Certificate::new(
            "d",
            Level::L0,
            vec![],
            0,
            vec![]
        )));
        let reason = RejectReason {
            cause: "WeakContract".to_string(),
            detail: "x".to_string(),
        };
        // An L3 cert with a reject (e.g. a WeakContract reject built on L0) does not
        // certify — the reject dominates.
        assert!(!cert_certifies(&Certificate::rejected(
            "e",
            vec![],
            false,
            reason
        )));
    }

    // #17 (e2e-vs-boundary REQ-3, R-SPEC-2): `assurance_scope` is additive — absent
    // on a plain cert and on the golden, defaulting `None`, so the frozen golden
    // `conformance/sum.cert.json` still deserializes. The oracle normalization makes
    // `None` (golden) oracle-equal to `Some(EndToEnd)` (a classified pure fn), so the
    // golden subset stays stable; a `Some(ToBoundary)` is oracle-distinct (verdict-
    // relevant, §9 / R-DEFER-9). Expected behavior traces to the design REQ-3 + the
    // Verification section (R-CHAR-3), not forge output.
    #[test]
    fn assurance_scope_is_additive_normalized_and_golden_stable() {
        let plain = Certificate::new("f", Level::L3, vec!["pure".to_string()], 0, vec![]);
        assert!(plain.assurance_scope.is_none(), "additive: defaults None");
        let json = serialize(&plain);
        assert!(
            !json.contains("assurance_scope"),
            "None assurance_scope is omitted:\n{json}"
        );

        // None and Some(EndToEnd) are oracle-equal (the normalization keeping the
        // golden stable).
        let e2e = plain.clone().with_assurance_scope(AssuranceScope::EndToEnd);
        assert!(
            oracle_eq(&plain, &e2e),
            "None and Some(EndToEnd) must be oracle-equal (golden stability)"
        );

        // Some(ToBoundary) is oracle-distinct from end-to-end (verdict-relevant).
        let to_boundary = plain
            .clone()
            .with_assurance_scope(AssuranceScope::ToBoundary {
                via: "ext_id".to_string(),
            });
        assert!(
            !oracle_eq(&plain, &to_boundary),
            "a to-the-boundary scope must be oracle-visible (§9 / R-DEFER-9)"
        );
        // The achieved level is untouched — scope ⊥ level (REQ-5).
        assert_eq!(to_boundary.level, Level::L3, "scope is orthogonal to level");

        // The frozen golden `sum.cert.json` (no assurance_scope) deserializes,
        // defaulting None, and is oracle-equal to a classified EndToEnd `sum`.
        let golden_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("conformance")
            .join("sum.cert.json");
        if let Ok(src) = std::fs::read_to_string(&golden_path) {
            let golden: Result<Certificate, _> = serde_json::from_str(&src);
            assert!(
                golden.is_ok(),
                "golden deserializes with the #17 additive field: {golden:?}"
            );
            if let Ok(g) = golden {
                assert!(g.assurance_scope.is_none(), "golden omits assurance_scope");
                let classified = g.clone().with_assurance_scope(AssuranceScope::EndToEnd);
                assert!(
                    oracle_eq(&g, &classified),
                    "the golden subset is stable once `sum` is classified EndToEnd"
                );
            }
        }
    }

    // effects_of covers the whole Effect enum, beyond `pure` (R-DEFER-8: fix
    // the whole class). Expected tokens are this module's documented mapping.
    #[test]
    fn effects_of_covers_every_variant() {
        assert_eq!(effects_of(&EffectRow::Pure), vec!["pure".to_string()]);
        let row = EffectRow::Set(vec![
            Effect::Read("x".to_string().into()),
            Effect::Write("y".to_string().into()),
            Effect::Net("z".to_string().into()),
            Effect::Alloc,
            Effect::Time,
            Effect::Rand,
            Effect::Panic,
            Effect::Diverge,
            Effect::Term,
        ]);
        assert_eq!(
            effects_of(&row),
            vec![
                "read(x)", "write(y)", "net(z)", "alloc", "time", "rand", "panic", "diverge",
                "term"
            ]
        );
    }

    // =======================================================================
    // REQ-10 (Target D) — the Verus-anchor for the project level-aggregation min
    // (`.design/verified/self-verification.md` REQ-10 / AC-10c, mechanism (c)).
    //
    // Placement deviation (Option B, orchestrator-authorized): the design doc names
    // a `manifest::verus_anchor` block (forge is binary-only, so an external test
    // cannot reach `AssuranceManifest::aggregate`/`Certificate`). Nested in the
    // existing `tests` module so the anti-pattern gate's `#[cfg(test)]` exemption
    // covers it. `thermite-verified` is a forge dev-dependency.
    //
    // AC-10c — the exhaustive `Level`-list equivalence: enumerate all per-fn `Level`
    // lists up to length 4 over the 4 levels (plus the empty list) and assert, for
    // each, that `AssuranceManifest::aggregate(certs).project` agrees with the verus-
    // proved fold-min `thermite_verified::aggregate_level`. The production `aggregate`
    // splits two orthogonal axes (REQ-2/REQ-6): a non-certifying fn (a plain `L0`
    // cert carries no rung — `cert_certifies` is false) caps the project at `Failed`,
    // independent of the min; when every fn certifies (the list is empty or over the
    // certifying rungs L1/L2/L3) the headline is `Certified(min)`. The anchor binds
    // the level min (D's §5.2 no-over-claim story) on the all-certifying lists —
    // `Certified(proved_min)` and headline ≤ every level — and confirms the
    // orthogonal `Failed`-cap fires iff an `L0` is present (so the enumeration is
    // exhaustive over the full 4-level alphabet, beyond the certifying subset).
    // Expected = the proved fold-min (R-CHAR-3, never forge's own output) — binding
    // the production min to the proved D1 (≤ every fn) + D2 (attained == the min).
    // =======================================================================
    mod verus_anchor {
        use super::*;
        use thermite_verified::{aggregate_level, Level as VLevel};

        /// The 4 production levels in rank order (`L0 < L1 < L2 < L3`), each paired
        /// with the verus-proved `thermite_verified::Level` mirror. The pairing is
        /// the representation bridge the anchor binds (R-CHAR-3 — the design's
        /// lattice, not forge output).
        const LEVELS: &[(Level, VLevel)] = &[
            (Level::L0, VLevel::L0),
            (Level::L1, VLevel::L1),
            (Level::L2, VLevel::L2),
            (Level::L3, VLevel::L3),
        ];

        /// Map a proved `thermite_verified::Level` back to the production `Level`
        /// via the lattice bridge. Total over the 4-level alphabet.
        fn prod_of(v: VLevel) -> Level {
            match v {
                VLevel::L0 => Level::L0,
                VLevel::L1 => Level::L1,
                VLevel::L2 => Level::L2,
                VLevel::L3 => Level::L3,
            }
        }

        /// Build a per-fn cert list from a production-level list. Each cert is
        /// `Certificate::new` (no reject); a plain `L0` cert does not certify
        /// (`cert_certifies` is false for `L0`), so a list containing `L0` exercises
        /// the orthogonal `Failed`-cap path, while a list over the certifying rungs
        /// (L1/L2/L3) exercises the min-over-functions path the D anchor binds.
        fn level_certs(levels: &[Level]) -> Vec<Certificate> {
            levels
                .iter()
                .enumerate()
                .map(|(i, &lvl)| {
                    Certificate::new(format!("f{i}"), lvl, vec!["pure".to_string()], 0, vec![])
                })
                .collect()
        }

        /// One enumerated `(Level, VLevel)` list element (the production level
        /// paired with its verus mirror).
        type LevelPair = (Level, VLevel);

        /// Recursively enumerate every [`LevelPair`] list up to `max_len`
        /// (inclusive, plus the empty list) and call `visit` on each. The 4-level
        /// alphabet over lengths 0..=4 is `1 + 4 + 16 + 64 + 256 = 341` lists.
        fn for_each_list(
            max_len: usize,
            acc: &mut Vec<LevelPair>,
            visit: &mut dyn FnMut(&[LevelPair]),
        ) {
            visit(acc);
            if acc.len() == max_len {
                return;
            }
            for &pair in LEVELS {
                acc.push(pair);
                for_each_list(max_len, acc, visit);
                acc.pop();
            }
        }

        /// AC-10c — over every `Level` list (length 0..=4) the production
        /// `AssuranceManifest::aggregate` project headline agrees with the verus-
        /// proved `aggregate_level`: on an all-certifying list (empty or over
        /// L1/L2/L3) it is `Certified(proved_min)` and ≤ every per-fn level (the
        /// §5.2 / R-DEFER-9 over-claim bound, proved D1); on a list with an `L0`
        /// (non-certifying) it is the orthogonal `Failed`-cap. 0 mismatches over the
        /// full finite domain.
        #[test]
        fn aggregate_project_min_matches_proved_aggregate_level_over_all_level_lists() {
            let mut checked = 0usize;
            let mut min_anchored = 0usize;
            let mut acc: Vec<(Level, VLevel)> = Vec::new();
            let mut visit = |list: &[(Level, VLevel)]| {
                let prod_levels: Vec<Level> = list.iter().map(|&(p, _)| p).collect();
                let v_levels: Vec<VLevel> = list.iter().map(|&(_, v)| v).collect();

                // R-CHAR-3: the expected min is the verus-proved fold, mapped back to
                // a production `Level` via the lattice bridge.
                let expected_min = prod_of(aggregate_level(&v_levels));

                let certs = level_certs(&prod_levels);
                let m = AssuranceManifest::aggregate(&certs);

                if prod_levels.contains(&Level::L0) {
                    // Orthogonal `Failed`-cap: a non-certifying (L0) fn caps the
                    // project regardless of the min (REQ-2/REQ-6).
                    assert_eq!(
                        m.project,
                        ProjectAssurance::Failed,
                        "an L0 fn must cap the project at Failed for {prod_levels:?}"
                    );
                } else {
                    // The min-over-functions path the D anchor binds.
                    assert_eq!(
                        m.project,
                        ProjectAssurance::Certified(expected_min),
                        "aggregate project min != proved aggregate_level for {prod_levels:?}"
                    );
                    // D1 observable: the headline is ≤ every per-fn level.
                    for &lvl in &prod_levels {
                        assert!(
                            expected_min <= lvl,
                            "project min {expected_min:?} must be <= every fn level (got {lvl:?})"
                        );
                    }
                    min_anchored += 1;
                }
                checked += 1;
            };
            for_each_list(4, &mut acc, &mut visit);
            // 1 + 4 + 16 + 64 + 256 = 341 lists over the 4-level alphabet (0..=4).
            assert_eq!(checked, 341, "all Level lists up to length 4 enumerated");
            // The min-anchored subset (no L0) is 1 + 3 + 9 + 27 + 81 = 121 lists.
            assert_eq!(
                min_anchored, 121,
                "the all-certifying (no-L0) lists bind the proved min"
            );
        }
    }
}
