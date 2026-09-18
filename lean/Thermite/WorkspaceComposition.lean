import Thermite.AssuranceComposition
import Mathlib.Data.Finset.Basic

/-!
# Workspace and build-matrix assurance composition

The build plan is an independent denominator.  Package, target, feature,
platform, generated-source, and source-item coordinates are fixed before any
project result is admitted.  A missing matrix result therefore remains an
explicit non-claim; it cannot disappear from the workspace conjunction.
-/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

structure WorkspaceBuildCoordinate where
  packageName : String
  target : String
  features : List String
  platform : String
  generatedSources : List String
deriving DecidableEq, Repr

structure WorkspaceMatrixPlan where
  coordinate : WorkspaceBuildCoordinate
  sourcePath : String
  intended : List ProjectItemIdentity
  intendedNodup : intended.Nodup

structure WorkspaceItemIdentity where
  matrix : WorkspaceBuildCoordinate
  sourcePath : String
  itemPath : String
deriving DecidableEq, Repr

def WorkspaceMatrixPlan.qualifiedItems (plan : WorkspaceMatrixPlan) :
    List WorkspaceItemIdentity :=
  plan.intended.map fun item =>
    ⟨plan.coordinate, item.sourcePath, item.itemPath⟩

inductive WorkspaceDisposition where
  | accepted
  | nonClaim (reason : String)
  | legacyUnversioned (legacyLevel : String)
deriving DecidableEq, Repr

structure WorkspacePopulationMember where
  identity : WorkspaceItemIdentity
  disposition : WorkspaceDisposition
deriving DecidableEq, Repr

/-- Exact matrix and item denominator. Matrix order is canonical in the
executable representation; the mathematical identity below is order-free. -/
structure WorkspacePopulation where
  workspaceSha256 : String
  matrices : List WorkspaceMatrixPlan
  coordinateNodup : (matrices.map WorkspaceMatrixPlan.coordinate).Nodup
  intended : List WorkspaceItemIdentity
  intendedExact : intended = matrices.flatMap WorkspaceMatrixPlan.qualifiedItems
  intendedNodup : intended.Nodup
  members : List WorkspacePopulationMember
  totalPartition : members.map WorkspacePopulationMember.identity = intended

def WorkspacePopulation.acceptedIdentities (population : WorkspacePopulation) :
    List WorkspaceItemIdentity :=
  (population.members.filter fun member =>
    match member.disposition with
    | .accepted => true
    | _ => false).map WorkspacePopulationMember.identity

def WorkspacePopulation.excludedIdentities (population : WorkspacePopulation) :
    List WorkspaceItemIdentity :=
  (population.members.filter fun member =>
    match member.disposition with
    | .accepted => false
    | _ => true).map WorkspacePopulationMember.identity

def WorkspacePopulation.allAccepted (population : WorkspacePopulation) : Bool :=
  population.members.all fun member =>
    match member.disposition with
    | .accepted => true
    | _ => false

theorem WorkspacePopulation.member_identities_are_exact
    (population : WorkspacePopulation) :
    population.members.map WorkspacePopulationMember.identity = population.intended :=
  population.totalPartition

theorem WorkspacePopulation.member_identities_are_nodup
    (population : WorkspacePopulation) :
    (population.members.map WorkspacePopulationMember.identity).Nodup := by
  rw [population.totalPartition]
  exact population.intendedNodup

/-- Canonical mathematical identity of a build matrix. `toFinset` makes the
identity independent of enumeration order and duplicate presentation. The
checked constructor separately rejects duplicate coordinates. -/
def canonicalWorkspaceCoordinates
    (coordinates : List WorkspaceBuildCoordinate) : Finset WorkspaceBuildCoordinate :=
  coordinates.toFinset

theorem canonical_workspace_coordinates_permutation_invariant
    {left right : List WorkspaceBuildCoordinate} (permuted : left.Perm right) :
    canonicalWorkspaceCoordinates left = canonicalWorkspaceCoordinates right := by
  apply Finset.ext
  intro coordinate
  simp only [canonicalWorkspaceCoordinates, List.mem_toFinset]
  exact permuted.mem_iff

@[simp] theorem canonical_workspace_coordinates_duplicate_invariant
    (coordinate : WorkspaceBuildCoordinate) (rest : List WorkspaceBuildCoordinate) :
    canonicalWorkspaceCoordinates (coordinate :: coordinate :: rest) =
      canonicalWorkspaceCoordinates (coordinate :: rest) := by
  simp [canonicalWorkspaceCoordinates]

inductive WorkspacePortraitScope where
  | noItems
  | acceptedSubset (numerator denominator : Nat)
      (excluded : List WorkspaceItemIdentity)
  | wholeWorkspace
deriving DecidableEq, Repr

def WorkspacePopulation.portraitScope (population : WorkspacePopulation) :
    WorkspacePortraitScope :=
  if population.intended = [] then .noItems
  else if population.allAccepted then .acceptedSubset
    population.acceptedIdentities.length population.intended.length []
  else .acceptedSubset population.acceptedIdentities.length
    population.intended.length population.excludedIdentities

theorem WorkspacePopulation.portraitScope_is_never_whole
    (population : WorkspacePopulation) :
    population.portraitScope ≠ .wholeWorkspace := by
  by_cases empty : population.intended = []
  · simp [WorkspacePopulation.portraitScope, empty]
  · by_cases accepted : population.allAccepted = true
    · simp [WorkspacePopulation.portraitScope, empty, accepted]
    · simp [WorkspacePopulation.portraitScope, empty, accepted]

theorem nonaccepted_workspace_member_suppresses_whole_workspace
    (population : WorkspacePopulation) (nonempty : population.intended ≠ [])
    (notAll : population.allAccepted = false) :
    population.portraitScope = .acceptedSubset
      population.acceptedIdentities.length population.intended.length
      population.excludedIdentities := by
  simp [WorkspacePopulation.portraitScope, nonempty, notAll]

structure LiftedProjectClaim where
  coordinate : WorkspaceBuildCoordinate
  claim : Program → Prop
  evidenceIdentity : String
  accepts : Program → Prop
  evidence : ∀ program, accepts program → claim program

def projectConjunction (projects : List LiftedProjectClaim) (program : Program) : Prop :=
  ∀ project, project ∈ projects → project.claim program

theorem project_conjunction_permutation_invariant
    {left right : List LiftedProjectClaim} (permuted : left.Perm right)
    (program : Program) :
    projectConjunction left program ↔ projectConjunction right program := by
  constructor
  · intro holds project member
    exact holds project (permuted.mem_iff.mpr member)
  · intro holds project member
    exact holds project (permuted.mem_iff.mp member)

theorem project_conjunction_duplicate_invariant
    (project : LiftedProjectClaim) (projects : List LiftedProjectClaim)
    (program : Program) :
    projectConjunction (project :: project :: projects) program ↔
      projectConjunction (project :: projects) program := by
  simp [projectConjunction]

/-- A workspace lift is non-vacuous and covers the exact matrix order. Each
entry is already a checked `ProjectLift`; this structure composes, rather than
reinterprets, those project judgments. -/
structure WorkspaceLift (population : WorkspacePopulation) where
  nonempty : population.intended ≠ []
  allAccepted : population.allAccepted = true
  projects : List LiftedProjectClaim
  complete : projects.map LiftedProjectClaim.coordinate =
    population.matrices.map WorkspaceMatrixPlan.coordinate

def WorkspaceLift.evidenceVector {population} (lift : WorkspaceLift population) :
    List String := lift.projects.map LiftedProjectClaim.evidenceIdentity

def WorkspaceLift.workspacePopulation {population} (lift : WorkspaceLift population)
    (program : Program) : Prop :=
  ∀ project, project ∈ lift.projects → project.accepts program

def WorkspaceLift.workspaceClaim {population} (lift : WorkspaceLift population)
    (program : Program) : Prop :=
  projectConjunction lift.projects program

structure WorkspaceEvidenceVector {population} (lift : WorkspaceLift population) where
  identities : List String
  sourceOrdered : identities = lift.evidenceVector

def WorkspaceJudgment {population} (lift : WorkspaceLift population)
    (evidence : WorkspaceEvidenceVector lift) (program : Program) : Prop :=
  evidence.identities = lift.evidenceVector ∧
    lift.workspacePopulation program ∧ lift.workspaceClaim program

def WorkspaceLift.realizedEvidence {population} (lift : WorkspaceLift population) :
    WorkspaceEvidenceVector lift := ⟨lift.evidenceVector, rfl⟩

theorem WorkspaceLift.certifiesWorkspace {population}
    (lift : WorkspaceLift population) (program : Program)
    (inside : lift.workspacePopulation program) :
    WorkspaceJudgment lift lift.realizedEvidence program := by
  refine ⟨rfl, inside, ?_⟩
  intro project member
  exact project.evidence program (inside project member)

def WorkspaceLift.portraitScope {population} (_ : WorkspaceLift population) :
    WorkspacePortraitScope := .wholeWorkspace

theorem WorkspaceLift.authorizes_whole_workspace {population}
    (lift : WorkspaceLift population) :
    lift.portraitScope = .wholeWorkspace := rfl

end Thermite.CertificationMetatheory
