import Std
import Thermite.LanguageCompleteness

namespace Thermite.CheckedTraversal

open Thermite.LanguageCompleteness

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

structure LockDecl where
  name : String
  guardedRegion : String
  guardedSegments : List String
  after : Option String
deriving DecidableEq, Repr

structure SemanticEventFact where
  node : Nat
  entering : Bool
  kind : String
  value : String
  segments : List String
  mode : String
  eligible : Bool
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
  lockDecls : List LockDecl
  events : List SemanticEventFact
  directFootprints : List Footprint
  calls : List CallRow
deriving DecidableEq, Repr

structure HeldScope where
  record : Nat
  loopDepth : Nat
deriving DecidableEq, Repr

structure SemanticState where
  holdings : List Holding := []
  sharedPlaces : List SharedPlace := []
  held : List HeldScope := []
  loopDepth : Nat := 0
  valid : Bool := true
deriving DecidableEq, Repr

def lookupLock (ast : CanonicalAst) (name : String) : Option LockDecl :=
  ast.lockDecls.find? (·.name == name)

def lockOfScope (records : List Holding) (scope : HeldScope) : String :=
  match records[scope.record]? with
  | some holding => holding.lock
  | none => ""

def updateHolding (records : List Holding) (index : Nat)
    (update : Holding → Holding) : List Holding :=
  records.zipIdx.map fun (holding, indexAt) =>
    if indexAt == index then update holding else holding

def addClose (records : List Holding) (scope : HeldScope) (node : Nat)
    (reason : String) (sequence : List String) : List Holding :=
  updateHolding records scope.record fun holding =>
    { holding with closeEdges := holding.closeEdges ++ [⟨node, reason, sequence⟩] }

def isPrefix : List String → List String → Bool
  | [], _ => true
  | _ :: _, [] => false
  | x :: xs, y :: ys => x == y && isPrefix xs ys

def regionsOverlap (left right : List String) : Bool :=
  isPrefix left right || isPrefix right left

def lockBefore (ast : CanonicalAst) (outer inner : String) : Bool :=
  match lookupLock ast inner with
  | some decl => decl.after == some outer
  | none => false

def orderValid (ast : CanonicalAst) (records : List Holding)
    (held : List HeldScope) (lock : String) : Bool :=
  let names := held.reverse.map (lockOfScope records)
  !names.contains lock &&
    match names.getLast? with
    | none => true
    | some outer => lockBefore ast outer lock

def closeAffected (state : SemanticState) (node : Nat) (reason : String)
    (applies : HeldScope → Bool) : SemanticState :=
  let affected := state.held.filter applies
  let sequence := affected.map (lockOfScope state.holdings)
  let records := affected.foldl
    (fun records scope => addClose records scope node reason sequence) state.holdings
  { state with holdings := records }

def deriveStep (ast : CanonicalAst) (state : SemanticState)
    (event : SemanticEventFact) : SemanticState :=
  if event.entering then
    if event.kind == "Loop" then
      { state with loopDepth := state.loopDepth + 1 }
    else if event.kind == "Holding" then
      match lookupLock ast event.value with
      | none => { state with valid := false }
      | some decl =>
        let incoming := state.held.reverse.map (lockOfScope state.holdings)
        let record := state.holdings.length
        let holding : Holding :=
          ⟨event.node, event.value, decl.guardedRegion, event.node,
            incoming, incoming, []⟩
        { state with
          holdings := state.holdings ++ [holding]
          held := ⟨record, state.loopDepth⟩ :: state.held
          valid := state.valid && orderValid ast state.holdings state.held event.value }
    else if event.kind == "Return" then
      closeAffected state event.node "Return" (fun _ => true)
    else if event.kind == "Break" then
      closeAffected state event.node "Break" (·.loopDepth == state.loopDepth)
    else if event.kind == "Continue" then
      closeAffected state event.node "Continue" (·.loopDepth == state.loopDepth)
    else if event.kind == "Place" && event.eligible then
      let heldNames := state.held.map (lockOfScope state.holdings)
      let authorizing := ast.lockDecls.filterMap fun decl =>
        if heldNames.contains decl.name && regionsOverlap decl.guardedSegments event.segments
        then some decl.name else none
      { state with sharedPlaces := state.sharedPlaces ++
          [⟨event.node, event.value, event.mode, authorizing⟩] }
    else state
  else if event.kind == "Holding" then
    match state.held with
    | [] => { state with valid := false }
    | scope :: rest =>
      let lock := lockOfScope state.holdings scope
      { state with
        holdings := addClose state.holdings scope event.node "Fallthrough" [lock]
        held := rest }
  else if event.kind == "Loop" then
    { state with loopDepth := state.loopDepth - 1 }
  else state

def deriveSemantics (ast : CanonicalAst) : SemanticState :=
  ast.events.foldl (deriveStep ast) {}

def derivedHoldings (ast : CanonicalAst) : List Holding := (deriveSemantics ast).holdings

def derivedSharedPlaces (ast : CanonicalAst) : List SharedPlace :=
  (deriveSemantics ast).sharedPlaces

def derivedAuthorityRequiredNodes (ast : CanonicalAst) : List Nat :=
  (derivedSharedPlaces ast).filterMap fun place =>
    if place.authorizingLocks.isEmpty then none else some place.node

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
    ((derivedAuthorityRequiredNodes ast).contains place.node ==
      !place.authorizingLocks.isEmpty)

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
  witness.holdings = derivedHoldings ast

def semanticDerivationSound (ast : CanonicalAst) (witness : Witness) : Prop :=
  (deriveSemantics ast).valid = true ∧
    witness.holdings = derivedHoldings ast ∧
    witness.sharedPlaces = derivedSharedPlaces ast

def verify (ast : CanonicalAst) (witness : Witness) : Bool :=
  witness.version == 3 &&
    (witness.digest == ast.digest &&
    (witness.nodeKinds == ast.nodeKinds &&
    (witness.nodeFacts == ast.nodeFacts &&
    (witness.edges == ast.edges &&
    (ast.edges.all (edgeWellFormed ast) &&
    (witness.directFootprints == ast.directFootprints &&
    (witness.calls == ast.calls &&
    (witness.footprints.all footprintWellFormed &&
    (callsWellFormed witness &&
    (footprintsClosed witness &&
    ((deriveSemantics ast).valid &&
    (witness.holdings == derivedHoldings ast &&
    (witness.holdings.all (holdingWellFormed ast) &&
    (witness.sharedPlaces == derivedSharedPlaces ast &&
      witness.sharedPlaces.all (sharedPlaceWellFormed ast)))))))))))))))

/-- The explicit rung-4 completeness fragment. This is declarative rather than
an alias for `verify`: every conjunct names one supported evidence obligation.
It deliberately stops at checked-traversal replay; prover and certification
policy are not premises of RFC-10 evidence completeness. -/
def SupportedRFC10 (ast : CanonicalAst) (witness : Witness) : Prop :=
  witness.version = 3 ∧
    witness.digest = ast.digest ∧
    witness.nodeKinds = ast.nodeKinds ∧
    witness.nodeFacts = ast.nodeFacts ∧
    witness.edges = ast.edges ∧
    ast.edges.all (edgeWellFormed ast) = true ∧
    witness.directFootprints = ast.directFootprints ∧
    witness.calls = ast.calls ∧
    witness.footprints.all footprintWellFormed = true ∧
    callsWellFormed witness = true ∧
    footprintClosureSound witness ∧
    (deriveSemantics ast).valid = true ∧
    holdingCoverageSound ast witness ∧
    witness.holdings.all (holdingWellFormed ast) = true ∧
    witness.sharedPlaces = derivedSharedPlaces ast ∧
    witness.sharedPlaces.all (sharedPlaceWellFormed ast) = true

/-- RFC-10's canonical AST projection into the language-wide neutral program. -/
def toLanguageProgram (ast : CanonicalAst) : Program :=
  { digest := ast.digest
    constructs := ast.nodeKinds
    facts := ast.nodeFacts }

/-- RFC-10 v1 is a named fragment of neutral programs. Membership means that
some checked-traversal witness realizes the program and satisfies the existing
declarative RFC-10 support predicate. -/
def rfc10FragmentV1 : Fragment :=
  { version := ⟨"rfc10-checked-traversal", 1⟩
    admits := fun program => ∃ ast witness,
      toLanguageProgram ast = program ∧ SupportedRFC10 ast witness }

/-- The existing RFC-10 predicate is an instance of neutral fragment
membership, rather than the foundation of the language-wide vocabulary. -/
theorem supportedRFC10_refines_language_fragment {ast : CanonicalAst}
    {witness : Witness} (supported : SupportedRFC10 ast witness) :
    rfc10FragmentV1.admits (toLanguageProgram ast) := by
  exact ⟨ast, witness, rfl, supported⟩

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
    holdings := derivedHoldings ast
    sharedPlaces := derivedSharedPlaces ast }

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
    (deriveSemantics ast).valid = true ∧
    (derivedHoldings ast).all (holdingWellFormed ast) = true ∧
    (derivedSharedPlaces ast).all (sharedPlaceWellFormed ast) = true

theorem produce_supported {ast : CanonicalAst} (supported : SupportedCanonicalAst ast) :
    SupportedRFC10 ast (produce ast) := by
  rcases supported with ⟨edges, calls, footprints, valid, holdings, places⟩
  simp [produce] at calls footprints
  simp [SupportedRFC10, produce, footprintClosureSound, holdingCoverageSound,
    footprintWellFormed, edges, calls, footprints, valid, holdings, places]

/-- Producer completeness: every supported canonical RFC-10 input has a
deterministically produced witness accepted by the executable checker. -/
theorem produce_complete {ast : CanonicalAst} (supported : SupportedCanonicalAst ast) :
    verify ast (produce ast) = true :=
  verify_complete (produce_supported supported)

theorem structural_complete_of_verify {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) : structuralComplete ast witness := by
  have supported := (verify_iff_supported (ast := ast) (witness := witness)).mp accepted
  rcases supported with
    ⟨_, digest, kinds, facts, edges, _, direct, calls, _, _, _, _, _, _, _, _⟩
  exact ⟨digest, kinds, facts, edges, direct, calls⟩

theorem evidence_well_formed_of_verify {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) : evidenceWellFormed ast witness := by
  have supported := (verify_iff_supported (ast := ast) (witness := witness)).mp accepted
  rcases supported with
    ⟨_, _, _, _, _, _, _, _, footprints, _, _, _, _, holdings, _, places⟩
  exact ⟨footprints, holdings, places⟩

theorem footprint_closure_sound_of_verify {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) : footprintClosureSound witness := by
  have supported := (verify_iff_supported (ast := ast) (witness := witness)).mp accepted
  exact supported.2.2.2.2.2.2.2.2.2.2.1

theorem holding_coverage_sound_of_verify {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) : holdingCoverageSound ast witness := by
  have supported := (verify_iff_supported (ast := ast) (witness := witness)).mp accepted
  exact supported.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem semantic_derivation_sound_of_verify {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) : semanticDerivationSound ast witness := by
  have supported := (verify_iff_supported (ast := ast) (witness := witness)).mp accepted
  rcases supported with
    ⟨_, _, _, _, _, _, _, _, _, _, _, valid, holdings, _, places, _⟩
  exact ⟨valid, holdings, places⟩

theorem verify_sound {ast : CanonicalAst} {witness : Witness}
    (accepted : verify ast witness = true) :
    structuralComplete ast witness ∧ footprintClosureSound witness ∧
      holdingCoverageSound ast witness ∧ evidenceWellFormed ast witness ∧
      semanticDerivationSound ast witness :=
  ⟨structural_complete_of_verify accepted, footprint_closure_sound_of_verify accepted,
    holding_coverage_sound_of_verify accepted, evidence_well_formed_of_verify accepted,
    semantic_derivation_sound_of_verify accepted⟩

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
