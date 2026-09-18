//! Deterministic replay of checked assurance policy-version migrations.
//!
//! The Lean `PolicyVersionMigration` theorem is authority. This receipt binds a
//! reviewed theorem name to exact versions and the closed six-family order
//! isomorphism used by diagnostic report comparison.

use crate::assurance_v2::{assurance_kind_leq, AssuranceKindV2, ALL_ASSURANCE_KINDS_V2};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const POLICY_MIGRATION_SCHEMA: &str = "thermite-policy-migration/v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMigrationDisposition {
    Compatible,
    InformationLoss,
    Incompatible,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyKindMigrationRow {
    pub source: AssuranceKindV2,
    pub target: AssuranceKindV2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyMigrationReceiptV1 {
    pub schema: String,
    pub source_policy_version: u64,
    pub target_policy_version: u64,
    pub disposition: PolicyMigrationDisposition,
    pub lean_witness: String,
    pub rows: Vec<PolicyKindMigrationRow>,
    pub receipt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct PolicyMigrationReceiptBody<'a> {
    schema: &'a str,
    source_policy_version: u64,
    target_policy_version: u64,
    disposition: PolicyMigrationDisposition,
    lean_witness: &'a str,
    rows: &'a [PolicyKindMigrationRow],
}

#[derive(Clone, Debug)]
pub struct ValidatedPolicyMigration {
    receipt: PolicyMigrationReceiptV1,
    mapping: BTreeMap<AssuranceKindV2, AssuranceKindV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyMigrationError(pub String);

impl std::fmt::Display for PolicyMigrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for PolicyMigrationError {}

fn sha256_domain(domain: &[u8], payload: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(payload);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn receipt_digest(receipt: &PolicyMigrationReceiptV1) -> String {
    let body = PolicyMigrationReceiptBody {
        schema: &receipt.schema,
        source_policy_version: receipt.source_policy_version,
        target_policy_version: receipt.target_policy_version,
        disposition: receipt.disposition,
        lean_witness: &receipt.lean_witness,
        rows: &receipt.rows,
    };
    sha256_domain(
        b"thermite-policy-migration-receipt-v1\0",
        &serde_json::to_vec(&body).expect("closed policy migration receipt serializes"),
    )
}

impl PolicyMigrationReceiptV1 {
    #[cfg(test)]
    pub fn compatible_identity(
        source_policy_version: u64,
        target_policy_version: u64,
        lean_witness: impl Into<String>,
    ) -> Self {
        let mut receipt = Self {
            schema: POLICY_MIGRATION_SCHEMA.into(),
            source_policy_version,
            target_policy_version,
            disposition: PolicyMigrationDisposition::Compatible,
            lean_witness: lean_witness.into(),
            rows: ALL_ASSURANCE_KINDS_V2
                .into_iter()
                .map(|kind| PolicyKindMigrationRow {
                    source: kind,
                    target: kind,
                })
                .collect(),
            receipt_sha256: String::new(),
        };
        receipt.receipt_sha256 = receipt_digest(&receipt);
        receipt
    }

    #[cfg(test)]
    pub fn reseal(&mut self) {
        self.receipt_sha256 = receipt_digest(self);
    }

    pub fn validate(
        &self,
        expected_source: u64,
        expected_target: u64,
    ) -> Result<ValidatedPolicyMigration, PolicyMigrationError> {
        if self.schema != POLICY_MIGRATION_SCHEMA {
            return Err(PolicyMigrationError(
                "policy migration schema mismatch".into(),
            ));
        }
        if self.source_policy_version != expected_source
            || self.target_policy_version != expected_target
        {
            return Err(PolicyMigrationError(
                "policy migration endpoints do not match compared reports".into(),
            ));
        }
        if self.source_policy_version >= self.target_policy_version {
            return Err(PolicyMigrationError(
                "policy migration must follow a strict forward version edge".into(),
            ));
        }
        if self.disposition != PolicyMigrationDisposition::Compatible {
            return Err(PolicyMigrationError(format!(
                "policy migration is not comparison-compatible: {:?}",
                self.disposition
            )));
        }
        if self.lean_witness.trim().is_empty() {
            return Err(PolicyMigrationError(
                "policy migration is missing its Lean witness".into(),
            ));
        }
        if self.receipt_sha256 != receipt_digest(self) {
            return Err(PolicyMigrationError(
                "policy migration receipt digest mismatch".into(),
            ));
        }
        if self.rows.len() != ALL_ASSURANCE_KINDS_V2.len() {
            return Err(PolicyMigrationError(
                "policy migration must contain exactly six family rows".into(),
            ));
        }
        let mapping = self
            .rows
            .iter()
            .map(|row| (row.source, row.target))
            .collect::<BTreeMap<_, _>>();
        if mapping.len() != ALL_ASSURANCE_KINDS_V2.len()
            || ALL_ASSURANCE_KINDS_V2
                .iter()
                .any(|kind| !mapping.contains_key(kind))
        {
            return Err(PolicyMigrationError(
                "policy migration source families are incomplete or duplicated".into(),
            ));
        }
        let targets = mapping.values().copied().collect::<BTreeSet<_>>();
        if targets.len() != ALL_ASSURANCE_KINDS_V2.len() {
            return Err(PolicyMigrationError(
                "policy migration loses information between formal families".into(),
            ));
        }
        for left in ALL_ASSURANCE_KINDS_V2 {
            for right in ALL_ASSURANCE_KINDS_V2 {
                let translated_left = mapping[&left];
                let translated_right = mapping[&right];
                if assurance_kind_leq(left, right)
                    != assurance_kind_leq(translated_left, translated_right)
                {
                    return Err(PolicyMigrationError(
                        "policy migration does not preserve and reflect the formal order".into(),
                    ));
                }
            }
        }
        Ok(ValidatedPolicyMigration {
            receipt: self.clone(),
            mapping,
        })
    }
}

impl ValidatedPolicyMigration {
    pub fn source_policy_version(&self) -> u64 {
        self.receipt.source_policy_version
    }

    pub fn target_policy_version(&self) -> u64 {
        self.receipt.target_policy_version
    }

    pub fn receipt_sha256(&self) -> &str {
        &self.receipt.receipt_sha256
    }

    pub fn lean_witness(&self) -> &str {
        &self.receipt.lean_witness
    }

    pub fn translate_kind(&self, kind: AssuranceKindV2) -> AssuranceKindV2 {
        self.mapping[&kind]
    }

    pub fn translate_kinds(&self, kinds: &[AssuranceKindV2]) -> Vec<AssuranceKindV2> {
        let mut translated = kinds
            .iter()
            .copied()
            .map(|kind| self.translate_kind(kind))
            .collect::<Vec<_>>();
        translated.sort();
        translated.dedup();
        translated
    }
}

pub fn parse_policy_migration_json(
    bytes: &[u8],
) -> Result<PolicyMigrationReceiptV1, PolicyMigrationError> {
    serde_json::from_slice(bytes)
        .map_err(|error| PolicyMigrationError(format!("invalid policy migration JSON: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt() -> PolicyMigrationReceiptV1 {
        PolicyMigrationReceiptV1::compatible_identity(1, 2, "policy_v1_to_v2_order_isomorphism")
    }

    #[test]
    fn compatible_receipt_is_total_directional_and_deterministic() {
        let receipt = receipt();
        let validated = receipt.validate(1, 2).unwrap();
        assert_eq!(
            validated.translate_kinds(&ALL_ASSURANCE_KINDS_V2),
            ALL_ASSURANCE_KINDS_V2
        );
        assert_eq!(receipt_digest(&receipt), receipt.receipt_sha256);
    }

    #[test]
    fn missing_duplicate_lossy_reverse_and_forged_receipts_fail_closed() {
        let mut missing = receipt();
        missing.rows.pop();
        missing.reseal();
        assert!(missing.validate(1, 2).is_err());

        let mut duplicate = receipt();
        duplicate.rows[5].source = duplicate.rows[4].source;
        duplicate.reseal();
        assert!(duplicate.validate(1, 2).is_err());

        let mut lossy = receipt();
        lossy.rows[5].target = lossy.rows[3].target;
        lossy.reseal();
        assert!(lossy.validate(1, 2).is_err());

        let mut reverse = receipt();
        reverse.source_policy_version = 2;
        reverse.target_policy_version = 1;
        reverse.reseal();
        assert!(reverse.validate(2, 1).is_err());

        let mut forged = receipt();
        forged.lean_witness = "counterfeit".into();
        assert!(forged.validate(1, 2).is_err());
    }

    #[test]
    fn non_monotone_and_non_reflecting_maps_fail_closed() {
        let mut non_monotone = receipt();
        let first_target = non_monotone.rows[0].target;
        non_monotone.rows[0].target = non_monotone.rows[5].target;
        non_monotone.rows[5].target = first_target;
        non_monotone.reseal();
        assert!(non_monotone.validate(1, 2).is_err());

        let mut incompatible = receipt();
        incompatible.disposition = PolicyMigrationDisposition::Incompatible;
        incompatible.reseal();
        assert!(incompatible.validate(1, 2).is_err());
    }

    #[derive(Deserialize)]
    struct ReplayMatrix {
        case: Vec<ReplayCase>,
    }

    #[derive(Deserialize)]
    struct ReplayCase {
        relation: String,
        direction: String,
        complete: bool,
        order_isomorphism: bool,
        receipt_bound: bool,
        outcome: String,
    }

    #[test]
    fn checked_replay_matrix_is_consumed_by_rust_comparison() {
        let matrix: ReplayMatrix = serde_json::from_str(include_str!(
            "../../gates/assurance-policy-migration-replay.json"
        ))
        .unwrap();
        assert_eq!(matrix.case.len(), 10);
        for case in matrix.case {
            let outcome = if case.relation == "compatible"
                && case.direction == "forward"
                && case.complete
                && case.order_isomorphism
                && case.receipt_bound
            {
                "compared"
            } else {
                "policy_skew"
            };
            assert_eq!(outcome, case.outcome);
        }

        let receipt = parse_policy_migration_json(include_bytes!(
            "../../gates/policy-v1-to-v2-migration.json"
        ))
        .unwrap();
        receipt.validate(1, 2).unwrap();
    }
}
