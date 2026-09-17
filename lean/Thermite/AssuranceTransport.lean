import Thermite.ImplementationModel

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
    (reindexProgram : Program → Program)
    (translateEvidence : SourceEvidence → TargetEvidence)
    (sourceObservation targetObservation : ObservationContract) where
  environment : source.environment = target.environment
  toolVersion : source.toolVersion = target.toolVersion
  resourceBudget : source.resourceBudget ≤ target.resourceBudget
  observation : ∀ program, sourceObservation.observes program →
    targetObservation.observes (reindexProgram program)
  failureReflection : ∀ program, targetObservation.observes (reindexProgram program) →
    sourceObservation.observes program

def ProcedureSimulation.identity
    (procedure : CertificationProcedure)
    (Evidence : Type) (observation : ObservationContract) :
    ProcedureSimulation procedure procedure Evidence Evidence id id observation observation := {
  environment := rfl
  toolVersion := rfl
  resourceBudget := Nat.le_refl _
  observation := fun _ observed => observed
  failureReflection := fun _ observed => observed
}

def ProcedureSimulation.comp
    {first second third : CertificationProcedure}
    {Evidence₁ Evidence₂ Evidence₃ : Type}
    {reindex₁₂ reindex₂₃ : Program → Program}
    {translate₁₂ : Evidence₁ → Evidence₂}
    {translate₂₃ : Evidence₂ → Evidence₃}
    {observation₁ observation₂ observation₃ : ObservationContract}
    (firstSecond : ProcedureSimulation first second Evidence₁ Evidence₂ reindex₁₂ translate₁₂
      observation₁ observation₂)
    (secondThird : ProcedureSimulation second third Evidence₂ Evidence₃ reindex₂₃ translate₂₃
      observation₂ observation₃) :
    ProcedureSimulation first third Evidence₁ Evidence₃ (reindex₂₃ ∘ reindex₁₂)
      (translate₂₃ ∘ translate₁₂) observation₁ observation₃ := {
  environment := firstSecond.environment.trans secondThird.environment
  toolVersion := firstSecond.toolVersion.trans secondThird.toolVersion
  resourceBudget := Nat.le_trans firstSecond.resourceBudget secondThird.resourceBudget
  observation := fun program observed =>
    secondThird.observation _ (firstSecond.observation program observed)
  failureReflection := fun program observed =>
    firstSecond.failureReflection program (secondThird.failureReflection _ observed)
}

/-- Implementation-model evolution is a separately typed obligation.  A
semantic-version transport cannot smuggle in a model-family change. -/
structure ImplementationModelVersionTransport (source target : SemanticFrame) where
  implementationModel : source.implementationModel = target.implementationModel
  implementationModelVersion :
    source.implementationModelVersion ≤ target.implementationModelVersion

def ImplementationModelVersionTransport.identity (frame : SemanticFrame) :
    ImplementationModelVersionTransport frame frame := {
  implementationModel := rfl
  implementationModelVersion := Nat.le_refl _
}

def ImplementationModelVersionTransport.comp
    {first second third : SemanticFrame}
    (firstSecond : ImplementationModelVersionTransport first second)
    (secondThird : ImplementationModelVersionTransport second third) :
    ImplementationModelVersionTransport first third := {
  implementationModel := firstSecond.implementationModel.trans
    secondThird.implementationModel
  implementationModelVersion := Nat.le_trans firstSecond.implementationModelVersion
    secondThird.implementationModelVersion
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
    (sourceObservation targetObservation : ObservationContract)
    (sourceCertifies : SourceEvidence → Program → Prop)
    (targetCertifies : TargetEvidence → Program → Prop)
    (reindexProgram : Program → Program)
    (translateEvidence : SourceEvidence → TargetEvidence) where
  semantics : source.semantics = target.semantics
  semanticsVersion : source.semanticsVersion ≤ target.semanticsVersion
  model : ImplementationModelVersionTransport source target
  context : ContextRefines sourceContext targetContext
  membership : ∀ program, sourceFragment.admits program →
    targetFragment.admits (reindexProgram program)
  claim : ∀ program, sourceClaim program → targetClaim (reindexProgram program)
  boundary : BoundaryRefines source.boundary target.boundary
  observation : ObservationRefines reindexProgram sourceObservation targetObservation
  certification : ∀ evidence program, sourceCertifies evidence program →
    targetCertifies (translateEvidence evidence) (reindexProgram program)
  sourceRefutationSoundness : RefutationSoundness sourceClaim sourceObservation
  targetRefutationSoundness : RefutationSoundness targetClaim targetObservation
  sourceRefutationCompleteness : RefutationCompleteness sourceClaim sourceObservation
  targetRefutationCompleteness : RefutationCompleteness targetClaim targetObservation

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
    (sourceObservation targetObservation : ObservationContract)
    (sourceCertifies : SourceEvidence → Program → Prop)
    (targetCertifies : TargetEvidence → Program → Prop) where
  reindexProgram : Program → Program
  translateEvidence : SourceEvidence → TargetEvidence
  semantic : SemanticVersionTransport sourceFrame targetFrame sourceContext targetContext
    sourceFragment targetFragment sourceClaim targetClaim SourceEvidence TargetEvidence
    sourceObservation targetObservation sourceCertifies targetCertifies reindexProgram
    translateEvidence
  procedure : ProcedureSimulation sourceProcedure targetProcedure SourceEvidence TargetEvidence
    reindexProgram translateEvidence sourceObservation targetObservation

def AssuranceTransport.identity
    (frame : SemanticFrame) (context : ResidualContext) (fragment : Fragment)
    (procedure : CertificationProcedure) (claim : Program → Prop)
    (Evidence : Type) (observation : ObservationContract)
    (certifies : Evidence → Program → Prop)
    (refutationSoundness : RefutationSoundness claim observation)
    (refutationCompleteness : RefutationCompleteness claim observation) :
    AssuranceTransport frame frame context context fragment fragment procedure procedure
      claim claim Evidence Evidence observation observation certifies certifies := {
  reindexProgram := id
  translateEvidence := id
  semantic := {
    semantics := rfl
    semanticsVersion := Nat.le_refl _
    model := ImplementationModelVersionTransport.identity _
    context := fun held => held
    membership := fun _ admitted => admitted
    claim := fun _ proved => proved
    boundary := fun _ qualified => qualified
    observation := fun _ observed => observed
    certification := fun _ _ certified => certified
    sourceRefutationSoundness := refutationSoundness
    targetRefutationSoundness := refutationSoundness
    sourceRefutationCompleteness := refutationCompleteness
    targetRefutationCompleteness := refutationCompleteness
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
    {certifies₁ : Evidence₁ → Program → Prop}
    {certifies₂ : Evidence₂ → Program → Prop}
    {certifies₃ : Evidence₃ → Program → Prop}
    (firstSecond : AssuranceTransport frame₁ frame₂ context₁ context₂ fragment₁ fragment₂
      procedure₁ procedure₂ claim₁ claim₂ Evidence₁ Evidence₂ observation₁ observation₂
      certifies₁ certifies₂)
    (secondThird : AssuranceTransport frame₂ frame₃ context₂ context₃ fragment₂ fragment₃
      procedure₂ procedure₃ claim₂ claim₃ Evidence₂ Evidence₃ observation₂ observation₃
      certifies₂ certifies₃) :
    AssuranceTransport frame₁ frame₃ context₁ context₃ fragment₁ fragment₃ procedure₁ procedure₃
      claim₁ claim₃ Evidence₁ Evidence₃ observation₁ observation₃ certifies₁ certifies₃ := {
  reindexProgram := secondThird.reindexProgram ∘ firstSecond.reindexProgram
  translateEvidence := secondThird.translateEvidence ∘ firstSecond.translateEvidence
  semantic := {
    semantics := firstSecond.semantic.semantics.trans secondThird.semantic.semantics
    semanticsVersion := Nat.le_trans firstSecond.semantic.semanticsVersion
      secondThird.semantic.semanticsVersion
    model := ImplementationModelVersionTransport.comp firstSecond.semantic.model
      secondThird.semantic.model
    context := fun held => secondThird.semantic.context (firstSecond.semantic.context held)
    membership := fun program admitted =>
      secondThird.semantic.membership _ (firstSecond.semantic.membership program admitted)
    claim := fun program proved =>
      secondThird.semantic.claim _ (firstSecond.semantic.claim program proved)
    boundary := fun program qualified =>
      secondThird.semantic.boundary program (firstSecond.semantic.boundary program qualified)
    observation := fun program observed =>
      secondThird.semantic.observation _ (firstSecond.semantic.observation program observed)
    certification := fun evidence program certified =>
      secondThird.semantic.certification _ _
        (firstSecond.semantic.certification evidence program certified)
    sourceRefutationSoundness := firstSecond.semantic.sourceRefutationSoundness
    targetRefutationSoundness := secondThird.semantic.targetRefutationSoundness
    sourceRefutationCompleteness := firstSecond.semantic.sourceRefutationCompleteness
    targetRefutationCompleteness := secondThird.semantic.targetRefutationCompleteness
  }
  procedure := ProcedureSimulation.comp firstSecond.procedure secondThird.procedure
}

/-! Cross-implementation-model-family transport. The existing
`AssuranceTransport` remains the version-only compatibility surface; this
separate type makes a family change impossible without a typed
`ModelRefinement` over the families' own input and behavior carriers. -/

structure CrossModelAssuranceTransport
    (sourceModel targetModel : ImplementationModelFamily)
    (sourceFrame targetFrame : SemanticFrame)
    (sourceContext targetContext : ResidualContext)
    (sourceFragment targetFragment : Fragment)
    (sourceProcedure targetProcedure : CertificationProcedure)
    (sourceClaim targetClaim : Program → Prop)
    (SourceEvidence TargetEvidence : Type)
    (sourceObservation targetObservation : ObservationContract)
    (sourceCertifies : SourceEvidence → Program → Prop)
    (targetCertifies : TargetEvidence → Program → Prop) where
  reindexProgram : Program → Program
  translateEvidence : SourceEvidence → TargetEvidence
  sourceFamily : sourceFrame.implementationModel = sourceModel.identity.family
  targetFamily : targetFrame.implementationModel = targetModel.identity.family
  sourceModelFragment : sourceFragment = sourceModel.fragment
  targetModelFragment : targetFragment = targetModel.fragment
  model : ModelRefinement sourceModel targetModel
  modelPrograms : ∀ input, reindexProgram (sourceModel.toProgram input) =
    targetModel.toProgram (model.translateInput input)
  semantics : sourceFrame.semantics = targetFrame.semantics
  semanticsVersion : sourceFrame.semanticsVersion ≤ targetFrame.semanticsVersion
  context : ContextRefines sourceContext targetContext
  membership : ∀ program, sourceFragment.admits program →
    targetFragment.admits (reindexProgram program)
  claim : ∀ program, sourceClaim program → targetClaim (reindexProgram program)
  boundary : BoundaryRefines sourceFrame.boundary targetFrame.boundary
  observation : ObservationRefines reindexProgram sourceObservation targetObservation
  certification : ∀ evidence program, sourceCertifies evidence program →
    targetCertifies (translateEvidence evidence) (reindexProgram program)
  sourceRefutationSoundness : RefutationSoundness sourceClaim sourceObservation
  targetRefutationSoundness : RefutationSoundness targetClaim targetObservation
  sourceRefutationCompleteness : RefutationCompleteness sourceClaim sourceObservation
  targetRefutationCompleteness : RefutationCompleteness targetClaim targetObservation
  procedure : ProcedureSimulation sourceProcedure targetProcedure SourceEvidence TargetEvidence
    reindexProgram translateEvidence sourceObservation targetObservation

theorem CrossModelAssuranceTransport.identity
    (modelFamily : ImplementationModelFamily)
    (frame : SemanticFrame)
    (familyBinding : frame.implementationModel = modelFamily.identity.family)
    (context : ResidualContext) (fragment : Fragment)
    (fragmentBinding : fragment = modelFamily.fragment)
    (procedure : CertificationProcedure) (claim : Program → Prop)
    (Evidence : Type) (observation : ObservationContract)
    (certifies : Evidence → Program → Prop)
    (refutationSoundness : RefutationSoundness claim observation)
    (refutationCompleteness : RefutationCompleteness claim observation) :
    Nonempty (CrossModelAssuranceTransport modelFamily modelFamily frame frame context context
      fragment fragment procedure procedure claim claim Evidence Evidence observation observation
      certifies certifies) := by
  let model : ModelRefinement modelFamily modelFamily := {
    translateInput := id
    translateBehavior := id
    membership := fun _ admitted => admitted
    denotation := fun _ _ modeled => modeled
  }
  exact ⟨{
    reindexProgram := id
    translateEvidence := id
    sourceFamily := familyBinding
    targetFamily := familyBinding
    sourceModelFragment := fragmentBinding
    targetModelFragment := fragmentBinding
    model := model
    modelPrograms := fun _ => rfl
    semantics := rfl
    semanticsVersion := Nat.le_refl _
    context := fun held => held
    membership := fun _ admitted => admitted
    claim := fun _ proved => proved
    boundary := fun _ qualified => qualified
    observation := fun _ observed => observed
    certification := fun _ _ certified => certified
    sourceRefutationSoundness := refutationSoundness
    targetRefutationSoundness := refutationSoundness
    sourceRefutationCompleteness := refutationCompleteness
    targetRefutationCompleteness := refutationCompleteness
    procedure := ProcedureSimulation.identity procedure Evidence observation
  }⟩

def CrossModelAssuranceTransport.comp
    {model₁ model₂ model₃ : ImplementationModelFamily}
    {frame₁ frame₂ frame₃ : SemanticFrame}
    {context₁ context₂ context₃ : ResidualContext}
    {fragment₁ fragment₂ fragment₃ : Fragment}
    {procedure₁ procedure₂ procedure₃ : CertificationProcedure}
    {claim₁ claim₂ claim₃ : Program → Prop}
    {Evidence₁ Evidence₂ Evidence₃ : Type}
    {observation₁ observation₂ observation₃ : ObservationContract}
    {certifies₁ : Evidence₁ → Program → Prop}
    {certifies₂ : Evidence₂ → Program → Prop}
    {certifies₃ : Evidence₃ → Program → Prop}
    (firstSecond : CrossModelAssuranceTransport model₁ model₂ frame₁ frame₂ context₁ context₂
      fragment₁ fragment₂ procedure₁ procedure₂ claim₁ claim₂ Evidence₁ Evidence₂
      observation₁ observation₂ certifies₁ certifies₂)
    (secondThird : CrossModelAssuranceTransport model₂ model₃ frame₂ frame₃ context₂ context₃
      fragment₂ fragment₃ procedure₂ procedure₃ claim₂ claim₃ Evidence₂ Evidence₃
      observation₂ observation₃ certifies₂ certifies₃) :
    CrossModelAssuranceTransport model₁ model₃ frame₁ frame₃ context₁ context₃
      fragment₁ fragment₃ procedure₁ procedure₃ claim₁ claim₃ Evidence₁ Evidence₃
      observation₁ observation₃ certifies₁ certifies₃ := {
  reindexProgram := secondThird.reindexProgram ∘ firstSecond.reindexProgram
  translateEvidence := secondThird.translateEvidence ∘ firstSecond.translateEvidence
  sourceFamily := firstSecond.sourceFamily
  targetFamily := secondThird.targetFamily
  sourceModelFragment := firstSecond.sourceModelFragment
  targetModelFragment := secondThird.targetModelFragment
  model := model_refinement_trans firstSecond.model secondThird.model
  modelPrograms := by
    intro input
    simp only [Function.comp_apply]
    rw [firstSecond.modelPrograms, secondThird.modelPrograms]
    rfl
  semantics := firstSecond.semantics.trans secondThird.semantics
  semanticsVersion := Nat.le_trans firstSecond.semanticsVersion secondThird.semanticsVersion
  context := fun held => secondThird.context (firstSecond.context held)
  membership := fun program admitted =>
    secondThird.membership _ (firstSecond.membership program admitted)
  claim := fun program proved => secondThird.claim _ (firstSecond.claim program proved)
  boundary := fun program qualified =>
    secondThird.boundary program (firstSecond.boundary program qualified)
  observation := fun program observed =>
    secondThird.observation _ (firstSecond.observation program observed)
  certification := fun evidence program certified =>
    secondThird.certification _ _ (firstSecond.certification evidence program certified)
  sourceRefutationSoundness := firstSecond.sourceRefutationSoundness
  targetRefutationSoundness := secondThird.targetRefutationSoundness
  sourceRefutationCompleteness := firstSecond.sourceRefutationCompleteness
  targetRefutationCompleteness := secondThird.targetRefutationCompleteness
  procedure := ProcedureSimulation.comp firstSecond.procedure secondThird.procedure
}

/-! Concrete cross-family witness on the currently proved fragment. It carries
the rustc-to-portable behavior translation from `ImplementationModel.lean` and
does not claim correspondence for programs outside `thermiteRustV1`. -/

def rustc195TransportFrame : SemanticFrame :=
  ⟨"thermite-language", 1, rustc195Identity.family, 1, unqualifiedBoundary⟩

def portableRustTransportFrame : SemanticFrame :=
  ⟨"thermite-language", 1, portableRustIdentity.family, 1, unqualifiedBoundary⟩

def fragmentRefutationObservation (fragment : Fragment) : ObservationContract :=
  ⟨"fragment-refutation", fun program => ¬ fragment.admits program⟩

def rustc195_to_portable_assurance_transport :
    CrossModelAssuranceTransport rustc195Family portableRustFamily
      rustc195TransportFrame portableRustTransportFrame noResiduals noResiduals
      thermiteRustV1 thermiteRustV1 logicalProcedure logicalProcedure
      thermiteRustV1.admits thermiteRustV1.admits Unit Unit
      (fragmentRefutationObservation thermiteRustV1)
      (fragmentRefutationObservation thermiteRustV1)
      (fun _ program => thermiteRustV1.admits program)
      (fun _ program => thermiteRustV1.admits program) := {
  reindexProgram := id
  translateEvidence := id
  sourceFamily := rfl
  targetFamily := rfl
  sourceModelFragment := rfl
  targetModelFragment := rfl
  model := rustc195_refines_portable_rust
  modelPrograms := fun _ => rfl
  semantics := rfl
  semanticsVersion := Nat.le_refl _
  context := fun held => held
  membership := fun _ admitted => admitted
  claim := fun _ proved => proved
  boundary := fun _ qualified => qualified
  observation := fun _ observed => observed
  certification := fun _ _ certified => certified
  sourceRefutationSoundness := ⟨fun _ refuted proved => refuted proved⟩
  targetRefutationSoundness := ⟨fun _ refuted proved => refuted proved⟩
  sourceRefutationCompleteness := ⟨fun _ refuted => refuted⟩
  targetRefutationCompleteness := ⟨fun _ refuted => refuted⟩
  procedure := ProcedureSimulation.identity logicalProcedure Unit
    (fragmentRefutationObservation thermiteRustV1)
}

theorem rustc195_to_portable_composes_with_target_identity :
    Nonempty (CrossModelAssuranceTransport rustc195Family portableRustFamily
      rustc195TransportFrame portableRustTransportFrame noResiduals noResiduals
      thermiteRustV1 thermiteRustV1 logicalProcedure logicalProcedure
      thermiteRustV1.admits thermiteRustV1.admits Unit Unit
      (fragmentRefutationObservation thermiteRustV1)
      (fragmentRefutationObservation thermiteRustV1)
      (fun _ program => thermiteRustV1.admits program)
      (fun _ program => thermiteRustV1.admits program)) := by
  rcases CrossModelAssuranceTransport.identity portableRustFamily
    portableRustTransportFrame rfl noResiduals thermiteRustV1 rfl logicalProcedure
    thermiteRustV1.admits Unit (fragmentRefutationObservation thermiteRustV1)
    (fun _ program => thermiteRustV1.admits program)
    ⟨fun _ refuted proved => refuted proved⟩ ⟨fun _ refuted => refuted⟩ with ⟨identity⟩
  exact ⟨CrossModelAssuranceTransport.comp rustc195_to_portable_assurance_transport identity⟩

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

/-- The finite relation vocabulary replayed by Rust and CI.  These are result
labels for an already checked typed relation; they are not assurance
coordinates or substitutes for the witnesses above. -/
inductive TransportRelationKind where
  | transported
  | incompatible
  | missing
deriving DecidableEq, Repr

inductive TransportFiberKind where
  | same
  | different
deriving DecidableEq, Repr

inductive TransportModelKind where
  | same
  | refines
  | reverseRefines
  | compatibilityBreak
  | unproved
deriving DecidableEq, Repr

inductive TransportBoundaryKind where
  | same
  | weaker
  | strengthening
  | unrelated
  | unproved
deriving DecidableEq, Repr

inductive TransportComparisonOutcome where
  | strengthened
  | weakened
  | incomparable
  | changedFiber
  | missingClassification
deriving DecidableEq, Repr

def classifyTransportReplay
    (relation : TransportRelationKind)
    (model : TransportModelKind)
    (fiber : TransportFiberKind)
    (boundary : TransportBoundaryKind) : Option TransportComparisonOutcome :=
  match relation with
  | .missing => some .missingClassification
  | .incompatible => some .incomparable
  | .transported =>
      match model with
      | .reverseRefines | .compatibilityBreak | .unproved => none
      | .same | .refines =>
          match boundary with
          | .strengthening | .unrelated | .unproved => none
          | .same =>
              if model = .refines ∨ fiber = .different then
                some .changedFiber
              else some .strengthened
          | .weaker =>
              if model = .refines ∨ fiber = .different then
                some .changedFiber
              else some .weakened

structure TransportReplayCase where
  id : String
  source : String
  target : String
  relation : TransportRelationKind
  model : TransportModelKind
  fiber : TransportFiberKind
  boundary : TransportBoundaryKind
  expected : Option TransportComparisonOutcome
deriving DecidableEq, Repr

def TransportReplayCase.accepts (row : TransportReplayCase) : Bool :=
  decide (classifyTransportReplay row.relation row.model row.fiber row.boundary = row.expected)

theorem assurance_transport_identity_left
    {frame targetFrame : SemanticFrame}
    {context targetContext : ResidualContext}
    {fragment targetFragment : Fragment}
    {procedure targetProcedure : CertificationProcedure}
    {claim targetClaim : Program → Prop}
    {Evidence TargetEvidence : Type}
    {observation targetObservation : ObservationContract}
    {certifies : Evidence → Program → Prop}
    {targetCertifies : TargetEvidence → Program → Prop}
    (transport : AssuranceTransport frame targetFrame context targetContext fragment targetFragment
      procedure targetProcedure claim targetClaim Evidence TargetEvidence observation targetObservation
      certifies targetCertifies) :
    AssuranceTransport.comp
      (AssuranceTransport.identity frame context fragment procedure claim Evidence observation certifies
        transport.semantic.sourceRefutationSoundness
        transport.semantic.sourceRefutationCompleteness)
      transport = transport := by
  cases transport
  simp [AssuranceTransport.comp, AssuranceTransport.identity,
    Function.comp_def]

theorem assurance_transport_identity_right
    {frame targetFrame : SemanticFrame}
    {context targetContext : ResidualContext}
    {fragment targetFragment : Fragment}
    {procedure targetProcedure : CertificationProcedure}
    {claim targetClaim : Program → Prop}
    {Evidence TargetEvidence : Type}
    {observation targetObservation : ObservationContract}
    {certifies : Evidence → Program → Prop}
    {targetCertifies : TargetEvidence → Program → Prop}
    (transport : AssuranceTransport frame targetFrame context targetContext fragment targetFragment
      procedure targetProcedure claim targetClaim Evidence TargetEvidence observation targetObservation
      certifies targetCertifies) :
    AssuranceTransport.comp transport
      (AssuranceTransport.identity targetFrame targetContext targetFragment targetProcedure
        targetClaim TargetEvidence targetObservation targetCertifies
        transport.semantic.targetRefutationSoundness
        transport.semantic.targetRefutationCompleteness) = transport := by
  cases transport
  simp [AssuranceTransport.comp, AssuranceTransport.identity,
    Function.comp_def]

end Thermite.CertificationMetatheory
