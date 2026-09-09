//! Rust replay of the Lean `AssurancePolicyV2` constructor-family laws.
//!
//! This module is deliberately parameter-free: it enumerates the six closed
//! constructor families, never execution identities, bounds, semantic/model
//! versions, contexts, or boundaries. Those values remain exact fiber keys.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceKindV2 {
    Runtime,
    Bounded,
    SolverIncomplete,
    SolverComplete,
    LeanEmpirical,
    LeanComplete,
}

pub const ALL_ASSURANCE_KINDS_V2: [AssuranceKindV2; 6] = [
    AssuranceKindV2::Runtime,
    AssuranceKindV2::Bounded,
    AssuranceKindV2::SolverIncomplete,
    AssuranceKindV2::SolverComplete,
    AssuranceKindV2::LeanEmpirical,
    AssuranceKindV2::LeanComplete,
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
enum PopulationKeyV2 {
    OneExecution { identity: String },
    ThroughBound { bound: u64 },
    AllInputs,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
enum BoundaryKeyV2 {
    EndToEnd,
    ToBoundary { via: String },
    ToPlatform { platform: String },
}

/// Metatheory fixture for the complete formal authority projection. Production
/// issue #56 authority is serialized by `manifest::FormalAuthorityRecordV2`;
/// this independent shape keeps the digest/presentation separation laws pinned.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct FormalAuthorityRecordV2 {
    subject: String,
    source_sha256: String,
    artifact_sha256: String,
    family: AssuranceKindV2,
    semantics: String,
    semantics_version: u64,
    implementation_model: String,
    implementation_model_version: u64,
    fragment_lineage: String,
    fragment_revision: u64,
    population: PopulationKeyV2,
    claim_identity: String,
    observation_contract: String,
    refutation_contract: String,
    classification_fragment: String,
    classification_verdict: String,
    residual_context: String,
    boundary: BoundaryKeyV2,
    procedure: String,
    procedure_version: u64,
    environment: String,
    tool_version: String,
    resource_budget: u64,
    residual_trust: String,
    axioms: Vec<String>,
    accepted_evidence_sha256: String,
    reconstruction_identity: String,
    composition_witnesses: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct EngineerDisplayRecordV1 {
    claim: String,
    coverage: BoundaryKeyV2,
}

#[derive(Serialize)]
struct PresentationRecord<'a> {
    authority_digest: &'a str,
    report_schema: u64,
    policy_version: u64,
    display: &'a EngineerDisplayRecordV1,
}

fn sha256_domain(domain: &[u8], payload: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(payload);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn authority_digest(record: &FormalAuthorityRecordV2) -> String {
    sha256_domain(
        b"thermite-assurance-authority-v2\0",
        &serde_json::to_vec(record).expect("closed formal authority record serializes"),
    )
}

fn presentation_digest(
    authority: &str,
    report_schema: u64,
    policy_version: u64,
    display: &EngineerDisplayRecordV1,
) -> String {
    sha256_domain(
        b"thermite-assurance-presentation-v1\0",
        &serde_json::to_vec(&PresentationRecord {
            authority_digest: authority,
            report_schema,
            policy_version,
            display,
        })
        .expect("closed presentation record serializes"),
    )
}

pub const fn assurance_kind_leq(left: AssuranceKindV2, right: AssuranceKindV2) -> bool {
    use AssuranceKindV2::{
        Bounded, LeanComplete, LeanEmpirical, Runtime, SolverComplete, SolverIncomplete,
    };
    matches!(
        (left, right),
        (Runtime, Runtime)
            | (Bounded, Bounded)
            | (
                SolverIncomplete,
                SolverIncomplete | SolverComplete | LeanEmpirical | LeanComplete
            )
            | (SolverComplete, SolverComplete | LeanComplete)
            | (LeanEmpirical, LeanEmpirical | LeanComplete)
            | (LeanComplete, LeanComplete)
    )
}

fn strictly_below(left: AssuranceKindV2, right: AssuranceKindV2) -> bool {
    assurance_kind_leq(left, right) && !assurance_kind_leq(right, left)
}

pub fn lower_bound_frontier(left: AssuranceKindV2, right: AssuranceKindV2) -> Vec<AssuranceKindV2> {
    let supported =
        |candidate| assurance_kind_leq(candidate, left) && assurance_kind_leq(candidate, right);
    ALL_ASSURANCE_KINDS_V2
        .into_iter()
        .filter(|candidate| {
            supported(*candidate)
                && !ALL_ASSURANCE_KINDS_V2
                    .into_iter()
                    .any(|other| supported(other) && strictly_below(*candidate, other))
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct AntichainNf(u8);

impl AntichainNf {
    pub fn from_generators(generators: &[AssuranceKindV2]) -> Self {
        let mut bits = 0_u8;
        for (index, candidate) in ALL_ASSURANCE_KINDS_V2.into_iter().enumerate() {
            if generators
                .iter()
                .any(|generator| assurance_kind_leq(candidate, *generator))
            {
                bits |= 1 << index;
            }
        }
        Self(bits)
    }

    const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    const fn empty() -> Self {
        Self(0)
    }

    fn supports(self, kind: AssuranceKindV2) -> bool {
        let index = ALL_ASSURANCE_KINDS_V2
            .iter()
            .position(|candidate| *candidate == kind)
            .expect("closed V2 constructor family");
        self.0 & (1 << index) != 0
    }

    fn frontier(self) -> Vec<AssuranceKindV2> {
        ALL_ASSURANCE_KINDS_V2
            .into_iter()
            .filter(|candidate| {
                self.supports(*candidate)
                    && !ALL_ASSURANCE_KINDS_V2
                        .into_iter()
                        .any(|other| self.supports(other) && strictly_below(*candidate, other))
            })
            .collect()
    }
}

/// Exact source/build identity for one project population. Lists are required
/// to be sorted and duplicate-free so logically identical builds serialize
/// identically rather than depending on caller insertion order.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectBuildIdentityV2 {
    pub crate_name: String,
    pub target: String,
    pub features: Vec<String>,
    pub platform: String,
    pub generated_sources: Vec<String>,
    pub artifact_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProjectItemIdentityV2 {
    pub source_path: String,
    pub item_path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ProjectDispositionV2 {
    Accepted,
    NonClaim { reason: String },
    LegacyUnversioned { legacy_level: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectPopulationMemberV2 {
    pub identity: ProjectItemIdentityV2,
    pub disposition: ProjectDispositionV2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectPopulationV2 {
    pub build: ProjectBuildIdentityV2,
    /// Independent source/build inventory in source order.
    pub intended: Vec<ProjectItemIdentityV2>,
    /// Total closed partition in exactly the same order as `intended`.
    pub members: Vec<ProjectPopulationMemberV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionError {
    pub reason: String,
}

impl CompositionError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for CompositionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid assurance composition: {}", self.reason)
    }
}

impl std::error::Error for CompositionError {}

fn nonempty(value: &str, field: &str) -> Result<(), CompositionError> {
    if value.is_empty() {
        Err(CompositionError::new(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

fn canonical_strings(values: &[String], field: &str) -> Result<(), CompositionError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(CompositionError::new(format!(
            "{field} must be sorted and duplicate-free"
        )));
    }
    if values.iter().any(String::is_empty) {
        return Err(CompositionError::new(format!(
            "{field} must not contain an empty identity"
        )));
    }
    Ok(())
}

/// Validated executable address of the exact Lean `ClaimFiberKeyV2` cell.
///
/// Lean keeps predicates in the key directly. Executable aggregation cannot
/// serialize functions, so it accepts only a separately validated lowercase
/// SHA-256 address of that complete typed cell rather than an informal name.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClaimFiberAddressV2 {
    sha256: String,
}

impl ClaimFiberAddressV2 {
    pub fn new(sha256: String) -> Result<Self, CompositionError> {
        if sha256.len() != 64
            || !sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(CompositionError::new(
                "claim fiber address must be a lowercase SHA-256 digest",
            ));
        }
        Ok(Self { sha256 })
    }

    pub fn as_str(&self) -> &str {
        &self.sha256
    }
}

impl ProjectBuildIdentityV2 {
    fn validate(&self) -> Result<(), CompositionError> {
        nonempty(&self.crate_name, "crate_name")?;
        nonempty(&self.target, "target")?;
        nonempty(&self.platform, "platform")?;
        nonempty(&self.artifact_sha256, "artifact_sha256")?;
        canonical_strings(&self.features, "features")?;
        canonical_strings(&self.generated_sources, "generated_sources")
    }
}

impl ProjectItemIdentityV2 {
    fn validate(&self) -> Result<(), CompositionError> {
        nonempty(&self.source_path, "source_path")?;
        nonempty(&self.item_path, "item_path")
    }
}

impl ProjectPopulationV2 {
    pub fn derive(
        build: ProjectBuildIdentityV2,
        intended: Vec<ProjectItemIdentityV2>,
        members: Vec<ProjectPopulationMemberV2>,
    ) -> Result<Self, CompositionError> {
        build.validate()?;
        let mut identities = std::collections::BTreeSet::new();
        for identity in &intended {
            identity.validate()?;
            if !identities.insert(identity.clone()) {
                return Err(CompositionError::new(format!(
                    "duplicate intended item {}::{}",
                    identity.source_path, identity.item_path
                )));
            }
        }
        if intended.len() != members.len() {
            return Err(CompositionError::new(format!(
                "population partition has {} rows for {} intended items",
                members.len(),
                intended.len()
            )));
        }
        for (index, (expected, member)) in intended.iter().zip(&members).enumerate() {
            if &member.identity != expected {
                return Err(CompositionError::new(format!(
                    "population row {index} is not the source-ordered intended item"
                )));
            }
            match &member.disposition {
                ProjectDispositionV2::Accepted => {}
                ProjectDispositionV2::NonClaim { reason } => {
                    nonempty(reason, "non-claim reason")?;
                }
                ProjectDispositionV2::LegacyUnversioned { legacy_level } => {
                    nonempty(legacy_level, "legacy level")?;
                }
            }
        }
        Ok(Self {
            build,
            intended,
            members,
        })
    }

    pub fn accepted_identities(&self) -> Vec<ProjectItemIdentityV2> {
        self.members
            .iter()
            .filter(|member| matches!(member.disposition, ProjectDispositionV2::Accepted))
            .map(|member| member.identity.clone())
            .collect()
    }

    pub fn excluded_identities(&self) -> Vec<ProjectItemIdentityV2> {
        self.members
            .iter()
            .filter(|member| !matches!(member.disposition, ProjectDispositionV2::Accepted))
            .map(|member| member.identity.clone())
            .collect()
    }

    pub fn all_accepted(&self) -> bool {
        self.members
            .iter()
            .all(|member| matches!(member.disposition, ProjectDispositionV2::Accepted))
    }

    pub fn identity_digest(&self) -> String {
        sha256_domain(
            b"thermite-project-population-v2\0",
            &serde_json::to_vec(self).expect("validated population serializes"),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClauseClaimSetV2 {
    pub address: String,
    pub evidence_identity: String,
    /// None means this clause could not be transported into the candidate
    /// claim fiber. Its item claim set therefore becomes empty.
    pub transported_generators: Option<Vec<AssuranceKindV2>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioLiftV2 {
    pub item: ProjectItemIdentityV2,
    pub expected_clauses: Vec<String>,
    pub clauses: Vec<ClauseClaimSetV2>,
    pub connective_witness: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClaimProvenanceV2 {
    pub item: ProjectItemIdentityV2,
    pub clause_addresses: Vec<String>,
    pub evidence_identities: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ItemClaimSetV2 {
    item: ProjectItemIdentityV2,
    claim_fiber: ClaimFiberAddressV2,
    normal: AntichainNf,
    provenance: ClaimProvenanceV2,
    heterogeneous: bool,
}

impl PortfolioLiftV2 {
    pub fn new(
        item: ProjectItemIdentityV2,
        expected_clauses: Vec<String>,
        clauses: Vec<ClauseClaimSetV2>,
        connective_witness: String,
    ) -> Result<Self, CompositionError> {
        item.validate()?;
        nonempty(&connective_witness, "connective witness")?;
        if expected_clauses.is_empty() {
            return Err(CompositionError::new(
                "a heterogeneous portfolio must contain at least one clause",
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for address in &expected_clauses {
            nonempty(address, "expected clause address")?;
            if !seen.insert(address) {
                return Err(CompositionError::new(format!(
                    "duplicate expected clause {address}"
                )));
            }
        }
        let actual = clauses
            .iter()
            .map(|clause| clause.address.clone())
            .collect::<Vec<_>>();
        if actual != expected_clauses {
            return Err(CompositionError::new(
                "addressed clauses do not exactly equal the complete source-ordered inventory",
            ));
        }
        for clause in &clauses {
            nonempty(&clause.evidence_identity, "clause evidence identity")?;
        }
        Ok(Self {
            item,
            expected_clauses,
            clauses,
            connective_witness,
        })
    }

    pub fn item_claim_set(
        &self,
        claim_fiber: ClaimFiberAddressV2,
    ) -> Result<ItemClaimSetV2, CompositionError> {
        let normal = self
            .clauses
            .iter()
            .fold(None::<AntichainNf>, |aggregate, clause| {
                let clause_set = clause
                    .transported_generators
                    .as_deref()
                    .map(AntichainNf::from_generators)
                    .unwrap_or_else(AntichainNf::empty);
                Some(match aggregate {
                    None => clause_set,
                    Some(current) => current.intersect(clause_set),
                })
            });
        Ok(ItemClaimSetV2 {
            item: self.item.clone(),
            claim_fiber,
            normal: normal.expect("validated heterogeneous portfolio is nonempty"),
            provenance: ClaimProvenanceV2 {
                item: self.item.clone(),
                clause_addresses: self.expected_clauses.clone(),
                evidence_identities: self
                    .clauses
                    .iter()
                    .map(|clause| clause.evidence_identity.clone())
                    .collect(),
            },
            heterogeneous: true,
        })
    }
}

impl ItemClaimSetV2 {
    pub fn item(&self) -> &ProjectItemIdentityV2 {
        &self.item
    }

    pub fn claim_fiber(&self) -> &ClaimFiberAddressV2 {
        &self.claim_fiber
    }

    pub fn provenance(&self) -> &ClaimProvenanceV2 {
        &self.provenance
    }

    /// Rebind the certificate-local subject address to the exact source/build
    /// population identity used by an assurance report.  This changes neither
    /// the claim fiber nor the represented downset; it only replaces the
    /// provisional `certificate://current` identity minted by the item-local
    /// authority seam.  Report construction performs this rebasing before any
    /// project aggregation so a missing, duplicated, or reordered source item
    /// cannot be hidden behind a certificate-local name.
    pub fn rebase_item_identity(
        mut self,
        item: ProjectItemIdentityV2,
    ) -> Result<Self, CompositionError> {
        item.validate()?;
        self.item = item.clone();
        self.provenance.item = item;
        Ok(self)
    }

    /// Maximal generators of this exact downset. Consumers compare or impose
    /// floors through these V2 policy points; no scalar representative is
    /// invented for heterogeneous or incomparable conjunctions.
    pub fn frontier(&self) -> Vec<AssuranceKindV2> {
        self.normal.frontier()
    }

    pub fn homogeneous(
        item: ProjectItemIdentityV2,
        claim_fiber: ClaimFiberAddressV2,
        point: AssuranceKindV2,
        evidence_identity: String,
    ) -> Result<Self, CompositionError> {
        item.validate()?;
        nonempty(&evidence_identity, "evidence identity")?;
        Ok(Self {
            item: item.clone(),
            claim_fiber,
            normal: AntichainNf::from_generators(&[point]),
            provenance: ClaimProvenanceV2 {
                item,
                clause_addresses: Vec::new(),
                evidence_identities: vec![evidence_identity],
            },
            heterogeneous: false,
        })
    }

    /// Conjoin additional, independently evidenced premises into this item's
    /// claim set. Each premise contributes its downset, so conjunction is exact
    /// intersection rather than a minimum over a legacy scalar ladder.
    pub fn conjoin(
        mut self,
        premises: &[(AssuranceKindV2, String)],
    ) -> Result<Self, CompositionError> {
        for (kind, evidence_identity) in premises {
            nonempty(evidence_identity, "conjunct evidence identity")?;
            self.normal = self
                .normal
                .intersect(AntichainNf::from_generators(&[*kind]));
            self.provenance
                .evidence_identities
                .push(evidence_identity.clone());
            self.heterogeneous = true;
        }
        Ok(self)
    }

    /// A heterogeneous portfolio has no singular representative position.
    pub fn representative_position(&self) -> Result<AssuranceKindV2, CompositionError> {
        if self.heterogeneous {
            return Err(CompositionError::new(
                "heterogeneous item claim sets have no representative position",
            ));
        }
        self.normal
            .frontier()
            .into_iter()
            .next()
            .ok_or_else(|| CompositionError::new("empty claim set has no representative"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PopulationClaimSetTransportV2 {
    Transported(ItemClaimSetV2),
    Unavailable {
        item: ProjectItemIdentityV2,
        reason: String,
    },
}

impl PopulationClaimSetTransportV2 {
    fn item(&self) -> &ProjectItemIdentityV2 {
        match self {
            Self::Transported(claim_set) => &claim_set.item,
            Self::Unavailable { item, .. } => item,
        }
    }

    fn normal(&self) -> AntichainNf {
        match self {
            Self::Transported(claim_set) => claim_set.normal,
            Self::Unavailable { .. } => AntichainNf::empty(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceFrontierEntryV2 {
    pub kind: AssuranceKindV2,
    pub supporters: Vec<ClaimProvenanceV2>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CommonClaimFrontierV2 {
    NoItems,
    Frontier(Vec<AssuranceKindV2>),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnavailableTransportV2 {
    pub item: ProjectItemIdentityV2,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProjectPortraitScopeV2 {
    CompletePopulationFloor {
        members: usize,
    },
    AcceptedSubset {
        numerator: usize,
        denominator: usize,
        excluded: Vec<ProjectItemIdentityV2>,
    },
    WholeProject {
        project_lift_sha256: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectFrontiersV2 {
    population_sha256: String,
    claim_fiber: ClaimFiberAddressV2,
    /// Every accepted item whose claim set could not be transported into this
    /// fiber. Keeping the exact item and reason prevents the empty common
    /// frontier from hiding which per-item evidence is unavailable.
    unavailable_transports: Vec<UnavailableTransportV2>,
    common_claim_frontier: CommonClaimFrontierV2,
    evidence_frontier: Vec<EvidenceFrontierEntryV2>,
    scope: ProjectPortraitScopeV2,
}

impl ProjectFrontiersV2 {
    pub fn population_sha256(&self) -> &str {
        &self.population_sha256
    }

    pub fn claim_fiber(&self) -> &ClaimFiberAddressV2 {
        &self.claim_fiber
    }

    pub fn unavailable_transports(&self) -> &[UnavailableTransportV2] {
        &self.unavailable_transports
    }

    pub fn common_claim_frontier(&self) -> &CommonClaimFrontierV2 {
        &self.common_claim_frontier
    }

    pub fn evidence_frontier(&self) -> &[EvidenceFrontierEntryV2] {
        &self.evidence_frontier
    }

    pub fn scope(&self) -> &ProjectPortraitScopeV2 {
        &self.scope
    }

    pub fn aggregate(
        population: &ProjectPopulationV2,
        claim_fiber: ClaimFiberAddressV2,
        entries: Vec<PopulationClaimSetTransportV2>,
    ) -> Result<Self, CompositionError> {
        let accepted = population.accepted_identities();
        let actual = entries
            .iter()
            .map(|entry| entry.item().clone())
            .collect::<Vec<_>>();
        if actual != accepted {
            return Err(CompositionError::new(
                "project claim-set entries must exactly equal every accepted population member",
            ));
        }
        for entry in &entries {
            match entry {
                PopulationClaimSetTransportV2::Transported(claim_set) => {
                    if claim_set.claim_fiber != claim_fiber {
                        return Err(CompositionError::new(
                            "cross-fiber item claim set entered project aggregation",
                        ));
                    }
                }
                PopulationClaimSetTransportV2::Unavailable { reason, .. } => {
                    nonempty(reason, "unavailable transport reason")?;
                }
            }
        }
        let common_claim_frontier = if entries.is_empty() {
            CommonClaimFrontierV2::NoItems
        } else {
            let common = entries
                .iter()
                .map(PopulationClaimSetTransportV2::normal)
                .reduce(AntichainNf::intersect)
                .expect("nonempty entries have a common intersection");
            CommonClaimFrontierV2::Frontier(common.frontier())
        };
        let evidence = entries
            .iter()
            .map(PopulationClaimSetTransportV2::normal)
            .fold(AntichainNf::empty(), AntichainNf::union);
        let evidence_frontier = evidence
            .frontier()
            .into_iter()
            .map(|kind| EvidenceFrontierEntryV2 {
                kind,
                supporters: entries
                    .iter()
                    .filter_map(|entry| match entry {
                        PopulationClaimSetTransportV2::Transported(claim_set)
                            if claim_set.normal.supports(kind) =>
                        {
                            Some(claim_set.provenance.clone())
                        }
                        _ => None,
                    })
                    .collect(),
            })
            .collect();
        let unavailable_transports = entries
            .iter()
            .filter_map(|entry| match entry {
                PopulationClaimSetTransportV2::Unavailable { item, reason } => {
                    Some(UnavailableTransportV2 {
                        item: item.clone(),
                        reason: reason.clone(),
                    })
                }
                PopulationClaimSetTransportV2::Transported(_) => None,
            })
            .collect();
        let scope = if population.all_accepted() {
            ProjectPortraitScopeV2::CompletePopulationFloor {
                members: population.members.len(),
            }
        } else {
            ProjectPortraitScopeV2::AcceptedSubset {
                numerator: accepted.len(),
                denominator: population.intended.len(),
                excluded: population.excluded_identities(),
            }
        };
        Ok(Self {
            population_sha256: population.identity_digest(),
            claim_fiber,
            unavailable_transports,
            common_claim_frontier,
            evidence_frontier,
            scope,
        })
    }

    pub fn authorize_project(&mut self, lift: &ProjectLiftV2) -> Result<(), CompositionError> {
        if lift.population_sha256 != self.population_sha256 {
            return Err(CompositionError::new(
                "project lift belongs to a different source/build population",
            ));
        }
        if lift.frontiers_sha256 != self.digest() || lift.claim_fiber != self.claim_fiber {
            return Err(CompositionError::new(
                "project lift belongs to a different exact claim-fiber aggregation",
            ));
        }
        self.scope = ProjectPortraitScopeV2::WholeProject {
            project_lift_sha256: lift.digest(),
        };
        Ok(())
    }

    fn digest(&self) -> String {
        sha256_domain(
            b"thermite-project-frontiers-v2\0",
            &serde_json::to_vec(self).expect("validated project frontiers serialize"),
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LiftedItemEvidenceV2 {
    pub item: ProjectItemIdentityV2,
    pub evidence_identity: String,
    pub refutation_complete: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectLiftV2 {
    population_sha256: String,
    frontiers_sha256: String,
    claim_fiber: ClaimFiberAddressV2,
    project_lift_witness: String,
    pub evidence_vector: Vec<LiftedItemEvidenceV2>,
    pub finite_scheduler_identity: Option<String>,
}

impl ProjectLiftV2 {
    pub fn claim_fiber(&self) -> &ClaimFiberAddressV2 {
        &self.claim_fiber
    }

    pub fn validate_against(
        &self,
        population: &ProjectPopulationV2,
        frontiers: &ProjectFrontiersV2,
    ) -> Result<(), CompositionError> {
        if self.population_sha256 != population.identity_digest()
            || self.claim_fiber != frontiers.claim_fiber
            || frontiers.population_sha256 != self.population_sha256
            || !population.all_accepted()
            || !frontiers.unavailable_transports.is_empty()
        {
            return Err(CompositionError::new(
                "project lift does not match its checked population/frontier inputs",
            ));
        }
        nonempty(&self.project_lift_witness, "project lift witness")?;
        if let Some(scheduler) = self.finite_scheduler_identity.as_deref() {
            nonempty(scheduler, "finite scheduler identity")?;
        }
        if self
            .evidence_vector
            .iter()
            .map(|evidence| evidence.item.clone())
            .collect::<Vec<_>>()
            != population.intended
            || self
                .evidence_vector
                .iter()
                .any(|evidence| evidence.evidence_identity.is_empty())
        {
            return Err(CompositionError::new(
                "project lift evidence does not cover the exact population",
            ));
        }
        if !matches!(
            &frontiers.scope,
            ProjectPortraitScopeV2::WholeProject { project_lift_sha256 }
                if project_lift_sha256 == &self.digest()
        ) {
            return Err(CompositionError::new(
                "project frontier does not carry this project lift authorization",
            ));
        }
        let mut preauthorization = frontiers.clone();
        preauthorization.scope = ProjectPortraitScopeV2::CompletePopulationFloor {
            members: population.members.len(),
        };
        if self.frontiers_sha256 != preauthorization.digest() {
            return Err(CompositionError::new(
                "project lift does not bind the pre-authorization frontier",
            ));
        }
        Ok(())
    }

    pub fn new(
        population: &ProjectPopulationV2,
        frontiers: &ProjectFrontiersV2,
        evidence_vector: Vec<LiftedItemEvidenceV2>,
        project_lift_witness: String,
        finite_scheduler_identity: Option<String>,
    ) -> Result<Self, CompositionError> {
        if population.intended.is_empty() {
            return Err(CompositionError::new(
                "an empty NoItems population cannot acquire a vacuous whole-project lift",
            ));
        }
        if !population.all_accepted() {
            return Err(CompositionError::new(
                "whole-project lift requires an entirely accepted population",
            ));
        }
        if frontiers.population_sha256 != population.identity_digest() {
            return Err(CompositionError::new(
                "project lift uses frontiers from a different source/build population",
            ));
        }
        if !frontiers.unavailable_transports.is_empty() {
            return Err(CompositionError::new(
                "whole-project lift requires every accepted item transport",
            ));
        }
        nonempty(&project_lift_witness, "project lift witness")?;
        let actual = evidence_vector
            .iter()
            .map(|evidence| evidence.item.clone())
            .collect::<Vec<_>>();
        if actual != population.intended {
            return Err(CompositionError::new(
                "project evidence vector must cover the complete source-ordered population",
            ));
        }
        for evidence in &evidence_vector {
            nonempty(&evidence.evidence_identity, "item evidence identity")?;
        }
        if let Some(scheduler) = finite_scheduler_identity.as_deref() {
            nonempty(scheduler, "finite scheduler identity")?;
        }
        Ok(Self {
            population_sha256: population.identity_digest(),
            frontiers_sha256: frontiers.digest(),
            claim_fiber: frontiers.claim_fiber.clone(),
            project_lift_witness,
            evidence_vector,
            finite_scheduler_identity,
        })
    }

    /// One observed false conjunct soundly refutes the conjunction exactly
    /// when it belongs to the complete project evidence vector.
    pub fn false_conjunct_refutes(&self, item: &ProjectItemIdentityV2) -> bool {
        self.evidence_vector
            .iter()
            .any(|evidence| &evidence.item == item)
    }

    /// Completeness is separate from soundness: it needs a total finite
    /// scheduler and every item's own completeness premise.
    pub fn refutation_is_complete(&self) -> bool {
        self.finite_scheduler_identity.is_some()
            && self
                .evidence_vector
                .iter()
                .all(|evidence| evidence.refutation_complete)
    }

    fn digest(&self) -> String {
        sha256_domain(
            b"thermite-project-lift-v2\0",
            &serde_json::to_vec(self).expect("validated project lift serializes"),
        )
    }
}

/// A scalar minimum exists only for genuinely comparable points. Incomparable
/// policies must remain an antichain rather than be minimized per coordinate.
pub fn comparable_policy_minimum(
    left: AssuranceKindV2,
    right: AssuranceKindV2,
) -> Result<AssuranceKindV2, CompositionError> {
    if assurance_kind_leq(left, right) {
        Ok(left)
    } else if assurance_kind_leq(right, left) {
        Ok(right)
    } else {
        Err(CompositionError::new(
            "incomparable policies have no representative scalar minimum",
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct ReplayPair {
    left: AssuranceKindV2,
    right: AssuranceKindV2,
    left_le_right: bool,
    right_le_left: bool,
    lower_bound_frontier: Vec<AssuranceKindV2>,
}

#[derive(Deserialize)]
struct ReplayMatrix {
    version: u64,
    families: Vec<AssuranceKindV2>,
    pair: Vec<ReplayPair>,
}

fn replay_matrix() -> ReplayMatrix {
    serde_json::from_str(include_str!("../../gates/assurance-v2-replay.json"))
        .expect("checked AssurancePolicyV2 replay matrix")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_identity() -> ProjectBuildIdentityV2 {
        ProjectBuildIdentityV2 {
            crate_name: "demo".into(),
            target: "lib".into(),
            features: vec!["default".into()],
            platform: "x86_64-unknown-linux-gnu".into(),
            generated_sources: vec!["target/generated.rs".into()],
            artifact_sha256: "artifact-sha256".into(),
        }
    }

    fn item(name: &str) -> ProjectItemIdentityV2 {
        ProjectItemIdentityV2 {
            source_path: "src/lib.th".into(),
            item_path: format!("crate::{name}"),
        }
    }

    fn accepted_population(names: &[&str]) -> ProjectPopulationV2 {
        let intended = names.iter().map(|name| item(name)).collect::<Vec<_>>();
        let members = intended
            .iter()
            .cloned()
            .map(|identity| ProjectPopulationMemberV2 {
                identity,
                disposition: ProjectDispositionV2::Accepted,
            })
            .collect();
        ProjectPopulationV2::derive(build_identity(), intended, members).unwrap()
    }

    fn claim_fiber() -> ClaimFiberAddressV2 {
        ClaimFiberAddressV2::new("0".repeat(64)).unwrap()
    }

    fn homogeneous(item: ProjectItemIdentityV2, kind: AssuranceKindV2) -> ItemClaimSetV2 {
        ItemClaimSetV2::homogeneous(item, claim_fiber(), kind, "evidence".into()).unwrap()
    }

    #[test]
    fn generated_matrix_replays_every_constructor_pair() {
        let matrix = replay_matrix();
        assert_eq!(matrix.version, 2);
        assert_eq!(matrix.families, ALL_ASSURANCE_KINDS_V2);
        assert_eq!(matrix.pair.len(), 36);

        let actual = ALL_ASSURANCE_KINDS_V2
            .into_iter()
            .flat_map(|left| {
                ALL_ASSURANCE_KINDS_V2
                    .into_iter()
                    .map(move |right| ReplayPair {
                        left,
                        right,
                        left_le_right: assurance_kind_leq(left, right),
                        right_le_left: assurance_kind_leq(right, left),
                        lower_bound_frontier: lower_bound_frontier(left, right),
                    })
            })
            .collect::<Vec<_>>();
        assert_eq!(matrix.pair, actual);
    }

    #[test]
    fn complete_routes_collapse_only_in_presentation() {
        assert!(!assurance_kind_leq(
            AssuranceKindV2::SolverComplete,
            AssuranceKindV2::LeanEmpirical
        ));
        assert!(!assurance_kind_leq(
            AssuranceKindV2::LeanEmpirical,
            AssuranceKindV2::SolverComplete
        ));
        assert_eq!(
            lower_bound_frontier(
                AssuranceKindV2::SolverComplete,
                AssuranceKindV2::LeanEmpirical
            ),
            vec![AssuranceKindV2::SolverIncomplete]
        );
        assert!(assurance_kind_leq(
            AssuranceKindV2::SolverComplete,
            AssuranceKindV2::LeanComplete
        ));
        assert!(assurance_kind_leq(
            AssuranceKindV2::LeanEmpirical,
            AssuranceKindV2::LeanComplete
        ));
    }

    #[test]
    fn antichain_intersection_is_permutation_and_duplicate_invariant() {
        let normal_forms = (0_u8..64)
            .map(|mask| {
                let generators = ALL_ASSURANCE_KINDS_V2
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, kind)| (mask & (1 << index) != 0).then_some(kind))
                    .collect::<Vec<_>>();
                AntichainNf::from_generators(&generators)
            })
            .collect::<Vec<_>>();

        for &left in &normal_forms {
            assert_eq!(left.intersect(left), left);
            let frontier = left.frontier();
            assert!(frontier.windows(2).all(|pair| pair[0] < pair[1]));
            for &kind in &ALL_ASSURANCE_KINDS_V2 {
                if left.supports(kind) {
                    assert!(frontier
                        .iter()
                        .any(|upper| assurance_kind_leq(kind, *upper)));
                }
            }
            for &right in &normal_forms {
                assert_eq!(left.intersect(right), right.intersect(left));
                for &third in &normal_forms {
                    assert_eq!(
                        left.intersect(right).intersect(third),
                        left.intersect(right.intersect(third))
                    );
                    assert_eq!(
                        left.intersect(right).intersect(third),
                        third.intersect(left).intersect(right)
                    );
                }
            }
        }
    }

    #[test]
    fn project_population_is_an_exact_total_partition() {
        assert!(ClaimFiberAddressV2::new("informal-fiber-name".into())
            .unwrap_err()
            .reason
            .contains("lowercase SHA-256"));

        let intended = vec![item("a"), item("b")];
        let missing = vec![ProjectPopulationMemberV2 {
            identity: item("a"),
            disposition: ProjectDispositionV2::Accepted,
        }];
        let error =
            ProjectPopulationV2::derive(build_identity(), intended.clone(), missing).unwrap_err();
        assert!(error.reason.contains("1 rows for 2 intended"));

        let reordered = vec![
            ProjectPopulationMemberV2 {
                identity: item("b"),
                disposition: ProjectDispositionV2::Accepted,
            },
            ProjectPopulationMemberV2 {
                identity: item("a"),
                disposition: ProjectDispositionV2::Accepted,
            },
        ];
        assert!(
            ProjectPopulationV2::derive(build_identity(), intended.clone(), reordered)
                .unwrap_err()
                .reason
                .contains("source-ordered")
        );

        let duplicated = vec![item("a"), item("a")];
        let duplicate_rows = duplicated
            .iter()
            .cloned()
            .map(|identity| ProjectPopulationMemberV2 {
                identity,
                disposition: ProjectDispositionV2::Accepted,
            })
            .collect();
        assert!(
            ProjectPopulationV2::derive(build_identity(), duplicated, duplicate_rows)
                .unwrap_err()
                .reason
                .contains("duplicate intended item")
        );
    }

    #[test]
    fn portfolio_lift_requires_every_source_ordered_clause() {
        let clauses = vec![ClauseClaimSetV2 {
            address: "requires[0]".into(),
            evidence_identity: "req-evidence".into(),
            transported_generators: Some(vec![AssuranceKindV2::SolverComplete]),
        }];
        let error = PortfolioLiftV2::new(
            item("mixed"),
            vec!["requires[0]".into(), "ensures[0]".into()],
            clauses,
            "and-introduction".into(),
        )
        .unwrap_err();
        assert!(error.reason.contains("complete source-ordered inventory"));
    }

    #[test]
    fn mixed_incomparable_portfolio_is_an_intersection_not_a_representative() {
        let lift = PortfolioLiftV2::new(
            item("mixed"),
            vec!["requires[0]".into(), "ensures[0]".into()],
            vec![
                ClauseClaimSetV2 {
                    address: "requires[0]".into(),
                    evidence_identity: "solver-evidence".into(),
                    transported_generators: Some(vec![AssuranceKindV2::SolverComplete]),
                },
                ClauseClaimSetV2 {
                    address: "ensures[0]".into(),
                    evidence_identity: "lean-evidence".into(),
                    transported_generators: Some(vec![AssuranceKindV2::LeanEmpirical]),
                },
            ],
            "item-conjunction-v1".into(),
        )
        .unwrap();
        let claim_set = lift.item_claim_set(claim_fiber()).unwrap();
        assert_eq!(
            claim_set.normal.frontier(),
            vec![AssuranceKindV2::SolverIncomplete]
        );
        assert!(claim_set
            .representative_position()
            .unwrap_err()
            .reason
            .contains("no representative position"));
        assert!(comparable_policy_minimum(
            AssuranceKindV2::SolverComplete,
            AssuranceKindV2::LeanEmpirical
        )
        .unwrap_err()
        .reason
        .contains("no representative scalar minimum"));
    }

    #[test]
    fn unavailable_clause_and_item_transport_contribute_empty_not_omission() {
        let lift = PortfolioLiftV2::new(
            item("mixed"),
            vec!["requires[0]".into(), "ensures[0]".into()],
            vec![
                ClauseClaimSetV2 {
                    address: "requires[0]".into(),
                    evidence_identity: "req".into(),
                    transported_generators: Some(vec![AssuranceKindV2::SolverComplete]),
                },
                ClauseClaimSetV2 {
                    address: "ensures[0]".into(),
                    evidence_identity: "ens".into(),
                    transported_generators: None,
                },
            ],
            "item-conjunction-v1".into(),
        )
        .unwrap();
        assert!(lift
            .item_claim_set(claim_fiber())
            .unwrap()
            .normal
            .frontier()
            .is_empty());

        let population = accepted_population(&["a", "b"]);
        let input = vec![PopulationClaimSetTransportV2::Transported(homogeneous(
            item("a"),
            AssuranceKindV2::SolverComplete,
        ))];
        assert!(
            ProjectFrontiersV2::aggregate(&population, claim_fiber(), input)
                .unwrap_err()
                .reason
                .contains("every accepted population member")
        );

        let input = vec![
            PopulationClaimSetTransportV2::Transported(homogeneous(
                item("a"),
                AssuranceKindV2::SolverComplete,
            )),
            PopulationClaimSetTransportV2::Unavailable {
                item: item("b"),
                reason: "no checked transport".into(),
            },
        ];
        let portrait = ProjectFrontiersV2::aggregate(&population, claim_fiber(), input).unwrap();
        assert_eq!(
            portrait.common_claim_frontier,
            CommonClaimFrontierV2::Frontier(Vec::new())
        );
        assert_eq!(portrait.evidence_frontier.len(), 1);
        assert_eq!(
            portrait.evidence_frontier[0].kind,
            AssuranceKindV2::SolverComplete
        );
        assert_eq!(
            portrait.unavailable_transports,
            vec![UnavailableTransportV2 {
                item: item("b"),
                reason: "no checked transport".into(),
            }]
        );
        let evidence = vec![
            LiftedItemEvidenceV2 {
                item: item("a"),
                evidence_identity: "a-evidence".into(),
                refutation_complete: true,
            },
            LiftedItemEvidenceV2 {
                item: item("b"),
                evidence_identity: "b-evidence".into(),
                refutation_complete: true,
            },
        ];
        assert!(ProjectLiftV2::new(
            &population,
            &portrait,
            evidence,
            "project-lift-witness".into(),
            Some("finite-scheduler".into()),
        )
        .unwrap_err()
        .reason
        .contains("every accepted item transport"));
    }

    #[test]
    fn common_and_evidence_frontiers_answer_different_questions() {
        let population = accepted_population(&["solver", "lean"]);
        let input = vec![
            PopulationClaimSetTransportV2::Transported(homogeneous(
                item("solver"),
                AssuranceKindV2::SolverComplete,
            )),
            PopulationClaimSetTransportV2::Transported(homogeneous(
                item("lean"),
                AssuranceKindV2::LeanEmpirical,
            )),
        ];
        let portrait = ProjectFrontiersV2::aggregate(&population, claim_fiber(), input).unwrap();
        assert_eq!(
            portrait.common_claim_frontier,
            CommonClaimFrontierV2::Frontier(vec![AssuranceKindV2::SolverIncomplete])
        );
        assert_eq!(
            portrait
                .evidence_frontier
                .iter()
                .map(|entry| entry.kind)
                .collect::<Vec<_>>(),
            vec![
                AssuranceKindV2::SolverComplete,
                AssuranceKindV2::LeanEmpirical
            ]
        );
        assert!(portrait
            .evidence_frontier
            .iter()
            .all(|entry| entry.supporters.len() == 1));
        assert_eq!(
            portrait.scope,
            ProjectPortraitScopeV2::CompletePopulationFloor { members: 2 }
        );
    }

    #[test]
    fn nonclaim_or_legacy_population_is_only_subset_qualified() {
        let intended = vec![item("accepted"), item("legacy")];
        let members = vec![
            ProjectPopulationMemberV2 {
                identity: item("accepted"),
                disposition: ProjectDispositionV2::Accepted,
            },
            ProjectPopulationMemberV2 {
                identity: item("legacy"),
                disposition: ProjectDispositionV2::LegacyUnversioned {
                    legacy_level: "L3".into(),
                },
            },
        ];
        let population = ProjectPopulationV2::derive(build_identity(), intended, members).unwrap();
        let portrait = ProjectFrontiersV2::aggregate(
            &population,
            claim_fiber(),
            vec![PopulationClaimSetTransportV2::Transported(homogeneous(
                item("accepted"),
                AssuranceKindV2::SolverComplete,
            ))],
        )
        .unwrap();
        assert_eq!(
            portrait.scope,
            ProjectPortraitScopeV2::AcceptedSubset {
                numerator: 1,
                denominator: 2,
                excluded: vec![item("legacy")],
            }
        );
        assert!(ProjectLiftV2::new(
            &population,
            &portrait,
            Vec::new(),
            "project-lift-witness".into(),
            None,
        )
        .unwrap_err()
        .reason
        .contains("entirely accepted population"));
    }

    #[test]
    fn project_lift_requires_complete_evidence_and_separate_completeness_premises() {
        let population = accepted_population(&["a", "b"]);
        let mut portrait = ProjectFrontiersV2::aggregate(
            &population,
            claim_fiber(),
            vec![
                PopulationClaimSetTransportV2::Transported(homogeneous(
                    item("a"),
                    AssuranceKindV2::SolverComplete,
                )),
                PopulationClaimSetTransportV2::Transported(homogeneous(
                    item("b"),
                    AssuranceKindV2::SolverComplete,
                )),
            ],
        )
        .unwrap();
        let one = vec![LiftedItemEvidenceV2 {
            item: item("a"),
            evidence_identity: "a-evidence".into(),
            refutation_complete: true,
        }];
        assert!(ProjectLiftV2::new(
            &population,
            &portrait,
            one,
            "project-lift-witness".into(),
            Some("scheduler".into()),
        )
        .unwrap_err()
        .reason
        .contains("complete source-ordered population"));

        let evidence = vec![
            LiftedItemEvidenceV2 {
                item: item("a"),
                evidence_identity: "a-evidence".into(),
                refutation_complete: true,
            },
            LiftedItemEvidenceV2 {
                item: item("b"),
                evidence_identity: "b-evidence".into(),
                refutation_complete: false,
            },
        ];
        let no_scheduler = ProjectLiftV2::new(
            &population,
            &portrait,
            evidence.clone(),
            "project-lift-witness".into(),
            None,
        )
        .unwrap();
        assert!(!no_scheduler.refutation_is_complete());
        let missing_item_premise = ProjectLiftV2::new(
            &population,
            &portrait,
            evidence,
            "project-lift-witness".into(),
            Some("finite-scheduler".into()),
        )
        .unwrap();
        assert!(!missing_item_premise.refutation_is_complete());
        assert!(missing_item_premise.false_conjunct_refutes(&item("b")));
        assert!(!missing_item_premise.false_conjunct_refutes(&item("outside")));

        let complete_evidence = missing_item_premise
            .evidence_vector
            .iter()
            .cloned()
            .map(|mut evidence| {
                evidence.refutation_complete = true;
                evidence
            })
            .collect();
        let complete = ProjectLiftV2::new(
            &population,
            &portrait,
            complete_evidence,
            "project-lift-witness".into(),
            Some("finite-scheduler".into()),
        )
        .unwrap();
        assert!(complete.refutation_is_complete());

        let other_fiber = ClaimFiberAddressV2::new("1".repeat(64)).unwrap();
        let other_item_set = |identity| {
            ItemClaimSetV2::homogeneous(
                identity,
                other_fiber.clone(),
                AssuranceKindV2::SolverComplete,
                "other-evidence".into(),
            )
            .unwrap()
        };
        let mut other_fiber_portrait = ProjectFrontiersV2::aggregate(
            &population,
            other_fiber.clone(),
            vec![
                PopulationClaimSetTransportV2::Transported(other_item_set(item("a"))),
                PopulationClaimSetTransportV2::Transported(other_item_set(item("b"))),
            ],
        )
        .unwrap();
        assert!(other_fiber_portrait
            .authorize_project(&complete)
            .unwrap_err()
            .reason
            .contains("different exact claim-fiber aggregation"));
        portrait.authorize_project(&complete).unwrap();
        assert!(matches!(
            portrait.scope,
            ProjectPortraitScopeV2::WholeProject { .. }
        ));
    }

    #[test]
    fn empty_population_is_no_items_not_top() {
        let population = accepted_population(&[]);
        let portrait =
            ProjectFrontiersV2::aggregate(&population, claim_fiber(), Vec::new()).unwrap();
        assert_eq!(
            portrait.common_claim_frontier,
            CommonClaimFrontierV2::NoItems
        );
        assert!(portrait.evidence_frontier.is_empty());
        assert!(ProjectLiftV2::new(
            &population,
            &portrait,
            Vec::new(),
            "project-lift-witness".into(),
            None,
        )
        .unwrap_err()
        .reason
        .contains("cannot acquire a vacuous whole-project lift"));
    }

    fn formal_fixture() -> FormalAuthorityRecordV2 {
        FormalAuthorityRecordV2 {
            subject: "crate::f".into(),
            source_sha256: "source".into(),
            artifact_sha256: "artifact".into(),
            family: AssuranceKindV2::SolverComplete,
            semantics: "thermite-language".into(),
            semantics_version: 1,
            implementation_model: "rustc".into(),
            implementation_model_version: 1,
            fragment_lineage: "thermite-core".into(),
            fragment_revision: 2,
            population: PopulationKeyV2::AllInputs,
            claim_identity: "req-implies-ens".into(),
            observation_contract: "complete-countermodel".into(),
            refutation_contract: "complete".into(),
            classification_fragment: "thermite-core-v2".into(),
            classification_verdict: "admitted".into(),
            residual_context: "solver".into(),
            boundary: BoundaryKeyV2::EndToEnd,
            procedure: "bv".into(),
            procedure_version: 1,
            environment: "linux".into(),
            tool_version: "z3-4.13".into(),
            resource_budget: 100,
            residual_trust: "solver".into(),
            axioms: vec!["Classical.choice".into()],
            accepted_evidence_sha256: "evidence".into(),
            reconstruction_identity: "solver-proof".into(),
            composition_witnesses: vec!["item-lift".into()],
        }
    }

    #[test]
    fn authority_and_presentation_digests_have_disjoint_mutation_domains() {
        let formal = formal_fixture();
        let authority = authority_digest(&formal);
        let display = EngineerDisplayRecordV1 {
            claim: "proved_all_with_concrete_witness".into(),
            coverage: BoundaryKeyV2::EndToEnd,
        };
        let presentation = presentation_digest(&authority, 1, 1, &display);

        let mut changed_formal = formal.clone();
        changed_formal.accepted_evidence_sha256 = "spliced".into();
        assert_ne!(authority_digest(&changed_formal), authority);
        let mut changed_position = formal.clone();
        changed_position.family = AssuranceKindV2::LeanComplete;
        assert_ne!(authority_digest(&changed_position), authority);
        let mut changed_reconstruction = formal.clone();
        changed_reconstruction.reconstruction_identity = "spliced-kernel-replay".into();
        assert_ne!(authority_digest(&changed_reconstruction), authority);

        let changed_display = EngineerDisplayRecordV1 {
            claim: "changed_presentational_label".into(),
            coverage: BoundaryKeyV2::EndToEnd,
        };
        assert_eq!(authority_digest(&formal), authority);
        assert_ne!(
            presentation_digest(&authority, 1, 1, &changed_display),
            presentation
        );
        let changed_coverage = EngineerDisplayRecordV1 {
            claim: display.claim.clone(),
            coverage: BoundaryKeyV2::ToBoundary { via: "ffi".into() },
        };
        assert_ne!(
            presentation_digest(&authority, 1, 1, &changed_coverage),
            presentation
        );
        assert_ne!(
            presentation_digest(&authority, 2, 1, &display),
            presentation
        );
        assert_ne!(
            presentation_digest(&authority, 1, 2, &display),
            presentation
        );
    }
}
