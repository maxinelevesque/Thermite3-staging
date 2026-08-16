import Thermite.LanguageCompleteness
import Thermite.CheckedTraversal

/-!
The neutral proof-theoretic layer beneath RFC-3 certification coordinates.

The Rust coordinate tuple is an executable projection.  This module owns the
semantic object it projects: a certification judgment indexed by its frame,
residual assumptions, language fragment, procedure, claim, evidence type, and
observable refutation contract.  Refinement translates every one of the
semantic carriers explicitly; no finite assurance label appears here.
-/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

/-- Boundary qualification is semantic coverage, not a display string. -/
structure BoundaryContext where
  name : String
  qualifies : Program → Prop

/-- Versioned semantics, implementation-model selection, and boundary context
for a judgment. -/
structure SemanticFrame where
  semantics : String
  semanticsVersion : Nat
  implementationModel : String
  implementationModelVersion : Nat
  boundary : BoundaryContext

/-- Residual assumptions are represented by their joint semantic obligation,
not by a count or an assurance label. -/
structure ResidualContext where
  name : String
  holds : Prop

/-- A versioned certification procedure and its explicit operational frame. -/
structure CertificationProcedure where
  name : String
  version : Nat
  environment : String
  toolVersion : String
  resourceBudget : Nat
deriving DecidableEq, Repr

/-- What observable behavior is promised when the certified claim is false. -/
structure ObservationContract where
  name : String
  observes : Program → Prop

/-- The full judgment underlying an RFC-3 coordinate projection.  All semantic
dimensions are indices, so changing a fragment, frame, procedure, claim,
evidence language, or observation contract changes the judgment's type. -/
structure CertificationJudgment
    (frame : SemanticFrame)
    (context : ResidualContext)
    (fragment : Fragment)
    (procedure : CertificationProcedure)
    (claim : Program → Prop)
    (Evidence : Type)
    (observation : ObservationContract) where
  certifies : Evidence → Program → Prop

/-- A judgment is semantically meaningful when accepted evidence only certifies
members of its named fragment that satisfy its named claim, under the residual
context carried by the judgment. -/
def SemanticMeaning {frame context fragment procedure claim Evidence observation}
    (judgment : CertificationJudgment frame context fragment procedure claim Evidence observation) :
    Prop :=
  context.holds → ∀ evidence program, judgment.certifies evidence program →
    fragment.admits program ∧ claim program

/-! Separately typed theorem families.  Keeping these as distinct structures is
intentional: a proof of classifier completeness is not producer totality, and
neither is proof or refutation soundness. -/

structure ClassifierSoundness (fragment : Fragment)
    (classify : Program → Bool) : Prop where
  sound : ∀ program, classify program = true → fragment.admits program

structure ClassifierCompleteness (fragment : Fragment)
    (classify : Program → Bool) : Prop where
  complete : ∀ program, fragment.admits program → classify program = true

structure ProducerTotality (fragment : Fragment) (Evidence : Type)
    (produce : Program → Evidence) : Prop where
  total : ∀ program, fragment.admits program → ∃ evidence, produce program = evidence

structure ProducerRefinement (fragment : Fragment)
    (LogicalEvidence ExecutableEvidence : Type)
    (logical : Program → LogicalEvidence)
    (executable : Program → ExecutableEvidence)
    (corresponds : LogicalEvidence → ExecutableEvidence → Prop) : Prop where
  refines : ∀ program, fragment.admits program →
    corresponds (logical program) (executable program)

structure ProofSoundness
    {frame context fragment procedure claim Evidence observation}
    (judgment : CertificationJudgment frame context fragment procedure claim Evidence observation) :
    Prop where
  meaningful : SemanticMeaning judgment

structure RefutationSoundness (claim : Program → Prop)
    (observation : ObservationContract) : Prop where
  sound : ∀ program, observation.observes program → ¬ claim program

structure RefutationCompleteness (claim : Program → Prop)
    (observation : ObservationContract) : Prop where
  complete : ∀ program, ¬ claim program → observation.observes program

structure StageCompleteness (fragment : Fragment) (valid : SemanticValidity)
    (stage : Stage) (run : Program → StageResult stage Unit) : Prop where
  complete : CompleteAt fragment valid stage run

/-- Stronger residual assumptions entail every assumption required by the
weaker judgment.  The direction is explicit because deleting an assumption is
not refinement. -/
def ContextRefines (strong weak : ResidualContext) : Prop :=
  strong.holds → weak.holds

/-- Stronger boundary coverage entails the weaker qualification. -/
def BoundaryRefines (strong weak : BoundaryContext) : Prop :=
  ∀ program, strong.qualifies program → weak.qualifies program

/-- Semantic/model versions remain exact while boundary qualification may
weaken explicitly. -/
structure FrameRefines (strong weak : SemanticFrame) : Prop where
  semantics : strong.semantics = weak.semantics
  semanticsVersion : strong.semanticsVersion = weak.semanticsVersion
  implementationModel : strong.implementationModel = weak.implementationModel
  implementationModelVersion :
    strong.implementationModelVersion = weak.implementationModelVersion
  boundary : BoundaryRefines strong.boundary weak.boundary

def ProcedureRefines (strong weak : CertificationProcedure) : Prop := strong = weak

theorem frame_refines_refl (frame : SemanticFrame) : FrameRefines frame frame := by
  exact ⟨rfl, rfl, rfl, rfl, fun _ qualified => qualified⟩

theorem frame_refines_trans {first second third : SemanticFrame}
    (firstSecond : FrameRefines first second)
    (secondThird : FrameRefines second third) : FrameRefines first third := by
  exact ⟨firstSecond.semantics.trans secondThird.semantics,
    firstSecond.semanticsVersion.trans secondThird.semanticsVersion,
    firstSecond.implementationModel.trans secondThird.implementationModel,
    firstSecond.implementationModelVersion.trans secondThird.implementationModelVersion,
    fun program qualified => secondThird.boundary program
      (firstSecond.boundary program qualified)⟩

/-- A stronger observable-failure contract preserves the weaker contract after
program reindexing. -/
def ObservationRefines (reindex : Program → Program)
    (strong weak : ObservationContract) : Prop :=
  ∀ program, strong.observes program → weak.observes (reindex program)

/-- Semantic refinement between full certification judgments.  Membership,
claims, evidence, residual assumptions, observable failure, and accepted
certifications are transported independently rather than collapsed into an
assurance score. -/
structure Refinement
    {strongFrame weakFrame : SemanticFrame}
    {strongContext weakContext : ResidualContext}
    {strongFragment weakFragment : Fragment}
    {strongProcedure weakProcedure : CertificationProcedure}
    {strongClaim weakClaim : Program → Prop}
    {StrongEvidence WeakEvidence : Type}
    {strongObservation weakObservation : ObservationContract}
    (strong : CertificationJudgment strongFrame strongContext strongFragment
      strongProcedure strongClaim StrongEvidence strongObservation)
    (weak : CertificationJudgment weakFrame weakContext weakFragment
      weakProcedure weakClaim WeakEvidence weakObservation) where
  reindexProgram : Program → Program
  translateEvidence : StrongEvidence → WeakEvidence
  frame : FrameRefines strongFrame weakFrame
  procedure : ProcedureRefines strongProcedure weakProcedure
  context : ContextRefines strongContext weakContext
  membership : ∀ program, strongFragment.admits program →
    weakFragment.admits (reindexProgram program)
  claim : ∀ program, strongClaim program → weakClaim (reindexProgram program)
  observation : ObservationRefines reindexProgram strongObservation weakObservation
  certification : ∀ evidence program, strong.certifies evidence program →
    weak.certifies (translateEvidence evidence) (reindexProgram program)

/-- Existence of an explicit refinement witness.  The witness carries data, so
the public relation packages it in `Nonempty` to remain a proposition. -/
def Refines
    {strongFrame weakFrame : SemanticFrame}
    {strongContext weakContext : ResidualContext}
    {strongFragment weakFragment : Fragment}
    {strongProcedure weakProcedure : CertificationProcedure}
    {strongClaim weakClaim : Program → Prop}
    {StrongEvidence WeakEvidence : Type}
    {strongObservation weakObservation : ObservationContract}
    (strong : CertificationJudgment strongFrame strongContext strongFragment
      strongProcedure strongClaim StrongEvidence strongObservation)
    (weak : CertificationJudgment weakFrame weakContext weakFragment
      weakProcedure weakClaim WeakEvidence weakObservation) : Prop :=
  Nonempty (Refinement strong weak)

theorem refines_refl {frame context fragment procedure claim Evidence observation}
    (judgment : CertificationJudgment frame context fragment procedure claim Evidence observation) :
    Refines judgment judgment := by
  exact ⟨{
    reindexProgram := id
    translateEvidence := id
    frame := frame_refines_refl _
    procedure := rfl
    context := fun held => held
    membership := fun _ admitted => admitted
    claim := fun _ proved => proved
    observation := fun _ observed => observed
    certification := fun _ _ certified => certified
  }⟩

theorem refines_trans
    {frame₁ frame₂ frame₃ : SemanticFrame}
    {context₁ context₂ context₃ : ResidualContext}
    {fragment₁ fragment₂ fragment₃ : Fragment}
    {procedure₁ procedure₂ procedure₃ : CertificationProcedure}
    {claim₁ claim₂ claim₃ : Program → Prop}
    {Evidence₁ Evidence₂ Evidence₃ : Type}
    {observation₁ observation₂ observation₃ : ObservationContract}
    {first : CertificationJudgment frame₁ context₁ fragment₁ procedure₁ claim₁
      Evidence₁ observation₁}
    {second : CertificationJudgment frame₂ context₂ fragment₂ procedure₂ claim₂
      Evidence₂ observation₂}
    {third : CertificationJudgment frame₃ context₃ fragment₃ procedure₃ claim₃
      Evidence₃ observation₃}
    (firstSecond : Refines first second) (secondThird : Refines second third) :
    Refines first third := by
  rcases firstSecond with ⟨firstSecond⟩
  rcases secondThird with ⟨secondThird⟩
  exact ⟨{
    reindexProgram := secondThird.reindexProgram ∘ firstSecond.reindexProgram
    translateEvidence := secondThird.translateEvidence ∘ firstSecond.translateEvidence
    frame := frame_refines_trans firstSecond.frame secondThird.frame
    procedure := firstSecond.procedure.trans secondThird.procedure
    context := fun held => secondThird.context (firstSecond.context held)
    membership := fun program admitted =>
      secondThird.membership _ (firstSecond.membership program admitted)
    claim := fun program proved => secondThird.claim _ (firstSecond.claim program proved)
    observation := fun program observed =>
      secondThird.observation _ (firstSecond.observation program observed)
    certification := fun evidence program certified =>
      secondThird.certification _ _ (firstSecond.certification evidence program certified)
  }⟩

/-! Checked bridges from the already-shipped language-completeness layer. -/

def unqualifiedBoundary : BoundaryContext := ⟨"unqualified", fun _ => True⟩
def neutralFrame : SemanticFrame :=
  ⟨"thermite-language", 1, "neutral", 1, unqualifiedBoundary⟩
def noResiduals : ResidualContext := ⟨"none", True⟩
def logicalProcedure : CertificationProcedure := ⟨"logical", 1, "kernel", "lean", 0⟩
def noObservation : ObservationContract := ⟨"none", fun _ => True⟩

def membershipJudgment (fragment : Fragment) :
    CertificationJudgment neutralFrame noResiduals fragment logicalProcedure
      fragment.admits Unit noObservation :=
  ⟨fun _ program => fragment.admits program⟩

/-- An ordinary fragment expansion is a semantic refinement of its membership
judgment. -/
theorem expansion_refines_membership {old new : Fragment} (expansion : Expands old new) :
    Refines (membershipJudgment old) (membershipJudgment new) := by
  exact ⟨{
    reindexProgram := id
    translateEvidence := id
    frame := frame_refines_refl _
    procedure := rfl
    context := fun held => held
    membership := expansion.2.2
    claim := expansion.2.2
    observation := fun _ _ => trivial
    certification := fun _ program certified => expansion.2.2 program certified
  }⟩

def guaranteeJudgment {stage : Stage} (guarantee : StageGuarantee stage)
    (fragment : Fragment) :
    CertificationJudgment neutralFrame noResiduals fragment logicalProcedure
      guarantee.holds Unit noObservation :=
  ⟨fun _ program => fragment.admits program ∧ guarantee.holds program⟩

/-- An explicit stage-composition premise induces semantic refinement between
the corresponding stage guarantee judgments. -/
theorem composition_refines_guarantee {sourceStage targetStage : Stage}
    {source : StageGuarantee sourceStage} {target : StageGuarantee targetStage}
    {fragment : Fragment}
    (premise : CompositionPremise sourceStage targetStage source target) :
    Refines (guaranteeJudgment source fragment) (guaranteeJudgment target fragment) := by
  exact ⟨{
    reindexProgram := id
    translateEvidence := id
    frame := frame_refines_refl _
    procedure := rfl
    context := fun held => held
    membership := fun _ admitted => admitted
    claim := premise.transport
    observation := fun _ _ => trivial
    certification := fun _ program certified =>
      ⟨certified.1, premise.transport program certified.2⟩
  }⟩

open Thermite.CheckedTraversal

def rfc10LogicalProducerJudgment :
    CertificationJudgment neutralFrame noResiduals rfc10FragmentV1 logicalProcedure
      rfc10FragmentV1.admits CanonicalAst noObservation :=
  ⟨fun ast program => toLanguageProgram ast = program ∧ SupportedCanonicalAst ast⟩

def rfc10WitnessJudgment :
    CertificationJudgment neutralFrame noResiduals rfc10FragmentV1 logicalProcedure
      rfc10FragmentV1.admits Witness noObservation :=
  ⟨fun witness program => ∃ ast, toLanguageProgram ast = program ∧ SupportedRFC10 ast witness⟩

/-- The shipped RFC-10 logical-producer theorem is an evidence translation in
the general refinement relation. -/
theorem rfc10_producer_refines_certification :
    Refines rfc10LogicalProducerJudgment rfc10WitnessJudgment := by
  exact ⟨{
    reindexProgram := id
    translateEvidence := produce
    frame := frame_refines_refl _
    procedure := rfl
    context := fun held => held
    membership := fun _ admitted => admitted
    claim := fun _ admitted => admitted
    observation := fun _ _ => trivial
    certification := fun ast program certified =>
      ⟨ast, certified.1, produce_supported certified.2⟩
  }⟩

/-! AC-3 probes: unequal bounds and one-way boundary weakening. -/

def BoundedScope (bound : Nat) (program : Program) : Prop :=
  program.constructs.length ≤ bound

def BoundRefines (strong weak : Nat) : Prop := strong ≤ weak

theorem bounded_scope_monotone {strong weak : Nat} (ordered : BoundRefines strong weak) :
    ∀ program, BoundedScope strong program → BoundedScope weak program := by
  intro program inside
  exact Nat.le_trans inside ordered

theorem bounded_two_refines_five : BoundRefines 2 5 := by simp [BoundRefines]

def endToEndBoundary : BoundaryContext :=
  ⟨"end-to-end", fun program => program.facts.contains "end-to-end"⟩

def platformBoundary : BoundaryContext :=
  ⟨"to-platform", fun _ => True⟩

def endToEndFrame : SemanticFrame :=
  ⟨"thermite-language", 1, "neutral", 1, endToEndBoundary⟩

def platformFrame : SemanticFrame :=
  ⟨"thermite-language", 1, "neutral", 1, platformBoundary⟩

theorem end_to_end_refines_platform : FrameRefines endToEndFrame platformFrame := by
  exact ⟨rfl, rfl, rfl, rfl, fun _ _ => trivial⟩

end Thermite.CertificationMetatheory
