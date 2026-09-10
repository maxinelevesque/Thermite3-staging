//! Layered, versioned assurance reporting over live `CurrentAssurance`.
//!
//! The report is deliberately a presentation and comparison surface.  It can
//! never be deserialized into authority, and repository floors are evaluated
//! only by [`LiveAssuranceReport`], the capability returned while validating
//! live certificates against the exact parsed source population.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thermite_syntax::{ForgeItem, Item, Program};

use crate::assurance_v2::{
    assurance_kind_leq, AssuranceKindV2, ClaimFiberAddressV2, CommonClaimFrontierV2,
    LiftedItemEvidenceV2, PopulationClaimSetTransportV2, ProjectBuildIdentityV2,
    ProjectDispositionV2, ProjectFrontiersV2, ProjectItemIdentityV2, ProjectLiftV2,
    ProjectPopulationMemberV2, ProjectPopulationV2, ProjectPortraitScopeV2,
};
use crate::audit::{AuditManifest, Tcb, Toolchain};
use crate::manifest::{
    AssuranceScope, Certificate, CertificationBoundary, CertificationPosition, ClaimConstraint,
    ClauseCertification, ContractQuality, CurrentAssurance, CurrentClaim, CurrentDisposition,
    ObligationResult,
};

pub const REPORT_SCHEMA: &str = "thermite-assurance-report/v1";
pub const REPORT_SCHEMA_VERSION: u64 = 1;
pub const COLLAPSE_POLICY_VERSION: u64 = 1;
pub const FLOOR_SCHEMA: &str = "thermite-assurance-floor/v1";
pub const MAX_REPORT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_JSON_DEPTH: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportTrust {
    LocalDiagnostic,
    UntrustedPullRequest,
    ProtectedExactSha,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportSource {
    pub revision: String,
    pub source_path: String,
    pub source_sha256: String,
    pub artifact_sha256: String,
    pub trust: ReportTrust,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", deny_unknown_fields)]
pub enum EngineerCoverage {
    EndToEnd,
    ToBoundary { via: String },
    ToPlatform { platform: String },
}

impl From<&CertificationBoundary> for EngineerCoverage {
    fn from(boundary: &CertificationBoundary) -> Self {
        match boundary {
            CertificationBoundary::EndToEnd => Self::EndToEnd,
            CertificationBoundary::ToBoundary { via } => Self::ToBoundary { via: via.clone() },
            CertificationBoundary::ToPlatform { platform } => Self::ToPlatform {
                platform: platform.clone(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineerClaim {
    CheckedThisExecution,
    CheckedThroughBound,
    ProvedAllInputsMayNotProduceWitness,
    ProvedAllInputsWithConcreteWitness,
    ProvedAllInputsWithEmpiricalFalsification,
}

impl EngineerClaim {
    fn from_kind(kind: AssuranceKindV2) -> Self {
        match kind {
            AssuranceKindV2::Runtime => Self::CheckedThisExecution,
            AssuranceKindV2::Bounded => Self::CheckedThroughBound,
            AssuranceKindV2::SolverIncomplete => Self::ProvedAllInputsMayNotProduceWitness,
            AssuranceKindV2::SolverComplete | AssuranceKindV2::LeanComplete => {
                Self::ProvedAllInputsWithConcreteWitness
            }
            AssuranceKindV2::LeanEmpirical => Self::ProvedAllInputsWithEmpiricalFalsification,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EngineerStatement {
    pub claim: EngineerClaim,
    pub coverage: EngineerCoverage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_bound: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", deny_unknown_fields)]
pub enum ItemDisposition {
    Accepted,
    NonClaim {
        class: String,
        detail: String,
    },
    NotCurrent {
        reason: String,
    },
    Historical {
        legacy_level: String,
        warning: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClausePortrait {
    pub address: String,
    pub certification: ClauseCertification,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ItemPortrait {
    pub identity: ProjectItemIdentityV2,
    pub source_ordinal: usize,
    pub disposition: ItemDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub engineer_statements: Vec<EngineerStatement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_points: Vec<AssuranceKindV2>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claim_fibers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formal_positions: Vec<CertificationPosition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clauses: Vec<ClausePortrait>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<ClaimConstraint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closure_scope: Option<AssuranceScope>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub residual_trust: Vec<String>,
    pub contract_quality: ContractQuality,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligations: Vec<ObligationResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_flow: Option<crate::manifest::ResourceFlowEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interference: Option<crate::manifest::InterferenceEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<crate::manifest::ProtocolEvidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationPortrait {
    pub covered: usize,
    pub total: usize,
    pub population_sha256: String,
    pub build: ProjectBuildIdentityV2,
    pub intended: Vec<ProjectItemIdentityV2>,
    pub members: Vec<ProjectPopulationMemberV2>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectException {
    pub identity: ProjectItemIdentityV2,
    pub class: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectPortrait {
    pub population: PopulationPortrait,
    pub frontiers: Vec<ProjectFrontiersV2>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub project_lifts: Vec<ProjectLiftV2>,
    pub whole_project_claim: bool,
    pub common_claim_qualifier: String,
    pub boundary_qualifier: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<ProjectException>,
    pub headline: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssuranceReportBody {
    pub source: ReportSource,
    pub collapse_policy_version: u64,
    pub project: ProjectPortrait,
    pub items: Vec<ItemPortrait>,
    pub tcb: Tcb,
}

/// Stable machine portrait.  Deserialization supports untrusted diagnostic
/// comparison only; it does not create [`LiveAssuranceReport`] and therefore
/// cannot satisfy a formal floor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssuranceReport {
    pub schema: String,
    pub report_sha256: String,
    pub body: AssuranceReportBody,
}

#[derive(Debug)]
pub struct ReportError {
    pub reason: String,
}

impl ReportError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid assurance report: {}", self.reason)
    }
}

impl std::error::Error for ReportError {}

#[derive(Clone, Debug)]
struct LiveFloorFrontier {
    fiber: String,
    common: Vec<AssuranceKindV2>,
    whole_project: bool,
}

/// Non-serializable capability proving that the report was constructed from
/// live, admitted authority in this process.  Uploaded JSON and HTML never
/// acquire this type.
#[derive(Debug)]
pub struct LiveAssuranceReport {
    report: AssuranceReport,
    floor_frontiers: Vec<LiveFloorFrontier>,
}

impl LiveAssuranceReport {
    pub fn report(&self) -> &AssuranceReport {
        &self.report
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn into_report(self) -> AssuranceReport {
        self.report
    }

    pub fn evaluate_floor(&self, policy: &FormalFloorPolicy) -> FloorEvaluation {
        let mut results = Vec::new();
        if policy.schema != FLOOR_SCHEMA {
            return FloorEvaluation {
                passed: false,
                results: vec![FloorResult {
                    fiber_sha256: String::new(),
                    minimum: AssuranceKindV2::Runtime,
                    passed: false,
                    reason: format!(
                        "floor schema skew: expected {FLOOR_SCHEMA}, found {}",
                        policy.schema
                    ),
                }],
            };
        }
        for floor in &policy.floors {
            let matching = self
                .floor_frontiers
                .iter()
                .find(|frontier| frontier.fiber == floor.fiber_sha256);
            let (passed, reason) = match matching {
                None => (false, "exact claim fiber is absent".to_string()),
                Some(frontier) if !policy.allow_without_project_lift && !frontier.whole_project => {
                    (
                        false,
                        "exact claim fiber lacks a checked project lift".to_string(),
                    )
                }
                Some(frontier) => {
                    let dominates = frontier
                        .common
                        .iter()
                        .any(|actual| assurance_kind_leq(floor.minimum, *actual));
                    (
                        dominates,
                        if dominates {
                            "live common-claim frontier dominates the declared floor".to_string()
                        } else {
                            "live common-claim frontier does not dominate the declared floor"
                                .to_string()
                        },
                    )
                }
            };
            results.push(FloorResult {
                fiber_sha256: floor.fiber_sha256.clone(),
                minimum: floor.minimum,
                passed,
                reason,
            });
        }
        FloorEvaluation {
            passed: !results.is_empty() && results.iter().all(|result| result.passed),
            results,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormalFloorPolicy {
    pub schema: String,
    #[serde(default)]
    pub allow_without_project_lift: bool,
    pub floors: Vec<FormalFloor>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormalFloor {
    pub fiber_sha256: String,
    pub minimum: AssuranceKindV2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FloorEvaluation {
    pub passed: bool,
    pub results: Vec<FloorResult>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FloorResult {
    pub fiber_sha256: String,
    pub minimum: AssuranceKindV2,
    pub passed: bool,
    pub reason: String,
}

fn digest(domain: &[u8], value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("closed report value serializes");
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn claim_subject_names(program: &Program) -> Vec<String> {
    program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(_)
            | Item::SpecFn(_)
            | Item::Struct(_)
            | Item::Enum(_)
            | Item::EffectDecl(_)
            | Item::Protocol(_) => Some(item.name().to_string()),
            Item::Forge(ForgeItem::Lemma(lemma)) => Some(lemma.name.clone()),
            Item::Forge(_) | Item::SharedDecl(_) | Item::Concurrent(_) | Item::LockDecl(_) => None,
        })
        .collect()
}

fn disposition_label(disposition: &CurrentDisposition<'_>) -> (&'static str, String) {
    match disposition {
        CurrentDisposition::VerusTimeout => {
            ("verus_timeout", "verification exhausted its budget".into())
        }
        CurrentDisposition::TimeoutDegrade => (
            "timeout_degrade",
            "a timeout produced a lower non-current result".into(),
        ),
        CurrentDisposition::EngineUnknown => (
            "engine_unknown",
            "the selected engine could not decide the claim".into(),
        ),
        CurrentDisposition::Refuted => ("refuted", "the claim was refuted".into()),
        CurrentDisposition::WeakContract => (
            "weak_contract",
            "the contract-quality floor rejected the claim".into(),
        ),
        CurrentDisposition::SemanticTautology => (
            "semantic_tautology",
            "the contract does not constrain the result".into(),
        ),
        CurrentDisposition::VacuousPrecondition => (
            "vacuous_precondition",
            "the precondition was vacuous".into(),
        ),
        CurrentDisposition::SettledOther(detail) => ("settled_other", (*detail).to_string()),
    }
}

fn formal_positions(claim: &CurrentClaim<'_>) -> Vec<CertificationPosition> {
    let mut positions = match claim {
        CurrentClaim::Homogeneous { position, .. } => vec![(*position).clone()],
        CurrentClaim::LiftedClausePortfolio { portfolio, .. } => portfolio
            .clauses
            .iter()
            .filter_map(|clause| clause.position.clone())
            .collect(),
    };
    positions.extend(
        claim
            .constraints()
            .iter()
            .map(|constraint| constraint.position.clone()),
    );
    positions
}

fn clauses(claim: &CurrentClaim<'_>) -> Vec<ClausePortrait> {
    match claim {
        CurrentClaim::Homogeneous { .. } => Vec::new(),
        CurrentClaim::LiftedClausePortfolio { portfolio, .. } => portfolio
            .clauses
            .iter()
            .cloned()
            .map(|certification| ClausePortrait {
                address: format!(
                    "{:?}[{}]",
                    certification.address.family, certification.address.index
                ),
                certification,
            })
            .collect(),
    }
}

fn engineer_statements(
    points: &[AssuranceKindV2],
    positions: &[CertificationPosition],
) -> Vec<EngineerStatement> {
    let mut statements = BTreeSet::new();
    for point in points {
        let matching = positions
            .iter()
            .filter(|position| position.assurance_kind_v2() == Some(*point))
            .collect::<Vec<_>>();
        for position in matching {
            let exact_bound = match &position.scope {
                crate::manifest::CertificationScope::Bounded { bound } => Some(bound.clone()),
                _ => None,
            };
            statements.insert(EngineerStatement {
                claim: EngineerClaim::from_kind(*point),
                coverage: EngineerCoverage::from(&position.boundary),
                exact_bound,
            });
        }
    }
    statements.into_iter().collect()
}

fn presentation_digest(authority: &str, item: &str, statements: &[EngineerStatement]) -> String {
    digest(
        b"thermite-assurance-report-presentation-v1\0",
        &(
            authority,
            item,
            REPORT_SCHEMA_VERSION,
            COLLAPSE_POLICY_VERSION,
            statements,
        ),
    )
}

fn residual_trust(cert: &Certificate, positions: &[CertificationPosition]) -> Vec<String> {
    let mut residual = BTreeSet::new();
    for position in positions {
        residual.insert(format!("{:?}", position.residual_trust).to_lowercase());
        residual.extend(position.discharged_trust.iter().cloned());
    }
    if let Some(attribution) = &cert.engine_attribution {
        residual.insert(format!("procedure:{}", attribution.engine));
        residual.extend(attribution.trust_profile.iter().cloned());
    }
    residual.into_iter().collect()
}

fn project_headline(
    covered: usize,
    total: usize,
    fiber_count: usize,
    whole_project_claim: bool,
    common_claim_qualifier: &str,
    boundary_qualifier: &str,
    exceptions: &[ProjectException],
) -> String {
    let core = if total == 0 {
        "No claim-bearing items are in the exact source/build population (0/0).".to_string()
    } else if whole_project_claim {
        format!(
            "Whole-project formal assurance covers {covered}/{total} source/build items across {fiber_count} exact claim fiber(s)."
        )
    } else if covered == total {
        format!(
            "Complete accepted population portrait, not a whole-project claim: {covered}/{total} source/build items are current across {fiber_count} exact claim fiber(s)."
        )
    } else {
        format!(
            "Accepted subset portrait, not a whole-project claim: {covered}/{total} source/build items are current across {fiber_count} exact claim fiber(s)."
        )
    };
    let exceptions = if exceptions.is_empty() {
        String::new()
    } else {
        format!(
            " Exceptions: {}.",
            exceptions
                .iter()
                .map(|exception| format!("{} ({})", exception.identity.item_path, exception.class))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!(
        "{core} Common claim: {common_claim_qualifier}. Boundary: {boundary_qualifier}.{exceptions}"
    )
}

fn common_claim_qualifier(frontiers: &[ProjectFrontiersV2]) -> String {
    let mut rendered = Vec::new();
    for frontier in frontiers {
        let points = match frontier.common_claim_frontier() {
            CommonClaimFrontierV2::NoItems => Vec::new(),
            CommonClaimFrontierV2::Frontier(points) => points
                .iter()
                .map(|point| engineer_claim_text(EngineerClaim::from_kind(*point)))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        };
        if !points.is_empty() {
            rendered.push(format!(
                "fiber {}: {}",
                &frontier.claim_fiber().as_str()[..12],
                points.join(" + ")
            ));
        }
    }
    if rendered.is_empty() {
        "no shared formal claim under the active exact fibers".into()
    } else {
        rendered.join("; ")
    }
}

fn project_exceptions(items: &[ItemPortrait]) -> Vec<ProjectException> {
    items
        .iter()
        .filter_map(|item| {
            let class = match &item.disposition {
                ItemDisposition::Accepted => return None,
                ItemDisposition::NonClaim { class, .. } => class.clone(),
                ItemDisposition::NotCurrent { .. } => "not_current".into(),
                ItemDisposition::Historical { .. } => "historical".into(),
            };
            Some(ProjectException {
                identity: item.identity.clone(),
                class,
            })
        })
        .collect()
}

fn boundary_qualifier(items: &[ItemPortrait]) -> String {
    let boundaries = items
        .iter()
        .flat_map(|item| &item.engineer_statements)
        .map(|statement| match &statement.coverage {
            EngineerCoverage::EndToEnd => "end to end".to_string(),
            EngineerCoverage::ToBoundary { via } => format!("to boundary {via}"),
            EngineerCoverage::ToPlatform { platform } => {
                format!("on frozen platform {platform}")
            }
        })
        .collect::<BTreeSet<_>>();
    if boundaries.is_empty() {
        "no current assurance boundary".to_string()
    } else {
        boundaries.into_iter().collect::<Vec<_>>().join("; ")
    }
}

/// Construct all four report layers from the same validated object graph.
#[allow(clippy::too_many_arguments)]
pub fn build_live_report(
    certs: &[Certificate],
    program: &Program,
    source_path: &str,
    source: &str,
    revision: &str,
    trust: ReportTrust,
    toolchain: Toolchain,
) -> Result<LiveAssuranceReport, ReportError> {
    if source_path.trim().is_empty() {
        return Err(ReportError::new("source path must not be empty"));
    }
    if !is_revision(revision) {
        return Err(ReportError::new(
            "revision must be an exact lowercase 40- or 64-hex identity",
        ));
    }
    let names = claim_subject_names(program);
    let mut by_name = BTreeMap::new();
    for cert in certs {
        if by_name.insert(cert.item.as_str(), cert).is_some() {
            return Err(ReportError::new(format!(
                "duplicate certificate for source item {}",
                cert.item
            )));
        }
    }
    let expected = names.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let actual = by_name.keys().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        let missing = expected.difference(&actual).copied().collect::<Vec<_>>();
        let extra = actual.difference(&expected).copied().collect::<Vec<_>>();
        return Err(ReportError::new(format!(
            "certificate population does not equal the source/build inventory; missing={missing:?}, extra={extra:?}"
        )));
    }

    let source_sha256 = digest(b"thermite-assurance-report-source-v1\0", &source);
    let artifact_sha256 = digest(
        b"thermite-assurance-report-artifact-v1\0",
        &(source_sha256.as_str(), format!("{program:?}"), &toolchain),
    );
    let item_identities = names
        .iter()
        .map(|name| ProjectItemIdentityV2 {
            source_path: source_path.to_string(),
            item_path: name.clone(),
        })
        .collect::<Vec<_>>();

    let mut population_members = Vec::with_capacity(names.len());
    let mut items = Vec::with_capacity(names.len());
    let mut accepted_sets: Vec<(
        ProjectItemIdentityV2,
        Vec<crate::assurance_v2::ItemClaimSetV2>,
        String,
    )> = Vec::new();

    for (source_ordinal, (name, identity)) in names.iter().zip(&item_identities).enumerate() {
        let cert = by_name[name.as_str()];
        let authority = cert.current_authority_digest().ok();
        let current = cert.current_assurance();
        let (disposition, points, fibers, positions, item_clauses, constraints) = match current {
            Ok(CurrentAssurance::Accepted { claim }) => {
                let mut sets = Vec::new();
                for set in claim.item_claim_sets() {
                    sets.push(
                        set.clone()
                            .rebase_item_identity(identity.clone())
                            .map_err(|error| ReportError::new(error.to_string()))?,
                    );
                }
                if sets.is_empty() {
                    return Err(ReportError::new(format!(
                        "accepted item {name} has no formal claim set"
                    )));
                }
                let mut points = claim.policy_points();
                points.sort();
                points.dedup();
                let mut fibers = sets
                    .iter()
                    .map(|set| set.claim_fiber().as_str().to_string())
                    .collect::<Vec<_>>();
                fibers.sort();
                fibers.dedup();
                let positions = formal_positions(&claim);
                let item_clauses = clauses(&claim);
                let constraints = claim.constraints().to_vec();
                accepted_sets.push((
                    identity.clone(),
                    sets,
                    authority
                        .clone()
                        .expect("accepted current authority has a digest"),
                ));
                population_members.push(ProjectPopulationMemberV2 {
                    identity: identity.clone(),
                    disposition: ProjectDispositionV2::Accepted,
                });
                (
                    ItemDisposition::Accepted,
                    points,
                    fibers,
                    positions,
                    item_clauses,
                    constraints,
                )
            }
            Ok(CurrentAssurance::NonClaim { disposition }) => {
                let (class, detail) = disposition_label(&disposition);
                population_members.push(ProjectPopulationMemberV2 {
                    identity: identity.clone(),
                    disposition: ProjectDispositionV2::NonClaim {
                        reason: format!("{class}: {detail}"),
                    },
                });
                (
                    ItemDisposition::NonClaim {
                        class: class.into(),
                        detail,
                    },
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                )
            }
            Err(error) => {
                population_members.push(ProjectPopulationMemberV2 {
                    identity: identity.clone(),
                    disposition: ProjectDispositionV2::NonClaim {
                        reason: format!("not_current: {}", error.reason),
                    },
                });
                (
                    ItemDisposition::NotCurrent {
                        reason: error.reason,
                    },
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                )
            }
        };
        let statements = engineer_statements(&points, &positions);
        let presentation_sha256 = authority
            .as_deref()
            .map(|authority| presentation_digest(authority, name, &statements));
        items.push(ItemPortrait {
            identity: identity.clone(),
            source_ordinal,
            disposition,
            authority_sha256: authority,
            presentation_sha256,
            engineer_statements: statements,
            policy_points: points,
            claim_fibers: fibers,
            formal_positions: positions.clone(),
            clauses: item_clauses,
            constraints,
            effects: cert.effects.clone(),
            closure_scope: cert.assurance_scope.clone(),
            residual_trust: residual_trust(cert, &positions),
            contract_quality: cert.contract_quality.clone(),
            obligations: cert.obligations.clone(),
            resource_flow: cert.resource_flow.clone(),
            interference: cert.interference.clone(),
            protocol: cert.protocol.clone(),
        });
    }

    let build = ProjectBuildIdentityV2 {
        crate_name: source_path.to_string(),
        target: "thermite-source".into(),
        features: Vec::new(),
        platform: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        generated_sources: Vec::new(),
        artifact_sha256: artifact_sha256.clone(),
    };
    let population = ProjectPopulationV2::derive(
        build.clone(),
        item_identities.clone(),
        population_members.clone(),
    )
    .map_err(|error| ReportError::new(error.to_string()))?;
    let population_sha256 = population.identity_digest();

    let mut fibers = BTreeMap::<String, ClaimFiberAddressV2>::new();
    for (_, sets, _) in &accepted_sets {
        for set in sets {
            fibers.insert(
                set.claim_fiber().as_str().to_string(),
                set.claim_fiber().clone(),
            );
        }
    }
    let mut frontiers = Vec::new();
    let mut project_lifts = Vec::new();
    let mut floor_frontiers = Vec::new();
    for (fiber_sha256, fiber) in fibers {
        let mut transports = Vec::with_capacity(accepted_sets.len());
        for (identity, sets, _) in &accepted_sets {
            let mut matching = sets
                .iter()
                .filter(|set| set.claim_fiber().as_str() == fiber_sha256)
                .cloned();
            match (matching.next(), matching.next()) {
                (Some(set), None) => {
                    transports.push(PopulationClaimSetTransportV2::Transported(set))
                }
                (None, None) => transports.push(PopulationClaimSetTransportV2::Unavailable {
                    item: identity.clone(),
                    reason: "no checked transport into this exact claim fiber".into(),
                }),
                (_, Some(_)) => {
                    return Err(ReportError::new(format!(
                        "item {} has duplicate claim sets in fiber {fiber_sha256}",
                        identity.item_path
                    )))
                }
            }
        }
        let mut frontier = ProjectFrontiersV2::aggregate(&population, fiber, transports)
            .map_err(|error| ReportError::new(error.to_string()))?;
        if population.all_accepted() && frontier.unavailable_transports().is_empty() {
            let evidence_vector = accepted_sets
                .iter()
                .map(|(identity, sets, authority)| LiftedItemEvidenceV2 {
                    item: identity.clone(),
                    evidence_identity: authority.clone(),
                    refutation_complete: sets
                        .iter()
                        .filter(|set| set.claim_fiber().as_str() == fiber_sha256)
                        .flat_map(|set| set.frontier())
                        .all(|kind| {
                            matches!(
                                kind,
                                AssuranceKindV2::Runtime
                                    | AssuranceKindV2::Bounded
                                    | AssuranceKindV2::SolverComplete
                                    | AssuranceKindV2::LeanComplete
                            )
                        }),
                })
                .collect();
            let lift = ProjectLiftV2::new(
                &population,
                &frontier,
                evidence_vector,
                "Thermite.CertificationMetatheory.ProjectLift.certifiesProject".into(),
                Some("thermite-project-source-order-scheduler-v1".into()),
            )
            .map_err(|error| ReportError::new(error.to_string()))?;
            frontier
                .authorize_project(&lift)
                .map_err(|error| ReportError::new(error.to_string()))?;
            project_lifts.push(lift);
        }
        let common = match frontier.common_claim_frontier() {
            CommonClaimFrontierV2::NoItems => Vec::new(),
            CommonClaimFrontierV2::Frontier(points) => points.clone(),
        };
        let whole_project = matches!(
            frontier.scope(),
            ProjectPortraitScopeV2::WholeProject { .. }
        );
        floor_frontiers.push(LiveFloorFrontier {
            fiber: fiber_sha256,
            common,
            whole_project,
        });
        frontiers.push(frontier);
    }

    let covered = population.accepted_identities().len();
    let total = population.members.len();
    let whole_project_claim = total > 0
        && covered == total
        && !frontiers.is_empty()
        && frontiers.iter().all(|frontier| {
            matches!(
                frontier.scope(),
                ProjectPortraitScopeV2::WholeProject { .. }
            )
        });
    let common_claim_qualifier = common_claim_qualifier(&frontiers);
    let boundary_qualifier = boundary_qualifier(&items);
    let exceptions = project_exceptions(&items);
    let headline = project_headline(
        covered,
        total,
        frontiers.len(),
        whole_project_claim,
        &common_claim_qualifier,
        &boundary_qualifier,
        &exceptions,
    );
    let audit = AuditManifest::from_certificates(certs, program, toolchain);
    let body = AssuranceReportBody {
        source: ReportSource {
            revision: revision.to_string(),
            source_path: source_path.to_string(),
            source_sha256,
            artifact_sha256,
            trust,
        },
        collapse_policy_version: COLLAPSE_POLICY_VERSION,
        project: ProjectPortrait {
            population: PopulationPortrait {
                covered,
                total,
                population_sha256,
                build,
                intended: item_identities,
                members: population_members,
            },
            frontiers,
            project_lifts,
            whole_project_claim,
            common_claim_qualifier,
            boundary_qualifier,
            exceptions,
            headline,
        },
        items,
        tcb: audit.tcb,
    };
    let report_sha256 = digest(b"thermite-assurance-report-v1\0", &body);
    let report = AssuranceReport {
        schema: REPORT_SCHEMA.into(),
        report_sha256,
        body,
    };
    report.validate()?;
    Ok(LiveAssuranceReport {
        report,
        floor_frontiers,
    })
}

impl AssuranceReport {
    pub fn validate(&self) -> Result<(), ReportError> {
        if self.schema != REPORT_SCHEMA {
            return Err(ReportError::new(format!(
                "schema skew: expected {REPORT_SCHEMA}, found {}",
                self.schema
            )));
        }
        if self.body.collapse_policy_version != COLLAPSE_POLICY_VERSION {
            return Err(ReportError::new(format!(
                "collapse-policy skew: expected {COLLAPSE_POLICY_VERSION}, found {}",
                self.body.collapse_policy_version
            )));
        }
        if !is_revision(&self.body.source.revision)
            || !is_sha256(&self.body.source.source_sha256)
            || !is_sha256(&self.body.source.artifact_sha256)
            || !is_sha256(&self.body.project.population.population_sha256)
        {
            return Err(ReportError::new(
                "invalid source, artifact, or population identity",
            ));
        }
        let population = &self.body.project.population;
        let derived_population = ProjectPopulationV2::derive(
            population.build.clone(),
            population.intended.clone(),
            population.members.clone(),
        )
        .map_err(|error| ReportError::new(error.to_string()))?;
        if population.population_sha256 != derived_population.identity_digest() {
            return Err(ReportError::new(
                "population identity does not match its exact build/inventory partition",
            ));
        }
        if population.total != population.members.len()
            || population.total != population.intended.len()
            || population.covered
                != population
                    .members
                    .iter()
                    .filter(|member| matches!(member.disposition, ProjectDispositionV2::Accepted))
                    .count()
            || self.body.items.len() != population.total
        {
            return Err(ReportError::new(
                "population counts do not match the complete member/item portraits",
            ));
        }
        for (ordinal, (member, item)) in population.members.iter().zip(&self.body.items).enumerate()
        {
            if member.identity != item.identity || item.source_ordinal != ordinal {
                return Err(ReportError::new(
                    "item portrait is not the exact source-ordered population",
                ));
            }
            let disposition_matches = matches!(
                (&member.disposition, &item.disposition),
                (ProjectDispositionV2::Accepted, ItemDisposition::Accepted)
                    | (
                        ProjectDispositionV2::NonClaim { .. },
                        ItemDisposition::NonClaim { .. } | ItemDisposition::NotCurrent { .. }
                    )
                    | (
                        ProjectDispositionV2::LegacyUnversioned { .. },
                        ItemDisposition::Historical { .. }
                    )
            );
            if !disposition_matches {
                return Err(ReportError::new(format!(
                    "population/item disposition mismatch for {}",
                    item.identity.item_path
                )));
            }
            if let (Some(authority), Some(presentation)) = (
                item.authority_sha256.as_deref(),
                item.presentation_sha256.as_deref(),
            ) {
                if presentation
                    != presentation_digest(
                        authority,
                        &item.identity.item_path,
                        &item.engineer_statements,
                    )
                {
                    return Err(ReportError::new(format!(
                        "presentation digest mismatch for {}",
                        item.identity.item_path
                    )));
                }
            }
            if item.authority_sha256.is_some() != item.presentation_sha256.is_some() {
                return Err(ReportError::new(format!(
                    "authority/presentation presence mismatch for {}",
                    item.identity.item_path
                )));
            }
            if item
                .authority_sha256
                .as_deref()
                .is_some_and(|authority| !is_sha256(authority))
            {
                return Err(ReportError::new(format!(
                    "invalid authority identity for {}",
                    item.identity.item_path
                )));
            }
        }
        if self
            .body
            .project
            .frontiers
            .iter()
            .any(|frontier| frontier.population_sha256() != population.population_sha256)
        {
            return Err(ReportError::new(
                "project frontier belongs to a different source/build population",
            ));
        }
        for lift in &self.body.project.project_lifts {
            let frontier = self
                .body
                .project
                .frontiers
                .iter()
                .find(|frontier| frontier.claim_fiber() == lift.claim_fiber())
                .ok_or_else(|| {
                    ReportError::new("project lift has no matching exact-fiber frontier")
                })?;
            lift.validate_against(&derived_population, frontier)
                .map_err(|error| ReportError::new(error.to_string()))?;
        }
        let whole_project_claim = population.total > 0
            && population.covered == population.total
            && !self.body.project.frontiers.is_empty()
            && self.body.project.frontiers.iter().all(|frontier| {
                matches!(
                    frontier.scope(),
                    ProjectPortraitScopeV2::WholeProject { .. }
                )
            })
            && self.body.project.project_lifts.len() == self.body.project.frontiers.len();
        let common_claim_qualifier = common_claim_qualifier(&self.body.project.frontiers);
        let boundary_qualifier = boundary_qualifier(&self.body.items);
        let exceptions = project_exceptions(&self.body.items);
        if self.body.project.whole_project_claim != whole_project_claim
            || self.body.project.common_claim_qualifier != common_claim_qualifier
            || self.body.project.boundary_qualifier != boundary_qualifier
            || self.body.project.exceptions != exceptions
            || self.body.project.headline
                != project_headline(
                    population.covered,
                    population.total,
                    self.body.project.frontiers.len(),
                    whole_project_claim,
                    &common_claim_qualifier,
                    &boundary_qualifier,
                    &exceptions,
                )
        {
            return Err(ReportError::new(
                "project headline does not match the checked population/lift state",
            ));
        }
        let expected = digest(b"thermite-assurance-report-v1\0", &self.body);
        if expected != self.report_sha256 {
            return Err(ReportError::new(
                "report digest does not match its normalized body",
            ));
        }
        Ok(())
    }

    pub fn normalized_json(&self) -> Result<String, ReportError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map(|json| format!("{json}\n"))
            .map_err(|error| ReportError::new(format!("JSON serialization failed: {error}")))
    }

    pub fn render_headline(&self) -> Result<String, ReportError> {
        self.validate()?;
        let trust = match self.body.source.trust {
            ReportTrust::LocalDiagnostic => "LOCAL DIAGNOSTIC",
            ReportTrust::UntrustedPullRequest => "UNTRUSTED PR DIAGNOSTIC",
            ReportTrust::ProtectedExactSha => "PROTECTED EXACT-SHA PORTRAIT",
        };
        Ok(format!(
            "{trust}\n{}\nrevision: {}\nreport: {}\n",
            self.body.project.headline, self.body.source.revision, self.report_sha256
        ))
    }

    pub fn render_items(&self) -> Result<String, ReportError> {
        let mut out = self.render_headline()?;
        let mut groups = BTreeMap::<String, Vec<String>>::new();
        for item in &self.body.items {
            let fibers = if item.claim_fibers.is_empty() {
                "no-current-fiber".to_string()
            } else {
                item.claim_fibers.join(",")
            };
            let statements = if item.engineer_statements.is_empty() {
                disposition_text(&item.disposition)
            } else {
                item.engineer_statements
                    .iter()
                    .map(statement_text)
                    .collect::<Vec<_>>()
                    .join(" + ")
            };
            groups
                .entry(format!("fiber={fibers} | {statements}"))
                .or_default()
                .push(item.identity.item_path.clone());
        }
        out.push_str("\ncrate groups (count/project-total):\n");
        for (group, identities) in groups {
            out.push_str(&format!(
                "  {}/{} — {} — {}\n",
                identities.len(),
                self.body.project.population.total,
                group,
                identities.join(", ")
            ));
        }
        out.push_str("\nitems (source order; use --explain <item> for clauses):\n");
        for item in &self.body.items {
            out.push_str(&format!(
                "  {}/{} {} — {}\n",
                item.source_ordinal + 1,
                self.body.project.population.total,
                item.identity.item_path,
                disposition_text(&item.disposition)
            ));
            for statement in &item.engineer_statements {
                out.push_str(&format!("    {}\n", statement_text(statement)));
            }
        }
        Ok(out)
    }

    pub fn render_explain(&self, subject: &str) -> Result<String, ReportError> {
        self.validate()?;
        if subject == "project" {
            let mut out = self.render_headline()?;
            out.push_str(&format!(
                "population: {}/{}\npopulation identity: {}\nsource: {}\nartifact: {}\ncollapse policy: {}\n",
                self.body.project.population.covered,
                self.body.project.population.total,
                self.body.project.population.population_sha256,
                self.body.source.source_sha256,
                self.body.source.artifact_sha256,
                self.body.collapse_policy_version
            ));
            out.push_str(
                "projection disclosure: engineer labels forget exact procedure, formal frame, residual trust, tool/resource premises, and evidence provenance; the formal blocks below retain them.\n",
            );
            for frontier in &self.body.project.frontiers {
                let formal = serde_json::to_string_pretty(frontier).map_err(|error| {
                    ReportError::new(format!("frontier serialization failed: {error}"))
                })?;
                out.push_str(&format!(
                    "fiber {}\n  common: {:?}\n  evidence: {:?}\n  scope: {:?}\n  formal frontier and normalization witnesses:\n{}\n",
                    frontier.claim_fiber().as_str(),
                    frontier.common_claim_frontier(),
                    frontier.evidence_frontier(),
                    frontier.scope(),
                    formal
                ));
            }
            out.push_str(&format!(
                "project lifts:\n{}\ntrusted computing base / tool and axiom premises:\n{}\n",
                serde_json::to_string_pretty(&self.body.project.project_lifts).map_err(
                    |error| ReportError::new(format!("project-lift serialization failed: {error}"))
                )?,
                serde_json::to_string_pretty(&self.body.tcb).map_err(|error| {
                    ReportError::new(format!("TCB serialization failed: {error}"))
                })?
            ));
            return Ok(out);
        }
        let item = self
            .body
            .items
            .iter()
            .find(|item| item.identity.item_path == subject)
            .ok_or_else(|| ReportError::new(format!("unknown report subject {subject}")))?;
        let mut out = format!(
            "{} — {}\nsource ordinal: {}/{}\nauthority: {}\npresentation: {}\n",
            item.identity.item_path,
            disposition_text(&item.disposition),
            item.source_ordinal + 1,
            self.body.project.population.total,
            item.authority_sha256.as_deref().unwrap_or("none"),
            item.presentation_sha256.as_deref().unwrap_or("none")
        );
        out.push_str(&format!(
            "engineer statements: {:?}\nclaim fibers: {:?}\nformal positions and boundaries: {:?}\nresidual trust and procedure context: {:?}\nconstraints: {:?}\nclosure scope: {:?}\ncontract quality: {:?}\n",
            item.engineer_statements,
            item.claim_fibers,
            item.formal_positions,
            item.residual_trust,
            item.constraints,
            item.closure_scope,
            item.contract_quality,
        ));
        for clause in &item.clauses {
            out.push_str(&format!(
                "clause {}: procedure={:?}, position={:?}, terminal={:?}, classification={:?}, route_evidence={:?}\n",
                clause.address,
                clause.certification.procedure,
                clause.certification.position,
                clause.certification.terminal,
                clause.certification.classification,
                clause.certification.evidence,
            ));
        }
        out.push_str(&format!(
            "obligations: {:?}\nresource-flow evidence: {:?}\ninterference evidence: {:?}\nprotocol evidence: {:?}\ntrusted computing base / tool and axiom premises:\n{}\n",
            item.obligations,
            item.resource_flow,
            item.interference,
            item.protocol,
            serde_json::to_string_pretty(&self.body.tcb).map_err(|error| {
                ReportError::new(format!("TCB serialization failed: {error}"))
            })?
        ));
        Ok(out)
    }

    pub fn render_html(&self) -> Result<String, ReportError> {
        self.validate()?;
        let mut items = String::new();
        for item in &self.body.items {
            let statements = item
                .engineer_statements
                .iter()
                .map(statement_text)
                .collect::<Vec<_>>()
                .join("; ");
            let clauses = item
                .clauses
                .iter()
                .map(|clause| {
                    format!(
                        "<li><code>{}</code>: procedure <code>{}</code>; position <code>{}</code>; terminal <code>{}</code></li>",
                        escape_html(&clause.address),
                        escape_html(&format!("{:?}", clause.certification.procedure)),
                        escape_html(&format!("{:?}", clause.certification.position)),
                        escape_html(&format!("{:?}", clause.certification.terminal)),
                    )
                })
                .collect::<String>();
            items.push_str(&format!(
                "<li id=\"item-{}\"><h3><code>{}</code></h3><p>{}</p><p>{}</p><p>fibers <code>{}</code></p><p>residual trust <code>{}</code></p><details><summary>Clause evidence ({})</summary><ul>{}</ul></details></li>",
                item.source_ordinal,
                escape_html(&item.identity.item_path),
                escape_html(&disposition_text(&item.disposition)),
                escape_html(&statements),
                escape_html(&item.claim_fibers.join(", ")),
                escape_html(&item.residual_trust.join(", ")),
                item.clauses.len(),
                clauses,
            ));
        }
        let frontiers = self
            .body
            .project
            .frontiers
            .iter()
            .map(|frontier| {
                format!(
                    "<li><code>{}</code><br>common <code>{}</code><br>evidence <code>{}</code><br>scope <code>{}</code></li>",
                    escape_html(frontier.claim_fiber().as_str()),
                    escape_html(&format!("{:?}", frontier.common_claim_frontier())),
                    escape_html(&format!("{:?}", frontier.evidence_frontier())),
                    escape_html(&format!("{:?}", frontier.scope())),
                )
            })
            .collect::<String>();
        Ok(format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'\"><title>Thermite assurance report</title></head><body><header><strong>{}</strong><h1>Thermite assurance report</h1><p>{}</p><p>source <code>{}</code></p><p>revision <code>{}</code></p><p>report <code>{}</code></p><nav><a href=\"report.json\">Complete formal JSON portrait</a></nav></header><main><section><h2>Common project claims</h2><p>{}</p><ul>{}</ul></section><section><h2>Items ({}/{})</h2><ol>{}</ol></section></main></body></html>\n",
            escape_html(match self.body.source.trust {
                ReportTrust::UntrustedPullRequest => "UNTRUSTED PR DIAGNOSTIC — NOT CERTIFICATION AUTHORITY",
                ReportTrust::ProtectedExactSha => "PROTECTED EXACT-SHA PORTRAIT",
                ReportTrust::LocalDiagnostic => "LOCAL DIAGNOSTIC — NOT CERTIFICATION AUTHORITY",
            }),
            escape_html(&self.body.project.headline),
            escape_html(&self.body.source.source_path),
            escape_html(&self.body.source.revision),
            escape_html(&self.report_sha256),
            escape_html(&self.body.project.common_claim_qualifier),
            frontiers,
            self.body.project.population.covered,
            self.body.project.population.total,
            items
        ))
    }
}

fn disposition_text(disposition: &ItemDisposition) -> String {
    match disposition {
        ItemDisposition::Accepted => "accepted current authority".into(),
        ItemDisposition::NonClaim { class, detail } => format!("non-claim ({class}: {detail})"),
        ItemDisposition::NotCurrent { reason } => format!("not current ({reason})"),
        ItemDisposition::Historical { warning, .. } => warning.clone(),
    }
}

fn engineer_claim_text(claim: EngineerClaim) -> String {
    match claim {
        EngineerClaim::CheckedThisExecution => "Checked for this execution".to_string(),
        EngineerClaim::CheckedThroughBound => "Checked through the declared bound".to_string(),
        EngineerClaim::ProvedAllInputsMayNotProduceWitness => {
            "Proven for all inputs; a false claim may not produce a complete witness".into()
        }
        EngineerClaim::ProvedAllInputsWithConcreteWitness => {
            "Proven for all inputs; a false claim has a concrete witness".into()
        }
        EngineerClaim::ProvedAllInputsWithEmpiricalFalsification => {
            "Proven for all inputs with empirical falsification".into()
        }
    }
}

fn statement_text(statement: &EngineerStatement) -> String {
    let claim = match statement.claim {
        EngineerClaim::CheckedThroughBound => format!(
            "Checked through bound {}",
            statement.exact_bound.as_deref().unwrap_or("<missing>")
        ),
        other => engineer_claim_text(other),
    };
    let coverage = match &statement.coverage {
        EngineerCoverage::EndToEnd => "end to end".into(),
        EngineerCoverage::ToBoundary { via } => format!("to boundary {via}"),
        EngineerCoverage::ToPlatform { platform } => format!("on frozen platform {platform}"),
    };
    format!("{claim} — {coverage}")
}

fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(character),
        }
    }
    out
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ComparisonStatus {
    Compared,
    NoComparison { reason: String },
    SchemaSkew { base: String, head: String },
    PolicySkew { base: u64, head: u64 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormalMovement {
    Equal,
    Strengthened,
    Weakened,
    Incomparable,
    ChangedFiber,
    ChangedBoundary,
    ChangedResidualContext,
    PopulationAdded,
    PopulationRemoved,
    NewNonClaim,
    NewlyHistorical,
    BecameCurrent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemComparison {
    pub identity: ProjectItemIdentityV2,
    pub movements: Vec<FormalMovement>,
    pub base_authority_sha256: Option<String>,
    pub head_authority_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReportComparison {
    pub status: ComparisonStatus,
    pub base_revision: String,
    pub head_revision: String,
    pub common_frontier_changed: bool,
    pub items: Vec<ItemComparison>,
}

pub fn compare_reports(
    base: &AssuranceReport,
    head: &AssuranceReport,
    expected_base_revision: &str,
) -> ReportComparison {
    let no_comparison = |reason: String| ReportComparison {
        status: ComparisonStatus::NoComparison { reason },
        base_revision: base.body.source.revision.clone(),
        head_revision: head.body.source.revision.clone(),
        common_frontier_changed: false,
        items: Vec::new(),
    };
    if base.schema != head.schema {
        return ReportComparison {
            status: ComparisonStatus::SchemaSkew {
                base: base.schema.clone(),
                head: head.schema.clone(),
            },
            base_revision: base.body.source.revision.clone(),
            head_revision: head.body.source.revision.clone(),
            common_frontier_changed: false,
            items: Vec::new(),
        };
    }
    if base.body.collapse_policy_version != head.body.collapse_policy_version {
        return ReportComparison {
            status: ComparisonStatus::PolicySkew {
                base: base.body.collapse_policy_version,
                head: head.body.collapse_policy_version,
            },
            base_revision: base.body.source.revision.clone(),
            head_revision: head.body.source.revision.clone(),
            common_frontier_changed: false,
            items: Vec::new(),
        };
    }
    if let Err(error) = base.validate() {
        return no_comparison(format!("base report failed validation: {}", error.reason));
    }
    if let Err(error) = head.validate() {
        return no_comparison(format!("head report failed validation: {}", error.reason));
    }
    if base.body.source.revision != expected_base_revision {
        return no_comparison(format!(
            "base report is bound to {}, not exact base {}",
            base.body.source.revision, expected_base_revision
        ));
    }
    let base_items = base
        .body
        .items
        .iter()
        .map(|item| (item.identity.clone(), item))
        .collect::<BTreeMap<_, _>>();
    let head_items = head
        .body
        .items
        .iter()
        .map(|item| (item.identity.clone(), item))
        .collect::<BTreeMap<_, _>>();
    let identities = base_items
        .keys()
        .chain(head_items.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut items = Vec::new();
    for identity in identities {
        let base_item = base_items.get(&identity).copied();
        let head_item = head_items.get(&identity).copied();
        let mut movements = match (base_item, head_item) {
            (None, Some(_)) => vec![FormalMovement::PopulationAdded],
            (Some(_), None) => vec![FormalMovement::PopulationRemoved],
            (Some(base_item), Some(head_item)) => compare_items(base_item, head_item),
            (None, None) => unreachable!(),
        };
        movements.sort();
        movements.dedup();
        if movements != [FormalMovement::Equal] {
            items.push(ItemComparison {
                identity,
                movements,
                base_authority_sha256: base_item.and_then(|item| item.authority_sha256.clone()),
                head_authority_sha256: head_item.and_then(|item| item.authority_sha256.clone()),
            });
        }
    }
    ReportComparison {
        status: ComparisonStatus::Compared,
        base_revision: base.body.source.revision.clone(),
        head_revision: head.body.source.revision.clone(),
        common_frontier_changed: base.body.project.frontiers != head.body.project.frontiers,
        items,
    }
}

fn json_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Array(values) => {
            1 + values.iter().map(json_depth).max().unwrap_or_default()
        }
        serde_json::Value::Object(values) => {
            1 + values.values().map(json_depth).max().unwrap_or_default()
        }
        _ => 1,
    }
}

fn bounded_json(bytes: &[u8]) -> Result<serde_json::Value, ReportError> {
    if bytes.len() > MAX_REPORT_BYTES {
        return Err(ReportError::new(format!(
            "JSON input exceeds the {MAX_REPORT_BYTES}-byte limit"
        )));
    }
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| ReportError::new(format!("invalid JSON: {error}")))?;
    if json_depth(&value) > MAX_JSON_DEPTH {
        return Err(ReportError::new(format!(
            "JSON input exceeds the maximum nesting depth {MAX_JSON_DEPTH}"
        )));
    }
    Ok(value)
}

pub fn parse_report_json(bytes: &[u8]) -> Result<AssuranceReport, ReportError> {
    let report = serde_json::from_value(bounded_json(bytes)?)
        .map_err(|error| ReportError::new(format!("invalid report schema: {error}")))?;
    Ok(report)
}

pub fn parse_floor_json(bytes: &[u8]) -> Result<FormalFloorPolicy, ReportError> {
    serde_json::from_value(bounded_json(bytes)?)
        .map_err(|error| ReportError::new(format!("invalid floor schema: {error}")))
}

fn compare_items(base: &ItemPortrait, head: &ItemPortrait) -> Vec<FormalMovement> {
    match (&base.disposition, &head.disposition) {
        (ItemDisposition::Accepted, ItemDisposition::NonClaim { .. })
        | (ItemDisposition::Accepted, ItemDisposition::NotCurrent { .. }) => {
            return vec![FormalMovement::NewNonClaim]
        }
        (ItemDisposition::Accepted, ItemDisposition::Historical { .. }) => {
            return vec![FormalMovement::NewlyHistorical]
        }
        (
            ItemDisposition::NonClaim { .. }
            | ItemDisposition::NotCurrent { .. }
            | ItemDisposition::Historical { .. },
            ItemDisposition::Accepted,
        ) => return vec![FormalMovement::BecameCurrent],
        _ => {}
    }
    let mut movements = Vec::new();
    if base.claim_fibers != head.claim_fibers {
        movements.push(FormalMovement::ChangedFiber);
    } else if base.policy_points == head.policy_points {
        movements.push(FormalMovement::Equal);
    } else {
        let base_below_head = base.policy_points.iter().all(|base_point| {
            head.policy_points
                .iter()
                .any(|head_point| assurance_kind_leq(*base_point, *head_point))
        });
        let head_below_base = head.policy_points.iter().all(|head_point| {
            base.policy_points
                .iter()
                .any(|base_point| assurance_kind_leq(*head_point, *base_point))
        });
        movements.push(match (base_below_head, head_below_base) {
            (true, false) => FormalMovement::Strengthened,
            (false, true) => FormalMovement::Weakened,
            (true, true) => FormalMovement::Equal,
            (false, false) => FormalMovement::Incomparable,
        });
    }
    let base_boundaries = base
        .formal_positions
        .iter()
        .map(|position| &position.boundary)
        .collect::<Vec<_>>();
    let head_boundaries = head
        .formal_positions
        .iter()
        .map(|position| &position.boundary)
        .collect::<Vec<_>>();
    if base_boundaries != head_boundaries {
        movements.push(FormalMovement::ChangedBoundary);
    }
    if base.residual_trust != head.residual_trust {
        movements.push(FormalMovement::ChangedResidualContext);
    }
    movements
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ClassificationCertificate, ClassificationVerdict, Level};

    fn parsed(source: &str) -> Program {
        let parsed = thermite_syntax::parse(source);
        assert!(parsed.is_clean(), "{:?}", parsed.errors);
        parsed.program
    }

    fn cert(name: &str, kind: AssuranceKindV2, boundary: CertificationBoundary) -> Certificate {
        let (scope, refutation, trust, level) = match kind {
            AssuranceKindV2::Runtime => (
                crate::manifest::CertificationScope::PerExecution,
                crate::manifest::RefutationChannel::Abort,
                crate::manifest::ResidualTrust::Fiat,
                Level::L1,
            ),
            AssuranceKindV2::Bounded => (
                crate::manifest::CertificationScope::Bounded { bound: "8".into() },
                crate::manifest::RefutationChannel::Trace { bound: "8".into() },
                crate::manifest::ResidualTrust::Solver,
                Level::L2,
            ),
            AssuranceKindV2::SolverIncomplete => (
                crate::manifest::CertificationScope::All,
                crate::manifest::RefutationChannel::Incomplete,
                crate::manifest::ResidualTrust::Solver,
                Level::L3,
            ),
            AssuranceKindV2::SolverComplete => (
                crate::manifest::CertificationScope::All,
                crate::manifest::RefutationChannel::Complete,
                crate::manifest::ResidualTrust::Solver,
                Level::L4,
            ),
            AssuranceKindV2::LeanEmpirical => (
                crate::manifest::CertificationScope::All,
                crate::manifest::RefutationChannel::Empirical,
                crate::manifest::ResidualTrust::LeanChecked,
                Level::L3,
            ),
            AssuranceKindV2::LeanComplete => (
                crate::manifest::CertificationScope::All,
                crate::manifest::RefutationChannel::Complete,
                crate::manifest::ResidualTrust::LeanChecked,
                Level::L4,
            ),
        };
        Certificate::new(name, level, vec!["pure".into()], 0, Vec::new())
            .with_rfc3_coordinates(
                CertificationPosition {
                    scope,
                    refutation,
                    residual_trust: trust,
                    discharged_trust: vec!["fixture-evidence".into()],
                    boundary,
                },
                ClassificationCertificate {
                    fragment: "fixture-fragment-v1".into(),
                    verdict: ClassificationVerdict::Admitted,
                },
            )
            .unwrap()
            .with_live_disposition(crate::manifest::LiveResultDisposition::Accepted)
    }

    fn report(kind: AssuranceKindV2, revision: char) -> LiveAssuranceReport {
        report_at_path(kind, revision, "src/main.th")
    }

    fn report_at_path(
        kind: AssuranceKindV2,
        revision: char,
        source_path: &str,
    ) -> LiveAssuranceReport {
        let source = "fn f(x: Int) -> Int ! pure requires true ensures result == x { x }";
        let program = parsed(source);
        build_live_report(
            &[cert("f", kind, CertificationBoundary::EndToEnd)],
            &program,
            source_path,
            source,
            &revision.to_string().repeat(40),
            ReportTrust::UntrustedPullRequest,
            Toolchain::new("verus-test"),
        )
        .unwrap()
    }

    fn rebind_project_portrait(report: &mut AssuranceReport) {
        let population = ProjectPopulationV2::derive(
            report.body.project.population.build.clone(),
            report.body.project.population.intended.clone(),
            report.body.project.population.members.clone(),
        )
        .unwrap();
        report.body.project.population.covered = population.accepted_identities().len();
        report.body.project.population.total = population.members.len();
        report.body.project.population.population_sha256 = population.identity_digest();
        report.body.project.frontiers.clear();
        report.body.project.project_lifts.clear();
        report.body.project.whole_project_claim = false;
        report.body.project.common_claim_qualifier = common_claim_qualifier(&[]);
        report.body.project.boundary_qualifier = boundary_qualifier(&report.body.items);
        report.body.project.exceptions = project_exceptions(&report.body.items);
        report.body.project.headline = project_headline(
            report.body.project.population.covered,
            report.body.project.population.total,
            0,
            false,
            &report.body.project.common_claim_qualifier,
            &report.body.project.boundary_qualifier,
            &report.body.project.exceptions,
        );
        report.report_sha256 = digest(b"thermite-assurance-report-v1\0", &report.body);
    }

    #[test]
    fn one_validated_object_drives_all_disclosure_layers_deterministically() {
        let first = report(AssuranceKindV2::SolverComplete, 'a');
        let second = report(AssuranceKindV2::SolverComplete, 'a');
        assert_eq!(
            first.report.normalized_json().unwrap(),
            second.report.normalized_json().unwrap()
        );
        assert!(first.report.render_headline().unwrap().contains("1/1"));
        assert!(first.report.render_items().unwrap().contains("f"));
        assert!(first
            .report
            .render_items()
            .unwrap()
            .contains("crate groups (count/project-total)"));
        assert!(first
            .report
            .render_explain("f")
            .unwrap()
            .contains("formal positions and boundaries"));
        assert!(first
            .report
            .render_explain("project")
            .unwrap()
            .contains("normalization witnesses"));
        let json = first.report.normalized_json().unwrap();
        assert!(json.contains("common_claim_frontier"));
        let headline = first.report.render_headline().unwrap();
        assert!(headline.contains("Boundary: end to end"));
        assert!(!headline.contains("L4"));
    }

    #[test]
    fn source_population_includes_certified_effect_and_protocol_declarations() {
        let source = "effect platform(d) = state(d) + io(sigma_d)\n\
                      protocol P { A { request: u32 }, B { response: u32 }, end }";
        let program = parsed(source);
        let live = build_live_report(
            &[
                cert(
                    "platform",
                    AssuranceKindV2::SolverIncomplete,
                    CertificationBoundary::EndToEnd,
                ),
                cert(
                    "P",
                    AssuranceKindV2::SolverIncomplete,
                    CertificationBoundary::EndToEnd,
                ),
            ],
            &program,
            "src/declarations.th",
            source,
            &"d".repeat(40),
            ReportTrust::UntrustedPullRequest,
            Toolchain::new("verus-test"),
        )
        .unwrap();

        assert_eq!(live.report().body.project.population.total, 2);
        assert_eq!(live.report().body.project.population.covered, 2);
        assert_eq!(
            live.report()
                .body
                .items
                .iter()
                .map(|item| item.identity.item_path.as_str())
                .collect::<Vec<_>>(),
            ["platform", "P"]
        );
    }

    #[test]
    fn population_and_presentation_tampering_fail_validation() {
        let mut tampered = report(AssuranceKindV2::SolverIncomplete, 'a').into_report();
        tampered.body.project.population.total += 1;
        assert!(tampered
            .validate()
            .unwrap_err()
            .reason
            .contains("population counts"));

        let mut report = report(AssuranceKindV2::SolverIncomplete, 'a').into_report();
        report.body.items[0].engineer_statements.clear();
        report.report_sha256 = digest(b"thermite-assurance-report-v1\0", &report.body);
        assert!(report
            .validate()
            .unwrap_err()
            .reason
            .contains("presentation digest"));
    }

    #[test]
    fn html_escapes_source_controlled_content_and_carries_restrictive_csp() {
        let report = report_at_path(
            AssuranceKindV2::SolverIncomplete,
            'a',
            "<script>alert(1)</script>.th",
        )
        .into_report();
        let html = report.render_html().unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("default-src 'none'"));
        assert!(html.contains("UNTRUSTED PR DIAGNOSTIC"));
        assert!(html.contains("Common project claims"));
        assert!(html.contains("Complete formal JSON portrait"));
    }

    #[test]
    fn exact_base_comparison_separates_formal_movements() {
        let base = report(AssuranceKindV2::SolverIncomplete, 'a').into_report();
        let head = report(AssuranceKindV2::SolverComplete, 'b').into_report();
        let comparison = compare_reports(&base, &head, &"a".repeat(40));
        assert_eq!(comparison.status, ComparisonStatus::Compared);
        assert_eq!(
            comparison.items[0].movements,
            vec![FormalMovement::Strengthened]
        );
        let absent = compare_reports(&base, &head, &"c".repeat(40));
        assert!(matches!(
            absent.status,
            ComparisonStatus::NoComparison { .. }
        ));

        let incomparable = report(AssuranceKindV2::LeanEmpirical, 'b').into_report();
        let comparison = compare_reports(&head, &incomparable, &"b".repeat(40));
        assert_eq!(
            comparison.items[0].movements,
            vec![
                FormalMovement::Incomparable,
                FormalMovement::ChangedResidualContext,
            ]
        );

        let mut schema_skew = incomparable.clone();
        schema_skew.schema = "thermite-assurance-report/v2".into();
        assert!(matches!(
            compare_reports(&head, &schema_skew, &"b".repeat(40)).status,
            ComparisonStatus::SchemaSkew { .. }
        ));
        let mut policy_skew = incomparable;
        policy_skew.body.collapse_policy_version += 1;
        assert!(matches!(
            compare_reports(&head, &policy_skew, &"b".repeat(40)).status,
            ComparisonStatus::PolicySkew { .. }
        ));
    }

    #[test]
    fn comparison_vocabulary_keeps_formal_changes_separate() {
        let current = report(AssuranceKindV2::SolverIncomplete, 'a').into_report();
        let base = current.body.items[0].clone();

        let stronger = report(AssuranceKindV2::SolverComplete, 'b')
            .into_report()
            .body
            .items
            .remove(0);
        assert_eq!(
            compare_items(&base, &stronger),
            vec![FormalMovement::Strengthened]
        );
        assert_eq!(
            compare_items(&stronger, &base),
            vec![FormalMovement::Weakened]
        );

        let mut boundary = base.clone();
        boundary.formal_positions[0].boundary = CertificationBoundary::ToBoundary {
            via: "ffi-contract-v1".into(),
        };
        assert_eq!(
            compare_items(&base, &boundary),
            vec![FormalMovement::Equal, FormalMovement::ChangedBoundary]
        );

        let mut residual = base.clone();
        residual.residual_trust.push("procedure:other".into());
        assert_eq!(
            compare_items(&base, &residual),
            vec![
                FormalMovement::Equal,
                FormalMovement::ChangedResidualContext
            ]
        );

        let mut different_fiber = base.clone();
        different_fiber.claim_fibers = vec!["0".repeat(64)];
        assert_eq!(
            compare_items(&base, &different_fiber),
            vec![FormalMovement::ChangedFiber]
        );

        let mut nonclaim = base.clone();
        nonclaim.disposition = ItemDisposition::NonClaim {
            class: "refuted".into(),
            detail: "counterexample".into(),
        };
        assert_eq!(
            compare_items(&base, &nonclaim),
            vec![FormalMovement::NewNonClaim]
        );
        assert_eq!(
            compare_items(&nonclaim, &base),
            vec![FormalMovement::BecameCurrent]
        );

        let mut historical = base.clone();
        historical.disposition = ItemDisposition::Historical {
            legacy_level: "L3".into(),
            warning: "historical; re-certification required".into(),
        };
        assert_eq!(
            compare_items(&base, &historical),
            vec![FormalMovement::NewlyHistorical]
        );

        let added_removed = compare_reports(
            &report_at_path(AssuranceKindV2::SolverIncomplete, 'a', "src/base.th").into_report(),
            &report_at_path(AssuranceKindV2::SolverIncomplete, 'b', "src/head.th").into_report(),
            &"a".repeat(40),
        );
        assert_eq!(added_removed.items.len(), 2);
        assert!(added_removed
            .items
            .iter()
            .any(|item| { item.movements == [FormalMovement::PopulationAdded] }));
        assert!(added_removed
            .items
            .iter()
            .any(|item| { item.movements == [FormalMovement::PopulationRemoved] }));
    }

    #[test]
    fn historical_and_policy_rejected_portraits_remain_loud_nonclaims() {
        let mut historical = report(AssuranceKindV2::SolverComplete, 'a').into_report();
        historical.body.project.population.members[0].disposition =
            ProjectDispositionV2::LegacyUnversioned {
                legacy_level: "L3".into(),
            };
        let item = &mut historical.body.items[0];
        item.disposition = ItemDisposition::Historical {
            legacy_level: "L3".into(),
            warning: "HISTORICAL CERTIFICATE — NOT VALID FOR CURRENT ASSURANCE DECISIONS; re-certification required".into(),
        };
        item.authority_sha256 = None;
        item.presentation_sha256 = None;
        item.engineer_statements.clear();
        item.policy_points.clear();
        item.claim_fibers.clear();
        item.formal_positions.clear();
        item.clauses.clear();
        item.constraints.clear();
        item.residual_trust.clear();
        rebind_project_portrait(&mut historical);
        historical.validate().unwrap();
        let rendered = historical.render_items().unwrap();
        assert!(rendered.contains("0/1"));
        assert!(rendered.contains("HISTORICAL CERTIFICATE"));
        assert!(rendered.contains("Exceptions: f (historical)"));

        let mut rejected = historical;
        rejected.body.project.population.members[0].disposition = ProjectDispositionV2::NonClaim {
            reason: "policy_rejected: declared floor".into(),
        };
        rejected.body.items[0].disposition = ItemDisposition::NonClaim {
            class: "policy_rejected".into(),
            detail: "declared floor".into(),
        };
        rebind_project_portrait(&mut rejected);
        rejected.validate().unwrap();
        assert!(rejected
            .render_headline()
            .unwrap()
            .contains("Exceptions: f (policy_rejected)"));
    }

    #[test]
    fn diagnostic_json_is_bounded_and_rejects_unknown_report_fields() {
        let report = report(AssuranceKindV2::SolverIncomplete, 'a').into_report();
        let mut value = serde_json::to_value(&report).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("future_authority".into(), serde_json::json!(true));
        assert!(parse_report_json(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .reason
            .contains("unknown field"));

        let oversized = vec![b' '; MAX_REPORT_BYTES + 1];
        assert!(parse_report_json(&oversized)
            .unwrap_err()
            .reason
            .contains("byte limit"));

        let mut nested = serde_json::json!(null);
        for _ in 0..=MAX_JSON_DEPTH {
            nested = serde_json::json!([nested]);
        }
        assert!(parse_report_json(&serde_json::to_vec(&nested).unwrap())
            .unwrap_err()
            .reason
            .contains("nesting depth"));
    }

    #[test]
    fn floors_consume_only_live_common_frontier_and_exact_fiber() {
        let live = report(AssuranceKindV2::SolverComplete, 'a');
        let fiber = live.report.body.items[0].claim_fibers[0].clone();
        let policy = FormalFloorPolicy {
            schema: FLOOR_SCHEMA.into(),
            allow_without_project_lift: false,
            floors: vec![FormalFloor {
                fiber_sha256: fiber,
                minimum: AssuranceKindV2::SolverIncomplete,
            }],
        };
        assert!(live.evaluate_floor(&policy).passed);
        let wrong = FormalFloorPolicy {
            floors: vec![FormalFloor {
                fiber_sha256: "0".repeat(64),
                minimum: AssuranceKindV2::Runtime,
            }],
            ..policy
        };
        assert!(!live.evaluate_floor(&wrong).passed);
    }
}
