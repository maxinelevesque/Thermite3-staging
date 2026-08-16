import Thermite.CertificationMetatheory

/-!
Executable finite probe of the realizable certification sub-poset.

This is deliberately not a lattice instance.  It enumerates representative
full judgments, computes minimal upper and maximal lower bounds, and reports
missing or non-unique operations as data.
-/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

inductive RepresentativePosition where
  | runtime
  | bounded
  | solverComplete
  | leanEmpirical
deriving DecidableEq, Repr

def allRepresentativePositions : List RepresentativePosition :=
  [.runtime, .bounded, .solverComplete, .leanEmpirical]

/-- The realizable order has two incomparable upper branches and deliberately
contains no artificial point above both of them. -/
def representativeLeq : RepresentativePosition → RepresentativePosition → Bool
  | .runtime, _ => true
  | .bounded, .bounded | .bounded, .solverComplete | .bounded, .leanEmpirical => true
  | .solverComplete, .solverComplete => true
  | .leanEmpirical, .leanEmpirical => true
  | _, _ => false

def RepresentativeLE (left right : RepresentativePosition) : Prop :=
  representativeLeq left right = true

instance representativeLEDecidable (left right : RepresentativePosition) :
    Decidable (RepresentativeLE left right) := by
  unfold RepresentativeLE
  infer_instance

theorem representative_le_refl : ∀ position, RepresentativeLE position position := by
  intro position
  cases position <;> decide

theorem representative_le_antisymm : ∀ {left right},
    RepresentativeLE left right → RepresentativeLE right left → left = right := by
  intro left right
  cases left <;> cases right <;> decide

theorem representative_le_trans : ∀ {first second third},
    RepresentativeLE first second → RepresentativeLE second third →
      RepresentativeLE first third := by
  intro first second third
  cases first <;> cases second <;> cases third <;> decide

def representativeFrame : RepresentativePosition → SemanticFrame
  | .leanEmpirical => endToEndFrame
  | _ => platformFrame

def representativeClaim : RepresentativePosition → Program → Prop
  | .runtime => fun _ => True
  | .bounded => BoundedScope 5
  | .solverComplete => coreV2.admits
  | .leanEmpirical => coreV2.admits

/-- Every position in the executable order denotes a full indexed judgment,
not merely an enum value or display label. -/
def representativeJudgment (position : RepresentativePosition) :
    CertificationJudgment (representativeFrame position) noResiduals coreV2
      logicalProcedure (representativeClaim position) Unit noObservation :=
  ⟨fun _ program => coreV2.admits program ∧ representativeClaim position program⟩

def upperBounds (left right : RepresentativePosition) : List RepresentativePosition :=
  allRepresentativePositions.filter fun candidate =>
    representativeLeq left candidate && representativeLeq right candidate

def lowerBounds (left right : RepresentativePosition) : List RepresentativePosition :=
  allRepresentativePositions.filter fun candidate =>
    representativeLeq candidate left && representativeLeq candidate right

def minimalUpperBounds (left right : RepresentativePosition) : List RepresentativePosition :=
  (upperBounds left right).filter fun candidate =>
    !(upperBounds left right).any fun other =>
      representativeLeq other candidate && !(representativeLeq candidate other)

def maximalLowerBounds (left right : RepresentativePosition) : List RepresentativePosition :=
  (lowerBounds left right).filter fun candidate =>
    !(lowerBounds left right).any fun other =>
      representativeLeq candidate other && !(representativeLeq other candidate)

inductive OperationReport where
  | present (position : RepresentativePosition)
  | absent
  | nonUnique (positions : List RepresentativePosition)
deriving DecidableEq, Repr

def reportCandidates : List RepresentativePosition → OperationReport
  | [] => .absent
  | [position] => .present position
  | positions => .nonUnique positions

def joinReport (left right : RepresentativePosition) : OperationReport :=
  reportCandidates (minimalUpperBounds left right)

def meetReport (left right : RepresentativePosition) : OperationReport :=
  reportCandidates (maximalLowerBounds left right)

structure PairOperationReport where
  left : RepresentativePosition
  right : RepresentativePosition
  join : OperationReport
  meet : OperationReport
deriving DecidableEq, Repr

def representativeOperationMatrix : List PairOperationReport :=
  allRepresentativePositions.flatMap fun left =>
    allRepresentativePositions.map fun right =>
      ⟨left, right, joinReport left right, meetReport left right⟩

theorem representative_population_is_complete :
    allRepresentativePositions.length = 4 := by decide

theorem incomparable_branches_have_no_join :
    joinReport .solverComplete .leanEmpirical = .absent := by decide

theorem incomparable_branches_have_bounded_meet :
    meetReport .solverComplete .leanEmpirical = .present .bounded := by decide

theorem operation_matrix_covers_all_pairs :
    representativeOperationMatrix.length = 16 := by decide

end Thermite.CertificationMetatheory
