import Thermite.CertificationPolicy

/-! Negative-space pin for RFC-3 metatheory AC-5. -/

namespace Thermite.CertificationMetatheory

/-- An unsound collapse maps the concrete solver branch onto the Lean branch. -/
def mutantAbstractPosition : RepresentativePosition → PolicyPoint
  | .solverComplete => .leanEmpirical
  | position => abstractPosition position

def mutantFloorAllows (actual floor : RepresentativePosition) : Bool :=
  policyLeq (mutantAbstractPosition floor) (mutantAbstractPosition actual)

/-- The mutant accepts solver-complete against a Lean-empirical floor although
the concrete positions are incomparable. -/
theorem unsound_policy_collapse_rejected :
    mutantFloorAllows .solverComplete .leanEmpirical = true ∧
      ¬ RepresentativeLE .leanEmpirical .solverComplete := by
  constructor <;> decide

end Thermite.CertificationMetatheory
