import Std

namespace Thermite.ResourceFlow

structure ReturningEdge where
  label : String
  live : List String
deriving DecidableEq, Repr

structure JoinFact where
  label : String
  incoming : List (List String)
  outgoing : List String
deriving DecidableEq, Repr

structure LoopFact where
  label : String
  header : List String
  backEdges : List (List String)
  exitEdges : List (List String)
deriving DecidableEq, Repr

structure ForgetFact where
  label : String
  place : Option String
  valueRegions : List String
  pricedRegions : List String
  declaredRegions : List String
deriving DecidableEq, Repr

structure FunctionFlow where
  function : String
  entryLive : List String
  returningEdges : List ReturningEdge
  joins : List JoinFact
  loops : List LoopFact
  forgets : List ForgetFact
deriving DecidableEq, Repr

structure Canonical where
  sourceDigest : String
  checkedDigest : String
  functions : List FunctionFlow
deriving DecidableEq, Repr

structure Witness where
  version : Nat
  sourceDigest : String
  checkedDigest : String
  functions : List FunctionFlow
deriving DecidableEq, Repr

def listSetEq (left right : List String) : Bool :=
  left.length == right.length && left.all right.contains && right.all left.contains

def returningEdgeSound (edge : ReturningEdge) : Bool := edge.live.isEmpty

def joinSound (join : JoinFact) : Bool :=
  !join.incoming.isEmpty && join.incoming.all (listSetEq join.outgoing)

def loopSound (loop : LoopFact) : Bool :=
  loop.backEdges.all (listSetEq loop.header) &&
    loop.exitEdges.all (listSetEq loop.header)

def forgetSound (forget : ForgetFact) : Bool :=
  !forget.valueRegions.isEmpty &&
    listSetEq forget.valueRegions forget.pricedRegions &&
    forget.pricedRegions.all forget.declaredRegions.contains

def terminalDispositionsUnique (function : FunctionFlow) : Bool :=
  let labels := function.forgets.map (·.label)
  labels.eraseDups.length == labels.length

def functionSound (function : FunctionFlow) : Bool :=
  function.returningEdges.all returningEdgeSound &&
    function.joins.all joinSound &&
    function.loops.all loopSound &&
    function.forgets.all forgetSound &&
    terminalDispositionsUnique function

def verify (canonical : Canonical) (witness : Witness) : Bool :=
  witness.version == 1 &&
    witness.sourceDigest == canonical.sourceDigest &&
    witness.checkedDigest == canonical.checkedDigest &&
    witness.functions == canonical.functions &&
    witness.functions.all functionSound

def SupportedRFC11 (canonical : Canonical) (witness : Witness) : Prop :=
  witness.version = 1 ∧
    witness.sourceDigest = canonical.sourceDigest ∧
    witness.checkedDigest = canonical.checkedDigest ∧
    witness.functions = canonical.functions ∧
    witness.functions.all functionSound = true

theorem verify_iff_supported {canonical : Canonical} {witness : Witness} :
    verify canonical witness = true ↔ SupportedRFC11 canonical witness := by
  constructor
  · intro accepted
    simp only [verify, Bool.and_eq_true, beq_iff_eq] at accepted
    exact ⟨accepted.1.1.1.1, accepted.1.1.1.2, accepted.1.1.2,
      accepted.1.2, accepted.2⟩
  · rintro ⟨version, source, checked, functions, sound⟩
    simp only [verify, Bool.and_eq_true, beq_iff_eq]
    exact ⟨⟨⟨⟨version, source⟩, checked⟩, functions⟩, sound⟩

theorem returning_paths_empty_of_verify {canonical : Canonical} {witness : Witness}
    (accepted : verify canonical witness = true) :
    witness.functions.all (fun function =>
      function.returningEdges.all returningEdgeSound) = true := by
  have supported := (verify_iff_supported (canonical := canonical) (witness := witness)).mp accepted
  rcases supported with ⟨_, _, _, _, sound⟩
  apply List.all_eq_true.mpr
  intro function member
  have function_sound := List.all_eq_true.mp sound function member
  simp only [functionSound, Bool.and_eq_true] at function_sound
  exact function_sound.1.1.1.1

theorem joins_preserve_live_set_of_verify {canonical : Canonical} {witness : Witness}
    (accepted : verify canonical witness = true) :
    witness.functions.all (fun function => function.joins.all joinSound) = true := by
  have supported := (verify_iff_supported (canonical := canonical) (witness := witness)).mp accepted
  rcases supported with ⟨_, _, _, _, sound⟩
  apply List.all_eq_true.mpr
  intro function member
  have function_sound := List.all_eq_true.mp sound function member
  simp only [functionSound, Bool.and_eq_true] at function_sound
  exact function_sound.1.1.1.2

theorem loops_preserve_live_set_of_verify {canonical : Canonical} {witness : Witness}
    (accepted : verify canonical witness = true) :
    witness.functions.all (fun function => function.loops.all loopSound) = true := by
  have supported := (verify_iff_supported (canonical := canonical) (witness := witness)).mp accepted
  rcases supported with ⟨_, _, _, _, sound⟩
  apply List.all_eq_true.mpr
  intro function member
  have function_sound := List.all_eq_true.mp sound function member
  simp only [functionSound, Bool.and_eq_true] at function_sound
  exact function_sound.1.1.2

theorem forget_footprints_exact_of_verify {canonical : Canonical} {witness : Witness}
    (accepted : verify canonical witness = true) :
    witness.functions.all (fun function => function.forgets.all forgetSound) = true := by
  have supported := (verify_iff_supported (canonical := canonical) (witness := witness)).mp accepted
  rcases supported with ⟨_, _, _, _, sound⟩
  apply List.all_eq_true.mpr
  intro function member
  have function_sound := List.all_eq_true.mp sound function member
  simp only [functionSound, Bool.and_eq_true] at function_sound
  exact function_sound.1.2

theorem terminal_dispositions_unique_of_verify {canonical : Canonical} {witness : Witness}
    (accepted : verify canonical witness = true) :
    witness.functions.all terminalDispositionsUnique = true := by
  have supported := (verify_iff_supported (canonical := canonical) (witness := witness)).mp accepted
  rcases supported with ⟨_, _, _, _, sound⟩
  apply List.all_eq_true.mpr
  intro function member
  have function_sound := List.all_eq_true.mp sound function member
  simp only [functionSound, Bool.and_eq_true] at function_sound
  exact function_sound.2

end Thermite.ResourceFlow
