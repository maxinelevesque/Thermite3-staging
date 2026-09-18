import Thermite.AssurancePolicyV2

/-!
# Checked assurance policy-version migration

Comparison-capable policy migrations are order isomorphisms over the closed
`AssuranceKindV2` vocabulary.  They preserve the exact claim fiber and may only
change how a policy version names the six formal constructor families.  A
directional `PolicyVersionMigration` adds the version edge and witness identity;
lossy or incompatible edges deliberately have no constructor of this type.
-/

namespace Thermite.CertificationMetatheory

/-- A fiber-preserving order isomorphism between two policy vocabularies. -/
structure PolicyTranslation (sourceVersion targetVersion : Nat) where
  toKind : AssuranceKindV2 → AssuranceKindV2
  fromKind : AssuranceKindV2 → AssuranceKindV2
  leftInverse : ∀ kind, fromKind (toKind kind) = kind
  rightInverse : ∀ kind, toKind (fromKind kind) = kind
  allowedIff : ∀ population kind,
    kindAllowed population (toKind kind) = kindAllowed population kind
  orderIff : ∀ left right,
    assuranceKindLeq (toKind left) (toKind right) =
      assuranceKindLeq left right

namespace PolicyTranslation

def identity (version : Nat) : PolicyTranslation version version where
  toKind := id
  fromKind := id
  leftInverse := by intro kind; rfl
  rightInverse := by intro kind; rfl
  allowedIff := by intro population kind; rfl
  orderIff := by intro left right; rfl

/-- Rename a policy version without changing the formal policy domain. -/
def renameVersion (source target : Nat) : PolicyTranslation source target where
  toKind := id
  fromKind := id
  leftInverse := by intro kind; rfl
  rightInverse := by intro kind; rfl
  allowedIff := by intro population kind; rfl
  orderIff := by intro left right; rfl

def comp {first second third : Nat}
    (left : PolicyTranslation first second)
    (right : PolicyTranslation second third) : PolicyTranslation first third where
  toKind := right.toKind ∘ left.toKind
  fromKind := left.fromKind ∘ right.fromKind
  leftInverse := by
    intro kind
    simp [right.leftInverse, left.leftInverse]
  rightInverse := by
    intro kind
    simp [left.rightInverse, right.rightInverse]
  allowedIff := by
    intro population kind
    change kindAllowed population (right.toKind (left.toKind kind)) =
      kindAllowed population kind
    rw [right.allowedIff, left.allowedIff]
  orderIff := by
    intro firstKind secondKind
    change assuranceKindLeq (right.toKind (left.toKind firstKind))
      (right.toKind (left.toKind secondKind)) =
        assuranceKindLeq firstKind secondKind
    rw [right.orderIff, left.orderIff]

def migratePoint {source target : Nat}
    (translation : PolicyTranslation source target)
    (point : AssurancePolicyV2) : AssurancePolicyV2 :=
  ⟨point.fiber, translation.toKind point.kind⟩

def reversePoint {source target : Nat}
    (translation : PolicyTranslation source target)
    (point : AssurancePolicyV2) : AssurancePolicyV2 :=
  ⟨point.fiber, translation.fromKind point.kind⟩

@[simp] theorem reverse_migrate_point {source target : Nat}
    (translation : PolicyTranslation source target)
    (point : AssurancePolicyV2) :
    translation.reversePoint (translation.migratePoint point) = point := by
  cases point
  simp [migratePoint, reversePoint, translation.leftInverse]

@[simp] theorem migrate_reverse_point {source target : Nat}
    (translation : PolicyTranslation source target)
    (point : AssurancePolicyV2) :
    translation.migratePoint (translation.reversePoint point) = point := by
  cases point
  simp [migratePoint, reversePoint, translation.rightInverse]

theorem migrate_preserves_order {source target : Nat}
    (translation : PolicyTranslation source target)
    {left right : AssurancePolicyV2} (ordered : AssuranceLeq left right) :
    AssuranceLeq (translation.migratePoint left) (translation.migratePoint right) := by
  rcases ordered with ⟨sameFiber, leftAllowed, rightAllowed, kindsOrdered⟩
  refine ⟨sameFiber, ?_, ?_, ?_⟩
  · simpa [migratePoint, translation.allowedIff] using leftAllowed
  · simpa [migratePoint, translation.allowedIff] using rightAllowed
  · simpa [migratePoint, translation.orderIff] using kindsOrdered

theorem migrate_reflects_order {source target : Nat}
    (translation : PolicyTranslation source target)
    {left right : AssurancePolicyV2}
    (ordered : AssuranceLeq (translation.migratePoint left)
      (translation.migratePoint right)) :
    AssuranceLeq left right := by
  rcases ordered with ⟨sameFiber, leftAllowed, rightAllowed, kindsOrdered⟩
  refine ⟨sameFiber, ?_, ?_, ?_⟩
  · have := leftAllowed
    simpa [migratePoint, translation.allowedIff] using this
  · have := rightAllowed
    simpa [migratePoint, translation.allowedIff] using this
  · simpa [migratePoint, translation.orderIff] using kindsOrdered

def migrateNF {source target : Nat} {fiber : ClaimFiberKeyV2}
    (translation : PolicyTranslation source target)
    (normal : AntichainNF fiber) : AntichainNF fiber where
  support := AssuranceKindSet.ofPredicate fun targetKind =>
    normal.support.contains (translation.fromKind targetKind)
  allowed := by
    intro kind member
    rw [AssuranceKindSet.contains_ofPredicate] at member
    have sourceAllowed := normal.allowed (translation.fromKind kind) member
    have translatedAllowed := translation.allowedIff fiber.population
      (translation.fromKind kind)
    rw [translation.rightInverse] at translatedAllowed
    exact translatedAllowed.symm ▸ sourceAllowed
  closed := by
    intro lower upper lowerAllowed upperAllowed lowerUpper upperMember
    rw [AssuranceKindSet.contains_ofPredicate] at upperMember
    rw [AssuranceKindSet.contains_ofPredicate]
    have lowerSourceAllowed :
        kindAllowed fiber.population (translation.fromKind lower) = true := by
      have translated := translation.allowedIff fiber.population
        (translation.fromKind lower)
      rw [translation.rightInverse] at translated
      exact translated ▸ lowerAllowed
    have upperSourceAllowed :
        kindAllowed fiber.population (translation.fromKind upper) = true := by
      have translated := translation.allowedIff fiber.population
        (translation.fromKind upper)
      rw [translation.rightInverse] at translated
      exact translated ▸ upperAllowed
    have sourceOrdered : assuranceKindLeq (translation.fromKind lower)
        (translation.fromKind upper) = true := by
      have translated := translation.orderIff (translation.fromKind lower)
        (translation.fromKind upper)
      rw [translation.rightInverse, translation.rightInverse] at translated
      exact translated ▸ lowerUpper
    exact normal.closed _ _ lowerSourceAllowed upperSourceAllowed sourceOrdered upperMember

theorem migrateNF_membership {source target : Nat} {fiber : ClaimFiberKeyV2}
    (translation : PolicyTranslation source target)
    (normal : AntichainNF fiber) (kind : AssuranceKindV2) :
    (translation.migrateNF normal).support.contains (translation.toKind kind) =
      normal.support.contains kind := by
  simp [migrateNF, translation.leftInverse]

theorem migrateNF_denotation {source target : Nat} {fiber : ClaimFiberKeyV2}
    (translation : PolicyTranslation source target)
    (normal : AntichainNF fiber) (point : AssurancePolicyV2) :
    (translation.migrateNF normal).denotes (translation.migratePoint point) ↔
      normal.denotes point := by
  cases point
  simp [AntichainNF.denotes, migrateNF, migratePoint, translation.leftInverse]

theorem migrateNF_intersection {source target : Nat} {fiber : ClaimFiberKeyV2}
    (translation : PolicyTranslation source target)
    (left right : AntichainNF fiber) :
    (translation.migrateNF (intersectNF left right)).support =
      (intersectNF (translation.migrateNF left)
        (translation.migrateNF right)).support := by
  apply AssuranceKindSet.ext
  intro kind
  simp [migrateNF, intersectNF]

theorem migrateNF_identity {version : Nat} {fiber : ClaimFiberKeyV2}
    (normal : AntichainNF fiber) :
    ((identity version).migrateNF normal).support = normal.support := by
  apply AssuranceKindSet.ext
  intro kind
  simp [migrateNF, identity]

theorem migrateNF_composition {first second third : Nat}
    {fiber : ClaimFiberKeyV2}
    (left : PolicyTranslation first second)
    (right : PolicyTranslation second third)
    (normal : AntichainNF fiber) :
    ((comp left right).migrateNF normal).support =
      (right.migrateNF (left.migrateNF normal)).support := by
  apply AssuranceKindSet.ext
  intro kind
  simp [migrateNF, comp, Function.comp_def]

end PolicyTranslation

/-- A comparison-capable, forward policy-version edge. -/
structure PolicyVersionMigration (sourceVersion targetVersion : Nat)
    extends PolicyTranslation sourceVersion targetVersion where
  forward : sourceVersion < targetVersion
  witness : String
  witnessNonempty : witness ≠ ""

inductive PolicyMigrationDisposition where
  | compatible
  | informationLoss
  | incompatible
  | missing
deriving DecidableEq, Repr

def comparisonEligible : PolicyMigrationDisposition → Bool
  | .compatible => true
  | .informationLoss | .incompatible | .missing => false

def policyV1ToV2Fixture : PolicyVersionMigration 1 2 where
  toPolicyTranslation := PolicyTranslation.renameVersion 1 2
  forward := by decide
  witness := "policy_v1_to_v2_order_isomorphism"
  witnessNonempty := by decide

theorem policy_v1_to_v2_preserves_incomparable_frontier :
    (policyV1ToV2Fixture.toPolicyTranslation.migrateNF incomparableForkNF).frontier =
      [.solverComplete, .leanEmpirical] := by
  exact multiple_maximal_frontier_fixture

theorem policy_v1_to_v2_preserves_intersection :
    (policyV1ToV2Fixture.toPolicyTranslation.migrateNF
      (intersectNF solverCompleteNF empiricalLeanNF)).frontier =
      [.solverIncomplete] := by
  exact incomparable_intersection_fixture

theorem policy_migration_loss_is_not_comparison_eligible :
    comparisonEligible .informationLoss = false ∧
      comparisonEligible .incompatible = false ∧
      comparisonEligible .missing = false := by decide

end Thermite.CertificationMetatheory
