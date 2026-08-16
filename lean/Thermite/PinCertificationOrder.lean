import Thermite.CertificationOrder

/-! Negative-space pins for RFC-3 metatheory AC-4. -/

namespace Thermite.CertificationMetatheory

/-- The two realizable upper branches must not be collapsed into an order. -/
theorem incomparable_branches_remain_incomparable :
    representativeLeq .solverComplete .leanEmpirical = false ∧
      representativeLeq .leanEmpirical .solverComplete = false := by decide

/-- A policy-only top must not be invented to make the realizable probe a
lattice. -/
theorem invented_join_rejected :
    joinReport .solverComplete .leanEmpirical ≠ .present .solverComplete ∧
      joinReport .solverComplete .leanEmpirical ≠ .present .leanEmpirical := by decide

end Thermite.CertificationMetatheory
