import Thermite.CertificationOrder

/-! Negative-space pins for RFC-3 metatheory AC-4. -/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

/-- A syntactically solver-shaped program with no premises is not semantically
valid: the all-false interpretation is a concrete countermodel. -/
def neverSolvedProgram : Program := ⟨"never-solved", ["SpecFn"], []⟩

theorem never_solved_shape_is_not_solver_certificate :
    ¬ representativeSemanticValidity neverSolvedProgram := by
  intro valid
  have refuted := valid (fun _ => false) (by simp [neverSolvedProgram])
  have := refuted "SpecFn" (by simp [neverSolvedProgram])
  contradiction

/-- The two realizable upper branches must not be collapsed into an order. -/
theorem incomparable_branches_remain_incomparable :
    ¬ RepresentativeLE .solverComplete .leanEmpirical ∧
      ¬ RepresentativeLE .leanEmpirical .solverComplete := by
  constructor <;> intro refined <;>
    have decision := refinement_implies_decision refined <;>
    simp [representativeLeq] at decision

/-- A policy-only top must not be invented to make the realizable probe a
lattice. -/
theorem invented_join_rejected :
    joinReport .solverComplete .leanEmpirical ≠ .present .solverComplete ∧
      joinReport .solverComplete .leanEmpirical ≠ .present .leanEmpirical := by decide

end Thermite.CertificationMetatheory
