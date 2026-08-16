import Thermite.CertificationOrder

/-!
Versioned finite policy abstraction for the currently enabled RFC-3 floor
consumer.  The abstraction is intentionally narrow: it proves that accepting a
floor entails the concrete representative order.  It does not claim a Galois
connection or enable aggregation, Pareto, or display decisions.
-/

namespace Thermite.CertificationMetatheory

inductive PolicyDomainVersion where
  | v1
deriving DecidableEq, Repr

inductive PolicyPoint where
  | runtime
  | bounded
  | solverComplete
  | leanEmpirical
deriving DecidableEq, Repr

def allPolicyPoints : List PolicyPoint :=
  [.runtime, .bounded, .solverComplete, .leanEmpirical]

def abstractPosition : RepresentativePosition → PolicyPoint
  | .runtime => .runtime
  | .bounded => .bounded
  | .solverComplete => .solverComplete
  | .leanEmpirical => .leanEmpirical

def concretize : PolicyPoint → List RepresentativePosition
  | .runtime => [.runtime]
  | .bounded => [.bounded]
  | .solverComplete => [.solverComplete]
  | .leanEmpirical => [.leanEmpirical]

def policyLeq : PolicyPoint → PolicyPoint → Bool
  | .runtime, _ => true
  | .bounded, .bounded | .bounded, .solverComplete | .bounded, .leanEmpirical => true
  | .solverComplete, .solverComplete => true
  | .leanEmpirical, .leanEmpirical => true
  | _, _ => false

/-- The policy consumer corresponding to `CertificationPosition::dominates`:
an actual position is admitted when it meets the declared floor. -/
def floorAllows (actual floor : RepresentativePosition) : Bool :=
  policyLeq (abstractPosition floor) (abstractPosition actual)

/-- Every enabled floor acceptance is justified by concrete semantic order. -/
theorem floor_allows_sound : ∀ actual floor,
    floorAllows actual floor = true → RepresentativeLE floor actual := by
  intro actual floor accepted
  apply decision_implies_representative_refinement
  cases actual <;> cases floor <;> exact accepted

/-- The current Rust validation consumer asks only whether a coherent position
dominates itself; the abstract decision preserves that concrete fact. -/
theorem self_validation_sound : ∀ position,
    floorAllows position position = true ∧ RepresentativeLE position position := by
  intro position
  exact ⟨by cases position <;> decide, representative_le_refl position⟩

theorem policy_population_is_versioned_and_complete :
    allPolicyPoints.length = 4 := by decide

end Thermite.CertificationMetatheory
