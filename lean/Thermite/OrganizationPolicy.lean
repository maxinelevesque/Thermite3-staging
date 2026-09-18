import Thermite.PolicyMigration
import Thermite.WorkspaceComposition

/-!
# Organization-level formal assurance floors

Organization governance is indexed by exact repository, workspace-plan, and
claim-fiber identities. A floor can be satisfied only by the common-claim
frontier of a live checked workspace lift. Presentation labels and evidence
frontiers are absent from these types. Policy inheritance may strengthen a
comparable floor, but weakening or incomparable updates are conflicts.
-/

namespace Thermite.CertificationMetatheory

structure OrganizationFloorKey where
  repository : String
  workspacePlanSha256 : String
  claimFiberSha256 : String
deriving DecidableEq, Repr

structure OrganizationFloor where
  key : OrganizationFloorKey
  minimum : AssuranceKindV2
  owner : String
deriving DecidableEq, Repr

inductive FloorResolution where
  | resolved (floor : OrganizationFloor)
  | conflict
deriving DecidableEq, Repr

/-- A child may keep or strengthen a comparable exact-scope floor. It cannot
weaken it, and an incomparable proposal is an explicit conflict. -/
def resolveInheritedFloor (parent child : OrganizationFloor) : FloorResolution :=
  if parent.key != child.key then .conflict
  else if parent.minimum = child.minimum then .resolved child
  else if assuranceKindLeq parent.minimum child.minimum then .resolved child
  else .conflict

theorem inherited_floor_cannot_weaken
    (parent child : OrganizationFloor)
    (sameKey : parent.key = child.key)
    (notStrengthening : assuranceKindLeq parent.minimum child.minimum = false)
    (different : child.minimum ≠ parent.minimum) :
    resolveInheritedFloor parent child = .conflict := by
  have reverseDifferent : parent.minimum ≠ child.minimum := fun equal => different equal.symm
  simp [resolveInheritedFloor, sameKey, reverseDifferent, notStrengthening]

theorem incomparable_inherited_floor_conflicts
    (parent child : OrganizationFloor)
    (sameKey : parent.key = child.key)
    (notForward : assuranceKindLeq parent.minimum child.minimum = false)
    (different : parent.minimum ≠ child.minimum) :
    resolveInheritedFloor parent child = .conflict := by
  simp [resolveInheritedFloor, sameKey, different, notForward]

def frontierDominates (frontier : List AssuranceKindV2)
    (minimum : AssuranceKindV2) : Bool :=
  frontier.any fun actual => assuranceKindLeq minimum actual

structure CheckedWorkspaceFloor where
  key : OrganizationFloorKey
  projectLiftPresent : Bool
  workspaceLiftPresent : Bool
  commonClaimFrontiers : List (List AssuranceKindV2)
deriving DecidableEq, Repr

/-- Every exact project common-claim frontier must dominate the floor, and
both project and workspace lifts must be present. -/
def CheckedWorkspaceFloor.satisfies (checked : CheckedWorkspaceFloor)
    (floor : OrganizationFloor) : Bool :=
  checked.key = floor.key &&
    checked.projectLiftPresent && checked.workspaceLiftPresent &&
    !checked.commonClaimFrontiers.isEmpty &&
    checked.commonClaimFrontiers.all fun frontier =>
      frontierDominates frontier floor.minimum

theorem missing_project_lift_cannot_satisfy
    (checked : CheckedWorkspaceFloor) (floor : OrganizationFloor)
    (missing : checked.projectLiftPresent = false) :
    checked.satisfies floor = false := by
  simp [CheckedWorkspaceFloor.satisfies, missing]

theorem missing_workspace_lift_cannot_satisfy
    (checked : CheckedWorkspaceFloor) (floor : OrganizationFloor)
    (missing : checked.workspaceLiftPresent = false) :
    checked.satisfies floor = false := by
  simp [CheckedWorkspaceFloor.satisfies, missing]

structure ScopedPolicyException where
  key : OrganizationFloorKey
  owner : String
  expiresAtEpoch : Nat
  auditProvenance : String
deriving DecidableEq, Repr

def ScopedPolicyException.activeFor (exception : ScopedPolicyException)
    (key : OrganizationFloorKey) (epoch : Nat) : Bool :=
  exception.key = key && epoch < exception.expiresAtEpoch

structure OrganizationGateDecision where
  formalSatisfied : Bool
  excepted : Bool
  gatePassed : Bool
deriving DecidableEq, Repr

def decideOrganizationGate (formalSatisfied exceptionActive : Bool) :
    OrganizationGateDecision :=
  ⟨formalSatisfied, !formalSatisfied && exceptionActive,
    formalSatisfied || exceptionActive⟩

theorem exception_never_rewrites_formal_satisfaction
    (formalSatisfied exceptionActive : Bool) :
    (decideOrganizationGate formalSatisfied exceptionActive).formalSatisfied =
      formalSatisfied := by rfl

theorem expired_exception_cannot_pass_a_failed_floor
    (exception : ScopedPolicyException) (key : OrganizationFloorKey) (epoch : Nat)
    (expired : exception.expiresAtEpoch ≤ epoch) :
    (decideOrganizationGate false (exception.activeFor key epoch)).gatePassed = false := by
  simp [decideOrganizationGate, ScopedPolicyException.activeFor]
  omega

theorem checked_policy_translation_preserves_floor_order
    {source target : Nat} (translation : PolicyTranslation source target)
    (required actual : AssuranceKindV2)
    (dominates : assuranceKindLeq required actual = true) :
    assuranceKindLeq (translation.toKind required)
      (translation.toKind actual) = true := by
  rw [translation.orderIff]
  exact dominates

end Thermite.CertificationMetatheory
