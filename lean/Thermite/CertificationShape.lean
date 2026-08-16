import Thermite.CertificationPolicy

/-! Checked replacement of the candidate N5 policy shape. -/

namespace Thermite.CertificationMetatheory

def IsUpperBound (left right candidate : RepresentativePosition) : Prop :=
  RepresentativeLE left candidate ∧ RepresentativeLE right candidate

def IsLeastUpperBound (left right candidate : RepresentativePosition) : Prop :=
  IsUpperBound left right candidate ∧
    ∀ other, IsUpperBound left right other → RepresentativeLE candidate other

/-- The realizable solver and Lean branches have no join; this is the concrete
counterexample that rejects the proposed N5/lattice interpretation. -/
theorem solver_lean_have_no_realizable_join :
    ¬ ∃ candidate, IsLeastUpperBound .solverComplete .leanEmpirical candidate := by
  intro alleged
  rcases alleged with ⟨candidate, upper, _least⟩
  cases candidate <;>
    simp [IsUpperBound, RepresentativeLE, representativeLeq] at upper

/-- The selected checked domain has four realizable points, not the five points
required by N5. -/
theorem selected_domain_is_not_five_point :
    allRepresentativePositions.length ≠ 5 := by decide

/-- The replacement shape is a four-point fork with a shared bounded meet and
no common realizable upper bound. -/
theorem replacement_shape_witness :
    meetReport .solverComplete .leanEmpirical = .present .bounded ∧
      joinReport .solverComplete .leanEmpirical = .absent := by decide

end Thermite.CertificationMetatheory
