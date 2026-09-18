//! Organization-level formal assurance policy.
//!
//! Policies address exact repository/workspace/claim-fiber coordinates. They
//! consume only non-serializable live workspace capabilities and checked
//! policy-version migrations. Presentation labels, percentages, badges, and
//! evidence frontiers do not occur in the admission schema.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::assurance_v2::{assurance_kind_leq, AssuranceKindV2};
use crate::policy_migration::PolicyMigrationReceiptV1;
use crate::workspace_assurance::LiveWorkspaceAssuranceReportV1;

pub const ORGANIZATION_PLAN_SCHEMA: &str = "thermite-organization-assurance-plan/v1";
pub const ORGANIZATION_POLICY_SCHEMA: &str = "thermite-organization-assurance-policy/v1";
pub const ORGANIZATION_EVALUATION_SCHEMA: &str = "thermite-organization-assurance-evaluation/v1";
pub const MAX_ORGANIZATION_JSON_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizationPolicyError(pub String);

impl std::fmt::Display for OrganizationPolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for OrganizationPolicyError {}

fn error(message: impl Into<String>) -> OrganizationPolicyError {
    OrganizationPolicyError(message.into())
}

fn nonempty(value: &str, field: &str) -> Result<(), OrganizationPolicyError> {
    if value.trim().is_empty() {
        Err(error(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digest(domain: &[u8], value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("closed organization policy value serializes");
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrganizationFloorKeyV1 {
    pub repository: String,
    pub workspace_plan_sha256: String,
    pub fiber_sha256: String,
}

impl OrganizationFloorKeyV1 {
    fn validate(&self) -> Result<(), OrganizationPolicyError> {
        nonempty(&self.repository, "repository identity")?;
        if !is_sha256(&self.workspace_plan_sha256) || !is_sha256(&self.fiber_sha256) {
            return Err(error(
                "workspace-plan and claim-fiber identities must be lowercase SHA-256 digests",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrganizationRepositoryPlanV1 {
    pub repository: String,
    pub revision: String,
    pub workspace_plan_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrganizationPlanV1 {
    pub schema: String,
    pub organization: String,
    pub repositories: Vec<OrganizationRepositoryPlanV1>,
}

impl OrganizationPlanV1 {
    pub fn new(
        organization: String,
        mut repositories: Vec<OrganizationRepositoryPlanV1>,
    ) -> Result<Self, OrganizationPolicyError> {
        nonempty(&organization, "organization identity")?;
        for repository in &repositories {
            nonempty(&repository.repository, "repository identity")?;
            if !matches!(repository.revision.len(), 40 | 64)
                || !repository
                    .revision
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(error(
                    "organization repositories require exact lowercase 40- or 64-hex revisions",
                ));
            }
            if !is_sha256(&repository.workspace_plan_sha256) {
                return Err(error(
                    "organization repository plans require lowercase SHA-256 workspace-plan identities",
                ));
            }
        }
        repositories.sort_by(|left, right| left.repository.cmp(&right.repository));
        if repositories
            .windows(2)
            .any(|pair| pair[0].repository == pair[1].repository)
        {
            return Err(error("organization repository identities must be unique"));
        }
        if repositories.is_empty() {
            return Err(error("organization plan must name at least one repository"));
        }
        Ok(Self {
            schema: ORGANIZATION_PLAN_SCHEMA.into(),
            organization,
            repositories,
        })
    }

    fn validate(&self) -> Result<(), OrganizationPolicyError> {
        if self.schema != ORGANIZATION_PLAN_SCHEMA {
            return Err(error(format!(
                "organization plan schema skew: expected {ORGANIZATION_PLAN_SCHEMA}, found {}",
                self.schema
            )));
        }
        if &Self::new(self.organization.clone(), self.repositories.clone())? != self {
            return Err(error(
                "organization plan is not in canonical repository order",
            ));
        }
        Ok(())
    }

    pub fn identity_digest(&self) -> String {
        digest(b"thermite-organization-plan-v1\0", self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyParentV1 {
    pub package_id: String,
    pub version: u64,
    pub package_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrganizationFloorV1 {
    pub key: OrganizationFloorKeyV1,
    pub minimum: AssuranceKindV2,
    pub owner: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDelegationV1 {
    pub key: OrganizationFloorKeyV1,
    pub delegate: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScopedExceptionV1 {
    pub key: OrganizationFloorKeyV1,
    pub owner: String,
    pub expires_at_epoch: u64,
    pub reason: String,
    pub audit_provenance: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrganizationPolicyPackageV1 {
    pub schema: String,
    pub package_id: String,
    pub version: u64,
    pub owner: String,
    pub organization_plan_sha256: String,
    pub collapse_policy_version: u64,
    pub valid_from_epoch: u64,
    pub expires_at_epoch: Option<u64>,
    pub parent: Option<PolicyParentV1>,
    pub floors: Vec<OrganizationFloorV1>,
    #[serde(default)]
    pub delegations: Vec<PolicyDelegationV1>,
    #[serde(default)]
    pub exceptions: Vec<ScopedExceptionV1>,
    pub package_sha256: String,
}

#[derive(Serialize)]
struct PolicyPackageBody<'a> {
    schema: &'a str,
    package_id: &'a str,
    version: u64,
    owner: &'a str,
    organization_plan_sha256: &'a str,
    collapse_policy_version: u64,
    valid_from_epoch: u64,
    expires_at_epoch: Option<u64>,
    parent: &'a Option<PolicyParentV1>,
    floors: &'a [OrganizationFloorV1],
    delegations: &'a [PolicyDelegationV1],
    exceptions: &'a [ScopedExceptionV1],
}

impl OrganizationPolicyPackageV1 {
    fn expected_digest(&self) -> String {
        digest(
            b"thermite-organization-policy-package-v1\0",
            &PolicyPackageBody {
                schema: &self.schema,
                package_id: &self.package_id,
                version: self.version,
                owner: &self.owner,
                organization_plan_sha256: &self.organization_plan_sha256,
                collapse_policy_version: self.collapse_policy_version,
                valid_from_epoch: self.valid_from_epoch,
                expires_at_epoch: self.expires_at_epoch,
                parent: &self.parent,
                floors: &self.floors,
                delegations: &self.delegations,
                exceptions: &self.exceptions,
            },
        )
    }

    fn validate_shape(&self, evaluation_epoch: u64) -> Result<(), OrganizationPolicyError> {
        if self.schema != ORGANIZATION_POLICY_SCHEMA {
            return Err(error(format!(
                "organization policy schema skew: expected {ORGANIZATION_POLICY_SCHEMA}, found {}",
                self.schema
            )));
        }
        nonempty(&self.package_id, "policy package identity")?;
        nonempty(&self.owner, "policy package owner")?;
        if !is_sha256(&self.organization_plan_sha256) {
            return Err(error("policy package must bind an exact organization plan"));
        }
        if self.version == 0 || self.collapse_policy_version == 0 {
            return Err(error(
                "policy and collapse-policy versions must be positive",
            ));
        }
        if evaluation_epoch < self.valid_from_epoch
            || self
                .expires_at_epoch
                .is_some_and(|expiry| evaluation_epoch >= expiry)
        {
            return Err(error(format!(
                "policy package {}@{} is not active at evaluation epoch {evaluation_epoch}",
                self.package_id, self.version
            )));
        }
        if self
            .expires_at_epoch
            .is_some_and(|expiry| expiry <= self.valid_from_epoch)
        {
            return Err(error("policy package expiry must follow its activation"));
        }
        if self.package_sha256 != self.expected_digest() {
            return Err(error("organization policy package digest mismatch"));
        }
        let mut floor_keys = BTreeSet::new();
        if self
            .floors
            .windows(2)
            .any(|pair| pair[0].key >= pair[1].key)
        {
            return Err(error("policy floors must be in canonical exact-key order"));
        }
        for floor in &self.floors {
            floor.key.validate()?;
            nonempty(&floor.owner, "floor owner")?;
            if floor.owner != self.owner {
                return Err(error(
                    "every floor must be owned by its declaring package owner",
                ));
            }
            if !floor_keys.insert(floor.key.clone()) {
                return Err(error(
                    "one policy package cannot duplicate an exact floor key",
                ));
            }
        }
        let mut delegation_keys = BTreeSet::new();
        if self
            .delegations
            .windows(2)
            .any(|pair| (&pair[0].key, &pair[0].delegate) >= (&pair[1].key, &pair[1].delegate))
        {
            return Err(error(
                "policy delegations must be in canonical scope/owner order",
            ));
        }
        for delegation in &self.delegations {
            delegation.key.validate()?;
            nonempty(&delegation.delegate, "delegated owner")?;
            if !delegation_keys.insert((delegation.key.clone(), delegation.delegate.clone())) {
                return Err(error("duplicate exact-scope delegation"));
            }
        }
        let mut exception_keys = BTreeSet::new();
        if self
            .exceptions
            .windows(2)
            .any(|pair| pair[0].key >= pair[1].key)
        {
            return Err(error(
                "policy exceptions must be in canonical exact-key order",
            ));
        }
        for exception in &self.exceptions {
            exception.key.validate()?;
            nonempty(&exception.owner, "exception owner")?;
            nonempty(&exception.reason, "exception reason")?;
            nonempty(&exception.audit_provenance, "exception audit provenance")?;
            if exception.owner != self.owner {
                return Err(error(
                    "every exception must be owned by its declaring package owner",
                ));
            }
            if exception.expires_at_epoch <= evaluation_epoch {
                continue;
            }
            if !exception_keys.insert(exception.key.clone()) {
                return Err(error(
                    "one policy package cannot duplicate an active exception key",
                ));
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn reseal(&mut self) {
        self.package_sha256 = self.expected_digest();
    }
}

#[derive(Clone, Debug)]
pub struct LiveOrganizationRepositoryV1<'a> {
    pub repository: String,
    pub workspace: &'a LiveWorkspaceAssuranceReportV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedExceptionV1 {
    pub owner: String,
    pub expires_at_epoch: u64,
    pub reason: String,
    pub audit_provenance: String,
    pub source_package_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrganizationFloorResultV1 {
    pub key: OrganizationFloorKeyV1,
    pub minimum: AssuranceKindV2,
    pub policy_owner: String,
    pub source_package_sha256: String,
    pub formal_satisfied: bool,
    pub excepted: bool,
    pub gate_passed: bool,
    pub reason: String,
    pub exception: Option<AppliedExceptionV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrganizationPolicyEvaluationV1 {
    pub schema: String,
    pub organization_plan_sha256: String,
    pub evaluation_epoch: u64,
    pub package_sha256: Vec<String>,
    pub migration_receipt_sha256: Vec<String>,
    pub passed: bool,
    pub floors: Vec<OrganizationFloorResultV1>,
    pub evaluation_sha256: String,
}

#[derive(Clone)]
struct EffectiveFloor {
    minimum: AssuranceKindV2,
    owner: String,
    source_package_sha256: String,
}

#[derive(Clone)]
struct EffectiveException {
    value: ScopedExceptionV1,
    source_package_sha256: String,
}

fn authorized(
    owner: &str,
    key: &OrganizationFloorKeyV1,
    root_owner: &str,
    delegations: &BTreeSet<(OrganizationFloorKeyV1, String)>,
) -> bool {
    owner == root_owner || delegations.contains(&(key.clone(), owner.to_string()))
}

/// Evaluate a complete organization plan from live workspace capabilities.
/// The explicit epoch is an input, making expiry replay deterministic.
pub fn evaluate_organization_policy(
    plan: &OrganizationPlanV1,
    packages: &[OrganizationPolicyPackageV1],
    repositories: &[LiveOrganizationRepositoryV1<'_>],
    migrations: &[PolicyMigrationReceiptV1],
    evaluation_epoch: u64,
) -> Result<OrganizationPolicyEvaluationV1, OrganizationPolicyError> {
    plan.validate()?;
    if packages.is_empty() {
        return Err(error("organization policy chain must not be empty"));
    }
    let plan_sha256 = plan.identity_digest();
    let expected_repositories = plan
        .repositories
        .iter()
        .map(|repository| repository.repository.as_str())
        .collect::<BTreeSet<_>>();
    let supplied_repositories = repositories
        .iter()
        .map(|repository| repository.repository.as_str())
        .collect::<BTreeSet<_>>();
    if supplied_repositories.len() != repositories.len()
        || supplied_repositories != expected_repositories
    {
        return Err(error(
            "live organization repositories must exactly equal the independent organization plan",
        ));
    }
    let live = repositories
        .iter()
        .map(|repository| (repository.repository.as_str(), repository.workspace))
        .collect::<BTreeMap<_, _>>();
    let mut runtime_policy_version = None;
    for expected in &plan.repositories {
        let workspace = live[expected.repository.as_str()];
        if workspace.plan_sha256() != expected.workspace_plan_sha256 {
            return Err(error(format!(
                "repository {} supplied a different exact workspace plan",
                expected.repository
            )));
        }
        if workspace.revision() != expected.revision {
            return Err(error(format!(
                "repository {} supplied a different exact revision",
                expected.repository
            )));
        }
        if workspace.trust() != &crate::assurance_report::ReportTrust::ProtectedExactSha {
            return Err(error(format!(
                "repository {} is not a protected exact-SHA live workspace",
                expected.repository
            )));
        }
        if runtime_policy_version
            .replace(workspace.collapse_policy_version())
            .is_some_and(|seen| seen != workspace.collapse_policy_version())
        {
            return Err(error(
                "live organization repositories have collapse-policy skew",
            ));
        }
    }
    let runtime_policy_version = runtime_policy_version.expect("nonempty plan has a repository");

    let mut package_digests = Vec::new();
    let mut migration_digests = BTreeSet::new();
    let root_owner = packages[0].owner.clone();
    let mut prior: Option<&OrganizationPolicyPackageV1> = None;
    let mut delegations = BTreeSet::new();
    let mut floors = BTreeMap::<OrganizationFloorKeyV1, EffectiveFloor>::new();
    let mut exceptions = BTreeMap::<OrganizationFloorKeyV1, EffectiveException>::new();
    let plan_repositories = plan
        .repositories
        .iter()
        .map(|repository| {
            (
                repository.repository.as_str(),
                repository.workspace_plan_sha256.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    for package in packages {
        package.validate_shape(evaluation_epoch)?;
        if package.organization_plan_sha256 != plan_sha256 {
            return Err(error(
                "policy package is bound to a different organization plan",
            ));
        }
        match (prior, &package.parent) {
            (None, None) => {}
            (None, Some(_)) => return Err(error("root policy package cannot name a parent")),
            (Some(parent), Some(reference))
                if reference.package_id == parent.package_id
                    && reference.version == parent.version
                    && reference.package_sha256 == parent.package_sha256
                    && package.package_id == parent.package_id
                    && package.version > parent.version => {}
            (Some(_), _) => return Err(error("policy inheritance edge is missing or misbound")),
        }
        let translation = if package.collapse_policy_version == runtime_policy_version {
            None
        } else {
            let receipt = migrations
                .iter()
                .find(|receipt| {
                    receipt.source_policy_version == package.collapse_policy_version
                        && receipt.target_policy_version == runtime_policy_version
                })
                .ok_or_else(|| error("policy-version skew lacks a checked normalization"))?;
            let validated = receipt
                .validate(package.collapse_policy_version, runtime_policy_version)
                .map_err(|reason| error(reason.to_string()))?;
            migration_digests.insert(validated.receipt_sha256().to_string());
            Some(validated)
        };
        for floor in &package.floors {
            if plan_repositories
                .get(floor.key.repository.as_str())
                .copied()
                != Some(floor.key.workspace_plan_sha256.as_str())
            {
                return Err(error("floor key is outside the exact organization plan"));
            }
            if prior.is_some() && !authorized(&package.owner, &floor.key, &root_owner, &delegations)
            {
                return Err(error(format!(
                    "owner {} is not delegated the exact floor scope",
                    package.owner
                )));
            }
            let minimum = translation.as_ref().map_or(floor.minimum, |migration| {
                migration.translate_kind(floor.minimum)
            });
            match floors.get(&floor.key) {
                None => {
                    floors.insert(
                        floor.key.clone(),
                        EffectiveFloor {
                            minimum,
                            owner: floor.owner.clone(),
                            source_package_sha256: package.package_sha256.clone(),
                        },
                    );
                }
                Some(existing) if existing.minimum == minimum => {}
                Some(existing) if assurance_kind_leq(existing.minimum, minimum) => {
                    floors.insert(
                        floor.key.clone(),
                        EffectiveFloor {
                            minimum,
                            owner: floor.owner.clone(),
                            source_package_sha256: package.package_sha256.clone(),
                        },
                    );
                }
                Some(existing) if assurance_kind_leq(minimum, existing.minimum) => {
                    return Err(error(
                        "inherited policy attempts to weaken an exact formal floor",
                    ));
                }
                Some(_) => {
                    return Err(error(
                        "inherited exact formal floors are incomparable; conflict is unresolved",
                    ));
                }
            }
        }
        for exception in &package.exceptions {
            if prior.is_some()
                && !authorized(&package.owner, &exception.key, &root_owner, &delegations)
            {
                return Err(error(format!(
                    "owner {} is not delegated the exact exception scope",
                    package.owner
                )));
            }
            if exception.expires_at_epoch > evaluation_epoch {
                exceptions.insert(
                    exception.key.clone(),
                    EffectiveException {
                        value: exception.clone(),
                        source_package_sha256: package.package_sha256.clone(),
                    },
                );
            }
        }
        for delegation in &package.delegations {
            if plan_repositories
                .get(delegation.key.repository.as_str())
                .copied()
                != Some(delegation.key.workspace_plan_sha256.as_str())
            {
                return Err(error(
                    "delegation key is outside the exact organization plan",
                ));
            }
            if prior.is_some()
                && !authorized(&package.owner, &delegation.key, &root_owner, &delegations)
            {
                return Err(error("undelegated owner cannot redelegate an exact scope"));
            }
            delegations.insert((delegation.key.clone(), delegation.delegate.clone()));
        }
        package_digests.push(package.package_sha256.clone());
        prior = Some(package);
    }
    if floors.is_empty() {
        return Err(error(
            "effective organization policy must contain a formal floor",
        ));
    }
    if let Some(orphan) = exceptions.keys().find(|key| !floors.contains_key(*key)) {
        return Err(error(format!(
            "scoped exception has no effective floor for {}",
            orphan.repository
        )));
    }

    let mut results = Vec::with_capacity(floors.len());
    for (key, floor) in floors {
        let workspace = live[key.repository.as_str()];
        let (formal_satisfied, formal_reason) =
            workspace.evaluate_exact_floor(&key.fiber_sha256, floor.minimum);
        let exception = (!formal_satisfied)
            .then(|| exceptions.get(&key))
            .flatten()
            .map(|exception| AppliedExceptionV1 {
                owner: exception.value.owner.clone(),
                expires_at_epoch: exception.value.expires_at_epoch,
                reason: exception.value.reason.clone(),
                audit_provenance: exception.value.audit_provenance.clone(),
                source_package_sha256: exception.source_package_sha256.clone(),
            });
        let excepted = exception.is_some();
        let gate_passed = formal_satisfied || excepted;
        let reason = if excepted {
            format!(
                "formal floor is not satisfied ({formal_reason}); an active exact-scope exception permits the gate but does not satisfy the floor"
            )
        } else {
            formal_reason
        };
        results.push(OrganizationFloorResultV1 {
            key,
            minimum: floor.minimum,
            policy_owner: floor.owner,
            source_package_sha256: floor.source_package_sha256,
            formal_satisfied,
            excepted,
            gate_passed,
            reason,
            exception,
        });
    }
    let mut evaluation = OrganizationPolicyEvaluationV1 {
        schema: ORGANIZATION_EVALUATION_SCHEMA.into(),
        organization_plan_sha256: plan_sha256,
        evaluation_epoch,
        package_sha256: package_digests,
        migration_receipt_sha256: migration_digests.into_iter().collect(),
        passed: results.iter().all(|result| result.gate_passed),
        floors: results,
        evaluation_sha256: String::new(),
    };
    evaluation.evaluation_sha256 = digest(
        b"thermite-organization-policy-evaluation-v1\0",
        &(
            &evaluation.schema,
            &evaluation.organization_plan_sha256,
            evaluation.evaluation_epoch,
            &evaluation.package_sha256,
            &evaluation.migration_receipt_sha256,
            evaluation.passed,
            &evaluation.floors,
        ),
    );
    Ok(evaluation)
}

impl OrganizationPolicyEvaluationV1 {
    pub fn normalized_json(&self) -> Result<String, OrganizationPolicyError> {
        let expected = digest(
            b"thermite-organization-policy-evaluation-v1\0",
            &(
                &self.schema,
                &self.organization_plan_sha256,
                self.evaluation_epoch,
                &self.package_sha256,
                &self.migration_receipt_sha256,
                self.passed,
                &self.floors,
            ),
        );
        if self.schema != ORGANIZATION_EVALUATION_SCHEMA || self.evaluation_sha256 != expected {
            return Err(error("organization evaluation envelope is not canonical"));
        }
        serde_json::to_string_pretty(self)
            .map(|json| format!("{json}\n"))
            .map_err(|reason| {
                error(format!(
                    "organization evaluation serialization failed: {reason}"
                ))
            })
    }
}

fn bounded_json(bytes: &[u8]) -> Result<serde_json::Value, OrganizationPolicyError> {
    if bytes.len() > MAX_ORGANIZATION_JSON_BYTES {
        return Err(error(format!(
            "organization JSON exceeds the {MAX_ORGANIZATION_JSON_BYTES}-byte limit"
        )));
    }
    serde_json::from_slice(bytes)
        .map_err(|reason| error(format!("invalid organization JSON: {reason}")))
}

pub fn parse_organization_plan_json(
    bytes: &[u8],
) -> Result<OrganizationPlanV1, OrganizationPolicyError> {
    let raw: OrganizationPlanV1 = serde_json::from_value(bounded_json(bytes)?)
        .map_err(|reason| error(format!("invalid organization plan schema: {reason}")))?;
    raw.validate()?;
    Ok(raw)
}

pub fn parse_policy_package_json(
    bytes: &[u8],
) -> Result<OrganizationPolicyPackageV1, OrganizationPolicyError> {
    serde_json::from_value(bounded_json(bytes)?)
        .map_err(|reason| error(format!("invalid organization policy schema: {reason}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assurance_report::{build_live_report_for_coordinate, ReportTrust};
    use crate::assurance_v2::{ProjectBuildCoordinateV2, ProjectItemIdentityV2};
    use crate::audit::Toolchain;
    use crate::manifest::{
        Certificate, CertificationBoundary, CertificationPosition, CertificationScope,
        ClassificationCertificate, ClassificationVerdict, Level, RefutationChannel, ResidualTrust,
    };
    use crate::workspace_assurance::{
        build_live_workspace_report, WorkspaceMatrixPlanV1, WorkspacePlanV1,
    };

    fn live_workspace(
        workspace_identity: char,
    ) -> (
        LiveWorkspaceAssuranceReportV1,
        OrganizationRepositoryPlanV1,
        String,
    ) {
        let source = "fn identity(x: Int) -> Int ! pure requires true ensures result == x { x }";
        let parsed = thermite_syntax::parse(source);
        assert!(parsed.is_clean());
        let cert = Certificate::new("identity", Level::L4, vec!["pure".into()], 0, Vec::new())
            .with_rfc3_coordinates(
                CertificationPosition {
                    scope: CertificationScope::All,
                    refutation: RefutationChannel::Complete,
                    residual_trust: ResidualTrust::Solver,
                    discharged_trust: vec!["organization-fixture".into()],
                    boundary: CertificationBoundary::EndToEnd,
                },
                ClassificationCertificate {
                    fragment: "organization-fixture-v1".into(),
                    verdict: ClassificationVerdict::Admitted,
                },
            )
            .unwrap()
            .with_live_disposition(crate::manifest::LiveResultDisposition::Accepted);
        let coordinate = ProjectBuildCoordinateV2 {
            crate_name: "fixture".into(),
            target: "lib".into(),
            features: vec!["default".into()],
            platform: "x86_64-unknown-linux-gnu".into(),
            generated_sources: Vec::new(),
        };
        let project = build_live_report_for_coordinate(
            &[cert],
            &parsed.program,
            "fixture/src/lib.th",
            source,
            &workspace_identity.to_string().repeat(40),
            ReportTrust::ProtectedExactSha,
            Toolchain::new("organization-fixture"),
            coordinate.clone(),
        )
        .unwrap();
        let fiber = project.report().body.project.frontiers[0]
            .claim_fiber()
            .as_str()
            .to_string();
        let plan = WorkspacePlanV1::new(
            workspace_identity.to_string().repeat(64),
            vec![WorkspaceMatrixPlanV1::new(
                coordinate,
                "fixture/src/lib.th".into(),
                vec![ProjectItemIdentityV2 {
                    source_path: "fixture/src/lib.th".into(),
                    item_path: "identity".into(),
                }],
            )
            .unwrap()],
        )
        .unwrap();
        let plan_sha256 = plan.identity_digest();
        let workspace = build_live_workspace_report(
            plan,
            &[&project],
            &workspace_identity.to_string().repeat(40),
            ReportTrust::ProtectedExactSha,
        )
        .unwrap();
        (
            workspace,
            OrganizationRepositoryPlanV1 {
                repository: format!("org/repo-{workspace_identity}"),
                revision: workspace_identity.to_string().repeat(40),
                workspace_plan_sha256: plan_sha256,
            },
            fiber,
        )
    }

    fn package(
        plan: &OrganizationPlanV1,
        owner: &str,
        version: u64,
        parent: Option<&OrganizationPolicyPackageV1>,
        floors: Vec<OrganizationFloorV1>,
        delegations: Vec<PolicyDelegationV1>,
        exceptions: Vec<ScopedExceptionV1>,
    ) -> OrganizationPolicyPackageV1 {
        let mut package = OrganizationPolicyPackageV1 {
            schema: ORGANIZATION_POLICY_SCHEMA.into(),
            package_id: "org-policy".into(),
            version,
            owner: owner.into(),
            organization_plan_sha256: plan.identity_digest(),
            collapse_policy_version: crate::assurance_report::COLLAPSE_POLICY_VERSION,
            valid_from_epoch: 10,
            expires_at_epoch: Some(100),
            parent: parent.map(|parent| PolicyParentV1 {
                package_id: parent.package_id.clone(),
                version: parent.version,
                package_sha256: parent.package_sha256.clone(),
            }),
            floors,
            delegations,
            exceptions,
            package_sha256: String::new(),
        };
        package.reseal();
        package
    }

    #[test]
    fn exact_live_common_claims_are_the_only_floor_authority() {
        let (workspace, repository, fiber) = live_workspace('a');
        let plan = OrganizationPlanV1::new("org".into(), vec![repository.clone()]).unwrap();
        let key = OrganizationFloorKeyV1 {
            repository: repository.repository.clone(),
            workspace_plan_sha256: repository.workspace_plan_sha256,
            fiber_sha256: fiber,
        };
        let root = package(
            &plan,
            "security",
            1,
            None,
            vec![OrganizationFloorV1 {
                key,
                minimum: AssuranceKindV2::SolverComplete,
                owner: "security".into(),
            }],
            Vec::new(),
            Vec::new(),
        );
        let evaluation = evaluate_organization_policy(
            &plan,
            &[root],
            &[LiveOrganizationRepositoryV1 {
                repository: repository.repository,
                workspace: &workspace,
            }],
            &[],
            20,
        )
        .unwrap();
        assert!(evaluation.passed);
        assert!(evaluation.floors[0].formal_satisfied);
        assert!(!evaluation.floors[0].excepted);
    }

    #[test]
    fn inheritance_delegation_exceptions_expiry_and_conflicts_fail_closed() {
        let (workspace, repository, fiber) = live_workspace('b');
        let plan = OrganizationPlanV1::new("org".into(), vec![repository.clone()]).unwrap();
        let key = OrganizationFloorKeyV1 {
            repository: repository.repository.clone(),
            workspace_plan_sha256: repository.workspace_plan_sha256.clone(),
            fiber_sha256: fiber,
        };
        let root = package(
            &plan,
            "security",
            1,
            None,
            vec![OrganizationFloorV1 {
                key: key.clone(),
                minimum: AssuranceKindV2::LeanEmpirical,
                owner: "security".into(),
            }],
            vec![PolicyDelegationV1 {
                key: key.clone(),
                delegate: "team-a".into(),
            }],
            Vec::new(),
        );
        let child = package(
            &plan,
            "team-a",
            2,
            Some(&root),
            Vec::new(),
            Vec::new(),
            vec![ScopedExceptionV1 {
                key: key.clone(),
                owner: "team-a".into(),
                expires_at_epoch: 30,
                reason: "bounded migration window".into(),
                audit_provenance: "audit://ticket/42".into(),
            }],
        );
        let input = [LiveOrganizationRepositoryV1 {
            repository: repository.repository.clone(),
            workspace: &workspace,
        }];
        let active =
            evaluate_organization_policy(&plan, &[root.clone(), child.clone()], &input, &[], 20)
                .unwrap();
        assert!(active.passed);
        assert!(!active.floors[0].formal_satisfied);
        assert!(active.floors[0].excepted);
        let expired =
            evaluate_organization_policy(&plan, &[root.clone(), child], &input, &[], 30).unwrap();
        assert!(!expired.passed);
        assert!(!expired.floors[0].excepted);

        let mut incomparable = package(
            &plan,
            "team-a",
            2,
            Some(&root),
            vec![OrganizationFloorV1 {
                key,
                minimum: AssuranceKindV2::SolverComplete,
                owner: "team-a".into(),
            }],
            Vec::new(),
            Vec::new(),
        );
        incomparable.parent.as_mut().unwrap().package_sha256 = root.package_sha256.clone();
        incomparable.reseal();
        assert!(
            evaluate_organization_policy(&plan, &[root, incomparable], &input, &[], 20)
                .unwrap_err()
                .0
                .contains("incomparable")
        );
    }

    #[test]
    fn checked_policy_normalization_is_required_and_preserves_the_exact_floor() {
        let (mut workspace, repository, fiber) = live_workspace('d');
        workspace.set_collapse_policy_version_for_test(2);
        let plan = OrganizationPlanV1::new("org".into(), vec![repository.clone()]).unwrap();
        let key = OrganizationFloorKeyV1 {
            repository: repository.repository.clone(),
            workspace_plan_sha256: repository.workspace_plan_sha256.clone(),
            fiber_sha256: fiber,
        };
        let root = package(
            &plan,
            "security",
            1,
            None,
            vec![OrganizationFloorV1 {
                key: key.clone(),
                minimum: AssuranceKindV2::SolverComplete,
                owner: "security".into(),
            }],
            Vec::new(),
            Vec::new(),
        );
        let input = [LiveOrganizationRepositoryV1 {
            repository: repository.repository,
            workspace: &workspace,
        }];
        assert!(
            evaluate_organization_policy(&plan, std::slice::from_ref(&root), &input, &[], 20,)
                .unwrap_err()
                .0
                .contains("lacks a checked normalization")
        );

        let migration = PolicyMigrationReceiptV1::compatible_identity(
            1,
            2,
            "Thermite.OrganizationPolicy.checked_policy_translation_preserves_floor_order",
        );
        let evaluation = evaluate_organization_policy(
            &plan,
            &[root],
            &input,
            std::slice::from_ref(&migration),
            20,
        )
        .unwrap();
        assert!(evaluation.passed);
        assert_eq!(evaluation.floors[0].key, key);
        assert_eq!(
            evaluation.floors[0].minimum,
            AssuranceKindV2::SolverComplete
        );
        assert_eq!(
            evaluation.migration_receipt_sha256,
            vec![migration.receipt_sha256]
        );
    }

    #[test]
    fn missing_population_skew_forgery_and_undelegated_changes_are_rejected() {
        let (workspace, repository, fiber) = live_workspace('c');
        let plan = OrganizationPlanV1::new("org".into(), vec![repository.clone()]).unwrap();
        let key = OrganizationFloorKeyV1 {
            repository: repository.repository.clone(),
            workspace_plan_sha256: repository.workspace_plan_sha256.clone(),
            fiber_sha256: fiber,
        };
        let root = package(
            &plan,
            "security",
            1,
            None,
            vec![OrganizationFloorV1 {
                key: key.clone(),
                minimum: AssuranceKindV2::SolverComplete,
                owner: "security".into(),
            }],
            Vec::new(),
            Vec::new(),
        );
        assert!(
            evaluate_organization_policy(&plan, std::slice::from_ref(&root), &[], &[], 20).is_err()
        );

        let mut forged = root.clone();
        forged.owner = "attacker".into();
        assert!(evaluate_organization_policy(
            &plan,
            &[forged],
            &[LiveOrganizationRepositoryV1 {
                repository: repository.repository.clone(),
                workspace: &workspace,
            }],
            &[],
            20,
        )
        .is_err());

        let child = package(
            &plan,
            "team-b",
            2,
            Some(&root),
            vec![OrganizationFloorV1 {
                key,
                minimum: AssuranceKindV2::LeanComplete,
                owner: "team-b".into(),
            }],
            Vec::new(),
            Vec::new(),
        );
        assert!(evaluate_organization_policy(
            &plan,
            &[root, child],
            &[LiveOrganizationRepositoryV1 {
                repository: repository.repository,
                workspace: &workspace,
            }],
            &[],
            20,
        )
        .unwrap_err()
        .0
        .contains("not delegated"));
    }
}
