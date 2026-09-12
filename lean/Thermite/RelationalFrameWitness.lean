/-
  Thermite/RelationalFrameWitness.lean — exact Tier-A artifact replay.

  A producer may compute this witness, but it cannot choose its semantic facts:
  body footprints and effect support are recomputed from the canonical input.
  The initial producer is deliberately source-only.  End-to-end authority is a
  later transport theorem, not a Boolean that the emitter may set.
-/

import Thermite.RelationalFrame

namespace Thermite.RelationalFrameWitness

open Thermite.RelationalFrame

inductive Projection where
  | result
  | writeFrame
  | outcome
  | termination
  | trace
  | accumulator
deriving DecidableEq, Repr

inductive Scope where
  | sourceOnly
  | endToEnd
deriving DecidableEq, Repr

structure CanonicalInput where
  artifactDigest : String
  body : Bounded.Program
  normalizedRow : List EffectKind
  semanticFragment : String
  requested : List Projection
deriving DecidableEq, Repr

structure Witness where
  artifactDigest : String
  body : Bounded.Program
  normalizedRow : List EffectKind
  readFootprint : List Region
  writeFootprint : List Region
  effectSupport : List ProjectionSupport
  semanticFragment : String
  requested : List Projection
  scope : Scope
deriving DecidableEq, Repr

def supportOf (projection : Projection) (support : ProjectionSupport) : Support :=
  match projection with
  | .result => support.result
  | .writeFrame => support.writeFrame
  | .outcome => support.outcome
  | .termination => support.termination
  | .trace => support.trace
  | .accumulator => support.accumulator

def claimable : Support → Bool
  | .derived => true
  | .conditional _ => true
  | .structural => true
  | .unavailable _ => false
  | .notApplicable => false

def rowSupports (row : List EffectKind) (projection : Projection) : Bool :=
  row.all fun kind => claimable (supportOf projection (supportForKind kind))

def requestedSupported (input : CanonicalInput) : Bool :=
  input.requested.all (rowSupports input.normalizedRow)

def produce (input : CanonicalInput) : Witness :=
  { artifactDigest := input.artifactDigest
    body := input.body
    normalizedRow := input.normalizedRow
    readFootprint := input.body.reads
    writeFootprint := input.body.writes
    effectSupport := input.normalizedRow.map supportForKind
    semanticFragment := input.semanticFragment
    requested := input.requested
    scope := .sourceOnly }

def producerRefines (input : CanonicalInput) (witness : Witness) : Bool :=
  decide (witness = produce input)

def verify (input : CanonicalInput) (witness : Witness) : Bool :=
  producerRefines input witness && requestedSupported input

def Supported (input : CanonicalInput) (witness : Witness) : Prop :=
  witness = produce input ∧ requestedSupported input = true

theorem producerRefines_iff {input : CanonicalInput} {witness : Witness} :
    producerRefines input witness = true ↔ witness = produce input := by
  simp [producerRefines]

theorem verify_iff_supported {input : CanonicalInput} {witness : Witness} :
    verify input witness = true ↔ Supported input witness := by
  simp [verify, Supported, producerRefines]

theorem produce_complete {input : CanonicalInput}
    (supported : requestedSupported input = true) :
    verify input (produce input) = true := by
  simp [verify, producerRefines, supported]

namespace Examples

open Thermite.RelationalFrame.Bounded.Examples

def canonical : CanonicalInput :=
  { artifactDigest := "sha256:tier-a-increment-a"
    body := setA
    normalizedRow := [.read, .write]
    semanticFragment := "tier-a-state-core-v1"
    requested := [.result, .writeFrame, .outcome, .termination] }

def witness : Witness := produce canonical

theorem accepted : verify canonical witness = true := by
  decide

theorem digest_mutant_rejected :
    verify canonical { witness with artifactDigest := "sha256:other" } = false := by
  decide

theorem body_mutant_rejected :
    verify canonical { witness with body := .ret (.literal (.bool false)) } = false := by
  decide

theorem row_mutant_rejected :
    verify canonical { witness with normalizedRow := [.read] } = false := by
  decide

theorem read_footprint_mutant_rejected :
    verify canonical { witness with readFootprint := [] } = false := by
  decide

theorem write_footprint_mutant_rejected :
    verify canonical { witness with writeFootprint := [] } = false := by
  decide

theorem region_mutant_rejected :
    verify canonical { witness with writeFootprint := [regionB] } = false := by
  decide

theorem law_mutant_rejected :
    verify canonical { witness with effectSupport := [stateSupport] } = false := by
  decide

theorem fragment_mutant_rejected :
    verify canonical { witness with semanticFragment := "other-fragment" } = false := by
  decide

theorem requested_projection_mutant_rejected :
    verify canonical { witness with requested := [.writeFrame] } = false := by
  decide

theorem scope_upgrade_rejected :
    verify canonical { witness with scope := .endToEnd } = false := by
  decide

def pureInput : CanonicalInput :=
  { artifactDigest := "sha256:pure"
    body := .ret (.literal (.bool false))
    normalizedRow := []
    semanticFragment := "tier-a-state-core-v1"
    requested := [.result, .writeFrame, .outcome, .termination] }

/-- The empty surface row is pure, not the unavailable bare-random primitive. -/
theorem pure_empty_row_is_supported :
    verify pureInput (produce pureInput) = true := by
  decide

end Examples

end Thermite.RelationalFrameWitness
