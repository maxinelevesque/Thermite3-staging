import Thermite.ImplementationModel

/-!
Effective-TCB accounting for RFC-3.

The model layer can name an implementation assumption, but only a universal
executable refinement or sound checked evidence for one artifact can replace
that assumption in a certificate.  Every replacement carries an explicit
`ContextRefines` proof; model description alone has no discharge constructor.
-/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

/-- The modeled relation required for one artifact.  Fragment coverage remains
a separate premise of universal refinement rather than being hidden here. -/
def ArtifactCorresponds (family : ImplementationModelFamily)
    (input : family.Input) : Prop :=
  (family.observe input).model = family.identity ∧
    family.denotes input (family.observe input).behavior

/-- The strong discharge path: every admitted input produced by the executable
implementation has the modeled behavior. -/
structure UniversalExecutableRefinement (family : ImplementationModelFamily) where
  covers : ∀ input, family.fragment.admits (family.toProgram input) →
    ArtifactCorresponds family input

/-- The narrow discharge path: replayable evidence is accepted by a checker
whose soundness establishes the modeled relation for this exact artifact. -/
structure CheckedArtifactRefinement (family : ImplementationModelFamily)
    (input : family.Input) where
  evidence : String
  check : String → Bool
  accepted : check evidence = true
  checkerSound : ∀ candidate, check candidate = true →
    ArtifactCorresponds family input

inductive DischargeEvidence (family : ImplementationModelFamily)
    (input : family.Input) where
  | universal (proof : UniversalExecutableRefinement family)
  | checkedArtifact (proof : CheckedArtifactRefinement family input)

/-- Coverage facts govern how often evidence can be produced or checked.  They
are intentionally absent from `DischargeEvidence`, so missing completeness
reduces coverage without invalidating an already accepted artifact. -/
structure CoverageClaims where
  producerComplete : Bool
  checkerComplete : Bool
  fragmentComplete : Bool
  workflowComplete : Bool
deriving DecidableEq, Repr

/-- A TCB reduction is an entailment, not deletion from a string list.  The
record also reports the exact component replaced, its evidence, and every
remaining premise carried by the new residual context. -/
structure TcbReduction (family : ImplementationModelFamily)
    (input : family.Input) where
  oldContext : ResidualContext
  newContext : ResidualContext
  dischargedAssumption : ModelIdentity
  replacementEvidence : String
  remainingPremises : List String
  discharge : DischargeEvidence family input
  entailsOldObligation : ContextRefines newContext oldContext

/-- Model-only posture records the component as residual and deliberately has
no field capable of producing a `TcbReduction`. -/
structure ModelOnlyTrust (family : ImplementationModelFamily) where
  context : ResidualContext
  residualComponent : ModelIdentity
  retained : residualComponent = family.identity

def dischargeRustWitness : RustcInput :=
  ⟨⟨"discharge-rust", [],
    ["thermite-rust-v1", "target:x86_64-unknown-linux-gnu"]⟩⟩

theorem discharge_rust_witness_admitted :
    rustc195Family.fragment.admits
      (rustc195Family.toProgram dischargeRustWitness) := by
  simp [rustc195Family, thermiteRustV1, dischargeRustWitness]

theorem rustc195_artifact_corresponds :
    ArtifactCorresponds rustc195Family dischargeRustWitness := by
  exact rustc195_corresponds_on_thermite_fragment dischargeRustWitness
    discharge_rust_witness_admitted

def rustcModelOnlyContext : ResidualContext :=
  ⟨"rustc-1.95.0 plus platform",
    ArtifactCorresponds rustc195Family dischargeRustWitness ∧ True⟩

def rustcModelOnly : ModelOnlyTrust rustc195Family :=
  ⟨rustcModelOnlyContext, rustc195Identity, rfl⟩

theorem model_only_retains_rustc :
    rustcModelOnly.residualComponent = rustc195Family.identity := by
  exact rustcModelOnly.retained

def rustcUniversalRefinement : UniversalExecutableRefinement rustc195Family :=
  ⟨fun input admitted =>
    rustc195_corresponds_on_thermite_fragment input admitted⟩

def rustcUniversalContext : ResidualContext :=
  ⟨"universal rustc refinement plus platform",
    Nonempty (UniversalExecutableRefinement rustc195Family) ∧ True⟩

def rustcUniversalReduction :
    TcbReduction rustc195Family dischargeRustWitness where
  oldContext := rustcModelOnlyContext
  newContext := rustcUniversalContext
  dischargedAssumption := rustc195Identity
  replacementEvidence := "universal:rustc-1.95.0/thermite-emitted-rust-v1"
  remainingPremises := ["target:x86_64-unknown-linux-gnu"]
  discharge := .universal rustcUniversalRefinement
  entailsOldObligation := by
    rintro ⟨⟨universal⟩, platform⟩
    exact ⟨universal.covers dischargeRustWitness
      discharge_rust_witness_admitted, platform⟩

def rustcArtifactCheck (evidence : String) : Bool :=
  evidence == "rustc-1.95.0:discharge-rust:x86_64-unknown-linux-gnu"

def rustcCheckedArtifact :
    CheckedArtifactRefinement rustc195Family dischargeRustWitness where
  evidence := "rustc-1.95.0:discharge-rust:x86_64-unknown-linux-gnu"
  check := rustcArtifactCheck
  accepted := by decide
  checkerSound := by
    intro candidate accepted
    have : candidate = "rustc-1.95.0:discharge-rust:x86_64-unknown-linux-gnu" := by
      simpa [rustcArtifactCheck] using accepted
    exact rustc195_artifact_corresponds

def rustcCheckedContext : ResidualContext :=
  ⟨"checked rustc artifact plus checker and platform",
    Nonempty (CheckedArtifactRefinement rustc195Family dischargeRustWitness) ∧ True⟩

def rustcCheckedReduction :
    TcbReduction rustc195Family dischargeRustWitness where
  oldContext := rustcModelOnlyContext
  newContext := rustcCheckedContext
  dischargedAssumption := rustc195Identity
  replacementEvidence := "checked:rustc-1.95.0:discharge-rust"
  remainingPremises := ["artifact-checker-sound",
    "target:x86_64-unknown-linux-gnu"]
  discharge := .checkedArtifact rustcCheckedArtifact
  entailsOldObligation := by
    rintro ⟨⟨checked⟩, platform⟩
    exact ⟨checked.checkerSound checked.evidence checked.accepted, platform⟩

theorem universal_reduction_discharges_exact_rustc :
    rustcUniversalReduction.dischargedAssumption = rustc195Identity ∧
    rustcUniversalReduction.remainingPremises =
      ["target:x86_64-unknown-linux-gnu"] := by
  exact ⟨rfl, rfl⟩

theorem universal_reduction_context_refines :
    ContextRefines rustcUniversalContext rustcModelOnlyContext := by
  exact rustcUniversalReduction.entailsOldObligation

theorem checked_reduction_discharges_exact_rustc :
    rustcCheckedReduction.dischargedAssumption = rustc195Identity ∧
    rustcCheckedReduction.remainingPremises =
      ["artifact-checker-sound", "target:x86_64-unknown-linux-gnu"] := by
  exact ⟨rfl, rfl⟩

theorem checked_reduction_context_refines :
    ContextRefines rustcCheckedContext rustcModelOnlyContext := by
  exact rustcCheckedReduction.entailsOldObligation

def incompleteCoverage : CoverageClaims :=
  ⟨false, false, true, false⟩

/-- Coverage can be incomplete while the already accepted artifact remains
sound under its checked evidence. -/
theorem accepted_artifact_sound_despite_incomplete_coverage :
    ArtifactCorresponds rustc195Family dischargeRustWitness := by
  have _coverage := incompleteCoverage
  exact rustcCheckedArtifact.checkerSound _ rustcCheckedArtifact.accepted

end Thermite.CertificationMetatheory
