import Std

namespace Thermite.CheckedTraversal

structure Edge where
  parent : Nat
  child : Nat
  role : String
deriving DecidableEq, Repr

structure CloseEdge where
  nodeAt : Nat
  reason : String
  innerToOuter : List String
deriving DecidableEq, Repr

structure Holding where
  node : Nat
  lock : String
  guardedRegion : String
  capabilityNode : Nat
  incomingHeld : List String
  outgoingHeld : List String
  closeEdges : List CloseEdge
deriving DecidableEq, Repr

structure SharedPlace where
  node : Nat
  path : String
  mode : String
  authorizingLocks : List String
deriving DecidableEq, Repr

structure Footprint where
  function : String
  effects : List String
deriving DecidableEq, Repr

structure CallRow where
  function : String
  callees : List String
deriving DecidableEq, Repr

/-- Projection made directly from the canonical syntax inventory, independently
of the optimized checked-program evidence producer. -/
structure CanonicalAst where
  digest : String
  nodeKinds : List String
  nodeFacts : List String
  edges : List Edge
  holdings : List Holding
  sharedPlaces : List SharedPlace
  authorityRequiredNodes : List Nat
  directFootprints : List Footprint
  calls : List CallRow
deriving DecidableEq, Repr

/-- The actual finite production witness projection consumed by Forge. -/
structure Witness where
  version : Nat
  digest : String
  nodeKinds : List String
  nodeFacts : List String
  edges : List Edge
  directFootprints : List Footprint
  calls : List CallRow
  footprints : List Footprint
  holdings : List Holding
  sharedPlaces : List SharedPlace
deriving DecidableEq, Repr

def nodeIds (ast : CanonicalAst) : List Nat := List.range ast.nodeKinds.length

def edgeWellFormed (ast : CanonicalAst) (edge : Edge) : Bool :=
  edge.parent < ast.nodeKinds.length && edge.child < ast.nodeKinds.length &&
    edge.parent < edge.child

def holdingWellFormed (ast : CanonicalAst) (holding : Holding) : Bool :=
  holding.node < ast.nodeFacts.length &&
    holding.capabilityNode == holding.node &&
    holding.outgoingHeld == holding.incomingHeld &&
    !holding.closeEdges.isEmpty &&
    holding.closeEdges.all fun edge =>
      edge.nodeAt < ast.nodeFacts.length && !edge.innerToOuter.isEmpty &&
        edge.innerToOuter.contains holding.lock

def sharedPlaceWellFormed (ast : CanonicalAst) (place : SharedPlace) : Bool :=
  place.node < ast.nodeFacts.length &&
    (place.mode == "Read" || place.mode == "Write") &&
    (ast.authorityRequiredNodes.contains place.node == !place.authorizingLocks.isEmpty)

def footprintWellFormed (_footprint : Footprint) : Bool := true

def lookupEffects (function : String) (rows : List Footprint) : List String :=
  match rows.find? (·.function == function) with
  | some row => row.effects
  | none => []

def unionEffects (left right : List String) : List String :=
  right.foldl (fun effects effect =>
    if effects.contains effect then effects else effects ++ [effect]) left

def closureStep (direct : List Footprint) (calls : List CallRow)
    (current : List Footprint) : List Footprint :=
  direct.map fun row =>
    let callees := match calls.find? (·.function == row.function) with
      | some call => call.callees
      | none => []
    ⟨row.function, (callees.foldl (fun effects callee =>
      unionEffects effects (lookupEffects callee current)) row.effects).mergeSort⟩

def closureFuel (direct : List Footprint) (calls : List CallRow) :
    Nat → List Footprint → List Footprint
  | 0, current => current
  | fuel + 1, current => closureFuel direct calls fuel (closureStep direct calls current)

def closure (direct : List Footprint) (calls : List CallRow) : List Footprint :=
  closureFuel direct calls direct.length direct

def listSetEq (left right : List String) : Bool :=
  left.length == right.length && left.all right.contains && right.all left.contains

def footprintsClosed (witness : Witness) : Bool :=
  let expected := closure witness.directFootprints witness.calls
  witness.footprints.length == expected.length && witness.footprints.all fun row =>
    listSetEq row.effects (lookupEffects row.function expected)

def callsWellFormed (witness : Witness) : Bool :=
  let functions := witness.directFootprints.map (·.function)
  witness.calls.map (·.function) == functions &&
    witness.footprints.map (·.function) == functions &&
    witness.calls.all fun row => row.callees.all functions.contains

def structuralComplete (ast : CanonicalAst) (witness : Witness) : Prop :=
  witness.digest = ast.digest ∧ witness.nodeKinds = ast.nodeKinds ∧
    witness.nodeFacts = ast.nodeFacts ∧ witness.edges = ast.edges ∧
    witness.directFootprints = ast.directFootprints ∧ witness.calls = ast.calls

def evidenceWellFormed (ast : CanonicalAst) (witness : Witness) : Prop :=
  (witness.footprints.all footprintWellFormed = true) ∧
  (witness.holdings.all (holdingWellFormed ast) = true) ∧
  (witness.sharedPlaces.all (sharedPlaceWellFormed ast) = true)

def footprintClosureSound (witness : Witness) : Prop := footprintsClosed witness = true

def holdingCoverageSound (ast : CanonicalAst) (witness : Witness) : Prop :=
  witness.holdings = ast.holdings

def verify (ast : CanonicalAst) (witness : Witness) : Bool :=
  witness.version == 3 && witness.digest == ast.digest &&
    witness.nodeKinds == ast.nodeKinds && witness.nodeFacts == ast.nodeFacts &&
    witness.edges == ast.edges && ast.edges.all (edgeWellFormed ast) &&
    witness.directFootprints == ast.directFootprints && witness.calls == ast.calls &&
    witness.footprints.all footprintWellFormed &&
    callsWellFormed witness &&
    footprintsClosed witness &&
    witness.holdings == ast.holdings &&
    witness.holdings.all (holdingWellFormed ast) &&
    witness.sharedPlaces == ast.sharedPlaces &&
    witness.sharedPlaces.all (sharedPlaceWellFormed ast)

/-- The explicit rung-4 completeness fragment. This is declarative rather than
an alias for `verify`: every conjunct names one supported evidence obligation.
It deliberately stops at checked-traversal replay; prover and certification
policy are not premises of RFC-10 evidence completeness. -/
def SupportedRFC10 (ast : CanonicalAst) (witness : Witness) : Prop :=
  (((((((((((((witness.version = 3 ∧ witness.digest = ast.digest) ∧
    witness.nodeKinds = ast.nodeKinds) ∧ witness.nodeFacts = ast.nodeFacts) ∧
    witness.edges = ast.edges) ∧ ast.edges.all (edgeWellFormed ast) = true) ∧
    witness.directFootprints = ast.directFootprints) ∧ witness.calls = ast.calls) ∧
    witness.footprints.all footprintWellFormed = true) ∧ callsWellFormed witness = true) ∧
    footprintClosureSound witness) ∧ holdingCoverageSound ast witness) ∧
    witness.holdings.all (holdingWellFormed ast) = true) ∧
    witness.sharedPlaces = ast.sharedPlaces) ∧
    witness.sharedPlaces.all (sharedPlaceWellFormed ast) = true

/-- Completeness of the executable replay checker over the explicitly declared
RFC-10 rung-4 evidence fragment. -/
theorem verify_complete {ast : CanonicalAst} {witness : Witness}
    (supported : SupportedRFC10 ast witness) : verify ast witness = true := by
  simpa [SupportedRFC10, verify, footprintClosureSound, holdingCoverageSound] using supported

/-- Exact acceptance characterization: future fragment expansions must extend
`SupportedRFC10` and preserve this equivalence. -/
theorem verify_iff_supported {ast : CanonicalAst} {witness : Witness} :
    verify ast witness = true ↔ SupportedRFC10 ast witness := by
  simp [SupportedRFC10, verify, footprintClosureSound, holdingCoverageSound]

/-- The logical rung-4 witness producer. Canonical interpretation supplies the
semantic evidence; the producer deterministically assembles it and computes
the transitive footprint closure. -/
def produce (ast : CanonicalAst) : Witness :=
  { version := 3
    digest := ast.digest
    nodeKinds := ast.nodeKinds
    nodeFacts := ast.nodeFacts
    edges := ast.edges
    directFootprints := ast.directFootprints
    calls := ast.calls
    footprints := closure ast.directFootprints ast.calls
    holdings := ast.holdings
    sharedPlaces := ast.sharedPlaces }

/-- Concrete producer refinement. Evidence is exact except that footprint
effect lists are mathematical sets: Rust's `BTreeSet` order and Lean's list
order are transport details, so each row is compared by `listSetEq`. -/
def producerRefines (ast : CanonicalAst) (witness : Witness) : Bool :=
  witness.version == (produce ast).version &&
  witness.digest == (produce ast).digest &&
  witness.nodeKinds == (produce ast).nodeKinds &&
  witness.nodeFacts == (produce ast).nodeFacts &&
  witness.edges == (produce ast).edges &&
  witness.directFootprints == (produce ast).directFootprints &&
  witness.calls == (produce ast).calls &&
  witness.holdings == (produce ast).holdings &&
  witness.sharedPlaces == (produce ast).sharedPlaces &&
  witness.footprints.length == (produce ast).footprints.length &&
  witness.footprints.all (fun row =>
    listSetEq row.effects (lookupEffects row.function (produce ast).footprints))

/-- Canonical inputs supported by the logical producer. Resource availability
is deliberately absent: it is an implementation premise, not language syntax. -/
def SupportedCanonicalAst (ast : CanonicalAst) : Prop :=
  ast.edges.all (edgeWellFormed ast) = true ∧
    callsWellFormed (produce ast) = true ∧
    footprintsClosed (produce ast) = true ∧
    ast.holdings.all (holdingWellFormed ast) = true ∧
    ast.sharedPlaces.all (sharedPlaceWellFormed ast) = true

theorem produce_supported {ast : CanonicalAst} (supported : SupportedCanonicalAst ast) :
    SupportedRFC10 ast (produce ast) := by
  rcases supported with ⟨edges, calls, footprints, holdings, places⟩
  simp [produce] at calls footprints
  simp [SupportedRFC10, produce, footprintClosureSound, holdingCoverageSound,
    footprintWellFormed, edges, calls, footprints, holdings, places]

/-- Producer completeness: every supported canonical RFC-10 input has a
deterministically produced witness accepted by the executable checker. -/
theorem produce_complete {ast : CanonicalAst} (supported : SupportedCanonicalAst ast) :
    verify ast (produce ast) = true :=
  verify_complete (produce_supported supported)

theorem structural_complete_of_verify {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) : structuralComplete ast witness := by
  simp only [verify, Bool.and_eq_true, beq_iff_eq] at accepted
  simp only [structuralComplete]
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, calls⟩
  rcases accepted with ⟨accepted, direct⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, edges⟩
  rcases accepted with ⟨accepted, facts⟩
  rcases accepted with ⟨accepted, kinds⟩
  rcases accepted with ⟨_, digest⟩
  exact ⟨digest, kinds, facts, edges, direct, calls⟩

theorem evidence_well_formed_of_verify {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) : evidenceWellFormed ast witness := by
  simp only [verify, Bool.and_eq_true, beq_iff_eq] at accepted
  simp only [evidenceWellFormed]
  rcases accepted with ⟨accepted, places⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, holdings⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨_, footprints⟩
  exact ⟨footprints, holdings, places⟩

theorem footprint_closure_sound_of_verify {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) : footprintClosureSound witness := by
  simp only [verify, Bool.and_eq_true, beq_iff_eq] at accepted
  simp only [footprintClosureSound]
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  exact accepted.2

theorem holding_coverage_sound_of_verify {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) : holdingCoverageSound ast witness := by
  simp only [verify, Bool.and_eq_true, beq_iff_eq] at accepted
  simp only [holdingCoverageSound]
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  rcases accepted with ⟨accepted, _⟩
  exact accepted.2

theorem verify_sound {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) :
    structuralComplete ast witness ∧ footprintClosureSound witness ∧
      holdingCoverageSound ast witness ∧ evidenceWellFormed ast witness :=
  ⟨structural_complete_of_verify accepted, footprint_closure_sound_of_verify accepted,
    holding_coverage_sound_of_verify accepted, evidence_well_formed_of_verify accepted⟩

inductive CheckResult where
  | accepted
  | rejected
  | resourceLimit
deriving DecidableEq, Repr

def certifying : CheckResult → Bool
  | .accepted => true
  | .rejected | .resourceLimit => false

theorem resource_limit_not_certifying : certifying .resourceLimit = false := rfl

end Thermite.CheckedTraversal
