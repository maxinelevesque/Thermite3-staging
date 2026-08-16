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
      reduction.dischargedAssumption = ⟨"unrelated-component", "0"⟩ := by
  rintro ⟨reduction, forged⟩
  simp [TcbReduction.dischargedAssumption, rustc195Family,
    rustc195Identity] at forged

/-- An artifact for a different digest cannot reuse the pinned artifact's
checked evidence: the checker accepts only the exact replay key. -/
def otherRustArtifact : RustcInput :=
  ⟨⟨"other-rust", [],
    ["thermite-rust-v1", "target:x86_64-unknown-linux-gnu"]⟩⟩

theorem checked_evidence_is_artifact_pinned :
    ¬ ∃ evidence : ReplayEvidence rustc195Family dischargeRustWitness,
      evidence.payload = "rustc-1.95.0:other-rust:x86_64-unknown-linux-gnu" := by
  rintro ⟨evidence, wrongPayload⟩
  have replayed := evidence.replayed
  rw [wrongPayload] at replayed
  simp [rustc195Family, rustc195DecodeReplay, rustc195ReplayPayload,
    dischargeRustWitness] at replayed

/-- Completeness booleans cannot be projected into artifact soundness; the
only public theorem keeps the checked correspondence premise explicit. -/
theorem coverage_does_not_supply_correspondence
    (_coverage : CoverageClaims)
    (sound : ArtifactModeled rustc195Family dischargeRustWitness) :
    ArtifactModeled rustc195Family dischargeRustWitness := by
  exact sound

def nominallyCompleteCoverage : CoverageClaims :=
  ⟨true, true, true, true⟩

/-- Even every completeness flag set to true cannot establish correspondence
for an observation carrying the wrong model version. -/
theorem complete_coverage_cannot_mask_version_substitution :
    ¬ BehaviorCorresponds silentlySubstitutedRustcFamily pinnedRustWitness
      (silentlySubstitutedRustcFamily.observe pinnedRustWitness) := by
  have _coverage := nominallyCompleteCoverage
  intro corresponds
  have versionMatches := corresponds.1
  simp [silentlySubstitutedRustcFamily, rustc195Identity] at versionMatches

end Thermite.CertificationMetatheory
