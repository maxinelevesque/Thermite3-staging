//! Commutation facts computed from effect-theory instances and operation sets.
//!
//! Governing design: `.design/syntax/effect-algebra.md` REQ-8. The result is
//! available for RFC-9's future conflict rule and does not alter effect-row
//! subsumption.
//!
//! ## REQ status
//!
//! | REQ | Status | Evidence |
//! |---|---|---|
//! | REQ-8 | SHIPPED | [`commutes`] computes the conservative meet over every summand pair. |
//! | REQ-10 | SHIPPED | [`concurrent_conflicts`] checks every unordered root pair over supplied transitive footprints. |
//! | REQ-14 | SHIPPED | [`effects_commute`] is the production region-to-algebra consumer and missing footprints fail closed. |

use std::collections::{BTreeMap, BTreeSet};

use thermite_syntax::effect_basis::{entry_for_effect, BasisEntry, Entry, Operation, Theory};
use thermite_syntax::{Effect, Item, Program, RegionPath};

use crate::regions::{effect_path, RegionIndex};

/// A computed commutation fact. There is no `Unknown` promotion to `Accept`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Commutation {
    Accept,
    Reject,
}

/// One deterministic RFC-9 rejection from a named concurrent composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrentConflict {
    pub composition: String,
    pub left_root: String,
    pub right_root: String,
    pub left_effect: Effect,
    pub right_effect: Effect,
    /// The shorter path when two explicit regions overlap by ancestry.
    pub overlap: Option<RegionPath>,
}

/// Check every unordered pair of roots using already-inferred transitive
/// footprints. The lowerer owns inference; this module remains the sole owner
/// of the commutation decision. Missing footprints fail closed by returning a
/// named resolution error rather than treating the root as pure.
pub fn concurrent_conflicts(
    program: &Program,
    regions: &RegionIndex,
    footprints: &BTreeMap<String, BTreeSet<Effect>>,
) -> Result<Vec<ConcurrentConflict>, String> {
    let mut conflicts = Vec::new();
    for item in &program.items {
        let Item::Concurrent(composition) = item else {
            continue;
        };
        for left_index in 0..composition.roots.len() {
            for right_index in left_index + 1..composition.roots.len() {
                let left_root = &composition.roots[left_index];
                let right_root = &composition.roots[right_index];
                let left = footprints.get(left_root).ok_or_else(|| {
                    format!(
                        "concurrent composition `{}` has no inferred footprint for `{left_root}`",
                        composition.name
                    )
                })?;
                let right = footprints.get(right_root).ok_or_else(|| {
                    format!(
                        "concurrent composition `{}` has no inferred footprint for `{right_root}`",
                        composition.name
                    )
                })?;
                for left_effect in left {
                    for right_effect in right {
                        if effects_commute(left_effect, right_effect, regions)
                            == Commutation::Reject
                        {
                            conflicts.push(ConcurrentConflict {
                                composition: composition.name.clone(),
                                left_root: left_root.clone(),
                                right_root: right_root.clone(),
                                left_effect: left_effect.clone(),
                                right_effect: right_effect.clone(),
                                overlap: overlapping_ancestor(left_effect, right_effect, regions),
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(conflicts)
}

fn overlapping_ancestor(
    left: &Effect,
    right: &Effect,
    regions: &RegionIndex,
) -> Option<RegionPath> {
    let (Some(left), Some(right)) = (effect_path(left), effect_path(right)) else {
        return None;
    };
    if !regions.overlaps(left, right) {
        return None;
    }
    if left.segments.len() <= right.segments.len() {
        Some(left.clone())
    } else {
        Some(right.clone())
    }
}

/// RFC-9's production bridge from resolved region ancestry to the RFC-8
/// algebra. Operation commutation remains derived from basis entries here;
/// consumers do not carry a second read/write conflict table.
pub fn effects_commute(left: &Effect, right: &Effect, regions: &RegionIndex) -> Commutation {
    let overlap = match (effect_path(left), effect_path(right)) {
        (Some(left), Some(right)) => regions.overlaps(left, right),
        _ => false,
    };
    commutes_with_overlap(&entry_for_effect(left), &entry_for_effect(right), overlap)
}

/// Compute whether two basis entries commute.
///
/// Combinations use the conservative meet: every pair of summands must
/// commute. This function accepts basis entries only, so the unresolved
/// `GivenAtom::Random` convention cannot be reported as a computed fact.
pub fn commutes(left: &BasisEntry, right: &BasisEntry) -> Commutation {
    commutes_with_overlap(left, right, false)
}

/// Compute commutation after region resolution has established whether two
/// operations address equal/ancestrally-overlapping storage. Overlap makes two
/// state entries one theory instance even when their source paths differ; the
/// operation table remains owned here rather than duplicated by the consumer.
pub fn commutes_with_overlap(
    left: &BasisEntry,
    right: &BasisEntry,
    regions_overlap: bool,
) -> Commutation {
    let left_entries = entries(left);
    let right_entries = entries(right);
    if left_entries.iter().all(|left| {
        right_entries
            .iter()
            .all(|right| entries_commute(left, right, regions_overlap))
    }) {
        Commutation::Accept
    } else {
        Commutation::Reject
    }
}

fn entries(entry: &BasisEntry) -> &[Entry] {
    match entry {
        BasisEntry::Primitive(entry) => std::slice::from_ref(entry),
        BasisEntry::Combination(entries) => entries,
    }
}

fn entries_commute(left: &Entry, right: &Entry, regions_overlap: bool) -> bool {
    match (&left.theory, &right.theory) {
        (Theory::State(left_instance), Theory::State(right_instance)) => {
            (!regions_overlap && left_instance != right_instance)
                || state_operations_commute(left, right)
        }
        (Theory::Io(left_signature), Theory::Io(right_signature)) => {
            left_signature != right_signature
        }
        (Theory::Accrues(left_monoid), Theory::Accrues(right_monoid)) => {
            left_monoid != right_monoid
        }
        (Theory::Exception, Theory::Exception) | (Theory::Partiality, Theory::Partiality) => false,
        _ => true,
    }
}

fn state_operations_commute(left: &Entry, right: &Entry) -> bool {
    !left.operations.contains(&Operation::Put) && !right.operations.contains(&Operation::Put)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use thermite_syntax::{effect_basis::entry_for_effect, Effect};

    #[test]
    fn state_table_is_computed_from_instances_and_operations() {
        let read_r = entry_for_effect(&Effect::Read("r".into()));
        let write_r = entry_for_effect(&Effect::Write("r".into()));
        let write_s = entry_for_effect(&Effect::Write("s".into()));

        assert_eq!(commutes(&read_r, &read_r), Commutation::Accept);
        assert_eq!(commutes(&read_r, &write_r), Commutation::Reject);
        assert_eq!(commutes(&write_r, &write_r), Commutation::Reject);
        assert_eq!(commutes(&write_r, &write_s), Commutation::Accept);
    }

    #[test]
    fn free_io_rejects_same_instance_and_accepts_independent_instances() {
        let io = |signature: &str| {
            BasisEntry::Primitive(Entry {
                theory: Theory::Io(signature.into()),
                operations: BTreeSet::from([Operation::InvokeIo]),
            })
        };
        let io_a = io("sigma_a");
        let io_b = io("sigma_b");

        assert_eq!(commutes(&io_a, &io_a), Commutation::Reject);
        assert_eq!(commutes(&io_a, &io_b), Commutation::Accept);
    }

    #[test]
    fn combination_uses_the_conservative_meet() {
        let net = entry_for_effect(&Effect::Net("socket".into()));
        let read = entry_for_effect(&Effect::Read("socket".into()));

        assert_eq!(commutes(&net, &read), Commutation::Reject);
    }

    #[test]
    fn containment_overlap_uses_the_state_operation_fact() {
        let parent_write = entry_for_effect(&Effect::Write("scheduler".into()));
        let child_read = entry_for_effect(&Effect::Read("scheduler.runqueue".into()));

        assert_eq!(commutes(&parent_write, &child_read), Commutation::Accept);
        assert_eq!(
            commutes_with_overlap(&parent_write, &child_read, true),
            Commutation::Reject
        );
    }
}
