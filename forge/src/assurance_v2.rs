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

/// Canonical replay model of the entire formal authority projection. This is
/// not yet a `Certificate` field: issue #56 performs that atomic authority
/// migration. Its shape pins what the future digest must cover.
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AntichainNf(u8);

impl AntichainNf {
    fn from_generators(generators: &[AssuranceKindV2]) -> Self {
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
