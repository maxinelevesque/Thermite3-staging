/-
  RFC-9's formal boundary: finite effect footprints, call closure, region
  ancestry, and pairwise concurrent acceptance.

  This module deliberately does not model paired executions, `hides`,
  couplings, or hyperproperties. Those belong to relational contracts.
-/

namespace Thermite.EffectRows

abbrev Region := List Nat

inductive Operation where
  | read
  | write
  | invoke
  deriving DecidableEq

structure Effect where
  operation : Operation
  region : Region
  deriving DecidableEq

abbrev Footprint := List Effect

def UpperBound (declared inferred : Footprint) : Prop :=
  inferred ⊆ declared

def Overlaps (left right : Region) : Prop :=
  left.IsPrefix right ∨ right.IsPrefix left

def ConcurrentAccept
    (commutes : Effect → Effect → Prop)
    (left right : Footprint) : Prop :=
  ∀ leftEffect ∈ left, ∀ rightEffect ∈ right,
    Overlaps leftEffect.region rightEffect.region →
      commutes leftEffect rightEffect

theorem call_footprint_closed
    (direct callee inferred : Footprint)
    (directClosed : direct ⊆ inferred)
    (calleeClosed : callee ⊆ inferred) :
    direct ++ callee ⊆ inferred := by
  intro effect member
  rw [List.mem_append] at member
  cases member with
  | inl member => exact directClosed member
  | inr member => exact calleeClosed member

theorem upper_bound_transitive
    (outer middle inner : Footprint)
    (outerBoundsMiddle : UpperBound outer middle)
    (middleBoundsInner : UpperBound middle inner) :
    UpperBound outer inner := by
  intro effect member
  exact outerBoundsMiddle (middleBoundsInner member)

theorem overlaps_reflexive (region : Region) : Overlaps region region := by
  exact Or.inl (List.prefix_refl region)

theorem overlaps_symmetric {left right : Region} :
    Overlaps left right → Overlaps right left := by
  intro overlap
  exact overlap.elim (fun ancestry => Or.inr ancestry) (fun ancestry => Or.inl ancestry)

theorem accepted_pair_commutes
    (commutes : Effect → Effect → Prop)
    (left right : Footprint)
    (accepted : ConcurrentAccept commutes left right)
    {leftEffect rightEffect : Effect}
    (leftMember : leftEffect ∈ left)
    (rightMember : rightEffect ∈ right)
    (overlap : Overlaps leftEffect.region rightEffect.region) :
    commutes leftEffect rightEffect := by
  exact accepted leftEffect leftMember rightEffect rightMember overlap

theorem disjoint_pairs_impose_no_commutation
    (commutes : Effect → Effect → Prop)
    (left right : Footprint)
    (disjoint : ∀ l ∈ left, ∀ r ∈ right, ¬ Overlaps l.region r.region) :
    ConcurrentAccept commutes left right := by
  intro l lMember r rMember overlap
  exact False.elim (disjoint l lMember r rMember overlap)

/- RFC-10 Tier A: a semantic item is characterized by the same finite effect
   list as the Rust consumer. `resultCongruent` is functional dependence on the
   declared footprint; `framesWrites` is the canonical write-frame obligation. -/
abbrev State := Region → Nat

def HasOperation (operation : Operation) (footprint : Footprint) (region : Region) : Prop :=
  ∃ effect ∈ footprint, effect.operation = operation ∧ effect.region = region

def AgreesOnFootprint (left right : State) (footprint : Footprint) : Prop :=
  ∀ effect ∈ footprint, left effect.region = right effect.region

structure ItemSemantics (Result : Type) (footprint : Footprint) where
  run : State → Result → State → Prop
  resultCongruent :
    ∀ {s₁ s₂ r₁ r₂ t₁ t₂},
      AgreesOnFootprint s₁ s₂ footprint → run s₁ r₁ t₁ → run s₂ r₂ t₂ → r₁ = r₂
  framesWrites :
    ∀ {s r t region}, run s r t → ¬ HasOperation .write footprint region → t region = s region

theorem relational_frame
    {Result : Type}
    (footprint : Footprint)
    (semantics : ItemSemantics Result footprint)
    {s₁ s₂ : State} {r₁ r₂ : Result} {t₁ t₂ : State}
    (agree : AgreesOnFootprint s₁ s₂ footprint)
    (run₁ : semantics.run s₁ r₁ t₁)
    (run₂ : semantics.run s₂ r₂ t₂) :
    r₁ = r₂ ∧
      (∀ region, ¬ HasOperation .write footprint region →
        t₁ region = s₁ region ∧ t₂ region = s₂ region) := by
  constructor
  · exact semantics.resultCongruent agree run₁ run₂
  · intro region outside
    exact ⟨semantics.framesWrites run₁ outside, semantics.framesWrites run₂ outside⟩

theorem pure_deterministic
    {Result : Type}
    (semantics : ItemSemantics Result [])
    {s₁ s₂ : State} {r₁ r₂ : Result} {t₁ t₂ : State}
    (run₁ : semantics.run s₁ r₁ t₁)
    (run₂ : semantics.run s₂ r₂ t₂) : r₁ = r₂ := by
  exact semantics.resultCongruent (by intro effect member; contradiction) run₁ run₂

theorem outside_write_equal
    {Result : Type}
    (footprint : Footprint)
    (semantics : ItemSemantics Result footprint)
    {s₁ s₂ : State} {r₁ r₂ : Result} {t₁ t₂ : State} {region : Region}
    (initiallyEqual : s₁ region = s₂ region)
    (outside : ¬ HasOperation .write footprint region)
    (run₁ : semantics.run s₁ r₁ t₁)
    (run₂ : semantics.run s₂ r₂ t₂) : t₁ region = t₂ region := by
  rw [semantics.framesWrites run₁ outside, semantics.framesWrites run₂ outside]
  exact initiallyEqual

end Thermite.EffectRows
