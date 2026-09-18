//! Exact workspace/build-matrix assurance composition.
//!
//! A workspace plan is an independent denominator: every package/target/
//! feature/platform/generated-source matrix cell and every intended item is
//! named before any project report is admitted. Missing cells become explicit
//! non-claims. Only live project-report capabilities can construct a live
//! workspace lift; serialized reports remain diagnostic inputs to comparison.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::assurance_report::{
    compare_reports, AssuranceReport, ComparisonStatus, LiveAssuranceReport, ReportTrust,
};
use crate::assurance_v2::{ProjectBuildCoordinateV2, ProjectDispositionV2, ProjectItemIdentityV2};

pub const WORKSPACE_PLAN_SCHEMA: &str = "thermite-workspace-assurance-plan/v1";
pub const WORKSPACE_REPORT_SCHEMA: &str = "thermite-workspace-assurance-report/v1";
pub const MAX_WORKSPACE_JSON_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_WORKSPACE_JSON_DEPTH: usize = 96;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceReportError {
    pub reason: String,
}

impl WorkspaceReportError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for WorkspaceReportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid workspace assurance report: {}",
            self.reason
        )
    }
}

impl std::error::Error for WorkspaceReportError {}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_sha256_prefix(value)
}

fn is_sha256_prefix(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digest(domain: &[u8], value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("closed workspace value serializes");
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceMatrixPlanV1 {
    pub coordinate: ProjectBuildCoordinateV2,
    pub source_path: String,
    /// Independent item inventory in source order for this exact matrix cell.
    pub intended: Vec<ProjectItemIdentityV2>,
}

impl WorkspaceMatrixPlanV1 {
    pub fn new(
        coordinate: ProjectBuildCoordinateV2,
        source_path: String,
        intended: Vec<ProjectItemIdentityV2>,
    ) -> Result<Self, WorkspaceReportError> {
        coordinate
            .validate()
            .map_err(|error| WorkspaceReportError::new(error.to_string()))?;
        if source_path.is_empty() {
            return Err(WorkspaceReportError::new(
                "workspace matrix source path must not be empty",
            ));
        }
        let mut seen = BTreeSet::new();
        for item in &intended {
            if item.source_path.is_empty() || item.item_path.is_empty() {
                return Err(WorkspaceReportError::new(
                    "workspace intended item identities must not be empty",
                ));
            }
            if !seen.insert(item.clone()) {
                return Err(WorkspaceReportError::new(format!(
                    "duplicate intended workspace item {}::{} in {}:{}",
                    item.source_path, item.item_path, coordinate.crate_name, coordinate.target
                )));
            }
            if item.source_path != source_path {
                return Err(WorkspaceReportError::new(
                    "workspace matrix intended items must belong to its exact source path",
                ));
            }
        }
        Ok(Self {
            coordinate,
            source_path,
            intended,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspacePlanV1 {
    pub schema: String,
    pub workspace_sha256: String,
    /// Canonical coordinate order; constructor input order is irrelevant.
    pub matrices: Vec<WorkspaceMatrixPlanV1>,
}

impl WorkspacePlanV1 {
    pub fn new(
        workspace_sha256: String,
        mut matrices: Vec<WorkspaceMatrixPlanV1>,
    ) -> Result<Self, WorkspaceReportError> {
        if !is_sha256(&workspace_sha256) {
            return Err(WorkspaceReportError::new(
                "workspace identity must be a lowercase SHA-256 digest",
            ));
        }
        for matrix in &matrices {
            WorkspaceMatrixPlanV1::new(
                matrix.coordinate.clone(),
                matrix.source_path.clone(),
                matrix.intended.clone(),
            )?;
        }
        matrices.sort_by(|left, right| left.coordinate.cmp(&right.coordinate));
        if matrices
            .windows(2)
            .any(|pair| pair[0].coordinate == pair[1].coordinate)
        {
            return Err(WorkspaceReportError::new(
                "workspace matrix coordinates must be duplicate-free",
            ));
        }
        Ok(Self {
            schema: WORKSPACE_PLAN_SCHEMA.into(),
            workspace_sha256,
            matrices,
        })
    }

    fn validate(&self) -> Result<(), WorkspaceReportError> {
        if self.schema != WORKSPACE_PLAN_SCHEMA {
            return Err(WorkspaceReportError::new(format!(
                "workspace plan schema skew: expected {WORKSPACE_PLAN_SCHEMA}, found {}",
                self.schema
            )));
        }
        let canonical = Self::new(self.workspace_sha256.clone(), self.matrices.clone())?;
        if canonical.matrices != self.matrices {
            return Err(WorkspaceReportError::new(
                "workspace matrix plan is not in canonical coordinate order",
            ));
        }
        Ok(())
    }

    pub fn identity_digest(&self) -> String {
        digest(b"thermite-workspace-plan-v1\0", self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceItemIdentityV1 {
    pub matrix: ProjectBuildCoordinateV2,
    pub source_path: String,
    pub item_path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspacePopulationMemberV1 {
    pub identity: WorkspaceItemIdentityV1,
    pub disposition: ProjectDispositionV2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspacePopulationV1 {
    pub plan_sha256: String,
    pub population_sha256: String,
    pub intended: Vec<WorkspaceItemIdentityV1>,
    pub members: Vec<WorkspacePopulationMemberV1>,
    pub covered: usize,
    pub total: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceMatrixPortraitV1 {
    pub plan: WorkspaceMatrixPlanV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_report: Option<AssuranceReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceProjectLiftBindingV1 {
    pub coordinate: ProjectBuildCoordinateV2,
    pub project_population_sha256: String,
    pub project_lift_sha256: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceLiftV1 {
    pub population_sha256: String,
    pub witness: String,
    pub projects: Vec<WorkspaceProjectLiftBindingV1>,
    pub workspace_lift_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", deny_unknown_fields)]
pub enum WorkspacePortraitScopeV1 {
    NoItems,
    AcceptedSubset {
        numerator: usize,
        denominator: usize,
        excluded: Vec<WorkspaceItemIdentityV1>,
        unavailable_matrices: Vec<ProjectBuildCoordinateV2>,
    },
    WholeWorkspace {
        workspace_lift_sha256: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceAssuranceReportBodyV1 {
    pub revision: String,
    pub trust: ReportTrust,
    pub collapse_policy_version: u64,
    pub plan: WorkspacePlanV1,
    pub population: WorkspacePopulationV1,
    pub matrices: Vec<WorkspaceMatrixPortraitV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_lift: Option<WorkspaceLiftV1>,
    pub scope: WorkspacePortraitScopeV1,
    pub whole_workspace_claim: bool,
    pub headline: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceAssuranceReportV1 {
    pub schema: String,
    pub report_sha256: String,
    pub body: WorkspaceAssuranceReportBodyV1,
}

#[derive(Debug)]
pub struct LiveWorkspaceAssuranceReportV1 {
    report: WorkspaceAssuranceReportV1,
}

impl LiveWorkspaceAssuranceReportV1 {
    pub fn report(&self) -> &WorkspaceAssuranceReportV1 {
        &self.report
    }

    #[allow(dead_code)]
    pub fn into_report(self) -> WorkspaceAssuranceReportV1 {
        self.report
    }

    /// Evaluate one exact formal floor across every project in this live
    /// workspace capability. Persisted workspace JSON cannot call this path.
    pub(crate) fn evaluate_exact_floor(
        &self,
        fiber_sha256: &str,
        minimum: crate::assurance_v2::AssuranceKindV2,
    ) -> (bool, String) {
        use crate::assurance_v2::{
            assurance_kind_leq, CommonClaimFrontierV2, ProjectPortraitScopeV2,
        };

        if !self.report.body.whole_workspace_claim || self.report.body.workspace_lift.is_none() {
            return (
                false,
                "exact workspace population lacks a checked WorkspaceLift".into(),
            );
        }
        if self.report.body.matrices.is_empty() {
            return (false, "exact workspace population is empty".into());
        }
        for matrix in &self.report.body.matrices {
            let Some(project) = matrix.project_report.as_ref() else {
                return (
                    false,
                    "an exact build-matrix cell lacks a live project report".into(),
                );
            };
            let Some(frontier) = project
                .body
                .project
                .frontiers
                .iter()
                .find(|frontier| frontier.claim_fiber().as_str() == fiber_sha256)
            else {
                return (
                    false,
                    format!(
                        "exact claim fiber is absent from {}:{}",
                        matrix.plan.coordinate.crate_name, matrix.plan.coordinate.target
                    ),
                );
            };
            if !matches!(
                frontier.scope(),
                ProjectPortraitScopeV2::WholeProject { .. }
            ) {
                return (
                    false,
                    format!(
                        "exact claim fiber lacks a checked ProjectLift in {}:{}",
                        matrix.plan.coordinate.crate_name, matrix.plan.coordinate.target
                    ),
                );
            }
            let dominates = match frontier.common_claim_frontier() {
                CommonClaimFrontierV2::NoItems => false,
                CommonClaimFrontierV2::Frontier(actual) => actual
                    .iter()
                    .any(|actual| assurance_kind_leq(minimum, *actual)),
            };
            if !dominates {
                return (
                    false,
                    format!(
                        "live common-claim frontier does not dominate the floor in {}:{}",
                        matrix.plan.coordinate.crate_name, matrix.plan.coordinate.target
                    ),
                );
            }
        }
        (
            true,
            "every exact build-matrix cell has a checked ProjectLift whose live common-claim frontier dominates the floor".into(),
        )
    }

    pub(crate) fn plan_sha256(&self) -> String {
        self.report.body.plan.identity_digest()
    }

    pub(crate) fn collapse_policy_version(&self) -> u64 {
        self.report.body.collapse_policy_version
    }

    #[cfg(test)]
    pub(crate) fn set_collapse_policy_version_for_test(&mut self, version: u64) {
        self.report.body.collapse_policy_version = version;
    }

    pub(crate) fn revision(&self) -> &str {
        &self.report.body.revision
    }

    pub(crate) fn trust(&self) -> &ReportTrust {
        &self.report.body.trust
    }
}

fn workspace_item(
    matrix: &ProjectBuildCoordinateV2,
    item: &ProjectItemIdentityV2,
) -> WorkspaceItemIdentityV1 {
    WorkspaceItemIdentityV1 {
        matrix: matrix.clone(),
        source_path: item.source_path.clone(),
        item_path: item.item_path.clone(),
    }
}

fn population_digest(
    plan_sha256: &str,
    matrices: &[WorkspaceMatrixPortraitV1],
    intended: &[WorkspaceItemIdentityV1],
    members: &[WorkspacePopulationMemberV1],
) -> String {
    let project_populations = matrices
        .iter()
        .map(|matrix| {
            matrix
                .project_report
                .as_ref()
                .map(|report| report.body.project.population.population_sha256.clone())
        })
        .collect::<Vec<_>>();
    digest(
        b"thermite-workspace-population-v1\0",
        &(plan_sha256, project_populations, intended, members),
    )
}

fn workspace_lift(
    population_sha256: &str,
    matrices: &[WorkspaceMatrixPortraitV1],
) -> WorkspaceLiftV1 {
    let projects = matrices
        .iter()
        .map(|matrix| {
            let report = matrix
                .project_report
                .as_ref()
                .expect("whole workspace matrices have project reports");
            WorkspaceProjectLiftBindingV1 {
                coordinate: matrix.plan.coordinate.clone(),
                project_population_sha256: report.body.project.population.population_sha256.clone(),
                project_lift_sha256: report
                    .body
                    .project
                    .project_lifts
                    .iter()
                    .map(|lift| lift.identity_digest())
                    .collect(),
            }
        })
        .collect::<Vec<_>>();
    let witness = "Thermite.CertificationMetatheory.WorkspaceLift.certifiesWorkspace".to_string();
    let workspace_lift_sha256 = digest(
        b"thermite-workspace-lift-v1\0",
        &(population_sha256, &witness, &projects),
    );
    WorkspaceLiftV1 {
        population_sha256: population_sha256.into(),
        witness,
        projects,
        workspace_lift_sha256,
    }
}

fn derive_body(
    plan: WorkspacePlanV1,
    reports: Vec<Option<AssuranceReport>>,
    revision: String,
    trust: ReportTrust,
) -> Result<WorkspaceAssuranceReportBodyV1, WorkspaceReportError> {
    plan.validate()?;
    if !is_revision(&revision) {
        return Err(WorkspaceReportError::new(
            "workspace revision must be an exact lowercase 40- or 64-hex identity",
        ));
    }
    if reports.len() != plan.matrices.len() {
        return Err(WorkspaceReportError::new(
            "workspace project-report slots must equal the exact matrix plan",
        ));
    }
    let mut matrices = Vec::with_capacity(plan.matrices.len());
    let mut intended = Vec::new();
    let mut members = Vec::new();
    let mut policy_version = None;
    for (matrix_plan, report) in plan.matrices.iter().cloned().zip(reports) {
        for item in &matrix_plan.intended {
            intended.push(workspace_item(&matrix_plan.coordinate, item));
        }
        match report {
            Some(report) => {
                report.validate().map_err(|error| {
                    WorkspaceReportError::new(format!(
                        "project report for {}:{} failed validation: {}",
                        matrix_plan.coordinate.crate_name,
                        matrix_plan.coordinate.target,
                        error.reason
                    ))
                })?;
                if report.body.source.revision != revision || report.body.source.trust != trust {
                    return Err(WorkspaceReportError::new(
                        "workspace project reports must share the exact revision and trust class",
                    ));
                }
                if report.body.project.population.build.coordinate() != matrix_plan.coordinate {
                    return Err(WorkspaceReportError::new(
                        "project report belongs to a different package/target/features/platform/generated-source cell",
                    ));
                }
                if report.body.source.source_path != matrix_plan.source_path {
                    return Err(WorkspaceReportError::new(
                        "project report source path does not equal the independent matrix plan",
                    ));
                }
                if report.body.project.population.intended != matrix_plan.intended {
                    return Err(WorkspaceReportError::new(
                        "project report item population does not equal the independent matrix plan",
                    ));
                }
                if policy_version
                    .replace(report.body.collapse_policy_version)
                    .is_some_and(|seen| seen != report.body.collapse_policy_version)
                {
                    return Err(WorkspaceReportError::new(
                        "workspace project reports have policy-version skew",
                    ));
                }
                members.extend(report.body.project.population.members.iter().map(|member| {
                    WorkspacePopulationMemberV1 {
                        identity: workspace_item(&matrix_plan.coordinate, &member.identity),
                        disposition: member.disposition.clone(),
                    }
                }));
                matrices.push(WorkspaceMatrixPortraitV1 {
                    plan: matrix_plan,
                    project_report: Some(report),
                    unavailable_reason: None,
                });
            }
            None => {
                members.extend(matrix_plan.intended.iter().map(|item| {
                    WorkspacePopulationMemberV1 {
                        identity: workspace_item(&matrix_plan.coordinate, item),
                        disposition: ProjectDispositionV2::NonClaim {
                            reason: "missing exact build-matrix report".into(),
                        },
                    }
                }));
                matrices.push(WorkspaceMatrixPortraitV1 {
                    plan: matrix_plan,
                    project_report: None,
                    unavailable_reason: Some("missing exact build-matrix report".into()),
                });
            }
        }
    }
    if intended.len() != members.len()
        || intended
            .iter()
            .zip(&members)
            .any(|(expected, member)| expected != &member.identity)
    {
        return Err(WorkspaceReportError::new(
            "workspace population is not an exact source-ordered total partition",
        ));
    }
    let mut unique = BTreeSet::new();
    if intended.iter().any(|item| !unique.insert(item.clone())) {
        return Err(WorkspaceReportError::new(
            "workspace population contains a duplicate matrix-qualified item",
        ));
    }
    let plan_sha256 = plan.identity_digest();
    let population_sha256 = population_digest(&plan_sha256, &matrices, &intended, &members);
    let covered = members
        .iter()
        .filter(|member| matches!(member.disposition, ProjectDispositionV2::Accepted))
        .count();
    let total = members.len();
    let all_projects_lifted = !matrices.is_empty()
        && matrices.iter().all(|matrix| {
            matrix.project_report.as_ref().is_some_and(|report| {
                report.body.project.whole_project_claim
                    && !report.body.project.project_lifts.is_empty()
                    && report.body.project.project_lifts.len()
                        == report.body.project.frontiers.len()
            })
        });
    let whole_workspace_claim = total > 0 && covered == total && all_projects_lifted;
    let workspace_lift =
        whole_workspace_claim.then(|| workspace_lift(&population_sha256, &matrices));
    let unavailable_matrices = matrices
        .iter()
        .filter(|matrix| matrix.project_report.is_none())
        .map(|matrix| matrix.plan.coordinate.clone())
        .collect::<Vec<_>>();
    let excluded = members
        .iter()
        .filter(|member| !matches!(member.disposition, ProjectDispositionV2::Accepted))
        .map(|member| member.identity.clone())
        .collect::<Vec<_>>();
    let scope = if total == 0 {
        WorkspacePortraitScopeV1::NoItems
    } else if let Some(lift) = &workspace_lift {
        WorkspacePortraitScopeV1::WholeWorkspace {
            workspace_lift_sha256: lift.workspace_lift_sha256.clone(),
        }
    } else {
        WorkspacePortraitScopeV1::AcceptedSubset {
            numerator: covered,
            denominator: total,
            excluded,
            unavailable_matrices,
        }
    };
    let headline = match &scope {
        WorkspacePortraitScopeV1::NoItems => format!(
            "No claim-bearing items are in the exact workspace build matrix (0/0 across {} matrix cell(s)); no workspace claim.",
            matrices.len()
        ),
        WorkspacePortraitScopeV1::WholeWorkspace { .. } => format!(
            "Whole-workspace formal conjunction covers {covered}/{total} items across {} exact package/target/feature/platform/generated-source matrix cell(s).",
            matrices.len()
        ),
        WorkspacePortraitScopeV1::AcceptedSubset {
            unavailable_matrices,
            ..
        } => format!(
            "Accepted subset portrait, not a whole-workspace claim: {covered}/{total} items across {} exact matrix cell(s); {} matrix cell(s) unavailable.",
            matrices.len(),
            unavailable_matrices.len()
        ),
    };
    Ok(WorkspaceAssuranceReportBodyV1 {
        revision,
        trust,
        collapse_policy_version: policy_version
            .unwrap_or(crate::assurance_report::COLLAPSE_POLICY_VERSION),
        plan,
        population: WorkspacePopulationV1 {
            plan_sha256,
            population_sha256,
            intended,
            members,
            covered,
            total,
        },
        matrices,
        workspace_lift,
        scope,
        whole_workspace_claim,
        headline,
    })
}

pub fn build_live_workspace_report(
    plan: WorkspacePlanV1,
    reports: &[&LiveAssuranceReport],
    revision: &str,
    trust: ReportTrust,
) -> Result<LiveWorkspaceAssuranceReportV1, WorkspaceReportError> {
    plan.validate()?;
    let mut by_coordinate = BTreeMap::new();
    for live in reports {
        let report = live.report();
        let coordinate = report.body.project.population.build.coordinate();
        if by_coordinate.insert(coordinate, report.clone()).is_some() {
            return Err(WorkspaceReportError::new(
                "duplicate live report for one workspace matrix coordinate",
            ));
        }
    }
    let planned = plan
        .matrices
        .iter()
        .map(|matrix| matrix.coordinate.clone())
        .collect::<BTreeSet<_>>();
    if let Some(extra) = by_coordinate
        .keys()
        .find(|coordinate| !planned.contains(*coordinate))
    {
        return Err(WorkspaceReportError::new(format!(
            "unplanned live matrix report {}:{}",
            extra.crate_name, extra.target
        )));
    }
    let report_slots = plan
        .matrices
        .iter()
        .map(|matrix| by_coordinate.remove(&matrix.coordinate))
        .collect();
    let body = derive_body(plan, report_slots, revision.into(), trust)?;
    let report_sha256 = digest(b"thermite-workspace-report-v1\0", &body);
    let report = WorkspaceAssuranceReportV1 {
        schema: WORKSPACE_REPORT_SCHEMA.into(),
        report_sha256,
        body,
    };
    report.validate()?;
    Ok(LiveWorkspaceAssuranceReportV1 { report })
}

impl WorkspaceAssuranceReportV1 {
    pub fn validate(&self) -> Result<(), WorkspaceReportError> {
        if self.schema != WORKSPACE_REPORT_SCHEMA {
            return Err(WorkspaceReportError::new(format!(
                "workspace report schema skew: expected {WORKSPACE_REPORT_SCHEMA}, found {}",
                self.schema
            )));
        }
        let slots = self
            .body
            .matrices
            .iter()
            .map(|matrix| matrix.project_report.clone())
            .collect();
        let derived = derive_body(
            self.body.plan.clone(),
            slots,
            self.body.revision.clone(),
            self.body.trust.clone(),
        )?;
        if derived != self.body {
            return Err(WorkspaceReportError::new(
                "workspace report body does not match its exact plan/project inputs",
            ));
        }
        let expected = digest(b"thermite-workspace-report-v1\0", &self.body);
        if self.report_sha256 != expected {
            return Err(WorkspaceReportError::new(
                "workspace report digest does not match its canonical body",
            ));
        }
        Ok(())
    }

    pub fn normalized_json(&self) -> Result<String, WorkspaceReportError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map(|json| format!("{json}\n"))
            .map_err(|error| WorkspaceReportError::new(format!("serialization failed: {error}")))
    }

    pub fn render_headline(&self) -> Result<String, WorkspaceReportError> {
        self.validate()?;
        Ok(format!("{}\n", self.body.headline))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", deny_unknown_fields)]
pub enum WorkspaceComparisonStatusV1 {
    Compared,
    NoComparison { reason: String },
    SchemaSkew { base: String, head: String },
    PolicySkew { base: u64, head: u64 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceMatrixMovementV1 {
    Added,
    Removed,
    IntendedPopulationChanged,
    ReportBecameUnavailable,
    ReportBecameAvailable,
    ProjectFormalMovement,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceMatrixComparisonV1 {
    pub coordinate: ProjectBuildCoordinateV2,
    pub movements: Vec<WorkspaceMatrixMovementV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceReportComparisonV1 {
    pub status: WorkspaceComparisonStatusV1,
    pub base_revision: String,
    pub head_revision: String,
    pub build_matrix_changed: bool,
    pub denominator_changed: bool,
    pub whole_workspace_claim_changed: bool,
    pub matrices: Vec<WorkspaceMatrixComparisonV1>,
}

pub fn compare_workspace_reports(
    base: &WorkspaceAssuranceReportV1,
    head: &WorkspaceAssuranceReportV1,
    expected_base_revision: &str,
) -> WorkspaceReportComparisonV1 {
    let empty = |status| WorkspaceReportComparisonV1 {
        status,
        base_revision: base.body.revision.clone(),
        head_revision: head.body.revision.clone(),
        build_matrix_changed: false,
        denominator_changed: false,
        whole_workspace_claim_changed: false,
        matrices: Vec::new(),
    };
    if base.schema != head.schema {
        return empty(WorkspaceComparisonStatusV1::SchemaSkew {
            base: base.schema.clone(),
            head: head.schema.clone(),
        });
    }
    if base.body.collapse_policy_version != head.body.collapse_policy_version {
        return empty(WorkspaceComparisonStatusV1::PolicySkew {
            base: base.body.collapse_policy_version,
            head: head.body.collapse_policy_version,
        });
    }
    if let Err(error) = base.validate() {
        return empty(WorkspaceComparisonStatusV1::NoComparison {
            reason: format!("base workspace report failed validation: {}", error.reason),
        });
    }
    if let Err(error) = head.validate() {
        return empty(WorkspaceComparisonStatusV1::NoComparison {
            reason: format!("head workspace report failed validation: {}", error.reason),
        });
    }
    if base.body.revision != expected_base_revision {
        return empty(WorkspaceComparisonStatusV1::NoComparison {
            reason: format!(
                "base workspace report is bound to {}, not exact base {}",
                base.body.revision, expected_base_revision
            ),
        });
    }
    let base_matrices = base
        .body
        .matrices
        .iter()
        .map(|matrix| (matrix.plan.coordinate.clone(), matrix))
        .collect::<BTreeMap<_, _>>();
    let head_matrices = head
        .body
        .matrices
        .iter()
        .map(|matrix| (matrix.plan.coordinate.clone(), matrix))
        .collect::<BTreeMap<_, _>>();
    let coordinates = base_matrices
        .keys()
        .chain(head_matrices.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut matrices = Vec::new();
    for coordinate in coordinates {
        let mut movements = Vec::new();
        match (
            base_matrices.get(&coordinate),
            head_matrices.get(&coordinate),
        ) {
            (None, Some(_)) => movements.push(WorkspaceMatrixMovementV1::Added),
            (Some(_), None) => movements.push(WorkspaceMatrixMovementV1::Removed),
            (Some(base_matrix), Some(head_matrix)) => {
                if base_matrix.plan.source_path != head_matrix.plan.source_path
                    || base_matrix.plan.intended != head_matrix.plan.intended
                {
                    movements.push(WorkspaceMatrixMovementV1::IntendedPopulationChanged);
                }
                match (&base_matrix.project_report, &head_matrix.project_report) {
                    (Some(_), None) => {
                        movements.push(WorkspaceMatrixMovementV1::ReportBecameUnavailable)
                    }
                    (None, Some(_)) => {
                        movements.push(WorkspaceMatrixMovementV1::ReportBecameAvailable)
                    }
                    (Some(base_project), Some(head_project)) => {
                        let comparison =
                            compare_reports(base_project, head_project, expected_base_revision);
                        if !matches!(comparison.status, ComparisonStatus::Compared)
                            || comparison.common_frontier_changed
                            || !comparison.items.is_empty()
                            || base_project.body.project.whole_project_claim
                                != head_project.body.project.whole_project_claim
                        {
                            movements.push(WorkspaceMatrixMovementV1::ProjectFormalMovement);
                        }
                    }
                    (None, None) => {}
                }
            }
            (None, None) => unreachable!(),
        }
        movements.sort();
        movements.dedup();
        if !movements.is_empty() {
            matrices.push(WorkspaceMatrixComparisonV1 {
                coordinate,
                movements,
            });
        }
    }
    WorkspaceReportComparisonV1 {
        status: WorkspaceComparisonStatusV1::Compared,
        base_revision: base.body.revision.clone(),
        head_revision: head.body.revision.clone(),
        build_matrix_changed: base.body.plan.matrices != head.body.plan.matrices,
        denominator_changed: base.body.population.total != head.body.population.total,
        whole_workspace_claim_changed: base.body.whole_workspace_claim
            != head.body.whole_workspace_claim,
        matrices,
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

fn bounded_json(bytes: &[u8]) -> Result<serde_json::Value, WorkspaceReportError> {
    if bytes.len() > MAX_WORKSPACE_JSON_BYTES {
        return Err(WorkspaceReportError::new(format!(
            "workspace JSON exceeds the {MAX_WORKSPACE_JSON_BYTES}-byte limit"
        )));
    }
    let value = serde_json::from_slice(bytes)
        .map_err(|error| WorkspaceReportError::new(format!("invalid JSON: {error}")))?;
    if json_depth(&value) > MAX_WORKSPACE_JSON_DEPTH {
        return Err(WorkspaceReportError::new(format!(
            "workspace JSON exceeds maximum depth {MAX_WORKSPACE_JSON_DEPTH}"
        )));
    }
    Ok(value)
}

pub fn parse_workspace_plan_json(bytes: &[u8]) -> Result<WorkspacePlanV1, WorkspaceReportError> {
    let raw: WorkspacePlanV1 = serde_json::from_value(bounded_json(bytes)?)
        .map_err(|error| WorkspaceReportError::new(format!("invalid plan schema: {error}")))?;
    let plan = WorkspacePlanV1::new(raw.workspace_sha256, raw.matrices)?;
    if raw.schema != WORKSPACE_PLAN_SCHEMA {
        return Err(WorkspaceReportError::new(format!(
            "workspace plan schema skew: expected {WORKSPACE_PLAN_SCHEMA}, found {}",
            raw.schema
        )));
    }
    Ok(plan)
}

pub fn parse_workspace_report_json(
    bytes: &[u8],
) -> Result<WorkspaceAssuranceReportV1, WorkspaceReportError> {
    serde_json::from_value(bounded_json(bytes)?)
        .map_err(|error| WorkspaceReportError::new(format!("invalid report schema: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assurance_report::{build_live_report_for_coordinate, ReportTrust};
    use crate::assurance_v2::AssuranceKindV2;
    use crate::audit::Toolchain;
    use crate::manifest::{
        Certificate, CertificationBoundary, CertificationPosition, CertificationScope,
        ClassificationCertificate, ClassificationVerdict, Level, RefutationChannel, ResidualTrust,
    };

    fn parsed(source: &str) -> thermite_syntax::Program {
        let parsed = thermite_syntax::parse(source);
        assert!(parsed.is_clean(), "{:?}", parsed.errors);
        parsed.program
    }

    fn coordinate(package: &str, target: &str) -> ProjectBuildCoordinateV2 {
        ProjectBuildCoordinateV2 {
            crate_name: package.into(),
            target: target.into(),
            features: vec!["default".into()],
            platform: "x86_64-unknown-linux-gnu".into(),
            generated_sources: vec!["generated/schema.rs".into()],
        }
    }

    fn cert(name: &str) -> Certificate {
        Certificate::new(name, Level::L4, vec!["pure".into()], 0, Vec::new())
            .with_rfc3_coordinates(
                CertificationPosition {
                    scope: CertificationScope::All,
                    refutation: RefutationChannel::Complete,
                    residual_trust: ResidualTrust::Solver,
                    discharged_trust: vec!["workspace-fixture".into()],
                    boundary: CertificationBoundary::EndToEnd,
                },
                ClassificationCertificate {
                    fragment: "workspace-fixture-v1".into(),
                    verdict: ClassificationVerdict::Admitted,
                },
            )
            .unwrap()
            .with_live_disposition(crate::manifest::LiveResultDisposition::Accepted)
    }

    fn project(
        package: &str,
        target: &str,
        item_name: &str,
        revision: char,
    ) -> LiveAssuranceReport {
        let source = format!(
            "fn {item_name}(x: Int) -> Int ! pure requires true ensures result == x {{ x }}"
        );
        let program = parsed(&source);
        build_live_report_for_coordinate(
            &[cert(item_name)],
            &program,
            &format!("{package}/src/lib.th"),
            &source,
            &revision.to_string().repeat(40),
            ReportTrust::UntrustedPullRequest,
            Toolchain::new("verus-workspace-test"),
            coordinate(package, target),
        )
        .unwrap()
    }

    fn matrix(live: &LiveAssuranceReport) -> WorkspaceMatrixPlanV1 {
        let population = &live.report().body.project.population;
        WorkspaceMatrixPlanV1::new(
            population.build.coordinate(),
            live.report().body.source.source_path.clone(),
            population.intended.clone(),
        )
        .unwrap()
    }

    #[test]
    fn composition_is_input_order_invariant_and_duplicates_fail_closed() {
        let a = project("alpha", "lib", "a", 'a');
        let b = project("beta", "bin", "b", 'a');
        let plan = WorkspacePlanV1::new("1".repeat(64), vec![matrix(&b), matrix(&a)]).unwrap();
        let first = build_live_workspace_report(
            plan.clone(),
            &[&a, &b],
            &"a".repeat(40),
            ReportTrust::UntrustedPullRequest,
        )
        .unwrap();
        let second = build_live_workspace_report(
            plan,
            &[&b, &a],
            &"a".repeat(40),
            ReportTrust::UntrustedPullRequest,
        )
        .unwrap();
        assert_eq!(first.report(), second.report());
        assert!(first.report().body.whole_workspace_claim);
        assert_eq!(first.report().body.population.total, 2);
        assert!(build_live_workspace_report(
            first.report().body.plan.clone(),
            &[&a, &a, &b],
            &"a".repeat(40),
            ReportTrust::UntrustedPullRequest,
        )
        .unwrap_err()
        .reason
        .contains("duplicate live report"));
    }

    #[test]
    fn omitted_matrix_preserves_denominator_and_suppresses_workspace_claim() {
        let a = project("alpha", "lib", "a", 'a');
        let b = project("beta", "bin", "b", 'a');
        let plan = WorkspacePlanV1::new("2".repeat(64), vec![matrix(&a), matrix(&b)]).unwrap();
        let workspace = build_live_workspace_report(
            plan,
            &[&a],
            &"a".repeat(40),
            ReportTrust::UntrustedPullRequest,
        )
        .unwrap();
        assert_eq!(workspace.report().body.population.total, 2);
        assert_eq!(workspace.report().body.population.covered, 1);
        assert!(!workspace.report().body.whole_workspace_claim);
        assert!(matches!(
            &workspace.report().body.scope,
            WorkspacePortraitScopeV1::AcceptedSubset {
                numerator: 1,
                denominator: 2,
                unavailable_matrices,
                ..
            } if unavailable_matrices == &[coordinate("beta", "bin")]
        ));
    }

    #[test]
    fn every_matrix_coordinate_dimension_changes_identity_and_comparison() {
        let base = project("alpha", "lib", "a", 'a');
        let head = project("alpha", "lib", "a", 'b');
        let base_plan = WorkspacePlanV1::new("3".repeat(64), vec![matrix(&base)]).unwrap();
        let base_workspace = build_live_workspace_report(
            base_plan.clone(),
            &[&base],
            &"a".repeat(40),
            ReportTrust::UntrustedPullRequest,
        )
        .unwrap();
        for mutate in 0..7 {
            let mut changed = base_plan.matrices[0].clone();
            match mutate {
                0 => changed.coordinate.crate_name = "other".into(),
                1 => changed.coordinate.target = "bin".into(),
                2 => changed.coordinate.features = vec!["extra".into()],
                3 => changed.coordinate.platform = "aarch64-apple-darwin".into(),
                4 => changed.coordinate.generated_sources = vec!["generated/other.rs".into()],
                5 => {
                    changed.source_path = "alpha/src/other.th".into();
                    for item in &mut changed.intended {
                        item.source_path = changed.source_path.clone();
                    }
                }
                _ => changed.intended[0].item_path = "other".into(),
            }
            let changed_plan = WorkspacePlanV1::new("3".repeat(64), vec![changed]).unwrap();
            assert_ne!(base_plan.identity_digest(), changed_plan.identity_digest());
        }
        let mut changed_plan = WorkspacePlanV1::new("3".repeat(64), vec![matrix(&head)]).unwrap();
        changed_plan.matrices[0].coordinate.features = vec!["extra".into()];
        changed_plan = WorkspacePlanV1::new("3".repeat(64), changed_plan.matrices).unwrap();
        let unavailable_head = build_live_workspace_report(
            changed_plan,
            &[],
            &"b".repeat(40),
            ReportTrust::UntrustedPullRequest,
        )
        .unwrap();
        let comparison = compare_workspace_reports(
            base_workspace.report(),
            unavailable_head.report(),
            &"a".repeat(40),
        );
        assert!(comparison.build_matrix_changed);
        assert!(comparison.whole_workspace_claim_changed);
    }

    #[test]
    fn matrix_item_omission_changes_denominator_and_fails_exact_report_binding() {
        let live = project("alpha", "lib", "a", 'a');
        let mut wrong = matrix(&live);
        wrong.intended.push(ProjectItemIdentityV2 {
            source_path: "alpha/src/lib.th".into(),
            item_path: "omitted".into(),
        });
        let plan = WorkspacePlanV1::new("4".repeat(64), vec![wrong]).unwrap();
        let error = build_live_workspace_report(
            plan,
            &[&live],
            &"a".repeat(40),
            ReportTrust::UntrustedPullRequest,
        )
        .unwrap_err();
        assert!(error
            .reason
            .contains("does not equal the independent matrix plan"));
    }

    #[test]
    fn workspace_json_is_diagnostic_and_tampering_is_detected() {
        let live = project("alpha", "lib", "a", 'a');
        let plan = WorkspacePlanV1::new("5".repeat(64), vec![matrix(&live)]).unwrap();
        let workspace = build_live_workspace_report(
            plan,
            &[&live],
            &"a".repeat(40),
            ReportTrust::UntrustedPullRequest,
        )
        .unwrap();
        let json = workspace.report().normalized_json().unwrap();
        let parsed = parse_workspace_report_json(json.as_bytes()).unwrap();
        parsed.validate().unwrap();
        let mut tampered = parsed;
        tampered.body.population.total += 1;
        assert!(tampered.validate().is_err());
        assert!(workspace.report().body.matrices[0]
            .project_report
            .as_ref()
            .unwrap()
            .body
            .items[0]
            .policy_points
            .contains(&AssuranceKindV2::SolverComplete));
    }
}
