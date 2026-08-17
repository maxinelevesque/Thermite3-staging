import Thermite.CertificationPolicy

/-!
The version-two assurance policy used by the engineer-assurance report.

This module deliberately separates three objects which older implementations
collapsed:

* `RealizedCertification` is an open semantic record carrying accepted
  evidence and the theorems which make that evidence meaningful;
* `AdmittedRealizedCertificationV2` is the closed six-family issuance boundary;
* `AssurancePolicyV2` is the parameterized policy abstraction used for
  comparison and finite downset computation.

Only the six constructor *families* are finite. Execution identities, bounds,
semantic/model versions, contexts, and boundaries remain typed parameters and
are never enumerated by the antichain algorithm.
-/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

inductive PopulationScope where
  | oneExecution (identity : String)
  | throughBound (bound : Nat)
  | allInputs
deriving DecidableEq, Repr

/-- The interpreted population quantified by an assurance statement. -/
structure ClaimPopulation where
  scope : PopulationScope
  contains : Program → Prop

/-- Canonical scope semantics. Fragment and boundary membership are conjoined
below; this cell prevents a scope tag from hiding a strict subset. -/
def PopulationScope.includes : PopulationScope → Program → Prop
  | .oneExecution identity, program => program.digest = identity
  | .throughBound bound, program => program.constructs.length ≤ bound
  | .allInputs, _ => True

inductive RefutationContractKind where
  | soundOnly
  | complete
  | empirical
deriving DecidableEq, Repr

inductive BoundaryCoverageV2 where
  | endToEnd
  | toBoundary (via : String)
  | toPlatform (platform : String)
deriving DecidableEq, Repr

inductive ResidualTrustV2 where
  | fiat
  | solver
  | leanChecked
deriving DecidableEq, Repr

/-- Semantic interpretation of a displayed boundary. A named crossing cannot
be licensed by replacing its predicate with `True`. -/
def BoundaryMeaningV2 (coverage : BoundaryCoverageV2)
    (boundary : BoundaryContext) : Prop :=
  match coverage with
  | .endToEnd => ∀ program, boundary.qualifies program
  | .toBoundary via => boundary.name = via ∧
      (∃ program, boundary.qualifies program) ∧
      ∃ program, ¬boundary.qualifies program
  | .toPlatform platform =>
      boundary.name = platform ∧
        (∃ program, boundary.qualifies program) ∧
        ∃ program, ¬boundary.qualifies program

/-- Current admitted fragments are interpreted proper subsets, not nominal
version strings or a vacuous `fun _ => True` predicate. -/
def FragmentMeaningV2 (fragment : Fragment) : Prop :=
  ∃ admittedWitness excludedWitness,
    fragment.admits admittedWitness ∧ ¬fragment.admits excludedWitness

/-- Refutation evidence remains attached to the actual claim and observation
contract. `empirical` deliberately carries soundness but no completeness. -/
inductive RealizedRefutation (claim : Program → Prop)
    (observation : ObservationContract) where
  | soundOnly (soundness : RefutationSoundness claim observation)
  | complete (soundness : RefutationSoundness claim observation)
      (completeness : RefutationCompleteness claim observation)
  | empirical (soundness : RefutationSoundness claim observation)

def RealizedRefutation.kind {claim observation} :
    RealizedRefutation claim observation → RefutationContractKind
  | .soundOnly _ => .soundOnly
  | .complete _ _ => .complete
  | .empirical _ => .empirical

theorem RealizedRefutation.sound {claim observation}
    (refutation : RealizedRefutation claim observation) :
    ∀ program, observation.observes program → ¬claim program := by
  cases refutation with
  | soundOnly soundness
  | complete soundness _
  | empirical soundness => exact soundness.sound

theorem RealizedRefutation.complete_of_kind {claim observation}
    (refutation : RealizedRefutation claim observation)
    (isComplete : refutation.kind = .complete) :
    ∀ program, ¬claim program → observation.observes program := by
  cases refutation with
  | soundOnly soundness => simp [RealizedRefutation.kind] at isComplete
  | complete soundness completeness => exact completeness.complete
  | empirical soundness => simp [RealizedRefutation.kind] at isComplete

/-- Exact evidence identity. The coarser claim fiber below is obtained only by
an explicit projection; procedure/tool/resource distinctions are not silently
part of policy equality. -/
structure EvidenceFiberKey where
  semantics : String
  semanticsVersion : Nat
  implementationModel : String
  implementationModelVersion : Nat
  fragmentLineage : String
  fragmentRevision : Nat
  procedure : String
  procedureVersion : Nat
  environment : String
  toolVersion : String
  resourceBudget : Nat
  residualContext : String
  boundary : String
  residualTrust : ResidualTrustV2
  axioms : List String
  reconstruction : String
deriving DecidableEq, Repr

/-- Open semantic carrier. It is intentionally not projectable until a closed
admitted-family witness is supplied. Accepted evidence covers every member of
the interpreted population, not merely one representative program. -/
structure RealizedCertification where
  frame : SemanticFrame
  context : ResidualContext
  fragment : Fragment
  procedure : CertificationProcedure
  claim : Program → Prop
  Evidence : Type
  observation : ObservationContract
  judgment : CertificationJudgment frame context fragment procedure claim Evidence observation
  population : ClaimPopulation
  coverage : BoundaryCoverageV2
  boundaryMeaning : BoundaryMeaningV2 coverage frame.boundary
  fragmentMeaning : FragmentMeaningV2 fragment
  populationMeaning : ∀ program, population.contains program ↔
    fragment.admits program ∧ frame.boundary.qualifies program ∧
      population.scope.includes program
  residualTrust : ResidualTrustV2
  axioms : List String
  reconstruction : String
  evidence : Evidence
  contextHolds : context.holds
  acceptsPopulation : ∀ program, population.contains program →
    judgment.certifies evidence program
  proofSoundness : ProofSoundness judgment
  refutation : RealizedRefutation claim observation

def RealizedCertification.evidenceFiber (realized : RealizedCertification) :
    EvidenceFiberKey :=
  ⟨realized.frame.semantics, realized.frame.semanticsVersion,
    realized.frame.implementationModel, realized.frame.implementationModelVersion,
    realized.fragment.version.lineage, realized.fragment.version.revision,
    realized.procedure.name, realized.procedure.version,
    realized.procedure.environment, realized.procedure.toolVersion,
    realized.procedure.resourceBudget, realized.context.name,
    realized.frame.boundary.name, realized.residualTrust, realized.axioms,
    realized.reconstruction⟩

/-- Accepted realized evidence proves the semantic claim for the whole named
population. -/
theorem RealizedCertification.claim_holds (realized : RealizedCertification)
    {program : Program} (inside : realized.population.contains program) :
    realized.fragment.admits program ∧ realized.claim program := by
  exact realized.proofSoundness.meaningful realized.contextHolds
    realized.evidence program (realized.acceptsPopulation program inside)

inductive AdmittedFamilyV2 where
  | runtime
  | bounded
  | solverIncomplete
  | solverComplete
  | leanEmpirical
  | leanComplete
deriving DecidableEq, Repr

def allAdmittedFamiliesV2 : List AdmittedFamilyV2 :=
  [.runtime, .bounded, .solverIncomplete, .solverComplete, .leanEmpirical, .leanComplete]

def AdmittedFamilyV2.expectedPopulation : AdmittedFamilyV2 → PopulationScope → Prop
  | .runtime, .oneExecution _ => True
  | .bounded, .throughBound _ => True
  | .solverIncomplete, .allInputs
  | .solverComplete, .allInputs
  | .leanEmpirical, .allInputs
  | .leanComplete, .allInputs => True
  | _, _ => False

def AdmittedFamilyV2.expectedRefutation : AdmittedFamilyV2 → RefutationContractKind
  | .runtime => .soundOnly
  | .bounded => .complete
  | .solverIncomplete => .soundOnly
  | .solverComplete => .complete
  | .leanEmpirical => .empirical
  | .leanComplete => .complete

def AdmittedFamilyV2.expectedTrust : AdmittedFamilyV2 → ResidualTrustV2
  | .runtime => .fiat
  | .bounded
  | .solverIncomplete
  | .solverComplete => .solver
  | .leanEmpirical
  | .leanComplete => .leanChecked

/-- Family witnesses are load-bearing: an open realized record does not become
current authority merely because a caller chooses a constructor name. -/
structure AdmittedFamilyWitness (family : AdmittedFamilyV2)
    (realized : RealizedCertification) : Prop where
  population : family.expectedPopulation realized.population.scope
  populationNonempty : ∃ program, realized.population.contains program
  refutation : realized.refutation.kind = family.expectedRefutation
  residualTrust : realized.residualTrust = family.expectedTrust
  fragmentNamed : ¬realized.fragment.version.lineage.isEmpty
  boundaryNamed : ¬realized.frame.boundary.name.isEmpty
  procedureNamed : ¬realized.procedure.name.isEmpty
  toolNamed : ¬realized.procedure.toolVersion.isEmpty

/-- Closed issuance wrapper. Adding a producer family makes every exhaustive
consumer in this module fail until the complete extension tax is paid. -/
inductive AdmittedRealizedCertificationV2 where
  | runtime (realized : RealizedCertification)
      (witness : AdmittedFamilyWitness .runtime realized)
  | bounded (realized : RealizedCertification)
      (witness : AdmittedFamilyWitness .bounded realized)
  | solverIncomplete (realized : RealizedCertification)
      (witness : AdmittedFamilyWitness .solverIncomplete realized)
  | solverComplete (realized : RealizedCertification)
      (witness : AdmittedFamilyWitness .solverComplete realized)
  | leanEmpirical (realized : RealizedCertification)
      (witness : AdmittedFamilyWitness .leanEmpirical realized)
  | leanComplete (realized : RealizedCertification)
      (witness : AdmittedFamilyWitness .leanComplete realized)

def AdmittedRealizedCertificationV2.family :
    AdmittedRealizedCertificationV2 → AdmittedFamilyV2
  | .runtime .. => .runtime
  | .bounded .. => .bounded
  | .solverIncomplete .. => .solverIncomplete
  | .solverComplete .. => .solverComplete
  | .leanEmpirical .. => .leanEmpirical
  | .leanComplete .. => .leanComplete

def AdmittedRealizedCertificationV2.realized :
    AdmittedRealizedCertificationV2 → RealizedCertification
  | .runtime realized _
  | .bounded realized _
  | .solverIncomplete realized _
  | .solverComplete realized _
  | .leanEmpirical realized _
  | .leanComplete realized _ => realized

theorem admitted_family_signature_is_exact :
    allAdmittedFamiliesV2.length = 6 ∧ allAdmittedFamiliesV2.Nodup := by decide

/-! ## Parameterized policy carrier -/

/-- Exact claim fiber. Population parameters are part of the key, so the
finite-family algorithm never samples arbitrary execution IDs or bounds. -/
structure ClaimFiberKeyV2 where
  semantics : String
  semanticsVersion : Nat
  implementationModel : String
  implementationModelVersion : Nat
  fragmentLineage : String
  fragmentRevision : Nat
  residualContext : String
  residualPremise : Prop
  boundary : BoundaryCoverageV2
  boundaryQualification : Program → Prop
  population : PopulationScope
  populationDenotation : Program → Prop
  fragmentDenotation : Program → Prop
  /-- The semantic claim itself is part of the formal fiber. Executable reports
  use a separately validated digest/address; Lean comparison never equates two
  claims merely because their coordinate strings match. -/
  claim : Program → Prop

def RealizedCertification.claimFiber (realized : RealizedCertification) : ClaimFiberKeyV2 :=
  ⟨realized.frame.semantics, realized.frame.semanticsVersion,
    realized.frame.implementationModel, realized.frame.implementationModelVersion,
    realized.fragment.version.lineage, realized.fragment.version.revision,
    realized.context.name, realized.context.holds, realized.coverage,
    realized.frame.boundary.qualifies, realized.population.scope,
    realized.population.contains, realized.fragment.admits, realized.claim⟩

inductive AssuranceKindV2 where
  | runtime
  | bounded
  | solverIncomplete
  | solverComplete
  | leanEmpirical
  | leanComplete
deriving DecidableEq, Repr

def allAssuranceKindsV2 : List AssuranceKindV2 :=
  [.runtime, .bounded, .solverIncomplete, .solverComplete, .leanEmpirical, .leanComplete]

/-! The two predecessor Lean carriers are related explicitly as partial family
projections. Neither historical carrier can denote the new
solver-incomplete/Lean-complete families or transport its old cross-population
order into an exact V2 claim fiber. -/

def representativePositionV2Family : RepresentativePosition → AssuranceKindV2
  | .runtime => .runtime
  | .bounded => .bounded
  | .solverComplete => .solverComplete
  | .leanEmpirical => .leanEmpirical

def policyPointV2Family : PolicyPoint → AssuranceKindV2
  | .runtime => .runtime
  | .bounded => .bounded
  | .solverComplete => .solverComplete
  | .leanEmpirical => .leanEmpirical

theorem predecessor_representative_image_is_exact :
    allRepresentativePositions.map representativePositionV2Family =
      [.runtime, .bounded, .solverComplete, .leanEmpirical] := by decide

theorem predecessor_policy_image_is_exact :
    allPolicyPoints.map policyPointV2Family =
      [.runtime, .bounded, .solverComplete, .leanEmpirical] := by decide

theorem predecessor_carriers_omit_new_v2_families :
    .solverIncomplete ∉ allRepresentativePositions.map representativePositionV2Family ∧
    .leanComplete ∉ allRepresentativePositions.map representativePositionV2Family ∧
    .solverIncomplete ∉ allPolicyPoints.map policyPointV2Family ∧
    .leanComplete ∉ allPolicyPoints.map policyPointV2Family := by decide

def AssuranceKindV2.ofFamily : AdmittedFamilyV2 → AssuranceKindV2
  | .runtime => .runtime
  | .bounded => .bounded
  | .solverIncomplete => .solverIncomplete
  | .solverComplete => .solverComplete
  | .leanEmpirical => .leanEmpirical
  | .leanComplete => .leanComplete

structure AssurancePolicyV2 where
  fiber : ClaimFiberKeyV2
  kind : AssuranceKindV2

/-- Constructor compatibility is indexed by the exact population. -/
def kindAllowed : PopulationScope → AssuranceKindV2 → Bool
  | .oneExecution _, .runtime => true
  | .throughBound _, .bounded => true
  | .allInputs, .solverIncomplete
  | .allInputs, .solverComplete
  | .allInputs, .leanEmpirical
  | .allInputs, .leanComplete => true
  | _, _ => false

/-- The predecessor order's `runtime ≤ bounded` edge cannot be transported
into one exact V2 population fiber: the two families have disjoint admitted
scope constructors. This is a checked incompatibility witness, not an omitted
or assumed order-preservation claim. -/
theorem predecessor_runtime_bounded_edge_crosses_v2_population :
    representativeLeq .runtime .bounded = true ∧
    policyLeq .runtime .bounded = true ∧
    ¬∃ scope,
      kindAllowed scope .runtime = true ∧ kindAllowed scope .bounded = true := by
  refine ⟨rfl, rfl, ?_⟩
  rintro ⟨scope, runtimeAllowed, boundedAllowed⟩
  cases scope <;> simp_all [kindAllowed]

/-- Likewise, the predecessor `bounded ≤ solverComplete` edge crosses from a
finite-bound population to all inputs and therefore has no same-fiber V2
transport. -/
theorem predecessor_bounded_solver_edge_crosses_v2_population :
    representativeLeq .bounded .solverComplete = true ∧
    policyLeq .bounded .solverComplete = true ∧
    ¬∃ scope,
      kindAllowed scope .bounded = true ∧
        kindAllowed scope .solverComplete = true := by
  refine ⟨rfl, rfl, ?_⟩
  rintro ⟨scope, boundedAllowed, solverAllowed⟩
  cases scope <;> simp_all [kindAllowed]

/-- The same checked incompatibility applies to the old bounded-to-empirical
edge. -/
theorem predecessor_bounded_empirical_edge_crosses_v2_population :
    representativeLeq .bounded .leanEmpirical = true ∧
    policyLeq .bounded .leanEmpirical = true ∧
    ¬∃ scope,
      kindAllowed scope .bounded = true ∧
        kindAllowed scope .leanEmpirical = true := by
  refine ⟨rfl, rfl, ?_⟩
  rintro ⟨scope, boundedAllowed, empiricalAllowed⟩
  cases scope <;> simp_all [kindAllowed]

/-- Exhaustive reconciliation of every predecessor pair. `preserved` is the
same-family reflexive case; every old true cross-family comparison is rejected
as population-incompatible; and every old false pair remains unordered. -/
inductive PredecessorEdgeDispositionV2 where
  | preserved
  | populationIncompatible
  | predecessorUnordered
deriving DecidableEq, Repr

def predecessorEdgeDispositionV2
    (left right : RepresentativePosition) : PredecessorEdgeDispositionV2 :=
  if representativeLeq left right then
    if left = right then .preserved else .populationIncompatible
  else .predecessorUnordered

def predecessorDispositionMeaningV2
    (left right : RepresentativePosition) : PredecessorEdgeDispositionV2 → Prop
  | .preserved =>
      left = right ∧ representativeLeq left right = true ∧
        policyLeq (abstractPosition left) (abstractPosition right) = true
  | .populationIncompatible =>
      left ≠ right ∧ representativeLeq left right = true ∧
        policyLeq (abstractPosition left) (abstractPosition right) = true ∧
        ¬∃ scope,
          kindAllowed scope (representativePositionV2Family left) = true ∧
            kindAllowed scope (representativePositionV2Family right) = true
  | .predecessorUnordered =>
      representativeLeq left right = false ∧
        policyLeq (abstractPosition left) (abstractPosition right) = false

/-- One proof covers all sixteen predecessor pairs, including the two direct
runtime-to-all-input edges that are not Hasse edges. -/
theorem predecessor_disposition_is_exact (left right : RepresentativePosition) :
    predecessorDispositionMeaningV2 left right
      (predecessorEdgeDispositionV2 left right) := by
  cases left <;> cases right <;>
    simp [predecessorEdgeDispositionV2, predecessorDispositionMeaningV2,
      representativeLeq, policyLeq, abstractPosition,
      representativePositionV2Family]
  all_goals
    intro scope
    cases scope <;> simp [kindAllowed]

def predecessorDispositionMatrixV2 :
    List (RepresentativePosition × RepresentativePosition × PredecessorEdgeDispositionV2) :=
  allRepresentativePositions.flatMap fun left =>
    allRepresentativePositions.map fun right =>
      (left, right, predecessorEdgeDispositionV2 left right)

theorem predecessor_disposition_matrix_covers_all_sixteen_pairs :
    predecessorDispositionMatrixV2.length = 16 := by decide

/-- Symbolic six-family order. In the all-input fiber, `leanComplete` is the
explicit upper bound of both complete-solver and empirical-Lean. The
solver-incomplete point is their common weaker claim. -/
def assuranceKindLeq : AssuranceKindV2 → AssuranceKindV2 → Bool
  | .runtime, .runtime => true
  | .bounded, .bounded => true
  | .solverIncomplete, .solverIncomplete
  | .solverIncomplete, .solverComplete
  | .solverIncomplete, .leanEmpirical
  | .solverIncomplete, .leanComplete => true
  | .solverComplete, .solverComplete
  | .solverComplete, .leanComplete => true
  | .leanEmpirical, .leanEmpirical
  | .leanEmpirical, .leanComplete => true
  | .leanComplete, .leanComplete => true
  | _, _ => false

def AssuranceLeq (left right : AssurancePolicyV2) : Prop :=
  left.fiber = right.fiber ∧
    kindAllowed left.fiber.population left.kind = true ∧
    kindAllowed right.fiber.population right.kind = true ∧
    assuranceKindLeq left.kind right.kind = true

theorem assuranceLeq_refl (point : AssurancePolicyV2)
    (allowed : kindAllowed point.fiber.population point.kind = true) :
    AssuranceLeq point point := by
  refine ⟨rfl, allowed, allowed, ?_⟩
  cases point.kind <;> rfl

theorem assuranceKindLeq_refl (kind : AssuranceKindV2) :
    assuranceKindLeq kind kind = true := by cases kind <;> rfl

theorem assuranceKindLeq_antisymm {left right : AssuranceKindV2}
    (forward : assuranceKindLeq left right = true)
    (reverse : assuranceKindLeq right left = true) : left = right := by
  cases left <;> cases right <;> simp_all [assuranceKindLeq]

theorem assuranceKindLeq_trans {first second third : AssuranceKindV2}
    (firstSecond : assuranceKindLeq first second = true)
    (secondThird : assuranceKindLeq second third = true) :
    assuranceKindLeq first third = true := by
  cases first <;> cases second <;> cases third <;> simp_all [assuranceKindLeq]

theorem assuranceLeq_trans {first second third : AssurancePolicyV2}
    (firstSecond : AssuranceLeq first second)
    (secondThird : AssuranceLeq second third) :
    AssuranceLeq first third := by
  rcases firstSecond with ⟨fiberFirstSecond, firstAllowed, _, kindFirstSecond⟩
  rcases secondThird with ⟨fiberSecondThird, _, thirdAllowed, kindSecondThird⟩
  exact ⟨fiberFirstSecond.trans fiberSecondThird, firstAllowed, thirdAllowed,
    assuranceKindLeq_trans kindFirstSecond kindSecondThird⟩

theorem assuranceLeq_rejects_different_fibers {left right : AssurancePolicyV2}
    (different : left.fiber ≠ right.fiber) : ¬AssuranceLeq left right := by
  intro ordered
  exact different ordered.1

theorem lean_complete_is_explicit_upper_bound :
    assuranceKindLeq .solverComplete .leanComplete = true ∧
    assuranceKindLeq .leanEmpirical .leanComplete = true := by decide

/-- Behavior/trust propositions denoted by policy families. The policy order is
licensed by implication between these meanings, not by enum rank. -/
structure PolicyCapabilitiesV2 where
  runtimeChecked : Prop
  boundedChecked : Prop
  provedAllInputs : Prop
  concreteWitness : Prop
  empiricalFalsification : Prop
  leanChecked : Prop
  completeSuppliesEmpirical : concreteWitness → empiricalFalsification

/-- The only capabilities used to license an admitted policy point are derived
from its realized evidence. In particular, complete refutation supplies an
empirical falsifier because every realized refutation already carries
soundness; callers do not postulate this edge. -/
def RealizedCertification.policyCapabilities (realized : RealizedCertification) :
    PolicyCapabilitiesV2 where
  runtimeChecked := ∀ program, realized.population.contains program →
    realized.fragment.admits program ∧ realized.claim program
  boundedChecked := ∀ program, realized.population.contains program →
    realized.fragment.admits program ∧ realized.claim program
  provedAllInputs := ∀ program, realized.population.contains program →
    realized.fragment.admits program ∧ realized.claim program
  concreteWitness := ∀ program, ¬realized.claim program →
    realized.observation.observes program
  empiricalFalsification := ∀ program, realized.observation.observes program →
    ¬realized.claim program
  leanChecked := realized.residualTrust = .leanChecked
  completeSuppliesEmpirical := by
    intro _
    exact realized.refutation.sound

def AssuranceKindV2.denotes (kind : AssuranceKindV2)
    (capabilities : PolicyCapabilitiesV2) : Prop :=
  match kind with
  | .runtime => capabilities.runtimeChecked
  | .bounded => capabilities.boundedChecked
  | .solverIncomplete => capabilities.provedAllInputs
  | .solverComplete => capabilities.provedAllInputs ∧ capabilities.concreteWitness
  | .leanEmpirical => capabilities.provedAllInputs ∧
      capabilities.empiricalFalsification ∧ capabilities.leanChecked
  | .leanComplete => capabilities.provedAllInputs ∧
      capabilities.concreteWitness ∧ capabilities.leanChecked

theorem assuranceKindLeq_semantically_sound {left right : AssuranceKindV2}
    (ordered : assuranceKindLeq left right = true)
    (capabilities : PolicyCapabilitiesV2)
    (strong : right.denotes capabilities) : left.denotes capabilities := by
  cases left <;> cases right <;>
    simp_all [assuranceKindLeq, AssuranceKindV2.denotes]
  exact capabilities.completeSuppliesEmpirical strong.2.1

theorem admitted_realizes_abstract_kind
    (admitted : AdmittedRealizedCertificationV2) :
    (AssuranceKindV2.ofFamily admitted.family).denotes
      admitted.realized.policyCapabilities := by
  have whole : ∀ program, admitted.realized.population.contains program →
      admitted.realized.fragment.admits program ∧ admitted.realized.claim program :=
    fun _ inside => admitted.realized.claim_holds inside
  have sound := admitted.realized.refutation.sound
  cases admitted with
  | runtime realized witness => exact whole
  | bounded realized witness => exact whole
  | solverIncomplete realized witness => exact whole
  | solverComplete realized witness =>
      exact ⟨whole, realized.refutation.complete_of_kind witness.refutation⟩
  | leanEmpirical realized witness =>
      exact ⟨whole, sound, witness.residualTrust⟩
  | leanComplete realized witness =>
      exact ⟨whole, realized.refutation.complete_of_kind witness.refutation,
        witness.residualTrust⟩

theorem admitted_order_is_semantically_sound
    (admitted : AdmittedRealizedCertificationV2) {lower : AssuranceKindV2}
    (ordered : assuranceKindLeq lower (AssuranceKindV2.ofFamily admitted.family) = true) :
    lower.denotes admitted.realized.policyCapabilities :=
  assuranceKindLeq_semantically_sound ordered _ (admitted_realizes_abstract_kind admitted)

def abstractAdmitted (admitted : AdmittedRealizedCertificationV2) : AssurancePolicyV2 :=
  { fiber := admitted.realized.claimFiber
    kind := AssuranceKindV2.ofFamily admitted.family }

theorem abstraction_is_total (admitted : AdmittedRealizedCertificationV2) :
    kindAllowed (abstractAdmitted admitted).fiber.population
      (abstractAdmitted admitted).kind = true := by
  cases admitted with
  | runtime realized witness =>
      cases scope : realized.population.scope with
      | oneExecution identity =>
          simp [abstractAdmitted, AdmittedRealizedCertificationV2.realized,
            AdmittedRealizedCertificationV2.family, AssuranceKindV2.ofFamily,
            RealizedCertification.claimFiber, kindAllowed, scope]
      | throughBound bound =>
          exfalso
          simpa [AdmittedFamilyV2.expectedPopulation, scope] using witness.population
      | allInputs =>
          exfalso
          simpa [AdmittedFamilyV2.expectedPopulation, scope] using witness.population
  | bounded realized witness =>
      cases scope : realized.population.scope with
      | oneExecution identity =>
          exfalso
          simpa [AdmittedFamilyV2.expectedPopulation, scope] using witness.population
      | throughBound bound =>
          simp [abstractAdmitted, AdmittedRealizedCertificationV2.realized,
            AdmittedRealizedCertificationV2.family, AssuranceKindV2.ofFamily,
            RealizedCertification.claimFiber, kindAllowed, scope]
      | allInputs =>
          exfalso
          simpa [AdmittedFamilyV2.expectedPopulation, scope] using witness.population
  | solverIncomplete realized witness
  | solverComplete realized witness
  | leanEmpirical realized witness
  | leanComplete realized witness =>
      cases scope : realized.population.scope with
      | oneExecution identity =>
          exfalso
          simpa [AdmittedFamilyV2.expectedPopulation, scope] using witness.population
      | throughBound bound =>
          exfalso
          simpa [AdmittedFamilyV2.expectedPopulation, scope] using witness.population
      | allInputs =>
          simp [abstractAdmitted, AdmittedRealizedCertificationV2.realized,
            AdmittedRealizedCertificationV2.family, AssuranceKindV2.ofFamily,
            RealizedCertification.claimFiber, kindAllowed, scope]

def AdmittedRealizedCertificationV2.familyWitness
    (admitted : AdmittedRealizedCertificationV2) :
    AdmittedFamilyWitness admitted.family admitted.realized := by
  cases admitted with
  | runtime _ witness
  | bounded _ witness
  | solverIncomplete _ witness
  | solverComplete _ witness
  | leanEmpirical _ witness
  | leanComplete _ witness => exact witness

theorem admitted_population_is_nonempty
    (admitted : AdmittedRealizedCertificationV2) :
    ∃ program, admitted.realized.population.contains program :=
  admitted.familyWitness.populationNonempty

theorem one_execution_membership_has_exact_identity (realized : RealizedCertification)
    (identity : String) (scope : realized.population.scope = .oneExecution identity)
    {program : Program} (inside : realized.population.contains program) :
    program.digest = identity := by
  have meaning := (realized.populationMeaning program).mp inside
  simpa [scope, PopulationScope.includes] using meaning.2.2

theorem bounded_membership_respects_exact_bound (realized : RealizedCertification)
    (bound : Nat) (scope : realized.population.scope = .throughBound bound)
    {program : Program} (inside : realized.population.contains program) :
    program.constructs.length ≤ bound := by
  have meaning := (realized.populationMeaning program).mp inside
  simpa [scope, PopulationScope.includes] using meaning.2.2

/-! ## Engineer projection: presentation derived from, never substituted for,
formal authority. -/

inductive EngineerClaimV1 where
  | checkedThisExecution (identity : String)
  | checkedThroughBound (bound : Nat)
  | provedAllInputsMayNotProduceWitness
  | provedAllInputsWithConcreteWitness
  | provedAllInputsWithEmpiricalFalsification
deriving DecidableEq, Repr

structure EngineerDisplayV1 where
  policyVersion : Nat
  claim : EngineerClaimV1
  coverage : BoundaryCoverageV2
deriving DecidableEq, Repr

def engineerClaimFor (point : AssurancePolicyV2) : Option EngineerClaimV1 :=
  match point.kind, point.fiber.population with
  | .runtime, .oneExecution identity => some (.checkedThisExecution identity)
  | .bounded, .throughBound bound => some (.checkedThroughBound bound)
  | .solverIncomplete, .allInputs => some .provedAllInputsMayNotProduceWitness
  | .solverComplete, .allInputs
  | .leanComplete, .allInputs => some .provedAllInputsWithConcreteWitness
  | .leanEmpirical, .allInputs => some .provedAllInputsWithEmpiricalFalsification
  | _, _ => none

def engineerDisplayFor (point : AssurancePolicyV2) : Option EngineerDisplayV1 :=
  (engineerClaimFor point).map fun claim => ⟨1, claim, point.fiber.boundary⟩

/-- Formal distinctions intentionally omitted by the primary label remain a
generated, typed disclosure. -/
structure EngineerDisclosureV1 where
  sourceFamily : AdmittedFamilyV2
  /-- The complete formal point is retained in disclosure even when the short
  label intentionally collapses route/trust or omits semantic identities. -/
  formalPosition : AssurancePolicyV2
  evidenceFiber : EvidenceFiberKey
  refutation : RefutationContractKind

def engineerDisclosureFor (admitted : AdmittedRealizedCertificationV2) :
    EngineerDisclosureV1 :=
  ⟨admitted.family, abstractAdmitted admitted, admitted.realized.evidenceFiber,
    admitted.realized.refutation.kind⟩

def EngineerMeaning (admitted : AdmittedRealizedCertificationV2)
    (display : EngineerDisplayV1) : Prop :=
  engineerDisplayFor (abstractAdmitted admitted) = some display ∧
    (∀ program, admitted.realized.population.contains program →
      admitted.realized.fragment.admits program ∧ admitted.realized.claim program) ∧
    (∀ program, admitted.realized.observation.observes program →
      ¬admitted.realized.claim program) ∧
    (admitted.family.expectedRefutation = .complete →
      ∀ program, ¬admitted.realized.claim program →
        admitted.realized.observation.observes program) ∧
    admitted.realized.refutation.kind = admitted.family.expectedRefutation ∧
    admitted.realized.residualTrust = admitted.family.expectedTrust ∧
    display.coverage = (abstractAdmitted admitted).fiber.boundary ∧
    BoundaryMeaningV2 admitted.realized.coverage admitted.realized.frame.boundary ∧
    FragmentMeaningV2 admitted.realized.fragment ∧
    (∀ program, admitted.realized.population.contains program ↔
      admitted.realized.fragment.admits program ∧
        admitted.realized.frame.boundary.qualifies program ∧
        admitted.realized.population.scope.includes program) ∧
    engineerDisclosureFor admitted =
      ⟨admitted.family, abstractAdmitted admitted, admitted.realized.evidenceFiber,
        admitted.realized.refutation.kind⟩

theorem engineer_projection_is_total (admitted : AdmittedRealizedCertificationV2) :
    ∃ display, engineerDisplayFor (abstractAdmitted admitted) = some display := by
  have allowed := abstraction_is_total admitted
  cases admitted with
  | runtime realized witness =>
      cases scope : realized.population.scope <;>
        simp_all [abstractAdmitted, AdmittedRealizedCertificationV2.realized,
          AdmittedRealizedCertificationV2.family, AssuranceKindV2.ofFamily,
          RealizedCertification.claimFiber, engineerDisplayFor, engineerClaimFor,
          kindAllowed]
  | bounded realized witness =>
      cases scope : realized.population.scope <;>
        simp_all [abstractAdmitted, AdmittedRealizedCertificationV2.realized,
          AdmittedRealizedCertificationV2.family, AssuranceKindV2.ofFamily,
          RealizedCertification.claimFiber, engineerDisplayFor, engineerClaimFor,
          kindAllowed]
  | solverIncomplete realized witness
  | solverComplete realized witness
  | leanEmpirical realized witness
  | leanComplete realized witness =>
      cases scope : realized.population.scope <;>
        simp_all [abstractAdmitted, AdmittedRealizedCertificationV2.realized,
          AdmittedRealizedCertificationV2.family, AssuranceKindV2.ofFamily,
          RealizedCertification.claimFiber, engineerDisplayFor, engineerClaimFor,
          kindAllowed]

theorem engineer_meaning_is_licensed (admitted : AdmittedRealizedCertificationV2) :
    ∃ display, EngineerMeaning admitted display := by
  rcases engineer_projection_is_total admitted with ⟨display, projected⟩
  refine ⟨display, projected, ?_, admitted.realized.refutation.sound, ?_,
    admitted.familyWitness.refutation, admitted.familyWitness.residualTrust, ?_,
    admitted.realized.boundaryMeaning, admitted.realized.fragmentMeaning,
    admitted.realized.populationMeaning, rfl⟩
  · intro program inside
    exact admitted.realized.claim_holds inside
  · intro complete
    exact admitted.realized.refutation.complete_of_kind
      (admitted.familyWitness.refutation.trans complete)
  · have mapped := congrArg (fun value => value.map EngineerDisplayV1.coverage) projected
    have coverage : (∃ claim, engineerClaimFor (abstractAdmitted admitted) = some claim) ∧
        (abstractAdmitted admitted).fiber.boundary = display.coverage := by
      simpa [engineerDisplayFor] using mapped
    exact coverage.2.symm

theorem complete_routes_share_label_but_not_formal_kind (fiber : ClaimFiberKeyV2)
    (allInputs : fiber.population = .allInputs) :
    engineerDisplayFor ⟨fiber, .solverComplete⟩ =
      engineerDisplayFor ⟨fiber, .leanComplete⟩ ∧
    (⟨fiber, .solverComplete⟩ : AssurancePolicyV2) ≠ ⟨fiber, .leanComplete⟩ := by
  constructor
  · simp [engineerDisplayFor, engineerClaimFor, allInputs]
  · intro equality
    cases equality

/-- The exact equivalence relation induced by the primary engineer display.
Every formal field not named here (semantic/model/fragment/context/claim and
population denotations) is intentionally forgotten by the short label and is
retained in `EngineerDisclosureV1.formalPosition`. -/
def SameEngineerDisplayFiber (left right : AssurancePolicyV2) : Prop :=
  left.fiber.population = right.fiber.population ∧
    left.fiber.boundary = right.fiber.boundary ∧
    (left.kind = right.kind ∨
      (left.kind = .solverComplete ∧ right.kind = .leanComplete) ∨
      (left.kind = .leanComplete ∧ right.kind = .solverComplete))

/-- Global exact-fiber theorem for the complete display, rather than a theorem
restricted to two kinds already placed in one formal fiber. -/
theorem engineer_projection_fiber_is_exact (left right : AssurancePolicyV2)
    (leftAllowed : kindAllowed left.fiber.population left.kind = true)
    (rightAllowed : kindAllowed right.fiber.population right.kind = true) :
    engineerDisplayFor left = engineerDisplayFor right ↔
      SameEngineerDisplayFiber left right := by
  cases left with
  | mk leftFiber leftKind =>
    cases right with
    | mk rightFiber rightKind =>
      cases leftFiber with
      | mk ls lsv lm lmv lfl lfr lcn lcp lb lbq lp lpd lfd lc =>
        cases rightFiber with
        | mk rs rsv rm rmv rfl rfr rcn rcp rb rbq rp rpd rfd rc =>
          cases lp <;> cases rp <;> cases leftKind <;> cases rightKind <;>
            simp_all [kindAllowed, engineerDisplayFor, engineerClaimFor,
              SameEngineerDisplayFiber]

theorem projection_covers_all_six_families :
    (allAssuranceKindsV2.map (fun kind =>
      match kind with
      | .runtime => "checked_this_execution"
      | .bounded => "checked_through_bound"
      | .solverIncomplete => "proved_all_may_not_witness"
      | .solverComplete => "proved_all_concrete_witness"
      | .leanEmpirical => "proved_all_empirical"
      | .leanComplete => "proved_all_concrete_witness")).length = 6 := by decide

/-! ## Realized fixtures

These fixtures use a nontrivial fragment and boundary predicate. They ensure
the six admitted constructors are inhabited by actual accepted evidence and
soundness/refutation theorems, rather than by coordinate strings. -/

def assuranceFixtureBoundary : BoundaryContext :=
  ⟨"fixture-boundary", fun program => program.facts.contains "fixture-boundary"⟩

def assuranceFixtureFrame : SemanticFrame :=
  ⟨"thermite-language", 1, "rustc", 1, assuranceFixtureBoundary⟩

def nominalTrueBoundary : BoundaryContext :=
  ⟨"fixture-boundary", fun _ => True⟩

def nominalTrueFragment : Fragment :=
  { version := ⟨"nominal-fragment", 1⟩
    admits := fun _ => True }

theorem named_boundary_cannot_be_nominal_true :
    ¬BoundaryMeaningV2 (.toBoundary "fixture-boundary") nominalTrueBoundary := by
  intro meaning
  rcases meaning.2.2 with ⟨program, excluded⟩
  exact excluded trivial

theorem admitted_fragment_cannot_be_nominal_true :
    ¬FragmentMeaningV2 nominalTrueFragment := by
  intro meaning
  rcases meaning with ⟨admitted, excluded, admittedProof, excludedProof⟩
  exact excludedProof trivial

def assuranceFixtureProcedure : CertificationProcedure :=
  ⟨"fixture-procedure", 1, "fixture-environment", "fixture-tool-1", 100⟩

def assuranceFixturePopulation (scope : PopulationScope) : ClaimPopulation :=
  { scope
    contains := fun program => coreV2.admits program ∧
      assuranceFixtureBoundary.qualifies program ∧
      match scope with
      | .oneExecution identity => program.digest = identity
      | .throughBound bound => program.constructs.length ≤ bound
      | .allInputs => True }

def assuranceFixtureObservation (scope : PopulationScope) : ObservationContract :=
  ⟨"fixture-observation", fun program =>
    ¬(assuranceFixturePopulation scope).contains program⟩

def assuranceFixtureJudgment (scope : PopulationScope) :
    CertificationJudgment assuranceFixtureFrame noResiduals coreV2
      assuranceFixtureProcedure (assuranceFixturePopulation scope).contains Unit
      (assuranceFixtureObservation scope) :=
  ⟨fun _ program => (assuranceFixturePopulation scope).contains program⟩

theorem assuranceFixturePopulation_admitted (scope : PopulationScope)
    {program : Program} (inside : (assuranceFixturePopulation scope).contains program) :
    coreV2.admits program := inside.1

def assuranceFixtureRefutation (scope : PopulationScope)
    (kind : RefutationContractKind) :
    RealizedRefutation (assuranceFixturePopulation scope).contains
      (assuranceFixtureObservation scope) :=
  match kind with
  | .soundOnly => .soundOnly ⟨by intro program observed claim; exact observed claim⟩
  | .complete => .complete
      ⟨by intro program observed claim; exact observed claim⟩
      ⟨by intro program falseClaim; exact falseClaim⟩
  | .empirical => .empirical ⟨by intro program observed claim; exact observed claim⟩

def assuranceFixtureRealized (scope : PopulationScope)
    (refutation : RefutationContractKind) (trust : ResidualTrustV2) :
    RealizedCertification where
  frame := assuranceFixtureFrame
  context := noResiduals
  fragment := coreV2
  procedure := assuranceFixtureProcedure
  claim := (assuranceFixturePopulation scope).contains
  Evidence := Unit
  observation := assuranceFixtureObservation scope
  judgment := assuranceFixtureJudgment scope
  population := assuranceFixturePopulation scope
  coverage := .toBoundary "fixture-boundary"
  boundaryMeaning := by
    refine ⟨rfl, ⟨⟨"inside-boundary", [], ["fixture-boundary"]⟩, ?_⟩,
      ⟨⟨"outside-boundary", [], []⟩, ?_⟩⟩
    · simp [assuranceFixtureFrame, assuranceFixtureBoundary]
    · simp [assuranceFixtureFrame, assuranceFixtureBoundary]
  fragmentMeaning := by
    refine ⟨⟨"admitted", ["Fn"], ["fixture-boundary"]⟩,
      ⟨"excluded", [], []⟩, ?_, ?_⟩
    · simp [coreV2]
    · simp [coreV2]
  populationMeaning := by
    intro program
    cases scope <;> rfl
  residualTrust := trust
  axioms := ["Classical.choice"]
  reconstruction := match trust with
    | .fiat => "runtime-monitor"
    | .solver => "solver-proof"
    | .leanChecked => "axiom-clean-kernel-replay"
  evidence := ()
  contextHolds := trivial
  acceptsPopulation := fun _ inside => inside
  proofSoundness := ⟨by
    intro _ evidence program certified
    exact ⟨assuranceFixturePopulation_admitted scope certified, certified⟩⟩
  refutation := assuranceFixtureRefutation scope refutation

def assuranceFixtureWitness (family : AdmittedFamilyV2)
    (scope : PopulationScope) (population : family.expectedPopulation scope)
    (populationNonempty : ∃ program,
      (assuranceFixturePopulation scope).contains program) :
    AdmittedFamilyWitness family
      (assuranceFixtureRealized scope family.expectedRefutation family.expectedTrust) where
  population := population
  populationNonempty := populationNonempty
  refutation := by cases family <;> rfl
  residualTrust := rfl
  fragmentNamed := by
    simp [assuranceFixtureRealized, coreV2]
  boundaryNamed := by
    simp [assuranceFixtureRealized, assuranceFixtureFrame, assuranceFixtureBoundary]
  procedureNamed := by
    simp [assuranceFixtureRealized, assuranceFixtureProcedure]
  toolNamed := by
    simp [assuranceFixtureRealized, assuranceFixtureProcedure]

def admittedRuntimeFixture : AdmittedRealizedCertificationV2 :=
  .runtime (assuranceFixtureRealized (.oneExecution "fixture-run") .soundOnly .fiat)
    (assuranceFixtureWitness .runtime (.oneExecution "fixture-run") trivial
      ⟨⟨"fixture-run", ["Fn"], ["fixture-boundary"]⟩, by
        simp [assuranceFixturePopulation, coreV2, assuranceFixtureBoundary]⟩)

def admittedBoundedFixture : AdmittedRealizedCertificationV2 :=
  .bounded (assuranceFixtureRealized (.throughBound 8) .complete .solver)
    (assuranceFixtureWitness .bounded (.throughBound 8) trivial
      ⟨⟨"bounded-run", ["Fn"], ["fixture-boundary"]⟩, by
        simp [assuranceFixturePopulation, coreV2, assuranceFixtureBoundary]⟩)

def admittedSolverIncompleteFixture : AdmittedRealizedCertificationV2 :=
  .solverIncomplete (assuranceFixtureRealized .allInputs .soundOnly .solver)
    (assuranceFixtureWitness .solverIncomplete .allInputs trivial
      ⟨⟨"all-input", ["Fn"], ["fixture-boundary"]⟩, by
        simp [assuranceFixturePopulation, coreV2, assuranceFixtureBoundary]⟩)

def admittedSolverCompleteFixture : AdmittedRealizedCertificationV2 :=
  .solverComplete (assuranceFixtureRealized .allInputs .complete .solver)
    (assuranceFixtureWitness .solverComplete .allInputs trivial
      ⟨⟨"all-input", ["Fn"], ["fixture-boundary"]⟩, by
        simp [assuranceFixturePopulation, coreV2, assuranceFixtureBoundary]⟩)

def admittedLeanEmpiricalFixture : AdmittedRealizedCertificationV2 :=
  .leanEmpirical (assuranceFixtureRealized .allInputs .empirical .leanChecked)
    (assuranceFixtureWitness .leanEmpirical .allInputs trivial
      ⟨⟨"all-input", ["Fn"], ["fixture-boundary"]⟩, by
        simp [assuranceFixturePopulation, coreV2, assuranceFixtureBoundary]⟩)

def admittedLeanCompleteFixture : AdmittedRealizedCertificationV2 :=
  .leanComplete (assuranceFixtureRealized .allInputs .complete .leanChecked)
    (assuranceFixtureWitness .leanComplete .allInputs trivial
      ⟨⟨"all-input", ["Fn"], ["fixture-boundary"]⟩, by
        simp [assuranceFixturePopulation, coreV2, assuranceFixtureBoundary]⟩)

theorem all_six_realized_fixtures_have_licensed_meanings :
    ∀ admitted ∈ [admittedRuntimeFixture, admittedBoundedFixture,
      admittedSolverIncompleteFixture, admittedSolverCompleteFixture,
      admittedLeanEmpiricalFixture, admittedLeanCompleteFixture],
      ∃ display, EngineerMeaning admitted display := by
  intro admitted _
  exact engineer_meaning_is_licensed admitted

/-! ## Canonical finite downset / antichain normal form -/

/-- A finite family mask. This enumerates constructor families only; the exact
fiber still contains arbitrary typed parameters. -/
structure AssuranceKindSet where
  runtime : Bool
  bounded : Bool
  solverIncomplete : Bool
  solverComplete : Bool
  leanEmpirical : Bool
  leanComplete : Bool
deriving DecidableEq, Repr

def AssuranceKindSet.contains (set : AssuranceKindSet) : AssuranceKindV2 → Bool
  | .runtime => set.runtime
  | .bounded => set.bounded
  | .solverIncomplete => set.solverIncomplete
  | .solverComplete => set.solverComplete
  | .leanEmpirical => set.leanEmpirical
  | .leanComplete => set.leanComplete

def AssuranceKindSet.ofPredicate (predicate : AssuranceKindV2 → Bool) : AssuranceKindSet :=
  ⟨predicate .runtime, predicate .bounded, predicate .solverIncomplete,
    predicate .solverComplete, predicate .leanEmpirical, predicate .leanComplete⟩

@[simp] theorem AssuranceKindSet.contains_ofPredicate
    (predicate : AssuranceKindV2 → Bool) (kind : AssuranceKindV2) :
    (AssuranceKindSet.ofPredicate predicate).contains kind = predicate kind := by
  cases kind <;> rfl

def AssuranceKindSet.inter (left right : AssuranceKindSet) : AssuranceKindSet :=
  AssuranceKindSet.ofPredicate fun kind => left.contains kind && right.contains kind

@[ext] theorem AssuranceKindSet.ext {left right : AssuranceKindSet}
    (same : ∀ kind, left.contains kind = right.contains kind) : left = right := by
  rcases left with ⟨lr, lb, li, ls, le, ll⟩
  rcases right with ⟨rr, rb, ri, rs, re, rl⟩
  have runtime := same .runtime
  have bounded := same .bounded
  have incomplete := same .solverIncomplete
  have solver := same .solverComplete
  have empirical := same .leanEmpirical
  have lean := same .leanComplete
  simp [AssuranceKindSet.contains] at runtime bounded incomplete solver empirical lean
  cases runtime
  cases bounded
  cases incomplete
  cases solver
  cases empirical
  cases lean
  rfl

@[simp] theorem AssuranceKindSet.contains_inter
    (left right : AssuranceKindSet) (kind : AssuranceKindV2) :
    (left.inter right).contains kind =
      (left.contains kind && right.contains kind) := by
  simp [AssuranceKindSet.inter]

def AssuranceKindSet.empty : AssuranceKindSet := ⟨false, false, false, false, false, false⟩

def downwardClosed (fiber : ClaimFiberKeyV2) (set : AssuranceKindSet) : Prop :=
  ∀ lower upper, kindAllowed fiber.population lower = true →
    kindAllowed fiber.population upper = true →
    assuranceKindLeq lower upper = true → set.contains upper = true →
    set.contains lower = true

/-- Canonical finite representation of a downset. Its serialized antichain is
derived in the one fixed constructor order below. -/
structure AntichainNF (fiber : ClaimFiberKeyV2) where
  support : AssuranceKindSet
  allowed : ∀ kind, support.contains kind = true →
    kindAllowed fiber.population kind = true
  closed : downwardClosed fiber support

def AntichainNF.denotes {fiber} (normal : AntichainNF fiber)
    (point : AssurancePolicyV2) : Prop :=
  point.fiber = fiber ∧ normal.support.contains point.kind = true

def principalNF (point : AssurancePolicyV2) : AntichainNF point.fiber where
  support := AssuranceKindSet.ofPredicate fun candidate =>
    kindAllowed point.fiber.population candidate && assuranceKindLeq candidate point.kind
  allowed := by
    intro kind member
    cases kind <;>
      simp_all [AssuranceKindSet.contains, AssuranceKindSet.ofPredicate]
  closed := by
    intro lower upper lowerAllowed upperAllowed lowerUpper upperMember
    rw [AssuranceKindSet.contains_ofPredicate] at upperMember
    change (kindAllowed point.fiber.population upper &&
      assuranceKindLeq upper point.kind) = true at upperMember
    rw [AssuranceKindSet.contains_ofPredicate]
    change (kindAllowed point.fiber.population lower &&
      assuranceKindLeq lower point.kind) = true
    apply Bool.and_eq_true_iff.mpr
    exact ⟨lowerAllowed, assuranceKindLeq_trans lowerUpper
      (Bool.and_eq_true_iff.mp upperMember).2⟩

theorem principalNF_contains_generator (point : AssurancePolicyV2)
    (allowed : kindAllowed point.fiber.population point.kind = true) :
    (principalNF point).support.contains point.kind = true := by
  change (AssuranceKindSet.ofPredicate fun candidate =>
    kindAllowed point.fiber.population candidate &&
      assuranceKindLeq candidate point.kind).contains point.kind = true
  rw [AssuranceKindSet.contains_ofPredicate]
  exact Bool.and_eq_true_iff.mpr ⟨allowed, assuranceKindLeq_refl point.kind⟩

def emptyNF (fiber : ClaimFiberKeyV2) : AntichainNF fiber where
  support := AssuranceKindSet.empty
  allowed := by intro kind member; cases kind <;> simp [AssuranceKindSet.empty,
    AssuranceKindSet.contains] at member
  closed := by intro lower upper _ _ _ member; cases upper <;>
    simp [AssuranceKindSet.empty, AssuranceKindSet.contains] at member

def intersectNF {fiber} (left right : AntichainNF fiber) : AntichainNF fiber where
  support := left.support.inter right.support
  allowed := by
    intro kind member
    rw [AssuranceKindSet.contains_inter] at member
    change (left.support.contains kind && right.support.contains kind) = true at member
    exact left.allowed kind (Bool.and_eq_true_iff.mp member).1
  closed := by
    intro lower upper lowerAllowed upperAllowed lowerUpper upperMember
    have leftMember : left.support.contains upper = true := by
      rw [AssuranceKindSet.contains_inter] at upperMember
      change (left.support.contains upper && right.support.contains upper) = true at upperMember
      exact (Bool.and_eq_true_iff.mp upperMember).1
    have rightMember : right.support.contains upper = true := by
      rw [AssuranceKindSet.contains_inter] at upperMember
      change (left.support.contains upper && right.support.contains upper) = true at upperMember
      exact (Bool.and_eq_true_iff.mp upperMember).2
    have leftLower := left.closed lower upper lowerAllowed upperAllowed lowerUpper leftMember
    have rightLower := right.closed lower upper lowerAllowed upperAllowed lowerUpper rightMember
    rw [AssuranceKindSet.contains_inter]
    change (left.support.contains lower && right.support.contains lower) = true
    exact Bool.and_eq_true_iff.mpr ⟨leftLower, rightLower⟩

theorem intersectNF_denotation {fiber} (left right : AntichainNF fiber)
    (point : AssurancePolicyV2) :
    (intersectNF left right).denotes point ↔
      left.denotes point ∧ right.denotes point := by
  rcases point with ⟨pointFiber, pointKind⟩
  cases pointKind <;>
    simp [AntichainNF.denotes, intersectNF, AssuranceKindSet.inter,
      AssuranceKindSet.ofPredicate, AssuranceKindSet.contains,
      and_assoc, and_left_comm]

theorem intersectNF_comm {fiber} (left right : AntichainNF fiber) :
    (intersectNF left right).support = (intersectNF right left).support := by
  cases left with | mk left _ _ =>
    cases right with | mk right _ _ =>
      cases left <;> cases right
      simp [intersectNF, AssuranceKindSet.inter, AssuranceKindSet.ofPredicate,
        AssuranceKindSet.contains, Bool.and_comm]

theorem intersectNF_assoc {fiber} (first second third : AntichainNF fiber) :
    (intersectNF (intersectNF first second) third).support =
      (intersectNF first (intersectNF second third)).support := by
  cases first with | mk first _ _ =>
    cases second with | mk second _ _ =>
      cases third with | mk third _ _ =>
        cases first <;> cases second <;> cases third
        simp [intersectNF, AssuranceKindSet.inter, AssuranceKindSet.ofPredicate,
          AssuranceKindSet.contains, Bool.and_assoc]

theorem intersectNF_idem {fiber} (normal : AntichainNF fiber) :
    (intersectNF normal normal).support = normal.support := by
  cases normal with | mk support _ _ =>
    cases support
    simp [intersectNF, AssuranceKindSet.inter, AssuranceKindSet.ofPredicate,
      AssuranceKindSet.contains]

@[ext] theorem AntichainNF.ext {fiber} {left right : AntichainNF fiber}
    (support : left.support = right.support) : left = right := by
  cases left
  cases right
  simp_all

theorem intersectNF_commutative {fiber} (left right : AntichainNF fiber) :
    intersectNF left right = intersectNF right left :=
  AntichainNF.ext (intersectNF_comm left right)

theorem intersectNF_associative {fiber} (first second third : AntichainNF fiber) :
    intersectNF (intersectNF first second) third =
      intersectNF first (intersectNF second third) :=
  AntichainNF.ext (intersectNF_assoc first second third)

theorem intersectNF_duplicate_invariant {fiber} (normal : AntichainNF fiber) :
    intersectNF normal normal = normal :=
  AntichainNF.ext (intersectNF_idem normal)

def strictlyBelow (left right : AssuranceKindV2) : Bool :=
  assuranceKindLeq left right && !(assuranceKindLeq right left)

/-- Fixed-order maximal-antichain serialization of the canonical support. -/
def AntichainNF.frontier {fiber} (normal : AntichainNF fiber) : List AssuranceKindV2 :=
  allAssuranceKindsV2.filter fun candidate =>
    normal.support.contains candidate &&
      !allAssuranceKindsV2.any fun other =>
        normal.support.contains other && strictlyBelow candidate other

theorem frontier_has_no_duplicates {fiber} (normal : AntichainNF fiber) :
    normal.frontier.Nodup := by
  exact List.Pairwise.filter _ (by decide)

def assuranceKindOrdinal : AssuranceKindV2 → Nat
  | .runtime => 0
  | .bounded => 1
  | .solverIncomplete => 2
  | .solverComplete => 3
  | .leanEmpirical => 4
  | .leanComplete => 5

theorem frontier_has_canonical_order {fiber} (normal : AntichainNF fiber) :
    normal.frontier.Pairwise fun left right =>
      assuranceKindOrdinal left < assuranceKindOrdinal right := by
  exact List.Pairwise.filter _ (by decide)

theorem frontier_members_are_supported {fiber} (normal : AntichainNF fiber)
    {kind : AssuranceKindV2} (member : kind ∈ normal.frontier) :
    normal.support.contains kind = true := by
  have filtered := (List.mem_filter.mp member).2
  exact (Bool.and_eq_true_iff.mp filtered).1

theorem frontier_members_are_maximal {fiber} (normal : AntichainNF fiber)
    {kind : AssuranceKindV2} (member : kind ∈ normal.frontier)
    {other : AssuranceKindV2} (otherSupported : normal.support.contains other = true)
    (below : assuranceKindLeq kind other = true) :
    assuranceKindLeq other kind = true := by
  have filtered := (List.mem_filter.mp member).2
  have noLarger : (allAssuranceKindsV2.any fun candidate =>
      normal.support.contains candidate && strictlyBelow kind candidate) = false := by
    simpa using (Bool.and_eq_true_iff.mp filtered).2
  cases reverse : assuranceKindLeq other kind with
  | true => rfl
  | false =>
      have strict : strictlyBelow kind other = true := by
        simp [strictlyBelow, below, reverse]
      have otherMember : other ∈ allAssuranceKindsV2 := by
        cases other <;> simp [allAssuranceKindsV2]
      have someLarger : (allAssuranceKindsV2.any fun candidate =>
          normal.support.contains candidate && strictlyBelow kind candidate) = true :=
        List.any_eq_true.mpr ⟨other, otherMember,
          Bool.and_eq_true_iff.mpr ⟨otherSupported, strict⟩⟩
      rw [someLarger] at noLarger
      contradiction

theorem frontier_members_are_pairwise_incomparable {fiber}
    (normal : AntichainNF fiber) {left right : AssuranceKindV2}
    (leftMember : left ∈ normal.frontier) (rightMember : right ∈ normal.frontier)
    (different : left ≠ right) :
    assuranceKindLeq left right = false ∧ assuranceKindLeq right left = false := by
  constructor
  · cases forward : assuranceKindLeq left right with
    | false => rfl
    | true =>
        have reverse := frontier_members_are_maximal normal leftMember
          (frontier_members_are_supported normal rightMember) forward
        exact False.elim (different (assuranceKindLeq_antisymm forward reverse))
  · cases forward : assuranceKindLeq right left with
    | false => rfl
    | true =>
        have reverse := frontier_members_are_maximal normal rightMember
          (frontier_members_are_supported normal leftMember) forward
        exact False.elim (different (assuranceKindLeq_antisymm reverse forward))

set_option maxHeartbeats 1000000 in
/-- Every supported family is below a serialized maximal family. This proof is
symbolic in the fiber parameters and exhaustive only over the six constructor
families. -/
theorem frontier_covers_support {fiber} (normal : AntichainNF fiber)
    (kind : AssuranceKindV2) (supported : normal.support.contains kind = true) :
    ∃ upper, upper ∈ normal.frontier ∧
      assuranceKindLeq kind upper = true := by
  rcases normal with ⟨⟨runtime, bounded, incomplete, solver, empirical, lean⟩,
    allowed, closed⟩
  cases kind <;> cases runtime <;> cases bounded <;> cases incomplete <;>
    cases solver <;> cases empirical <;> cases lean <;>
    simp_all [AntichainNF.frontier, allAssuranceKindsV2,
      AssuranceKindSet.contains, strictlyBelow, assuranceKindLeq]

theorem support_iff_below_frontier {fiber} (normal : AntichainNF fiber)
    (kind : AssuranceKindV2)
    (allowed : kindAllowed fiber.population kind = true) :
    normal.support.contains kind = true ↔
      ∃ upper, upper ∈ normal.frontier ∧ assuranceKindLeq kind upper = true := by
  constructor
  · exact frontier_covers_support normal kind
  · rintro ⟨upper, upperMember, below⟩
    exact normal.closed kind upper allowed
      (normal.allowed upper (frontier_members_are_supported normal upperMember))
      below (frontier_members_are_supported normal upperMember)

theorem antichain_nf_is_extensional {fiber} (left right : AntichainNF fiber)
    (sameDenotation : ∀ kind,
      left.support.contains kind = right.support.contains kind) : left = right := by
  exact AntichainNF.ext (AssuranceKindSet.ext sameDenotation)

inductive FiniteIntersectionV2 (fiber : ClaimFiberKeyV2) where
  | noItems
  | common (normal : AntichainNF fiber)

def intersectNonemptyNF {fiber} (first : AntichainNF fiber) :
    List (AntichainNF fiber) → AntichainNF fiber
  | [] => first
  | next :: rest => intersectNF next (intersectNonemptyNF first rest)

def intersectAllNF {fiber} : List (AntichainNF fiber) → FiniteIntersectionV2 fiber
  | [] => .noItems
  | first :: rest => .common (intersectNonemptyNF first rest)

theorem intersectAllNF_empty_is_explicit_no_items {fiber} :
    intersectAllNF ([] : List (AntichainNF fiber)) = .noItems := rfl

theorem intersectNonemptyNF_denotation {fiber} (first : AntichainNF fiber)
    (rest : List (AntichainNF fiber)) (point : AssurancePolicyV2) :
    (intersectNonemptyNF first rest).denotes point ↔
      first.denotes point ∧ ∀ normal ∈ rest, normal.denotes point := by
  induction rest with
  | nil => simp [intersectNonemptyNF]
  | cons next rest induction =>
      rw [intersectNonemptyNF, intersectNF_denotation, induction]
      constructor
      · rintro ⟨nextDenotes, firstDenotes, restDenotes⟩
        exact ⟨firstDenotes, by
          intro normal member
          simp only [List.mem_cons] at member
          rcases member with equal | member
          · subst normal
            exact nextDenotes
          · exact restDenotes normal member⟩
      · rintro ⟨firstDenotes, allDenote⟩
        exact ⟨allDenote next (by simp), firstDenotes, fun normal member =>
          allDenote normal (by simp [member])⟩

theorem intersectNonemptyNF_permutation_invariant {fiber}
    (first : AntichainNF fiber) {left right : List (AntichainNF fiber)}
    (permutation : left.Perm right) :
    intersectNonemptyNF first left = intersectNonemptyNF first right := by
  apply antichain_nf_is_extensional
  intro kind
  have semantic :
      (intersectNonemptyNF first left).denotes ⟨fiber, kind⟩ ↔
        (intersectNonemptyNF first right).denotes ⟨fiber, kind⟩ := by
    rw [intersectNonemptyNF_denotation, intersectNonemptyNF_denotation]
    constructor <;> rintro ⟨firstDenotes, allDenote⟩ <;>
      refine ⟨firstDenotes, ?_⟩
    · intro normal member
      exact allDenote normal (permutation.mem_iff.mpr member)
    · intro normal member
      exact allDenote normal (permutation.mem_iff.mp member)
  cases leftMember : (intersectNonemptyNF first left).support.contains kind <;>
    cases rightMember : (intersectNonemptyNF first right).support.contains kind <;>
    simp_all [AntichainNF.denotes]

theorem intersectNonemptyNF_head_swap_invariant {fiber}
    (first second : AntichainNF fiber) (rest : List (AntichainNF fiber)) :
    intersectNonemptyNF first (second :: rest) =
      intersectNonemptyNF second (first :: rest) := by
  apply antichain_nf_is_extensional
  intro kind
  have semantic :
      (intersectNonemptyNF first (second :: rest)).denotes ⟨fiber, kind⟩ ↔
        (intersectNonemptyNF second (first :: rest)).denotes ⟨fiber, kind⟩ := by
    rw [intersectNonemptyNF_denotation, intersectNonemptyNF_denotation]
    simp [and_left_comm]
  cases leftMember : (intersectNonemptyNF first (second :: rest)).support.contains kind <;>
    cases rightMember : (intersectNonemptyNF second (first :: rest)).support.contains kind <;>
    simp_all [AntichainNF.denotes]

theorem constructor_signature_is_finite_but_parameters_are_symbolic :
    allAssuranceKindsV2.length = 6 ∧ allAssuranceKindsV2.Nodup := by decide

/-- The symbolic pair law used by generated Rust replay. -/
def lowerBoundFrontierKinds (left right : AssuranceKindV2) : List AssuranceKindV2 :=
  let support := AssuranceKindSet.ofPredicate fun candidate =>
    assuranceKindLeq candidate left && assuranceKindLeq candidate right
  allAssuranceKindsV2.filter fun candidate =>
    support.contains candidate &&
      !allAssuranceKindsV2.any fun other =>
        support.contains other && strictlyBelow candidate other

structure ConstructorPairLawV2 where
  left : AssuranceKindV2
  right : AssuranceKindV2
  leftLeRight : Bool
  rightLeLeft : Bool
  lowerBounds : List AssuranceKindV2
deriving DecidableEq, Repr

def constructorPairLawsV2 : List ConstructorPairLawV2 :=
  allAssuranceKindsV2.flatMap fun left =>
    allAssuranceKindsV2.map fun right =>
      ⟨left, right, assuranceKindLeq left right, assuranceKindLeq right left,
        lowerBoundFrontierKinds left right⟩

theorem constructor_pair_matrix_is_exact : constructorPairLawsV2.length = 36 := by decide

set_option maxHeartbeats 1000000 in
theorem lower_bound_frontier_is_sound (left right candidate : AssuranceKindV2)
    (member : candidate ∈ lowerBoundFrontierKinds left right) :
    assuranceKindLeq candidate left = true ∧
      assuranceKindLeq candidate right = true := by
  cases left <;> cases right <;> cases candidate <;>
    simp_all [lowerBoundFrontierKinds, allAssuranceKindsV2,
      AssuranceKindSet.contains, AssuranceKindSet.ofPredicate,
      strictlyBelow, assuranceKindLeq]

set_option maxHeartbeats 1000000 in
theorem lower_bound_frontier_is_complete (left right candidate : AssuranceKindV2)
    (leftBound : assuranceKindLeq candidate left = true)
    (rightBound : assuranceKindLeq candidate right = true) :
    ∃ maximal, maximal ∈ lowerBoundFrontierKinds left right ∧
      assuranceKindLeq candidate maximal = true := by
  cases left <;> cases right <;> cases candidate <;>
    simp_all [lowerBoundFrontierKinds, allAssuranceKindsV2,
      AssuranceKindSet.contains, AssuranceKindSet.ofPredicate,
      strictlyBelow, assuranceKindLeq]

theorem singleton_lower_bound_frontier_is_glb
    (left right meet : AssuranceKindV2)
    (singleton : lowerBoundFrontierKinds left right = [meet]) :
    assuranceKindLeq meet left = true ∧
      assuranceKindLeq meet right = true ∧
      ∀ candidate, assuranceKindLeq candidate left = true →
        assuranceKindLeq candidate right = true →
        assuranceKindLeq candidate meet = true := by
  have meetMember : meet ∈ lowerBoundFrontierKinds left right := by
    rw [singleton]
    simp
  have sound := lower_bound_frontier_is_sound left right meet meetMember
  refine ⟨sound.1, sound.2, ?_⟩
  intro candidate leftBound rightBound
  rcases lower_bound_frontier_is_complete left right candidate leftBound rightBound with
    ⟨maximal, member, below⟩
  rw [singleton] at member
  have maximalIsMeet : maximal = meet := by simpa using member
  subst maximal
  exact below

theorem complete_solver_and_empirical_have_incomplete_common_claim :
    lowerBoundFrontierKinds .solverComplete .leanEmpirical = [.solverIncomplete] := by decide

theorem lean_complete_and_solver_complete_agree_on_solver_complete_lower_bound :
    lowerBoundFrontierKinds .leanComplete .solverComplete = [.solverComplete] := by decide

theorem complete_route_disclosures_remain_distinct :
    engineerDisclosureFor admittedSolverCompleteFixture ≠
      engineerDisclosureFor admittedLeanCompleteFixture := by
  intro equal
  have families := congrArg EngineerDisclosureV1.sourceFamily equal
  simp [engineerDisclosureFor, admittedSolverCompleteFixture,
    admittedLeanCompleteFixture, AdmittedRealizedCertificationV2.family] at families

/-! Executable antichain fixtures. They pin singleton, comparable,
incomparable, and multiple-frontier behavior without enumerating parameter
values. -/

def allInputFixtureFiber : ClaimFiberKeyV2 :=
  (assuranceFixtureRealized .allInputs .complete .solver).claimFiber

def solverCompleteNF : AntichainNF allInputFixtureFiber :=
  principalNF ⟨allInputFixtureFiber, .solverComplete⟩

def empiricalLeanNF : AntichainNF allInputFixtureFiber :=
  principalNF ⟨allInputFixtureFiber, .leanEmpirical⟩

def leanCompleteNF : AntichainNF allInputFixtureFiber :=
  principalNF ⟨allInputFixtureFiber, .leanComplete⟩

def incomparableForkNF : AntichainNF allInputFixtureFiber where
  support := ⟨false, false, true, true, true, false⟩
  allowed := by
    intro kind member
    cases kind <;>
      simp_all [allInputFixtureFiber, RealizedCertification.claimFiber,
        assuranceFixtureRealized, assuranceFixturePopulation,
        AssuranceKindSet.contains, kindAllowed]
  closed := by
    intro lower upper lowerAllowed upperAllowed lowerUpper member
    cases lower <;> cases upper <;>
      simp_all [AssuranceKindSet.contains, assuranceKindLeq]

theorem singleton_frontier_fixture : solverCompleteNF.frontier = [.solverComplete] := by decide

theorem incomparable_intersection_fixture :
    (intersectNF solverCompleteNF empiricalLeanNF).frontier = [.solverIncomplete] := by decide

theorem multiple_maximal_frontier_fixture :
    incomparableForkNF.frontier = [.solverComplete, .leanEmpirical] := by decide

theorem multiple_frontier_survives_stronger_intersection :
    (intersectNF incomparableForkNF leanCompleteNF).frontier =
      [.solverComplete, .leanEmpirical] := by decide

def boundedFixtureFiber (bound : Nat) : ClaimFiberKeyV2 :=
  (assuranceFixtureRealized (.throughBound bound) .complete .solver).claimFiber

theorem arbitrary_bounds_are_exact_not_sampled {left right : Nat}
    (different : left ≠ right) : boundedFixtureFiber left ≠ boundedFixtureFiber right := by
  intro equal
  have populations := congrArg ClaimFiberKeyV2.population equal
  have boundsEqual : left = right := by
    simpa [boundedFixtureFiber, RealizedCertification.claimFiber,
      assuranceFixtureRealized, assuranceFixturePopulation] using populations
  exact different boundsEqual

end Thermite.CertificationMetatheory
