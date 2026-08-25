import Thermite.LanguageCompleteness

/-!
Negative-space pin for fragment evolution.

The mutant silently relabels the concrete core-v2 → spec-only narrowing as an
ordinary expansion.  `narrowing_is_not_expansion` must refute that claim using
the checked compatibility witness; removing the membership implication or
reusing the old lineage makes this pin stop expressing the required boundary.
-/

namespace Thermite.LanguageCompleteness

def mutantSilentNarrowing : Prop := Expands coreV2 specOnlyV1

theorem mutant_silent_narrowing_rejected : ¬mutantSilentNarrowing := by
  exact narrowing_is_not_expansion

end Thermite.LanguageCompleteness
