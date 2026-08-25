import Thermite.CertificationMetatheory

/-! Negative-space pins for RFC-3 metatheory AC-3. -/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

/-- Reversing unequal bounded-scope refinement would admit five constructs
under a bound of two. -/
theorem unequal_bound_reverse_rejected : ¬ BoundRefines 5 2 := by simp [BoundRefines]

def platformOnlyWitness : Program :=
  { digest := "platform-only"
    constructs := []
    facts := [] }

/-- A boundary-qualified result cannot silently be upgraded to end-to-end. -/
theorem boundary_upgrade_rejected : ¬ FrameRefines platformFrame endToEndFrame := by
  intro refinement
  have qualified : endToEndBoundary.qualifies platformOnlyWitness :=
    refinement.boundary platformOnlyWitness trivial
  simp [endToEndBoundary, platformOnlyWitness] at qualified

end Thermite.CertificationMetatheory
