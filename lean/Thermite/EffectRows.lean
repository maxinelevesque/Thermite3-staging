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

end Thermite.EffectRows
