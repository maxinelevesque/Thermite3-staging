//! Opaque checked-lowering evidence for the homogeneous general-Verus route.
//!
//! Classification belongs before discharge: this module binds the isolated
//! Verus source, target item, declared effect row, and stable fragment identity
//! in one value before the solver runs. Certificate assembly may inspect but
//! cannot independently author those facts.

use sha2::{Digest, Sha256};
use thermite_syntax::{EffectRow, Item, Program};

use crate::lower::LowerError;

/// Checked pre-discharge evidence for one general-Verus query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L3Artifact {
    source: String,
    item: String,
    effect_row: Option<EffectRow>,
    query_identity: String,
    classifier_fragment: &'static str,
    resource_witness: Option<crate::ResourceFlowWitness>,
}

impl L3Artifact {
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn item(&self) -> &str {
        &self.item
    }

    /// Executable functions retain their exact declared row. Other item kinds
    /// use Forge's existing canonical `pure` projection.
    pub fn effect_row(&self) -> Option<&EffectRow> {
        self.effect_row.as_ref()
    }

    pub fn query_identity(&self) -> &str {
        &self.query_identity
    }

    pub fn classifier_fragment(&self) -> &'static str {
        self.classifier_fragment
    }

    pub fn resource_witness(&self) -> Option<&crate::ResourceFlowWitness> {
        self.resource_witness.as_ref()
    }
}

/// Lower one already-isolated item program and bind its Verus classifier and
/// query identity before execution.
pub fn lower_l3_artifact(program: &Program, item: &str) -> Result<L3Artifact, LowerError> {
    let checked = crate::checked::require_checked(program)?;
    let source_program = checked.source();
    let routed = source_program
        .items
        .iter()
        .find(|candidate| candidate.name() == item)
        .ok_or_else(|| LowerError::Unsupported {
            what: format!("L3 artifact item `{item}` is absent from the isolated program"),
            span: crate::l1::zero_span(),
        })?;
    let effect_row = match routed {
        Item::Fn(function) => Some(function.contract.effects.clone()),
        _ => None,
    };
    let source = crate::lower::lower(source_program)?;
    let digest = format!("{:x}", Sha256::digest(source.as_bytes()));
    let resource_witness = crate::checked::first_rfc11_span(source_program)
        .is_some()
        .then(|| crate::emit_resource_witness(&checked));
    let query_identity = if let Some(resource) = &resource_witness {
        format!(
            "thermite-verus-query-v1:{item}:sha256:{digest}:resource-sha256:{}",
            resource.checked_resource_sha256
        )
    } else {
        format!("thermite-verus-query-v1:{item}:sha256:{digest}")
    };
    Ok(L3Artifact {
        source,
        item: item.to_string(),
        effect_row,
        query_identity,
        classifier_fragment: "thermite-verus-v1",
        resource_witness,
    })
}
