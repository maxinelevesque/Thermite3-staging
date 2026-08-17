import Thermite.CertificationPolicy
import Thermite.ImplementationModel

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

inductive ReplayScope where
  | all | bounded (bound : Nat) | perExecution | none
deriving DecidableEq, Repr

inductive ReplayRefutation where
  | complete | incomplete | empirical | trace (bound : Nat) | abort | none
deriving DecidableEq, Repr

inductive ReplayResidualTrust where
  | leanChecked | solver | fiat
deriving DecidableEq, Repr

inductive ReplayBoundary where
  | endToEnd | toBoundary (via : String) | toPlatform (platform : String)
deriving DecidableEq, Repr

inductive ReplayClassification where
  | admitted | rejected | unknown
deriving DecidableEq, Repr

/-- Seven families of coherent RFC-3 positions. `bounded` is one family with a
parameter, not a claim that one chosen bound exhausts the family. -/
inductive CertificationCell where
  | noClaim | runtime | bounded | incompleteSolver | completeSolver
  | empiricalLean | completeLean
deriving DecidableEq, Repr

def allCertificationCells : List CertificationCell :=
  [.noClaim, .runtime, .bounded, .incompleteSolver, .completeSolver,
    .empiricalLean, .completeLean]

/-- Typed formal data decoded from the interchange row. These are the same
metatheory objects used elsewhere: `ModelIdentity`, `FragmentVersion`, and
`PolicyPoint`; frames, contexts, and fragments are constructed below. -/
structure FormalReplayProjection where
  cell : CertificationCell
  scope : ReplayScope
  refutation : ReplayRefutation
  residualTrust : ReplayResidualTrust
  dischargedTrust : List String
  boundary : ReplayBoundary
  model : ModelIdentity
  semantics : FragmentVersion
  residualContextName : String
  classificationFragment : FragmentVersion
  classification : ReplayClassification
  policy : PolicyPoint
deriving DecidableEq, Repr

def FormalReplayProjection.frame (projection : FormalReplayProjection) : SemanticFrame :=
  ⟨projection.semantics.lineage, projection.semantics.revision,
    projection.model.family ++ ":" ++ projection.model.version, 1,
    ⟨reprStr projection.boundary, fun _ => True⟩⟩

def FormalReplayProjection.context (projection : FormalReplayProjection) : ResidualContext :=
  ⟨projection.residualContextName, True⟩

def FormalReplayProjection.fragment (projection : FormalReplayProjection) : Fragment :=
  ⟨projection.classificationFragment, fun _ => True⟩

def boundedProjection (bound : Nat) : FormalReplayProjection :=
  ⟨.bounded, .bounded bound, .trace bound, .solver, [],
    .toPlatform "x86_64-unknown-linux-gnu", rustc195Identity,
    ⟨"thermite-language", 1⟩, "solver and platform",
    ⟨"thermite-core", 2⟩, .rejected, .bounded⟩

def coherentProjection (projection : FormalReplayProjection) : Bool :=
  match projection.cell, projection.scope, projection.refutation,
      projection.residualTrust, projection.policy with
  | .noClaim, .none, .none, .fiat, .runtime => true
  | .runtime, .perExecution, .abort, .fiat, .runtime => true
  | .bounded, .bounded scopeBound, .trace traceBound, .solver, .bounded =>
      scopeBound == traceBound
  | .incompleteSolver, .all, .incomplete, .solver, .solverComplete => true
  | .completeSolver, .all, .complete, .solver, .solverComplete => true
  | .empiricalLean, .all, .empirical, .leanChecked, .leanEmpirical => true
  | .completeLean, .all, .complete, .leanChecked, .leanEmpirical => true
  | _, _, _, _, _ => false

theorem every_bound_has_a_coherent_bounded_projection (bound : Nat) :
    coherentProjection (boundedProjection bound) = true := by simp [coherentProjection, boundedProjection]

end Thermite.CertificationMetatheory
