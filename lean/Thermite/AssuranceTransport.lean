import Thermite.CertificationMetatheory

/-!
Checked cross-version, cross-model, and cross-procedure assurance transport.

This layer is deliberately separate from `AssurancePolicyV2`: policy points
may be compared only after one of these witnesses establishes that the two
authorities inhabit a compatible target fiber.
-/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

/-- The operational part of a procedure simulation.  Version changes are
allowed, but environment/tool identity and resource monotonicity must be
proved explicitly rather than inferred from a procedure name. -/
structure ProcedureSimulation
    (source target : CertificationProcedure)
    (SourceEvidence TargetEvidence : Type)
    (sourceObservation targetObservation : ObservationContract) where
  reindexProgram : Program → Program
  translateEvidence : SourceEvidence → TargetEvidence
  environment : source.environment = target.environment
  toolVersion : source.toolVersion = target.toolVersion
  resourceBudget : source.resourceBudget ≤ target.resourceBudget
  observation : ∀ program, sourceObservation.observes program →
    targetObservation.observes (reindexProgram program)

def ProcedureSimulation.identity
    (procedure : CertificationProcedure)
    (Evidence : Type) (observation : ObservationContract) :
    ProcedureSimulation procedure procedure Evidence Evidence observation observation := {
  reindexProgram := id
  translateEvidence := id
  environment := rfl
  toolVersion := rfl
  resourceBudget := Nat.le_refl _
  observation := fun _ observed => observed
}

def ProcedureSimulation.comp
    {first second third : CertificationProcedure}
    {Evidence₁ Evidence₂ Evidence₃ : Type}
    {observation₁ observation₂ observation₃ : ObservationContract}
    (firstSecond : ProcedureSimulation first second Evidence₁ Evidence₂
      observation₁ observation₂)
    (secondThird : ProcedureSimulation second third Evidence₂ Evidence₃
      observation₂ observation₃) :
    ProcedureSimulation first third Evidence₁ Evidence₃ observation₁ observation₃ := {
  reindexProgram := secondThird.reindexProgram ∘ firstSecond.reindexProgram
  translateEvidence := secondThird.translateEvidence ∘ firstSecond.translateEvidence
  environment := firstSecond.environment.trans secondThird.environment
  toolVersion := firstSecond.toolVersion.trans secondThird.toolVersion
  resourceBudget := Nat.le_trans firstSecond.resourceBudget secondThird.resourceBudget
  observation := fun program observed =>
    secondThird.observation _ (firstSecond.observation program observed)
}

/-- A semantic-version transport keeps the source and target identities
visible.  The semantic and model equalities state lineage; the version
relations and preservation fields carry the actual compatibility proof. -/
structure SemanticVersionTransport
    (source target : SemanticFrame)
    (sourceContext targetContext : ResidualContext)
    (sourceFragment targetFragment : Fragment)
    (sourceClaim targetClaim : Program → Prop)
    (SourceEvidence TargetEvidence : Type)
    (sourceObservation targetObservation : ObservationContract) where
  reindexProgram : Program → Program
  translateEvidence : SourceEvidence → TargetEvidence
  semantics : source.semantics = target.semantics
  semanticsVersion : source.semanticsVersion ≤ target.semanticsVersion
  implementationModel : source.implementationModel = target.implementationModel
  implementationModelVersion :
    source.implementationModelVersion ≤ target.implementationModelVersion
  context : ContextRefines sourceContext targetContext
  membership : ∀ program, sourceFragment.admits program →
    targetFragment.admits (reindexProgram program)
  claim : ∀ program, sourceClaim program → targetClaim (reindexProgram program)
  boundary : BoundaryRefines source.boundary target.boundary
  observation : ObservationRefines reindexProgram sourceObservation targetObservation
  targetEvidenceValid : ∀ (evidence : SourceEvidence) (program : Program),
    sourceFragment.admits program → sourceClaim program →
    targetFragment.admits (reindexProgram program) ∧
      targetClaim (reindexProgram program)

/-- A full transport composes semantic translation with a typed procedure
simulation.  Keeping these as separate fields prevents a procedure change
from being smuggled in as a semantic-version change. -/
structure AssuranceTransport
    (sourceFrame targetFrame : SemanticFrame)
    (sourceContext targetContext : ResidualContext)
    (sourceFragment targetFragment : Fragment)
    (sourceProcedure targetProcedure : CertificationProcedure)
    (sourceClaim targetClaim : Program → Prop)
    (SourceEvidence TargetEvidence : Type)
    (sourceObservation targetObservation : ObservationContract) where
  semantic : SemanticVersionTransport sourceFrame targetFrame sourceContext targetContext
    sourceFragment targetFragment sourceClaim targetClaim SourceEvidence TargetEvidence
    sourceObservation targetObservation
  procedure : ProcedureSimulation sourceProcedure targetProcedure SourceEvidence TargetEvidence
    sourceObservation targetObservation

def AssuranceTransport.identity
    (frame : SemanticFrame) (context : ResidualContext) (fragment : Fragment)
    (procedure : CertificationProcedure) (claim : Program → Prop)
    (Evidence : Type) (observation : ObservationContract) :
    AssuranceTransport frame frame context context fragment fragment procedure procedure
      claim claim Evidence Evidence observation observation := {
  semantic := {
    reindexProgram := id
    translateEvidence := id
    semantics := rfl
    semanticsVersion := Nat.le_refl _
    implementationModel := rfl
    implementationModelVersion := Nat.le_refl _
    context := fun held => held
    membership := fun _ admitted => admitted
    claim := fun _ proved => proved
    boundary := fun _ qualified => qualified
    observation := fun _ observed => observed
    targetEvidenceValid := fun _ _ admitted proved => ⟨admitted, proved⟩
  }
  procedure := ProcedureSimulation.identity procedure Evidence observation
}

def AssuranceTransport.comp
    {frame₁ frame₂ frame₃ : SemanticFrame}
    {context₁ context₂ context₃ : ResidualContext}
    {fragment₁ fragment₂ fragment₃ : Fragment}
    {procedure₁ procedure₂ procedure₃ : CertificationProcedure}
    {claim₁ claim₂ claim₃ : Program → Prop}
    {Evidence₁ Evidence₂ Evidence₃ : Type}
    {observation₁ observation₂ observation₃ : ObservationContract}
    (firstSecond : AssuranceTransport frame₁ frame₂ context₁ context₂ fragment₁ fragment₂
      procedure₁ procedure₂ claim₁ claim₂ Evidence₁ Evidence₂ observation₁ observation₂)
    (secondThird : AssuranceTransport frame₂ frame₃ context₂ context₃ fragment₂ fragment₃
      procedure₂ procedure₃ claim₂ claim₃ Evidence₂ Evidence₃ observation₂ observation₃) :
    AssuranceTransport frame₁ frame₃ context₁ context₃ fragment₁ fragment₃ procedure₁ procedure₃
      claim₁ claim₃ Evidence₁ Evidence₃ observation₁ observation₃ := {
  semantic := {
    reindexProgram := secondThird.semantic.reindexProgram ∘ firstSecond.semantic.reindexProgram
    translateEvidence := secondThird.semantic.translateEvidence ∘ firstSecond.semantic.translateEvidence
    semantics := firstSecond.semantic.semantics.trans secondThird.semantic.semantics
    semanticsVersion := Nat.le_trans firstSecond.semantic.semanticsVersion
      secondThird.semantic.semanticsVersion
    implementationModel := firstSecond.semantic.implementationModel.trans
      secondThird.semantic.implementationModel
    implementationModelVersion := Nat.le_trans firstSecond.semantic.implementationModelVersion
      secondThird.semantic.implementationModelVersion
    context := fun held => secondThird.semantic.context (firstSecond.semantic.context held)
    membership := fun program admitted =>
      secondThird.semantic.membership _ (firstSecond.semantic.membership program admitted)
    claim := fun program proved =>
      secondThird.semantic.claim _ (firstSecond.semantic.claim program proved)
    boundary := fun program qualified =>
      secondThird.semantic.boundary program (firstSecond.semantic.boundary program qualified)
    observation := fun program observed =>
      secondThird.semantic.observation _ (firstSecond.semantic.observation program observed)
    targetEvidenceValid := fun evidence program admitted claim =>
      secondThird.semantic.targetEvidenceValid
        (firstSecond.semantic.translateEvidence evidence) _
        (firstSecond.semantic.membership program admitted)
        (firstSecond.semantic.claim program claim)
  }
  procedure := ProcedureSimulation.comp firstSecond.procedure secondThird.procedure
}

inductive IncompatibilityReason where
  | semanticFork
  | modelBreak
  | unsupportedProcedure
  | boundaryConflict
  | contextConflict
deriving DecidableEq, Repr

/-- A checked incompatibility is distinct from an absent row. -/
structure IncompatibilityWitness where
  source : String
  target : String
  reason : IncompatibilityReason
  explanation : String
  sourceIndependent : Prop
  targetIndependent : Prop
  noTransport : Prop
deriving Repr

inductive PredecessorRelation where
  | transported (source target : String) (receipt : String)
  | incompatible (source target : String) (witness : IncompatibilityWitness)
deriving Repr

def PredecessorRelation.source : PredecessorRelation → String
  | .transported source .. | .incompatible source .. => source

def PredecessorRelation.target : PredecessorRelation → String
  | .transported _ target .. | .incompatible _ target .. => target

def predecessorRelationKeys (relations : List PredecessorRelation) : List (String × String) :=
  relations.map fun relation => (relation.source, relation.target)

def predecessorDomainComplete
    (declared : List (String × String)) (relations : List PredecessorRelation) : Prop :=
  (∀ edge, edge ∈ declared → edge ∈ predecessorRelationKeys relations) ∧
  (∀ edge, edge ∈ predecessorRelationKeys relations → edge ∈ declared) ∧
  (relations.length = (predecessorRelationKeys relations).eraseDups.length)

theorem predecessorDomainComplete_refl
    (relations : List PredecessorRelation)
    (complete : predecessorDomainComplete (predecessorRelationKeys relations) relations) :
    relations.length = (predecessorRelationKeys relations).eraseDups.length :=
  complete.2.2

theorem assurance_transport_identity_left
    {frame targetFrame : SemanticFrame}
    {context targetContext : ResidualContext}
    {fragment targetFragment : Fragment}
    {procedure targetProcedure : CertificationProcedure}
    {claim targetClaim : Program → Prop}
    {Evidence TargetEvidence : Type}
    {observation targetObservation : ObservationContract}
    (transport : AssuranceTransport frame targetFrame context targetContext fragment targetFragment
      procedure targetProcedure claim targetClaim Evidence TargetEvidence observation targetObservation) :
    AssuranceTransport.comp (AssuranceTransport.identity frame context fragment procedure claim Evidence observation)
      transport = transport := by
  cases transport
  simp [AssuranceTransport.comp, AssuranceTransport.identity,
    ProcedureSimulation.comp, ProcedureSimulation.identity, Function.comp_def]

theorem assurance_transport_identity_right
    {frame targetFrame : SemanticFrame}
    {context targetContext : ResidualContext}
    {fragment targetFragment : Fragment}
    {procedure targetProcedure : CertificationProcedure}
    {claim targetClaim : Program → Prop}
    {Evidence TargetEvidence : Type}
    {observation targetObservation : ObservationContract}
    (transport : AssuranceTransport frame targetFrame context targetContext fragment targetFragment
      procedure targetProcedure claim targetClaim Evidence TargetEvidence observation targetObservation) :
    AssuranceTransport.comp transport
      (AssuranceTransport.identity targetFrame targetContext targetFragment targetProcedure
        targetClaim TargetEvidence targetObservation) = transport := by
  cases transport
  simp [AssuranceTransport.comp, AssuranceTransport.identity,
    ProcedureSimulation.comp, ProcedureSimulation.identity, Function.comp_def]

end Thermite.CertificationMetatheory
