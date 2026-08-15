import Std

/-!
Language-wide, versioned completeness vocabulary.

This module is deliberately neutral with respect to RFC-10 and to any concrete
checker.  It separates language membership, semantic validity, operational
stage outcomes, and completeness claims so no downstream stage follows merely
from success at an upstream one.
-/

namespace Thermite.LanguageCompleteness

/-- A stable projection of a parsed language program.  Feature-specific models
refine into this representation; it is not a serialization format. -/
structure Program where
  digest : String
  constructs : List String
  facts : List String
deriving DecidableEq, Repr

/-- Immutable fragment identity. A narrowing changes `lineage`; an ordinary
expansion increments `revision` within the same lineage. -/
structure FragmentVersion where
  lineage : String
  revision : Nat
deriving DecidableEq, Repr

/-- A named, versioned subset of canonical programs. -/
structure Fragment where
  version : FragmentVersion
  admits : Program → Prop

/-- Semantic validity is independent of whether any operational stage runs. -/
abbrev SemanticValidity := Program → Prop

inductive Stage where
  | parser
  | validator
  | canonicalSemantics
  | checkedIr
  | lowering
  | proofRoute
  | policy
  | certification
deriving DecidableEq, Repr

/-- Closed outcome vocabulary for an attempted stage. Environment and resource
failures are neither language exclusions nor verification refutations. -/
inductive Outcome (α : Type) where
  | success (value : α)
  | unsupportedLanguage (fragment : FragmentVersion) (detail : String)
  | invalidSource (detail : String)
  | unsupportedPolicy (detail : String)
  | resourceExhausted (resource : String)
  | toolUnavailable (tool : String)
  | toolIncompatible (tool version : String)
  | counterexample (witness : String)
  | proofFailure (detail : String)
  | soundnessAlarm (detail : String)
deriving DecidableEq, Repr

/-- A stage-indexed computation. The stage index prevents a result from being
silently reused as evidence about a different stage. -/
structure StageResult (stage : Stage) (α : Type) where
  outcome : Outcome α
deriving DecidableEq, Repr

/-- Ordinary version growth: same lineage and old membership implies new. -/
def Expands (old new : Fragment) : Prop :=
  old.version.lineage = new.version.lineage ∧
    old.version.revision ≤ new.version.revision ∧
    ∀ program, old.admits program → new.admits program

theorem expands_refl (fragment : Fragment) : Expands fragment fragment := by
  exact ⟨rfl, Nat.le_refl _, fun _ admitted => admitted⟩

theorem expands_trans {first second third : Fragment}
    (firstSecond : Expands first second) (secondThird : Expands second third) :
    Expands first third := by
  rcases firstSecond with ⟨lineage₁, revision₁, membership₁⟩
  rcases secondThird with ⟨lineage₂, revision₂, membership₂⟩
  exact ⟨lineage₁.trans lineage₂, Nat.le_trans revision₁ revision₂,
    fun program admitted => membership₂ program (membership₁ program admitted)⟩

/-- A compatibility break is explicit: the lineage changes and a concrete old
member witnesses that the new fragment is a narrowing. -/
structure CompatibilityBreak (old new : Fragment) where
  lineageChanged : old.version.lineage ≠ new.version.lineage
  witness : Program
  admittedBefore : old.admits witness
  excludedAfter : ¬new.admits witness

/-- Completeness claims are stage-specific and require semantic validity as a
separate premise. -/
def CompleteAt (fragment : Fragment) (valid : SemanticValidity) (stage : Stage)
    (run : Program → StageResult stage Unit) : Prop :=
  ∀ program, fragment.admits program → valid program →
    ∃ value, run program = ⟨Outcome.success value⟩

end Thermite.LanguageCompleteness
