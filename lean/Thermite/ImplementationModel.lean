import Thermite.CertificationMetatheory

/-!
Typed implementation-model families for RFC-3.

A family owns its input and behavior types, denotation, named language fragment,
and executable observation.  Observations carry the exact model identity that
produced them: model/version substitution is therefore a failed
correspondence, not an implicit change of semantics.

This layer only describes and relates models.  It deliberately contains no TCB
reduction: universal and checked-per-artifact discharge are separate AC-8/AC-9
obligations.
-/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

structure ModelIdentity where
  family : String
  version : String
deriving DecidableEq, Repr

structure ModelObservation (Behavior : Type) where
  model : ModelIdentity
  behavior : Behavior
deriving DecidableEq, Repr

/-- Component-specific semantic family.  Unlike a common untyped model record,
this permits rustc, a solver, and a checker to retain unrelated behavior types. -/
structure ImplementationModelFamily where
  Input : Type
  Behavior : Type
  identity : ModelIdentity
  toProgram : Input → Program
  inputIdentity : Input → String
  fragment : Fragment
  denotes : Input → Behavior → Prop
  observe : Input → ModelObservation Behavior
  decodeReplay : Input → String → Option (ModelObservation Behavior)

/-- Executable observation corresponds only when both its exact model version
and its behavior agree with the family denotation. -/
def ModelCorresponds (family : ImplementationModelFamily) : Prop :=
  ∀ input, family.fragment.admits (family.toProgram input) →
    (family.observe input).model = family.identity ∧
    family.denotes input (family.observe input).behavior

/-- A typed refinement maps both component-specific carriers while preserving
fragment membership and denotation. -/
structure ModelRefinement (source target : ImplementationModelFamily) where
  translateInput : source.Input → target.Input
  translateBehavior : source.Behavior → target.Behavior
  membership : ∀ input, source.fragment.admits (source.toProgram input) →
    target.fragment.admits (target.toProgram (translateInput input))
  denotation : ∀ input behavior, source.denotes input behavior →
    target.denotes (translateInput input) (translateBehavior behavior)

theorem model_refinement_refl (family : ImplementationModelFamily) :
    Nonempty (ModelRefinement family family) := by
  exact ⟨{
    translateInput := id
    translateBehavior := id
    membership := fun _ admitted => admitted
    denotation := fun _ _ modeled => modeled
  }⟩

def model_refinement_trans {first second third : ImplementationModelFamily}
    (firstSecond : ModelRefinement first second)
    (secondThird : ModelRefinement second third) :
    ModelRefinement first third := by
  exact {
    translateInput := secondThird.translateInput ∘ firstSecond.translateInput
    translateBehavior := secondThird.translateBehavior ∘ firstSecond.translateBehavior
    membership := fun input admitted =>
      secondThird.membership _ (firstSecond.membership input admitted)
    denotation := fun input behavior modeled =>
      secondThird.denotation _ _ (firstSecond.denotation input behavior modeled)
  }

/-- Ordinary family growth keeps the model identity exact, increases the named
fragment monotonically, and preserves both old denotation and observation. -/
structure ModelExpansion (old new : ImplementationModelFamily)
    (sameInput : old.Input = new.Input)
    (sameBehavior : old.Behavior = new.Behavior) where
  identity : old.identity = new.identity
  fragment : Expands old.fragment new.fragment
  semanticRefinement : ModelRefinement old new

/-- Semantic narrowing is a compatibility event, never an ordinary version
increment.  The existing fragment witness discipline is reused directly. -/
structure ModelCompatibilityBreak (old new : ImplementationModelFamily) where
  family : old.identity.family = new.identity.family
  fragment : CompatibilityBreak old.fragment new.fragment

/-! First substantive family: the exact rustc behavior Thermite consumes. -/

structure RustcInput where
  emitted : Program
deriving DecidableEq, Repr

inductive RustcDisposition where
  | accepted
  | rejected
deriving DecidableEq, Repr

structure RustcBehavior where
  disposition : RustcDisposition
  targetTriple : String
deriving DecidableEq, Repr

def thermiteRustV1Version : FragmentVersion :=
  ⟨"thermite-emitted-rust", 1⟩

def thermiteRustV1 : Fragment :=
  ⟨thermiteRustV1Version,
    fun program => program.facts.contains "thermite-rust-v1" = true ∧
      program.facts.contains "target:x86_64-unknown-linux-gnu" = true⟩

def thermiteRustV2 : Fragment :=
  ⟨⟨"thermite-emitted-rust", 2⟩,
    fun program =>
      (program.facts.contains "thermite-rust-v1" = true ∨
        program.facts.contains "thermite-rust-v2" = true) ∧
      program.facts.contains "target:x86_64-unknown-linux-gnu" = true⟩

def thermiteRustNarrow : Fragment :=
  ⟨⟨"thermite-emitted-rust-narrow", 1⟩,
    fun program => program.facts.contains "thermite-rust-v1" = true ∧
      program.facts.contains "target:x86_64-unknown-linux-gnu" = true⟩

def rustc195Identity : ModelIdentity := ⟨"rustc", "1.95.0"⟩

def thermiteRustV1Admits (input : RustcInput) : Bool :=
  input.emitted.facts.contains "thermite-rust-v1" &&
    input.emitted.facts.contains "target:x86_64-unknown-linux-gnu"

def rustc195Behavior (input : RustcInput) : RustcBehavior :=
  if thermiteRustV1Admits input then
    ⟨.accepted, "x86_64-unknown-linux-gnu"⟩
  else
    ⟨.rejected, "x86_64-unknown-linux-gnu"⟩

def rustc195ReplayPayload (input : RustcInput) : String :=
  "rustc-1.95.0:" ++ input.emitted.digest ++ ":x86_64-unknown-linux-gnu"

def rustc195DecodeReplay (input : RustcInput) (payload : String) :
    Option (ModelObservation RustcBehavior) :=
  if payload = rustc195ReplayPayload input then
    some ⟨rustc195Identity, rustc195Behavior input⟩
  else none

/-- Declarative behavior contract, stated independently of the executable
observation function. -/
def rustc195Denotation (input : RustcInput) (behavior : RustcBehavior) : Prop :=
  behavior.targetTriple = "x86_64-unknown-linux-gnu" ∧
    if thermiteRustV1Admits input then
      behavior.disposition = .accepted
    else
      behavior.disposition = .rejected

def rustc195Family : ImplementationModelFamily where
  Input := RustcInput
  Behavior := RustcBehavior
  identity := rustc195Identity
  toProgram := RustcInput.emitted
  inputIdentity := fun input => input.emitted.digest
  fragment := thermiteRustV1
  denotes := rustc195Denotation
  observe := fun input => ⟨rustc195Identity, rustc195Behavior input⟩
  decodeReplay := rustc195DecodeReplay

/-- Correspondence is deliberately limited to the named Thermite-emitted
fragment; it says nothing about whole-Rust behavior. -/
theorem rustc195_corresponds_on_thermite_fragment :
    ModelCorresponds rustc195Family := by
  intro input _
  constructor
  · rfl
  · by_cases admitted : thermiteRustV1Admits input = true <;>
      simp [rustc195Family, rustc195Denotation, rustc195Behavior, admitted]

/-! A second, deliberately different model family over the same named input
fragment. Its behavior carrier projects rustc's disposition into the portable
acceptance observation consumed by assurance transport. This is the largest
cross-family fragment justified by the current metatheory: the named
Thermite-emitted Rust fragment, not whole Rust. -/

structure PortableRustBehavior where
  accepted : Bool
  platform : String
deriving DecidableEq, Repr

def portableRustIdentity : ModelIdentity := ⟨"thermite-portable-rust", "1"⟩

def portableRustBehavior (input : RustcInput) : PortableRustBehavior :=
  ⟨thermiteRustV1Admits input, "x86_64-unknown-linux-gnu"⟩

def portableRustDenotation (input : RustcInput) (behavior : PortableRustBehavior) : Prop :=
  behavior.platform = "x86_64-unknown-linux-gnu" ∧
    behavior.accepted = thermiteRustV1Admits input

def portableRustReplayPayload (input : RustcInput) : String :=
  "thermite-portable-rust-1:" ++ input.emitted.digest ++ ":x86_64-unknown-linux-gnu"

def portableRustDecodeReplay (input : RustcInput) (payload : String) :
    Option (ModelObservation PortableRustBehavior) :=
  if payload = portableRustReplayPayload input then
    some ⟨portableRustIdentity, portableRustBehavior input⟩
  else none

def portableRustFamily : ImplementationModelFamily where
  Input := RustcInput
  Behavior := PortableRustBehavior
  identity := portableRustIdentity
  toProgram := RustcInput.emitted
  inputIdentity := fun input => input.emitted.digest
  fragment := thermiteRustV1
  denotes := portableRustDenotation
  observe := fun input => ⟨portableRustIdentity, portableRustBehavior input⟩
  decodeReplay := portableRustDecodeReplay

theorem portable_rust_corresponds_on_thermite_fragment :
    ModelCorresponds portableRustFamily := by
  intro input _
  exact ⟨rfl, rfl, rfl⟩

def rustcBehaviorToPortable (behavior : RustcBehavior) : PortableRustBehavior :=
  ⟨decide (behavior.disposition = .accepted), behavior.targetTriple⟩

/-- Checked denotation correspondence from the concrete rustc model family to
the portable assurance-observation family on exactly `thermiteRustV1`. -/
def rustc195_refines_portable_rust :
    ModelRefinement rustc195Family portableRustFamily := {
  translateInput := id
  translateBehavior := rustcBehaviorToPortable
  membership := fun _ admitted => admitted
  denotation := by
    intro input behavior modeled
    constructor
    · exact modeled.1
    · by_cases admitted : thermiteRustV1Admits input = true
      · have accepted : behavior.disposition = .accepted := by
          simpa [rustc195Denotation, admitted] using modeled.2
        simp [rustcBehaviorToPortable, admitted, accepted]
      · have rejected : behavior.disposition = .rejected := by
          simpa [rustc195Denotation, admitted] using modeled.2
        simp [rustcBehaviorToPortable, admitted, rejected]
}

theorem thermite_rust_v1_expands_to_v2 : Expands thermiteRustV1 thermiteRustV2 := by
  exact ⟨rfl, by decide, fun _ admitted => ⟨Or.inl admitted.1, admitted.2⟩⟩

def rustV2OnlyWitness : Program :=
  ⟨"rust-v2-only", [],
    ["thermite-rust-v2", "target:x86_64-unknown-linux-gnu"]⟩

def thermite_rust_narrowing_is_explicit :
    CompatibilityBreak thermiteRustV2 thermiteRustNarrow := by
  exact {
    lineageChanged := by decide
    witness := rustV2OnlyWitness
    admittedBefore := by simp [thermiteRustV2, rustV2OnlyWitness]
    excludedAfter := by simp [thermiteRustNarrow, rustV2OnlyWitness]
  }

end Thermite.CertificationMetatheory
