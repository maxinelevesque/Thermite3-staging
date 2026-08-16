import Thermite.ImplementationModel

/-! Negative-space pins for RFC-3 metatheory AC-7. -/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

/-- An observation labeled as a different rustc version cannot correspond to
the pinned 1.95.0 model even when its behavior payload is unchanged. -/
def silentlySubstitutedRustcFamily : ImplementationModelFamily where
  Input := RustcInput
  Behavior := RustcBehavior
  identity := rustc195Identity
  toProgram := RustcInput.emitted
  fragment := thermiteRustV1
  denotes := fun input behavior => behavior = rustc195Behavior input
  observe := fun input =>
    ⟨⟨"rustc", "1.96.0"⟩, rustc195Behavior input⟩

def pinnedRustWitness : RustcInput :=
  ⟨⟨"pinned-rust", [],
    ["thermite-rust-v1", "target:x86_64-unknown-linux-gnu"]⟩⟩

theorem silent_rustc_version_substitution_rejected :
    ¬ ModelCorresponds silentlySubstitutedRustcFamily := by
  intro corresponds
  have versionMatches :=
    (corresponds pinnedRustWitness
      (by simp [silentlySubstitutedRustcFamily, thermiteRustV1, pinnedRustWitness])).1
  simp [silentlySubstitutedRustcFamily, rustc195Identity] at versionMatches

def thermiteRustSilentNarrow : Fragment :=
  ⟨⟨"thermite-emitted-rust", 3⟩,
    fun program => program.facts.contains "thermite-rust-v1" = true ∧
      program.facts.contains "target:x86_64-unknown-linux-gnu" = true⟩

/-- The v2-only witness prevents a semantic narrowing from masquerading as an
ordinary same-lineage expansion. -/
theorem silent_rust_fragment_narrowing_rejected :
    ¬ Expands thermiteRustV2 thermiteRustSilentNarrow := by
  intro expansion
  have admitted := expansion.2.2 rustV2OnlyWitness
    (by simp [thermiteRustV2, rustV2OnlyWitness])
  simp [thermiteRustSilentNarrow, rustV2OnlyWitness] at admitted

end Thermite.CertificationMetatheory
