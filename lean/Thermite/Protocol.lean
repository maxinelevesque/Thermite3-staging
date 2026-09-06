import Std

namespace Thermite.Protocol

structure Action where
  kind : String
  payload : List String
deriving DecidableEq, Repr

structure RoleProjection where
  role : String
  actions : List Action
deriving DecidableEq, Repr

structure Definition where
  protocol : String
  roles : List String
  projections : List RoleProjection
  repeats : Bool
  compatible : Bool
deriving DecidableEq, Repr

structure Endpoint where
  binding : String
  protocol : String
  role : String
deriving DecidableEq, Repr

structure Transition where
  endpoint : String
  step : Nat
  action : String
deriving DecidableEq, Repr

structure FunctionFlow where
  function : String
  endpoints : List Endpoint
  transitions : List Transition
  completed : List String
deriving DecidableEq, Repr

structure Canonical where
  sourceDigest : String
  checkedDigest : String
  definitions : List Definition
  functions : List FunctionFlow
deriving DecidableEq, Repr

structure Witness where
  version : Nat
  sourceDigest : String
  checkedDigest : String
  definitions : List Definition
  functions : List FunctionFlow
deriving DecidableEq, Repr

def complementary (left right : Action) : Bool :=
  left.payload == right.payload &&
    ((left.kind == "send" && right.kind == "receive") ||
     (left.kind == "receive" && right.kind == "send") ||
     (left.kind == "choose_repeat_or_end" && right.kind == "await_repeat_or_end") ||
     (left.kind == "await_repeat_or_end" && right.kind == "choose_repeat_or_end"))

def projectionsComplementary (left right : RoleProjection) : Bool :=
  left.role != right.role && left.actions.length == right.actions.length &&
    (left.actions.zip right.actions).all (fun pair => complementary pair.1 pair.2)

def definitionSound (definition : Definition) : Bool :=
  !definition.protocol.isEmpty && definition.roles.length == 2 &&
    definition.roles.eraseDups.length == 2 && definition.projections.length == 2 &&
    definition.compatible &&
    match definition.projections with
    | [left, right] =>
        definition.roles.contains left.role && definition.roles.contains right.role &&
          projectionsComplementary left right
    | _ => false

def endpointSound (definitions : List Definition) (endpoint : Endpoint) : Bool :=
  !endpoint.binding.isEmpty &&
    match definitions.find? (fun definition => definition.protocol == endpoint.protocol) with
    | some definition => definition.roles.contains endpoint.role
    | none => false

def transitionSound (endpoints : List Endpoint) (transition : Transition) : Bool :=
  endpoints.any (fun endpoint => endpoint.binding == transition.endpoint) &&
    ["send", "receive", "repeat-choice", "end-choice", "repeat-receive", "end-receive"].contains transition.action

def actionMatches (action : Action) (transition : Transition) : Bool :=
  (action.kind == "send" && transition.action == "send") ||
    (action.kind == "receive" && transition.action == "receive") ||
    (action.kind == "choose_repeat_or_end" &&
      (transition.action == "repeat-choice" || transition.action == "end-choice")) ||
    (action.kind == "await_repeat_or_end" &&
      (transition.action == "repeat-receive" || transition.action == "end-receive"))

def roundSound (actions : List Action) (transitions : List Transition) : Bool :=
  let round := transitions.take actions.length
  round.length == actions.length &&
    ((List.range actions.length).zip (actions.zip round)).all (fun row =>
      row.2.2.step == row.1 && actionMatches row.2.1 row.2.2)

def consumesProjectionFuel : Nat → List Action → List Transition → Bool
  | 0, _, _ => false
  | fuel + 1, actions, transitions =>
      if actions.isEmpty || !roundSound actions transitions then false
      else
        let round := transitions.take actions.length
        let rest := transitions.drop actions.length
        match round.getLast? with
        | none => false
        | some last =>
            if last.action == "repeat-choice" || last.action == "repeat-receive" then
              !rest.isEmpty && consumesProjectionFuel fuel actions rest
            else
              rest.isEmpty

def consumesProjection (actions : List Action) (transitions : List Transition) : Bool :=
  consumesProjectionFuel (transitions.length + 1) actions transitions

def findProjection (definitions : List Definition) (endpoint : Endpoint) : Option RoleProjection :=
  match definitions.find? (fun definition => definition.protocol == endpoint.protocol) with
  | some definition => definition.projections.find? (fun projection => projection.role == endpoint.role)
  | none => none

def endpointFlowSound
    (definitions : List Definition) (flow : FunctionFlow) (endpoint : Endpoint) : Bool :=
  flow.completed.contains endpoint.binding &&
    let transitions := flow.transitions.filter (fun transition => transition.endpoint == endpoint.binding)
    match findProjection definitions endpoint with
    | some projection => consumesProjection projection.actions transitions
    | none => false

def functionSound (definitions : List Definition) (flow : FunctionFlow) : Bool :=
  !flow.function.isEmpty && !flow.endpoints.isEmpty &&
    flow.endpoints.all (endpointSound definitions) &&
    let bindings := flow.endpoints.map (·.binding)
    bindings.eraseDups.length == bindings.length &&
      flow.completed.eraseDups.length == flow.completed.length &&
      flow.completed.length == bindings.length &&
      flow.completed.all bindings.contains && bindings.all flow.completed.contains &&
      flow.transitions.all (transitionSound flow.endpoints) &&
      flow.endpoints.all (endpointFlowSound definitions flow)

def verify (canonical : Canonical) (witness : Witness) : Bool :=
  witness.version == 1 &&
    witness.sourceDigest == canonical.sourceDigest &&
    witness.checkedDigest == canonical.checkedDigest &&
    witness.definitions == canonical.definitions &&
    witness.functions == canonical.functions &&
    !witness.definitions.isEmpty && witness.definitions.all definitionSound &&
    witness.functions.all (functionSound witness.definitions)

def SupportedRFC13 (canonical : Canonical) (witness : Witness) : Prop :=
  verify canonical witness = true

theorem verify_iff_supported {canonical : Canonical} {witness : Witness} :
    verify canonical witness = true ↔ SupportedRFC13 canonical witness := by
  rfl

theorem projections_compatible_of_verify {canonical : Canonical} {witness : Witness}
    (accepted : verify canonical witness = true) :
    witness.definitions.all definitionSound = true := by
  simp only [verify, Bool.and_eq_true] at accepted
  exact accepted.1.2

theorem endpoints_complete_of_verify {canonical : Canonical} {witness : Witness}
    (accepted : verify canonical witness = true) :
    witness.functions.all (functionSound witness.definitions) = true := by
  simp only [verify, Bool.and_eq_true] at accepted
  exact accepted.2

end Thermite.Protocol
