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

/-- The checker is indexed by the exact artifact. Soundness consumes accepted,
decoded evidence and establishes the decoded modeled behavior. -/
structure ArtifactChecker (family : ImplementationModelFamily) (input : family.Input) where
  Evidence : Type
  check : Evidence → Bool
  decode : Evidence → Option (ModelObservation family.Behavior)
  evidenceId : Evidence → String
  sound : ∀ evidence observation, check evidence = true →
    decode evidence = some observation → BehaviorCorresponds family input observation

structure CheckedArtifactRefinement (family : ImplementationModelFamily)
    (input : family.Input) (checker : ArtifactChecker family input) where
  evidence : checker.Evidence
  observation : ModelObservation family.Behavior
  accepted : checker.check evidence = true
  decoded : checker.decode evidence = some observation

def CheckedArtifactRefinement.corresponds
    {family : ImplementationModelFamily} {input : family.Input}
    {checker : ArtifactChecker family input}
    (refinement : CheckedArtifactRefinement family input checker) :
    BehaviorCorresponds family input refinement.observation :=
  checker.sound refinement.evidence refinement.observation
    refinement.accepted refinement.decoded

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

structure RustcArtifactEvidence where
  model : ModelIdentity
  digest : String
  target : String
  behavior : RustcBehavior
deriving DecidableEq, Repr

def expectedRustcEvidence : RustcArtifactEvidence :=
  ⟨rustc195Identity, "discharge-rust", "x86_64-unknown-linux-gnu",
    rustc195Behavior dischargeRustWitness⟩

def rustcEvidenceValid (evidence : RustcArtifactEvidence) : Bool :=
  decide (evidence = expectedRustcEvidence)

def rustcArtifactChecker : ArtifactChecker rustc195Family dischargeRustWitness where
  Evidence := RustcArtifactEvidence
  check := rustcEvidenceValid
  decode := fun evidence => if rustcEvidenceValid evidence then
    some ⟨evidence.model, evidence.behavior⟩ else none
  evidenceId := fun evidence => evidence.model.version ++ ":" ++ evidence.digest ++
    ":" ++ evidence.target
  sound := by
    intro evidence observation accepted decoded
    have exactEvidence : evidence = expectedRustcEvidence := by
      simpa [rustcEvidenceValid] using accepted
    subst evidence
    simp [rustcEvidenceValid] at decoded
    cases decoded
    exact ⟨rfl, by
      simp [expectedRustcEvidence, dischargeRustWitness, thermiteRustV1Admits,
        rustc195Family, rustc195Denotation, rustc195Behavior]⟩

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
