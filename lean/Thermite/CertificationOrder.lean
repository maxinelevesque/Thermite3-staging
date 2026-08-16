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

/-- Executable decision procedure for the four representative judgments.  Its
semantic adequacy is proved below; consumers do not treat this table as the
definition of the order. -/
def representativeLeq : RepresentativePosition → RepresentativePosition → Bool
  | .runtime, _ => true
  | .bounded, .bounded | .bounded, .solverComplete | .bounded, .leanEmpirical => true
  | .solverComplete, .solverComplete => true
  | .leanEmpirical, .leanEmpirical => true
  | _, _ => false

def runtimeBoundary : BoundaryContext := ⟨"runtime", fun _ => True⟩
def boundedBoundary : BoundaryContext :=
  ⟨"bounded", fun program => program.constructs.length ≤ 5⟩
def solverBoundary : BoundaryContext :=
  ⟨"solver-complete", fun program => program.constructs.length ≤ 2⟩
def leanEmpiricalBoundary : BoundaryContext :=
  ⟨"lean-empirical", fun program => program.constructs.length ≤ 3 ∧
    ∀ construct ∈ program.constructs, construct = "Fn"⟩

def representativeBoundary : RepresentativePosition → BoundaryContext
  | .runtime => runtimeBoundary
  | .bounded => boundedBoundary
  | .solverComplete => solverBoundary
  | .leanEmpirical => leanEmpiricalBoundary

def representativeFrame (position : RepresentativePosition) : SemanticFrame :=
  ⟨"thermite-language", 1, "neutral", 1, representativeBoundary position⟩

/-- The semantic validity obligation consumed by representative proof routes.
It is intentionally opaque at the neutral `Program` projection: this layer has
no canonical denotation from which to invent it.  A future production bridge
must supply proofs from the actual language semantics.  In particular, facts,
digest strings, and construct shape cannot reduce or construct this predicate. -/
opaque representativeSemanticValidity : SemanticValidity

/-- A solver certificate is a kernel-checked derivation of the exact semantic
shape represented by the solver-complete point.  It is deliberately not a
self-authenticating digest string. -/
structure SolverCertificate (program : Program) : Type where
  bounded : program.constructs.length ≤ 2
  derivation : ∀ construct ∈ program.constructs, construct = "SpecFn"
  valid : representativeSemanticValidity program

/-- The empirical Lean branch likewise carries a checked proof object for its
exact program rather than a nominal route label. -/
structure LeanCertificate (program : Program) : Type where
  bounded : program.constructs.length ≤ 3
  derivation : ∀ construct ∈ program.constructs, construct = "Fn"
  valid : representativeSemanticValidity program

/-- Route evidence is program-bound. Runtime/bounded receipts identify their
operational runs; proof-complete branches additionally require dependent,
kernel-checked certificates for this exact program. -/
structure RepresentativeEvidence where
  program : Program
  runtimeReceipt : String
  boundedReceipt : String
  solverCertificate : Option (SolverCertificate program)
  leanCertificate : Option (LeanCertificate program)

def certificateMatches (routePrefix certificate : String) (program : Program) : Bool :=
  certificate == routePrefix ++ program.digest

def runtimeEvidenceAccepted (evidence : RepresentativeEvidence) (program : Program) : Bool :=
  decide (evidence.program = program) &&
    certificateMatches "runtime:" evidence.runtimeReceipt program

def boundedEvidenceAccepted (evidence : RepresentativeEvidence) (program : Program) : Bool :=
  runtimeEvidenceAccepted evidence program &&
    certificateMatches "bounded:" evidence.boundedReceipt program &&
    decide (program.constructs.length ≤ 5)

def solverEvidenceAccepted (evidence : RepresentativeEvidence) (program : Program) : Bool :=
  boundedEvidenceAccepted evidence program &&
    evidence.solverCertificate.isSome

def leanEvidenceAccepted (evidence : RepresentativeEvidence) (program : Program) : Bool :=
  boundedEvidenceAccepted evidence program &&
    evidence.leanCertificate.isSome

def representativeEvidenceAccepted :
    RepresentativePosition → RepresentativeEvidence → Program → Bool
  | .runtime => runtimeEvidenceAccepted
  | .bounded => boundedEvidenceAccepted
  | .solverComplete => solverEvidenceAccepted
  | .leanEmpirical => leanEvidenceAccepted

/-- Acceptance at the solver-complete point exposes a dependent proof object
for this exact program; formatted receipt strings alone cannot establish it. -/
theorem solver_accepted_supplies_checked_derivation
    (evidence : RepresentativeEvidence) (program : Program)
    (accepted : solverEvidenceAccepted evidence program = true) :
    Nonempty (SolverCertificate program) := by
  simp [solverEvidenceAccepted, boundedEvidenceAccepted,
    runtimeEvidenceAccepted] at accepted
  have programBound : evidence.program = program := accepted.1.1.1.1
  subst program
  cases certificate : evidence.solverCertificate with
  | none => simp [certificate] at accepted
  | some checked => exact ⟨checked⟩

/-- Solver acceptance is load-bearing for the fixed semantic claim, regardless
of whether the certificate was produced by an external solver or directly in
the kernel. -/
theorem solver_accepted_proves_semantic_validity
    (evidence : RepresentativeEvidence) (program : Program)
    (accepted : solverEvidenceAccepted evidence program = true) :
    representativeSemanticValidity program := by
  let ⟨certificate⟩ := solver_accepted_supplies_checked_derivation evidence program accepted
  exact certificate.valid

/-- The claim at each point is the existence of accepted, program-bound route
evidence, not a display label or a trivially inhabited proposition. -/
def representativeClaim (position : RepresentativePosition) : Program → Prop :=
  fun program => ∃ evidence, representativeEvidenceAccepted position evidence program = true

/-- Every position in the executable order denotes a full indexed judgment,
not merely an enum value or display label. -/
def representativeJudgment (position : RepresentativePosition) :
    CertificationJudgment (representativeFrame position) noResiduals coreV2
      logicalProcedure (representativeClaim position) RepresentativeEvidence noObservation :=
  ⟨fun evidence program =>
    representativeEvidenceAccepted position evidence program = true ∧
      coreV2.admits program ∧ (representativeBoundary position).qualifies program⟩

/-- Semantic order: `right` refines `left` as full indexed judgments. -/
def RepresentativeLE (left right : RepresentativePosition) : Prop :=
  Refines (representativeJudgment right) (representativeJudgment left)

def boundaryRefinesOfDecision : ∀ {left right}, representativeLeq left right = true →
    BoundaryRefines (representativeBoundary right) (representativeBoundary left)
  | .runtime, _, _ => by
      intro _ _
      trivial
  | .bounded, .bounded, _ => fun _ qualified => qualified
  | .bounded, .solverComplete, _ => by
      intro _ qualified
      exact Nat.le_trans qualified (by decide)
  | .bounded, .leanEmpirical, _ => by
      intro _ qualified
      exact Nat.le_trans qualified.1 (by decide)
  | .solverComplete, .solverComplete, _ => fun _ qualified => qualified
  | .leanEmpirical, .leanEmpirical, _ => fun _ qualified => qualified
  | .bounded, .runtime, accepted
  | .solverComplete, .runtime, accepted
  | .solverComplete, .bounded, accepted
  | .solverComplete, .leanEmpirical, accepted
  | .leanEmpirical, .runtime, accepted
  | .leanEmpirical, .bounded, accepted
  | .leanEmpirical, .solverComplete, accepted => by
      simp [representativeLeq] at accepted

theorem decision_implies_representative_refinement : ∀ {left right},
    representativeLeq left right = true → RepresentativeLE left right := by
  intro left right accepted
  exact ⟨{
    reindexProgram := id
    translateEvidence := id
    frame := ⟨rfl, rfl, rfl, rfl, boundaryRefinesOfDecision accepted⟩
    procedure := rfl
    context := fun held => held
    membership := fun _ admitted => admitted
    claim := by
      intro program claim
      rcases claim with ⟨evidence, acceptedEvidence⟩
      exact ⟨evidence, by
        cases left <;> cases right <;>
          simp_all [representativeLeq, representativeEvidenceAccepted,
            runtimeEvidenceAccepted, boundedEvidenceAccepted,
            solverEvidenceAccepted, leanEvidenceAccepted] <;>
          exact acceptedEvidence.2⟩
    observation := fun _ _ => trivial
    certification := by
      intro evidence program certified
      refine ⟨?_, certified.2.1,
        boundaryRefinesOfDecision accepted program certified.2.2⟩
      cases left <;> cases right <;>
        simp_all [representativeJudgment, representativeLeq,
          representativeEvidenceAccepted,
          runtimeEvidenceAccepted, boundedEvidenceAccepted,
          solverEvidenceAccepted, leanEvidenceAccepted] <;>
        exact certified.1.2
  }⟩

def runtimeOnlyWitness : Program :=
  ⟨"runtime-only", ["Fn", "Fn", "Fn", "Fn", "Fn", "Fn"], []⟩
def boundedOnlyWitness : Program :=
  ⟨"bounded-only", ["SpecFn", "SpecFn", "SpecFn", "SpecFn"], []⟩
def solverOnlyWitness : Program := ⟨"solver-only", ["SpecFn"], []⟩
def leanOnlyWitness : Program := ⟨"lean-only", ["Fn", "Fn", "Fn"], []⟩

theorem refinement_implies_decision : ∀ {left right},
    RepresentativeLE left right → representativeLeq left right = true := by
  intro left right refined
  rcases refined with ⟨refined⟩
  cases left <;> cases right <;> try rfl
  · have := refined.frame.boundary runtimeOnlyWitness trivial
    simp [representativeFrame, representativeBoundary, boundedBoundary,
      runtimeOnlyWitness] at this
  · have := refined.frame.boundary runtimeOnlyWitness trivial
    simp [representativeFrame, representativeBoundary, solverBoundary,
      runtimeOnlyWitness] at this
  · have := refined.frame.boundary boundedOnlyWitness (by
      simp [representativeFrame, representativeBoundary, boundedBoundary,
        boundedOnlyWitness])
    simp [representativeFrame, representativeBoundary, solverBoundary,
      boundedOnlyWitness] at this
  · have := refined.frame.boundary leanOnlyWitness (by
      simp [representativeFrame, representativeBoundary, leanEmpiricalBoundary,
        leanOnlyWitness])
    simp [representativeFrame, representativeBoundary, solverBoundary,
      leanOnlyWitness] at this
  · have := refined.frame.boundary runtimeOnlyWitness trivial
    simp [representativeFrame, representativeBoundary, leanEmpiricalBoundary,
      runtimeOnlyWitness] at this
  · have := refined.frame.boundary boundedOnlyWitness (by
      simp [representativeFrame, representativeBoundary, boundedBoundary,
        boundedOnlyWitness])
    simp [representativeFrame, representativeBoundary, leanEmpiricalBoundary,
      boundedOnlyWitness] at this
  · have := refined.frame.boundary solverOnlyWitness (by
      simp [representativeFrame, representativeBoundary, solverBoundary,
        solverOnlyWitness])
    simp [representativeFrame, representativeBoundary, leanEmpiricalBoundary,
      solverOnlyWitness] at this

theorem representative_decision_iff_refines : ∀ left right,
    representativeLeq left right = true ↔ RepresentativeLE left right := by
  intro left right
  exact ⟨decision_implies_representative_refinement,
    refinement_implies_decision⟩

theorem representative_le_refl : ∀ position, RepresentativeLE position position := by
  intro position
  exact decision_implies_representative_refinement (by cases position <;> rfl)

theorem representative_le_antisymm : ∀ {left right},
    RepresentativeLE left right → RepresentativeLE right left → left = right := by
  intro left right forward reverse
  have f := refinement_implies_decision forward
  have r := refinement_implies_decision reverse
  cases left <;> cases right <;> simp [representativeLeq] at f r ⊢

theorem representative_le_trans : ∀ {first second third},
    RepresentativeLE first second → RepresentativeLE second third →
      RepresentativeLE first third := by
  intro first second third firstSecond secondThird
  exact refines_trans secondThird firstSecond

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
