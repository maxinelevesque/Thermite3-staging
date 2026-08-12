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
//! | REQ-10 | NOT-STARTED | Deferred to future RFC-9 conflict-rule consumption. |
//! | REQ-14 | NOT-STARTED | Deferred to future RFC-9's production consumer. |

use thermite_syntax::effect_basis::{BasisEntry, Entry, Operation, Theory};

/// A computed commutation fact. There is no `Unknown` promotion to `Accept`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Commutation {
    Accept,
    Reject,
}

/// Compute whether two basis entries commute.
///
/// Combinations use the conservative meet: every pair of summands must
/// commute. This function accepts basis entries only, so the unresolved
/// `GivenAtom::Random` convention cannot be reported as a computed fact.
pub fn commutes(left: &BasisEntry, right: &BasisEntry) -> Commutation {
    let left_entries = entries(left);
    let right_entries = entries(right);
    if left_entries.iter().all(|left| {
        right_entries
            .iter()
            .all(|right| entries_commute(left, right))
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

fn entries_commute(left: &Entry, right: &Entry) -> bool {
    match (&left.theory, &right.theory) {
        (Theory::State(left_instance), Theory::State(right_instance)) => {
            left_instance != right_instance || state_operations_commute(left, right)
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
}
