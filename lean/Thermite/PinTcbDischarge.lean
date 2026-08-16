import Thermite.TcbDischarge
import Thermite.PinImplementationModel

/-! Negative-space pins for RFC-3 metatheory AC-8 and AC-9. -/

namespace Thermite.CertificationMetatheory

/-- A mutation that simply deletes a required assumption changes `False` to
`True`; no residual-context entailment exists, so no `TcbReduction` can carry
this transition. -/
def assumptionRequiredContext : ResidualContext :=
  ⟨"required rustc assumption", False⟩

def illicitDeletionContext : ResidualContext :=
  ⟨"assumption silently deleted", True⟩

theorem illicit_assumption_deletion_rejected :
    ¬ ContextRefines illicitDeletionContext assumptionRequiredContext := by
  intro refinement
  exact refinement trivial

theorem illicit_tcb_reduction_rejected :
    ¬ ∃ reduction : TcbReduction rustc195Family dischargeRustWitness,
      reduction.oldContext = assumptionRequiredContext ∧
      reduction.newContext = illicitDeletionContext := by
  rintro ⟨reduction, oldContext, newContext⟩
  have entailment := reduction.entailsOldObligation
  rw [oldContext, newContext] at entailment
  exact illicit_assumption_deletion_rejected entailment

/-- An artifact for a different digest cannot reuse the pinned artifact's
checked evidence: the checker accepts only the exact replay key. -/
def otherRustArtifact : RustcInput :=
  ⟨⟨"other-rust", [],
    ["thermite-rust-v1", "target:x86_64-unknown-linux-gnu"]⟩⟩

def wrongArtifactCheck (evidence : String) : Bool :=
  evidence == "rustc-1.95.0:discharge-rust:x86_64-unknown-linux-gnu"

theorem checked_evidence_is_artifact_pinned :
    wrongArtifactCheck
      "rustc-1.95.0:other-rust:x86_64-unknown-linux-gnu" = false := by
  decide

/-- Completeness booleans cannot be projected into artifact soundness; the
only public theorem keeps the checked correspondence premise explicit. -/
theorem coverage_does_not_supply_correspondence
    (_coverage : CoverageClaims)
    (sound : ArtifactCorresponds rustc195Family dischargeRustWitness) :
    ArtifactCorresponds rustc195Family dischargeRustWitness := by
  exact sound

def nominallyCompleteCoverage : CoverageClaims :=
  ⟨true, true, true, true⟩

/-- Even every completeness flag set to true cannot establish correspondence
for an observation carrying the wrong model version. -/
theorem complete_coverage_cannot_mask_version_substitution :
    ¬ ArtifactCorresponds silentlySubstitutedRustcFamily pinnedRustWitness := by
  have _coverage := nominallyCompleteCoverage
  intro corresponds
  have versionMatches := corresponds.1
  simp [silentlySubstitutedRustcFamily, rustc195Identity] at versionMatches

end Thermite.CertificationMetatheory
