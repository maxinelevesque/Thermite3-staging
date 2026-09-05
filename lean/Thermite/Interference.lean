import Std

namespace Thermite.Interference

structure Atom where
  place : String
  kind : String
deriving DecidableEq, Repr

structure FunctionContract where
  function : String
  asks : List Atom
  promises : List Atom
deriving DecidableEq, Repr

structure Obligation where
  composition : String
  guarantor : String
  relying : String
deriving DecidableEq, Repr

structure Requirement where
  composition : String
  leftRoot : String
  rightRoot : String
  leftPriority : Option Nat
  rightPriority : Option Nat
  overlaps : List String
deriving DecidableEq, Repr

structure Canonical where
  sourceDigest : String
  checkedDigest : String
  functions : List FunctionContract
  requirements : List Requirement
  obligations : List Obligation
deriving DecidableEq, Repr

structure Witness where
  version : Nat
  sourceDigest : String
  checkedDigest : String
  functions : List FunctionContract
  requirements : List Requirement
  obligations : List Obligation
deriving DecidableEq, Repr

def validKind (kind : String) : Bool :=
  kind == "ordered" || kind == "bit_set" || kind == "boolean"

def atomListSound (atoms : List Atom) : Bool :=
  atoms.all (fun atom => !atom.place.isEmpty && validKind atom.kind) &&
    atoms.eraseDups.length == atoms.length

def functionSound (contract : FunctionContract) : Bool :=
  !contract.function.isEmpty && atomListSound contract.asks && atomListSound contract.promises

def findFunction (functions : List FunctionContract) (name : String) : Option FunctionContract :=
  functions.find? (fun function => function.function == name)

def relationImplies (stronger weaker : List Atom) : Bool :=
  weaker.all stronger.contains

def obligationSound (functions : List FunctionContract) (obligation : Obligation) : Bool :=
  !obligation.composition.isEmpty && obligation.guarantor != obligation.relying &&
    match findFunction functions obligation.guarantor, findFunction functions obligation.relying with
    | some guarantor, some relying => relationImplies guarantor.promises relying.asks
    | _, _ => false

def expectedObligations (requirement : Requirement) : List Obligation :=
  match requirement.leftPriority, requirement.rightPriority with
  | none, none =>
      [⟨requirement.composition, requirement.leftRoot, requirement.rightRoot⟩,
       ⟨requirement.composition, requirement.rightRoot, requirement.leftRoot⟩]
  | some left, some right =>
      if left > right then
        [⟨requirement.composition, requirement.leftRoot, requirement.rightRoot⟩]
      else if right > left then
        [⟨requirement.composition, requirement.rightRoot, requirement.leftRoot⟩]
      else []
  | _, _ => []

def expectedGraph (requirements : List Requirement) : List Obligation :=
  requirements.flatMap expectedObligations

def relationCovers (atoms : List Atom) (place : String) : Bool :=
  atoms.any (fun atom => atom.place == place)

def obligationCovers
    (functions : List FunctionContract) (overlaps : List String) (obligation : Obligation) : Bool :=
  match findFunction functions obligation.guarantor, findFunction functions obligation.relying with
  | some guarantor, some relying =>
      overlaps.all (fun place =>
        relationCovers guarantor.promises place && relationCovers relying.asks place)
  | _, _ => false

def requirementSound (functions : List FunctionContract) (requirement : Requirement) : Bool :=
  !requirement.composition.isEmpty && requirement.leftRoot != requirement.rightRoot &&
    !requirement.overlaps.isEmpty &&
    requirement.overlaps.eraseDups.length == requirement.overlaps.length &&
    let expected := expectedObligations requirement
    !expected.isEmpty && expected.all (fun obligation =>
      obligationSound functions obligation && obligationCovers functions requirement.overlaps obligation)

def uniqueFunctions (functions : List FunctionContract) : Bool :=
  let names := functions.map (·.function)
  names.eraseDups.length == names.length

def uniqueObligations (obligations : List Obligation) : Bool :=
  obligations.eraseDups.length == obligations.length

def verify (canonical : Canonical) (witness : Witness) : Bool :=
  witness.version == 1 &&
    witness.sourceDigest == canonical.sourceDigest &&
    witness.checkedDigest == canonical.checkedDigest &&
    witness.functions == canonical.functions &&
    witness.requirements == canonical.requirements &&
    witness.obligations == canonical.obligations &&
    uniqueFunctions witness.functions &&
    witness.functions.all functionSound &&
    witness.requirements.eraseDups.length == witness.requirements.length &&
    witness.requirements.all (requirementSound witness.functions) &&
    expectedGraph witness.requirements == witness.obligations &&
    uniqueObligations witness.obligations &&
    witness.obligations.all (obligationSound witness.functions)

def SupportedRFC12 (canonical : Canonical) (witness : Witness) : Prop :=
  verify canonical witness = true

theorem verify_iff_supported {canonical : Canonical} {witness : Witness} :
    verify canonical witness = true ↔ SupportedRFC12 canonical witness := by
  rfl

theorem obligations_compatible_of_verify {canonical : Canonical} {witness : Witness}
    (accepted : verify canonical witness = true) :
    witness.obligations.all (obligationSound witness.functions) = true := by
  simp only [verify, Bool.and_eq_true] at accepted
  exact accepted.2

end Thermite.Interference
