import Thermite.ImplementationModel

/-! Structurally bound effective-TCB discharge for RFC-3. -/
namespace Thermite.CertificationMetatheory
open Thermite.LanguageCompleteness

def BehaviorCorresponds (family : ImplementationModelFamily)
    (input : family.Input) (observed : ModelObservation family.Behavior) : Prop :=
  observed.model = family.identity ∧ family.denotes input observed.behavior

def ArtifactModeled (family : ImplementationModelFamily) (input : family.Input) : Prop :=
  ∃ observed, BehaviorCorresponds family input observed

/-- A distinct executable carrier; this is not `family.observe`. -/
structure ExecutableImplementation (family : ImplementationModelFamily) where
  identity : ModelIdentity
  run : family.Input → ModelObservation family.Behavior

structure UniversalExecutableRefinement (family : ImplementationModelFamily)
    (implementation : ExecutableImplementation family) where
  identityMatches : implementation.identity = family.identity
  covers : ∀ input, family.fragment.admits (family.toProgram input) →
    (implementation.run input).model = implementation.identity ∧
    family.denotes input (implementation.run input).behavior

/-- Replay evidence is intrinsically bound to the model and exact input.  Its
payload cannot be empty, and the observation names the same model identity.
Thus a content-free carrier such as `Unit` cannot instantiate checked
refinement. -/
structure ReplayEvidence (family : ImplementationModelFamily) (input : family.Input) where
  model : ModelIdentity
  inputId : String
  observation : ModelObservation family.Behavior
  payload : String
  payloadNonempty : payload ≠ ""
  modelBound : model = family.identity
  inputBound : inputId = family.inputIdentity input
  observationModel : observation.model = model
  replayed : family.decodeReplay input payload = some observation

/-- The checker is indexed by the exact artifact and consumes only a replay
envelope carrying that artifact's identity and decoded observation. -/
structure ArtifactChecker (family : ImplementationModelFamily) (input : family.Input) where
  check : ReplayEvidence family input → Bool
  decode : ReplayEvidence family input → Option (ModelObservation family.Behavior)
  evidenceId : ReplayEvidence family input → String
  decodeBinds : ∀ evidence observation, decode evidence = some observation →
    observation = evidence.observation
  sound : ∀ evidence, check evidence = true →
    family.denotes input evidence.observation.behavior

structure CheckedArtifactRefinement (family : ImplementationModelFamily)
    (input : family.Input) (checker : ArtifactChecker family input) where
  evidence : ReplayEvidence family input
  observation : ModelObservation family.Behavior
  accepted : checker.check evidence = true
  decoded : checker.decode evidence = some observation

def CheckedArtifactRefinement.corresponds
    {family : ImplementationModelFamily} {input : family.Input}
    {checker : ArtifactChecker family input}
    (refinement : CheckedArtifactRefinement family input checker) :
    BehaviorCorresponds family input refinement.observation := by
  have bound := checker.decodeBinds refinement.evidence refinement.observation
    refinement.decoded
  rw [bound]
  exact ⟨refinement.evidence.observationModel.trans refinement.evidence.modelBound,
    checker.sound refinement.evidence refinement.accepted⟩

structure UniversalDischarge (family : ImplementationModelFamily) (input : family.Input) where
  implementation : ExecutableImplementation family
  refinement : UniversalExecutableRefinement family implementation
  admitted : family.fragment.admits (family.toProgram input)

structure CheckedDischarge (family : ImplementationModelFamily) (input : family.Input) where
  checker : ArtifactChecker family input
  refinement : CheckedArtifactRefinement family input checker

inductive DischargeEvidence (family : ImplementationModelFamily) (input : family.Input) where
  | universal (proof : UniversalDischarge family input)
  | checkedArtifact (proof : CheckedDischarge family input)

structure CoverageClaims where
  producerComplete : Bool
  checkerComplete : Bool
  fragmentComplete : Bool
  workflowComplete : Bool
deriving DecidableEq, Repr

def artifactOldContext (family : ImplementationModelFamily)
    (input : family.Input) : ResidualContext :=
  ⟨"modeled artifact plus platform", ArtifactModeled family input ∧ True⟩

def universalNewContext {family : ImplementationModelFamily} {input : family.Input}
    (proof : UniversalDischarge family input) : ResidualContext :=
  ⟨"universal executable refinement plus platform",
    Nonempty (UniversalExecutableRefinement family proof.implementation) ∧
      family.fragment.admits (family.toProgram input) ∧ True⟩

def checkedNewContext {family : ImplementationModelFamily} {input : family.Input}
    (proof : CheckedDischarge family input) : ResidualContext :=
  ⟨"checked artifact evidence plus checker and platform",
    Nonempty (CheckedArtifactRefinement family input proof.checker) ∧ True⟩

/-- A reduction contains no free report fields. All public metadata and both
contexts are derived from this dependent witness. -/
structure TcbReduction (family : ImplementationModelFamily) (input : family.Input) where
  discharge : DischargeEvidence family input

def TcbReduction.dischargedAssumption {family input}
    (_ : TcbReduction family input) : ModelIdentity := family.identity

def TcbReduction.replacementEvidence {family input}
    (reduction : TcbReduction family input) : String :=
  match reduction.discharge with
  | .universal proof => "universal:" ++ proof.implementation.identity.family ++
      "/" ++ proof.implementation.identity.version
  | .checkedArtifact proof =>
      "checked:" ++ proof.checker.evidenceId proof.refinement.evidence

def TcbReduction.remainingPremises {family input}
    (reduction : TcbReduction family input) : List String :=
  match reduction.discharge with
  | .universal _ => ["executable-identity", "universal-refinement", "platform"]
  | .checkedArtifact _ => ["checker-soundness", "decoded-evidence", "platform"]

def TcbReduction.oldContext {family input} (_ : TcbReduction family input) : ResidualContext :=
  artifactOldContext family input

def TcbReduction.newContext {family input}
    (reduction : TcbReduction family input) : ResidualContext :=
  match reduction.discharge with
  | .universal proof => universalNewContext proof
  | .checkedArtifact proof => checkedNewContext proof

theorem tcbReduction_context_refines {family input}
    (reduction : TcbReduction family input) :
    ContextRefines reduction.newContext reduction.oldContext := by
  rcases reduction with ⟨discharge⟩
  cases discharge with
  | universal proof =>
      rintro ⟨⟨refinement⟩, admitted, platform⟩
      have covered := refinement.covers input admitted
      exact ⟨⟨proof.implementation.run input,
        ⟨covered.1.trans refinement.identityMatches, covered.2⟩⟩, platform⟩
  | checkedArtifact proof =>
      rintro ⟨⟨refinement⟩, platform⟩
      exact ⟨⟨refinement.observation, refinement.corresponds⟩, platform⟩

structure ModelOnlyTrust (family : ImplementationModelFamily) where
  context : ResidualContext
  residualComponent : ModelIdentity
  retained : residualComponent = family.identity

def dischargeRustWitness : RustcInput :=
  ⟨⟨"discharge-rust", [],
    ["thermite-rust-v1", "target:x86_64-unknown-linux-gnu"]⟩⟩

theorem discharge_rust_witness_admitted :
    rustc195Family.fragment.admits (rustc195Family.toProgram dischargeRustWitness) := by
  simp [rustc195Family, thermiteRustV1, dischargeRustWitness]

def fixtureRustcExecutable : ExecutableImplementation rustc195Family :=
  ⟨rustc195Identity, fun input => ⟨rustc195Identity, rustc195Behavior input⟩⟩

def fixtureRustcUniversalRefinement :
    UniversalExecutableRefinement rustc195Family fixtureRustcExecutable :=
  ⟨rfl, fun input _ => ⟨rfl, by
    by_cases admitted : thermiteRustV1Admits input = true <;>
      simp [fixtureRustcExecutable, rustc195Family, rustc195Denotation,
        rustc195Behavior, admitted]⟩⟩

def rustcUniversalReduction : TcbReduction rustc195Family dischargeRustWitness :=
  ⟨.universal ⟨fixtureRustcExecutable, fixtureRustcUniversalRefinement,
    discharge_rust_witness_admitted⟩⟩

def expectedRustcEvidence : ReplayEvidence rustc195Family dischargeRustWitness :=
  { model := rustc195Identity
    inputId := "discharge-rust"
    observation := ⟨rustc195Identity, rustc195Behavior dischargeRustWitness⟩
    payload := "rustc-1.95.0:discharge-rust:x86_64-unknown-linux-gnu"
    payloadNonempty := by decide
    modelBound := rfl
    inputBound := rfl
    observationModel := rfl
    replayed := by
      simp [rustc195Family, rustc195DecodeReplay, rustc195ReplayPayload,
        dischargeRustWitness, rustc195Behavior, thermiteRustV1Admits] }

def replayRustcBehavior
    (evidence : ReplayEvidence rustc195Family dischargeRustWitness) : RustcBehavior :=
  evidence.observation.behavior

def rustcEvidenceValid
    (evidence : ReplayEvidence rustc195Family dischargeRustWitness) : Bool :=
  decide (evidence.model = expectedRustcEvidence.model ∧
    evidence.inputId = expectedRustcEvidence.inputId ∧
    evidence.observation.model = expectedRustcEvidence.observation.model ∧
    replayRustcBehavior evidence = replayRustcBehavior expectedRustcEvidence ∧
    evidence.payload = expectedRustcEvidence.payload)

def rustcArtifactChecker : ArtifactChecker rustc195Family dischargeRustWitness where
  check := rustcEvidenceValid
  decode := fun evidence => if rustcEvidenceValid evidence then
    some evidence.observation else none
  evidenceId := fun evidence => evidence.model.version ++ ":" ++ evidence.inputId ++
    ":" ++ evidence.payload
  decodeBinds := by
    intro evidence observation decoded
    by_cases accepted : rustcEvidenceValid evidence = true <;>
      simp [accepted] at decoded
    exact decoded.symm
  sound := by
    intro evidence accepted
    simp only [rustcEvidenceValid] at accepted
    have facts := of_decide_eq_true accepted
    change rustc195Family.denotes dischargeRustWitness (replayRustcBehavior evidence)
    rw [facts.2.2.2.1]
    simp [replayRustcBehavior, expectedRustcEvidence, dischargeRustWitness, thermiteRustV1Admits,
      rustc195Family, rustc195Denotation, rustc195Behavior]

def rustcCheckedRefinement :
    CheckedArtifactRefinement rustc195Family dischargeRustWitness rustcArtifactChecker :=
  ⟨expectedRustcEvidence, ⟨rustc195Identity, rustc195Behavior dischargeRustWitness⟩,
    by decide, by simp [rustcArtifactChecker, rustcEvidenceValid,
      expectedRustcEvidence]⟩

def rustcCheckedReduction : TcbReduction rustc195Family dischargeRustWitness :=
  ⟨.checkedArtifact ⟨rustcArtifactChecker, rustcCheckedRefinement⟩⟩

def rustcModelOnly : ModelOnlyTrust rustc195Family :=
  ⟨artifactOldContext rustc195Family dischargeRustWitness, rustc195Identity, rfl⟩

theorem rustc195_artifact_corresponds : ArtifactModeled rustc195Family dischargeRustWitness :=
  ⟨fixtureRustcExecutable.run dischargeRustWitness, ⟨rfl, by
    simp [fixtureRustcExecutable, dischargeRustWitness, thermiteRustV1Admits,
      rustc195Family, rustc195Denotation, rustc195Behavior]⟩⟩

theorem model_only_retains_rustc :
    rustcModelOnly.residualComponent = rustc195Family.identity := rustcModelOnly.retained

theorem universal_reduction_discharges_exact_rustc :
    rustcUniversalReduction.dischargedAssumption = rustc195Identity := rfl
theorem universal_reduction_context_refines :
    ContextRefines rustcUniversalReduction.newContext rustcUniversalReduction.oldContext :=
  tcbReduction_context_refines _
theorem checked_reduction_discharges_exact_rustc :
    rustcCheckedReduction.dischargedAssumption = rustc195Identity := rfl
theorem checked_reduction_context_refines :
    ContextRefines rustcCheckedReduction.newContext rustcCheckedReduction.oldContext :=
  tcbReduction_context_refines _

def incompleteCoverage : CoverageClaims := ⟨false, false, true, false⟩
theorem accepted_artifact_sound_despite_incomplete_coverage :
    ArtifactModeled rustc195Family dischargeRustWitness := by
  have _coverage := incompleteCoverage
  exact ⟨rustcCheckedRefinement.observation, rustcCheckedRefinement.corresponds⟩

end Thermite.CertificationMetatheory
