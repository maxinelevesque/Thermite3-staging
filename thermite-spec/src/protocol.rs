//! RFC-13 binary protocol declaration and endpoint-flow checking.

use std::collections::{BTreeMap, BTreeSet};

use thermite_syntax::{Block, Expr, FnItem, IndexArg, Item, PrimType, Program, Span, Stmt, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolErrorKind {
    InvalidDeclaration,
    UnknownEndpoint,
    SharedEndpoint,
    WrongTurn,
    PayloadMismatch,
    DuplicateEndpoint,
    UnfinishedEndpoint,
    BranchMismatch,
    InvalidRepeatExit,
    UnsupportedControlFlow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    pub kind: ProtocolErrorKind,
    pub function: Option<String>,
    pub endpoint: Option<String>,
    pub detail: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolTransition {
    pub endpoint: String,
    pub step: usize,
    pub action: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProtocolFunctionFlow {
    pub endpoints: BTreeMap<String, ProtocolEndpointBinding>,
    pub transitions: Vec<ProtocolTransition>,
    pub completed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolEndpointBinding {
    pub protocol: String,
    pub role: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProtocolReport {
    pub definitions: BTreeMap<String, ProtocolDefinition>,
    pub functions: BTreeMap<String, ProtocolFunctionFlow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolDefinition {
    pub roles: Vec<String>,
    pub projections: BTreeMap<String, Vec<ProtocolAction>>,
    pub repeat: bool,
    pub compatible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolAction {
    Send(Vec<Type>),
    Receive(Vec<Type>),
    ChooseRepeatOrEnd,
    AwaitRepeatOrEnd,
}

#[derive(Debug, Clone)]
struct Protocol {
    roles: Vec<String>,
    senders: Vec<String>,
    payload_types: Vec<Vec<Type>>,
    repeat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Endpoint {
    protocol: String,
    role: String,
    step: usize,
    awaiting_choice: bool,
    complete: bool,
}

type State = BTreeMap<String, Endpoint>;

/// Validate RFC-13's binary global declarations and each function-local endpoint
/// projection. The check is deliberately syntax-directed: protocol actions are
/// accepted only as direct method calls on a named endpoint binding.
pub fn check_protocols(program: &Program) -> Result<ProtocolReport, Vec<ProtocolError>> {
    let mut errors = Vec::new();
    let mut protocols = BTreeMap::new();

    for item in &program.items {
        let Item::Protocol(item) = item else { continue };
        let mut roles = Vec::new();
        let mut seen_roles = BTreeSet::new();
        for turn in &item.turns {
            if !turn.role.chars().next().is_some_and(char::is_uppercase) {
                errors.push(error(
                    ProtocolErrorKind::InvalidDeclaration,
                    None,
                    None,
                    format!(
                        "protocol `{}` role `{}` must be uppercase-initial",
                        item.name, turn.role
                    ),
                    turn.span,
                ));
            }
            if seen_roles.insert(turn.role.clone()) {
                roles.push(turn.role.clone());
            }
            let mut fields = BTreeSet::new();
            for field in &turn.fields {
                if !fields.insert(field.name.clone()) {
                    errors.push(error(
                        ProtocolErrorKind::InvalidDeclaration,
                        None,
                        None,
                        format!(
                            "protocol `{}` turn `{}` repeats payload field `{}`",
                            item.name, turn.role, field.name
                        ),
                        turn.span,
                    ));
                }
                if contains_endpoint(&field.ty) {
                    errors.push(error(
                        ProtocolErrorKind::InvalidDeclaration,
                        None,
                        None,
                        format!(
                            "protocol `{}` payloads may not contain protocol endpoints",
                            item.name
                        ),
                        turn.span,
                    ));
                }
            }
        }
        if item.turns.is_empty() || roles.len() != 2 {
            errors.push(error(
                ProtocolErrorKind::InvalidDeclaration,
                None,
                None,
                format!(
                    "protocol `{}` must contain turns from exactly two roles",
                    item.name
                ),
                item.span,
            ));
        }
        protocols.insert(
            item.name.clone(),
            Protocol {
                roles,
                senders: item.turns.iter().map(|turn| turn.role.clone()).collect(),
                payload_types: item
                    .turns
                    .iter()
                    .map(|turn| turn.fields.iter().map(|field| field.ty.clone()).collect())
                    .collect(),
                repeat: item.repeat,
            },
        );
    }

    for item in &program.items {
        if let Item::SharedDecl(shared) = item {
            if contains_endpoint(&shared.ty) {
                errors.push(error(
                    ProtocolErrorKind::SharedEndpoint,
                    None,
                    Some(shared.name.clone()),
                    format!(
                        "shared state `{}` may not contain a protocol endpoint",
                        shared.name
                    ),
                    shared.span,
                ));
            }
        }
    }

    let mut report = ProtocolReport {
        definitions: protocols
            .iter()
            .map(|(name, protocol)| (name.clone(), protocol_definition(protocol)))
            .collect(),
        functions: BTreeMap::new(),
    };
    for item in &program.items {
        if let Item::Fn(function) = item {
            check_function(function, &protocols, &mut report, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(report)
    } else {
        Err(errors)
    }
}

fn protocol_definition(protocol: &Protocol) -> ProtocolDefinition {
    let mut projections = BTreeMap::new();
    for role in &protocol.roles {
        let mut actions = protocol
            .senders
            .iter()
            .zip(&protocol.payload_types)
            .map(|(sender, payload)| {
                if sender == role {
                    ProtocolAction::Send(payload.clone())
                } else {
                    ProtocolAction::Receive(payload.clone())
                }
            })
            .collect::<Vec<_>>();
        if protocol.repeat {
            if protocol.senders.first() == Some(role) {
                actions.push(ProtocolAction::ChooseRepeatOrEnd);
            } else {
                actions.push(ProtocolAction::AwaitRepeatOrEnd);
            }
        }
        projections.insert(role.clone(), actions);
    }
    let compatible = protocol.roles.len() == 2
        && protocol
            .senders
            .iter()
            .all(|sender| protocol.roles.contains(sender));
    ProtocolDefinition {
        roles: protocol.roles.clone(),
        projections,
        repeat: protocol.repeat,
        compatible,
    }
}

fn check_function(
    function: &FnItem,
    protocols: &BTreeMap<String, Protocol>,
    report: &mut ProtocolReport,
    errors: &mut Vec<ProtocolError>,
) {
    let mut value_types = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    if let Some(body) = &function.body {
        collect_declared_types(body, &mut value_types);
    }
    let mut state = State::new();
    for param in &function.params {
        let Type::ProtocolEndpoint { protocol, role } = &param.ty else {
            continue;
        };
        match resolve_endpoint(protocols, protocol, role) {
            Ok(()) => {
                state.insert(
                    param.name.clone(),
                    Endpoint {
                        protocol: protocol.clone(),
                        role: role.clone(),
                        step: 0,
                        awaiting_choice: false,
                        complete: false,
                    },
                );
            }
            Err(detail) => errors.push(error(
                ProtocolErrorKind::UnknownEndpoint,
                Some(function.name.clone()),
                Some(param.name.clone()),
                detail,
                function.span,
            )),
        }
    }
    if state.is_empty() {
        return;
    }
    if !matches!(&function.contract.effects, thermite_syntax::EffectRow::Set(effects) if effects.contains(&thermite_syntax::Effect::Blocks))
    {
        errors.push(error(
            ProtocolErrorKind::WrongTurn,
            Some(function.name.clone()),
            None,
            format!(
                "function `{}` owns a protocol endpoint but does not declare `fx blocks`",
                function.name
            ),
            function.span,
        ));
    }
    let mut flow = ProtocolFunctionFlow {
        endpoints: state
            .iter()
            .map(|(name, endpoint)| {
                (
                    name.clone(),
                    ProtocolEndpointBinding {
                        protocol: endpoint.protocol.clone(),
                        role: endpoint.role.clone(),
                    },
                )
            })
            .collect(),
        ..ProtocolFunctionFlow::default()
    };
    if let Some(body) = &function.body {
        let (state, returned) = check_block(
            body,
            state,
            function,
            protocols,
            &value_types,
            &mut flow,
            errors,
        );
        if !returned {
            check_completion(&state, function, errors);
        }
        flow.completed = state
            .iter()
            .filter(|(_, ep)| ep.complete)
            .map(|(name, _)| name.clone())
            .collect();
    } else {
        check_completion(&state, function, errors);
    }
    report.functions.insert(function.name.clone(), flow);
}

fn check_block(
    block: &Block,
    mut state: State,
    function: &FnItem,
    protocols: &BTreeMap<String, Protocol>,
    value_types: &BTreeMap<String, Type>,
    flow: &mut ProtocolFunctionFlow,
    errors: &mut Vec<ProtocolError>,
) -> (State, bool) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Expr(expr) => check_expr(
                expr,
                &mut state,
                function,
                protocols,
                value_types,
                flow,
                errors,
            ),
            Stmt::Let { name, ty, init, .. } => {
                if let Some((binding, expected)) = receive_payload_binding(init, &state, protocols)
                {
                    match ty {
                        Some(found) if found == &expected => {}
                        Some(found) => errors.push(error(
                            ProtocolErrorKind::PayloadMismatch,
                            Some(function.name.clone()),
                            Some(binding),
                            format!(
                                "received protocol payload expects local type {expected:?}, found {found:?}"
                            ),
                            function.span,
                        )),
                        None => errors.push(error(
                            ProtocolErrorKind::PayloadMismatch,
                            Some(function.name.clone()),
                            Some(binding),
                            "a `receive_payload()` binding requires an explicit payload type"
                                .into(),
                            function.span,
                        )),
                    }
                    check_expr(
                        init,
                        &mut state,
                        function,
                        protocols,
                        value_types,
                        flow,
                        errors,
                    );
                    continue;
                }
                reject_endpoint_value_use(init, &state, function, errors);
                if matches!(ty, Some(Type::ProtocolEndpoint { .. })) {
                    errors.push(error(
                        ProtocolErrorKind::DuplicateEndpoint,
                        Some(function.name.clone()),
                        Some(name.clone()),
                        "protocol endpoints may enter a function only as owned parameters; a local binding cannot mint or alias one".into(),
                        function.span,
                    ));
                }
            }
            Stmt::Return(value) => {
                if let Some(value) = value {
                    reject_endpoint_value_use(value, &state, function, errors);
                }
                check_completion(&state, function, errors);
                return (state, true);
            }
            Stmt::If { cond, then, else_ } => {
                reject_endpoint_value_use(cond, &state, function, errors);
                if block_has_protocol_action(then, &state)
                    || else_
                        .as_ref()
                        .is_some_and(|block| block_has_protocol_action(block, &state))
                {
                    errors.push(error(
                        ProtocolErrorKind::UnsupportedControlFlow,
                        Some(function.name.clone()),
                        None,
                        "protocol actions in conditional branches are not supported by the RFC-13 v1 path witness; move the branch outside the session flow".into(),
                        function.span,
                    ));
                    continue;
                }
                let (then_state, then_returned) = check_block(
                    then,
                    state.clone(),
                    function,
                    protocols,
                    value_types,
                    flow,
                    errors,
                );
                let (else_state, else_returned) = if let Some(block) = else_ {
                    check_block(
                        block,
                        state.clone(),
                        function,
                        protocols,
                        value_types,
                        flow,
                        errors,
                    )
                } else {
                    (state.clone(), false)
                };
                match (then_returned, else_returned) {
                    (false, false) if then_state == else_state => state = then_state,
                    (true, false) => state = else_state,
                    (false, true) => state = then_state,
                    (true, true) => return (state, true),
                    _ => errors.push(error(
                        ProtocolErrorKind::BranchMismatch,
                        Some(function.name.clone()),
                        None,
                        "protocol endpoint states differ across `if` branches".into(),
                        function.span,
                    )),
                }
            }
            Stmt::Holding { body, .. } => {
                let (next, returned) = check_block(
                    body,
                    state.clone(),
                    function,
                    protocols,
                    value_types,
                    flow,
                    errors,
                );
                state = next;
                if returned {
                    return (state, true);
                }
            }
            Stmt::Loop(loop_) => {
                if block_has_protocol_action(&loop_.body, &state) {
                    errors.push(error(
                        ProtocolErrorKind::UnsupportedControlFlow,
                        Some(function.name.clone()),
                        None,
                        "protocol actions in general loops are not supported by the RFC-13 v1 path witness; use the protocol's explicit `repeat | end` actions".into(),
                        loop_.span,
                    ));
                    continue;
                }
                let (next, returned) = check_block(
                    &loop_.body,
                    state.clone(),
                    function,
                    protocols,
                    value_types,
                    flow,
                    errors,
                );
                if !returned && next != state {
                    errors.push(error(
                        ProtocolErrorKind::BranchMismatch, Some(function.name.clone()), None,
                        "a loop must restore every protocol endpoint to its header state or complete it on all exits".into(), loop_.span,
                    ));
                }
            }
            Stmt::Assign { target, value } => {
                reject_endpoint_value_use(target, &state, function, errors);
                reject_endpoint_value_use(value, &state, function, errors);
            }
            Stmt::Forget { value, .. } => {
                reject_endpoint_value_use(value, &state, function, errors)
            }
            Stmt::Break | Stmt::Continue => {}
        }
    }
    if let Some(tail) = &block.tail {
        reject_endpoint_value_use(tail, &state, function, errors);
    }
    (state, false)
}

fn block_has_protocol_action(block: &Block, state: &State) -> bool {
    block.stmts.iter().any(|stmt| match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) | Stmt::Forget { value: init, .. } => {
            expr_has_protocol_action(init, state)
        }
        Stmt::Assign { target, value } => {
            expr_has_protocol_action(target, state) || expr_has_protocol_action(value, state)
        }
        Stmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_has_protocol_action(value, state)),
        Stmt::If { cond, then, else_ } => {
            expr_has_protocol_action(cond, state)
                || block_has_protocol_action(then, state)
                || else_
                    .as_ref()
                    .is_some_and(|else_| block_has_protocol_action(else_, state))
        }
        Stmt::Loop(loop_) => block_has_protocol_action(&loop_.body, state),
        Stmt::Holding { body, .. } => block_has_protocol_action(body, state),
        Stmt::Break | Stmt::Continue => false,
    }) || block
        .tail
        .as_ref()
        .is_some_and(|tail| expr_has_protocol_action(tail, state))
}

fn expr_has_protocol_action(expr: &Expr, state: &State) -> bool {
    let visit = |expr: &Expr| expr_has_protocol_action(expr, state);
    match expr {
        Expr::MethodCall { receiver, args, .. } => {
            matches!(receiver.as_ref(), Expr::Path(path) if path.len() == 1 && state.contains_key(&path[0]))
                || visit(receiver)
                || args.iter().any(visit)
        }
        Expr::Call { callee, args } => visit(callee) || args.iter().any(visit),
        Expr::Field { receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Closure { body: receiver, .. }
        | Expr::Is {
            scrutinee: receiver,
            ..
        } => visit(receiver),
        Expr::Match { scrutinee, arms } => {
            visit(scrutinee)
                || arms
                    .iter()
                    .any(|arm| arm.guard.as_ref().is_some_and(visit) || visit(&arm.body))
        }
        Expr::If { cond, then, else_ } => {
            visit(cond)
                || block_has_protocol_action(then, state)
                || block_has_protocol_action(else_, state)
        }
        Expr::Binary { lhs, rhs, .. } => visit(lhs) || visit(rhs),
        Expr::Index { base, index } => {
            visit(base)
                || match index {
                    IndexArg::Single(expr)
                    | IndexArg::RangeTo(expr)
                    | IndexArg::RangeFrom(expr) => visit(expr),
                    IndexArg::Range(lo, hi) => visit(lo) || visit(hi),
                }
        }
        Expr::StructLit { fields, .. } => fields.iter().any(|(_, value)| visit(value)),
        Expr::Tuple(items) => items.iter().any(visit),
        Expr::Quantifier { domain, body, .. } => visit(domain) || visit(body),
        Expr::Path(_) | Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::StrLit(_) => false,
    }
}

fn check_expr(
    expr: &Expr,
    state: &mut State,
    function: &FnItem,
    protocols: &BTreeMap<String, Protocol>,
    value_types: &BTreeMap<String, Type>,
    flow: &mut ProtocolFunctionFlow,
    errors: &mut Vec<ProtocolError>,
) {
    let Expr::MethodCall {
        receiver,
        name,
        args,
    } = expr
    else {
        reject_endpoint_value_use(expr, state, function, errors);
        return;
    };
    let Expr::Path(path) = receiver.as_ref() else {
        reject_endpoint_value_use(expr, state, function, errors);
        return;
    };
    let [binding] = path.as_slice() else {
        reject_endpoint_value_use(expr, state, function, errors);
        return;
    };
    let Some(_) = state.get(binding) else {
        reject_endpoint_value_use(expr, state, function, errors);
        return;
    };
    for arg in args {
        reject_endpoint_value_use(arg, state, function, errors);
    }
    let Some(endpoint) = state.get_mut(binding) else {
        return;
    };
    let Some(protocol) = protocols.get(&endpoint.protocol) else {
        return;
    };
    if endpoint.complete {
        errors.push(error(
            ProtocolErrorKind::WrongTurn,
            Some(function.name.clone()),
            Some(binding.clone()),
            format!("endpoint `{binding}` is already complete"),
            function.span,
        ));
        return;
    }
    let action = name.as_str();
    if endpoint.awaiting_choice {
        let chooser = protocol.senders.first() == Some(&endpoint.role);
        let valid_action = if chooser {
            matches!(action, "repeat" | "end")
        } else {
            matches!(action, "receive_repeat" | "receive_end")
        };
        if !protocol.repeat || !valid_action || !args.is_empty() {
            errors.push(error(
                ProtocolErrorKind::InvalidRepeatExit,
                Some(function.name.clone()),
                Some(binding.clone()),
                format!(
                    "endpoint `{binding}` must {} after the final turn",
                    if chooser {
                        "choose `repeat()` or `end()`"
                    } else {
                        "receive the chooser's discriminant with `receive_repeat()` or `receive_end()`"
                    }
                ),
                function.span,
            ));
            return;
        }
        flow.transitions.push(ProtocolTransition {
            endpoint: binding.clone(),
            step: endpoint.step,
            action: match action {
                "repeat" => "repeat-choice",
                "end" => "end-choice",
                "receive_repeat" => "repeat-receive",
                _ => "end-receive",
            },
        });
        if matches!(action, "repeat" | "receive_repeat") {
            endpoint.step = 0;
            endpoint.awaiting_choice = false;
        } else {
            endpoint.complete = true;
        }
        return;
    }
    let Some(sender) = protocol.senders.get(endpoint.step) else {
        errors.push(error(
            ProtocolErrorKind::WrongTurn,
            Some(function.name.clone()),
            Some(binding.clone()),
            format!(
                "endpoint `{binding}` has no protocol action at step {}",
                endpoint.step
            ),
            function.span,
        ));
        return;
    };
    let expected = if sender == &endpoint.role {
        "send"
    } else {
        "receive"
    };
    let action_matches =
        action == expected || (expected == "receive" && action == "receive_payload");
    if !action_matches {
        errors.push(error(
            ProtocolErrorKind::WrongTurn,
            Some(function.name.clone()),
            Some(binding.clone()),
            format!(
                "endpoint `{binding}` at step {} must `{expected}`, not `{action}`",
                endpoint.step
            ),
            function.span,
        ));
        return;
    }
    let expected_types: &[Type] = if expected == "send" {
        &protocol.payload_types[endpoint.step]
    } else {
        &[]
    };
    if args.len() != expected_types.len() {
        errors.push(error(ProtocolErrorKind::PayloadMismatch, Some(function.name.clone()), Some(binding.clone()), format!("endpoint `{binding}` `{expected}` at step {} expects {} payload value(s), found {}", endpoint.step, expected_types.len(), args.len()), function.span));
        return;
    }
    for (index, (arg, expected_ty)) in args.iter().zip(expected_types).enumerate() {
        let Some(found_ty) = infer_expr_type(arg, value_types) else {
            errors.push(error(
                ProtocolErrorKind::PayloadMismatch,
                Some(function.name.clone()),
                Some(binding.clone()),
                format!(
                    "endpoint `{binding}` payload {index} at step {} has a type the RFC-13 v1 checker cannot infer; bind it to an explicitly typed local before sending",
                    endpoint.step
                ),
                function.span,
            ));
            return;
        };
        if !type_compatible(expected_ty, &found_ty, matches!(arg, Expr::IntLit { .. })) {
            errors.push(error(
                ProtocolErrorKind::PayloadMismatch,
                Some(function.name.clone()),
                Some(binding.clone()),
                format!(
                    "endpoint `{binding}` payload {index} at step {} expects {expected_ty:?}, found {found_ty:?}",
                    endpoint.step
                ),
                function.span,
            ));
            return;
        }
    }
    flow.transitions.push(ProtocolTransition {
        endpoint: binding.clone(),
        step: endpoint.step,
        action: if expected == "send" {
            "send"
        } else {
            "receive"
        },
    });
    endpoint.step += 1;
    if endpoint.step == protocol.senders.len() {
        if protocol.repeat {
            endpoint.awaiting_choice = true;
        } else {
            endpoint.complete = true;
        }
    }
}

fn receive_payload_binding(
    expr: &Expr,
    state: &State,
    protocols: &BTreeMap<String, Protocol>,
) -> Option<(String, Type)> {
    let Expr::MethodCall {
        receiver,
        name,
        args,
    } = expr
    else {
        return None;
    };
    if name != "receive_payload" || !args.is_empty() {
        return None;
    }
    let Expr::Path(path) = receiver.as_ref() else {
        return None;
    };
    let [binding] = path.as_slice() else {
        return None;
    };
    let endpoint = state.get(binding)?;
    let protocol = protocols.get(&endpoint.protocol)?;
    let sender = protocol.senders.get(endpoint.step)?;
    if sender == &endpoint.role {
        return None;
    }
    let payload = protocol.payload_types.get(endpoint.step)?;
    let ty = match payload.as_slice() {
        [] => Type::Unit,
        [single] => single.clone(),
        many => Type::Tuple(many.to_vec()),
    };
    Some((binding.clone(), ty))
}

fn infer_expr_type(expr: &Expr, value_types: &BTreeMap<String, Type>) -> Option<Type> {
    match expr {
        Expr::BoolLit(_) => Some(Type::Prim(PrimType::Bool)),
        Expr::IntLit { .. } => Some(Type::Prim(PrimType::U64)),
        Expr::Cast { ty, .. } => Some(ty.clone()),
        Expr::Path(path) if path.len() == 1 => value_types.get(&path[0]).cloned(),
        Expr::Tuple(items) => items
            .iter()
            .map(|item| infer_expr_type(item, value_types))
            .collect::<Option<Vec<_>>>()
            .map(Type::Tuple),
        _ => None,
    }
}

fn collect_declared_types(block: &Block, value_types: &mut BTreeMap<String, Type>) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Let {
                name, ty: Some(ty), ..
            } => {
                value_types.insert(name.clone(), ty.clone());
            }
            Stmt::If { then, else_, .. } => {
                collect_declared_types(then, value_types);
                if let Some(else_) = else_ {
                    collect_declared_types(else_, value_types);
                }
            }
            Stmt::Loop(loop_) => collect_declared_types(&loop_.body, value_types),
            Stmt::Holding { body, .. } => collect_declared_types(body, value_types),
            _ => {}
        }
    }
}

fn type_compatible(expected: &Type, found: &Type, unsuffixed_integer: bool) -> bool {
    expected == found
        || (unsuffixed_integer
            && matches!(
                expected,
                Type::Prim(
                    PrimType::U8 | PrimType::U16 | PrimType::U32 | PrimType::U64 | PrimType::Usize
                )
            )
            && matches!(found, Type::Prim(PrimType::U64)))
}

fn reject_endpoint_value_use(
    expr: &Expr,
    state: &State,
    function: &FnItem,
    errors: &mut Vec<ProtocolError>,
) {
    let mut uses = BTreeSet::new();
    collect_endpoint_uses(expr, state, &mut uses);
    for name in uses {
        errors.push(error(
            ProtocolErrorKind::DuplicateEndpoint,
            Some(function.name.clone()),
            Some(name.clone()),
            format!("endpoint `{name}` may only be consumed through its protocol actions"),
            function.span,
        ));
    }
}

fn collect_endpoint_uses(expr: &Expr, state: &State, uses: &mut BTreeSet<String>) {
    match expr {
        Expr::Path(path) => {
            if let [name] = path.as_slice() {
                if state.contains_key(name) {
                    uses.insert(name.clone());
                }
            }
        }
        Expr::Call { callee, args } => {
            collect_endpoint_uses(callee, state, uses);
            for arg in args {
                collect_endpoint_uses(arg, state, uses);
            }
        }
        Expr::MethodCall { receiver, args, .. } => {
            collect_endpoint_uses(receiver, state, uses);
            for arg in args {
                collect_endpoint_uses(arg, state, uses);
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. }
        | Expr::Unary { expr: receiver, .. } => collect_endpoint_uses(receiver, state, uses),
        Expr::Closure { body, .. } => collect_endpoint_uses(body, state, uses),
        Expr::Match { scrutinee, arms } => {
            collect_endpoint_uses(scrutinee, state, uses);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_endpoint_uses(guard, state, uses);
                }
                collect_endpoint_uses(&arm.body, state, uses);
            }
        }
        Expr::If { cond, then, else_ } => {
            collect_endpoint_uses(cond, state, uses);
            collect_block_endpoint_uses(then, state, uses);
            collect_block_endpoint_uses(else_, state, uses);
        }
        Expr::Binary { lhs, rhs, .. } => {
            collect_endpoint_uses(lhs, state, uses);
            collect_endpoint_uses(rhs, state, uses);
        }
        Expr::Index { base, index } => {
            collect_endpoint_uses(base, state, uses);
            match index {
                thermite_syntax::IndexArg::Single(expr)
                | thermite_syntax::IndexArg::RangeTo(expr)
                | thermite_syntax::IndexArg::RangeFrom(expr) => {
                    collect_endpoint_uses(expr, state, uses)
                }
                thermite_syntax::IndexArg::Range(lo, hi) => {
                    collect_endpoint_uses(lo, state, uses);
                    collect_endpoint_uses(hi, state, uses);
                }
            }
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                collect_endpoint_uses(value, state, uses);
            }
        }
        Expr::Is { scrutinee, .. } => collect_endpoint_uses(scrutinee, state, uses),
        Expr::Tuple(items) => {
            for item in items {
                collect_endpoint_uses(item, state, uses);
            }
        }
        Expr::Quantifier { domain, body, .. } => {
            collect_endpoint_uses(domain, state, uses);
            collect_endpoint_uses(body, state, uses);
        }
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::StrLit(_) => {}
    }
}

fn collect_block_endpoint_uses(block: &Block, state: &State, uses: &mut BTreeSet<String>) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Let { init, .. } => collect_endpoint_uses(init, state, uses),
            Stmt::Assign { target, value } => {
                collect_endpoint_uses(target, state, uses);
                collect_endpoint_uses(value, state, uses);
            }
            Stmt::Return(value) => {
                if let Some(value) = value {
                    collect_endpoint_uses(value, state, uses);
                }
            }
            Stmt::If { cond, then, else_ } => {
                collect_endpoint_uses(cond, state, uses);
                collect_block_endpoint_uses(then, state, uses);
                if let Some(else_) = else_ {
                    collect_block_endpoint_uses(else_, state, uses);
                }
            }
            Stmt::Loop(loop_) => collect_block_endpoint_uses(&loop_.body, state, uses),
            Stmt::Holding { body, .. } => collect_block_endpoint_uses(body, state, uses),
            Stmt::Forget { value, .. } | Stmt::Expr(value) => {
                collect_endpoint_uses(value, state, uses)
            }
            Stmt::Break | Stmt::Continue => {}
        }
    }
    if let Some(tail) = &block.tail {
        collect_endpoint_uses(tail, state, uses);
    }
}

fn check_completion(state: &State, function: &FnItem, errors: &mut Vec<ProtocolError>) {
    for (name, endpoint) in state {
        if !endpoint.complete {
            errors.push(error(
                ProtocolErrorKind::UnfinishedEndpoint, Some(function.name.clone()), Some(name.clone()),
                format!("function `{}` returns with endpoint `{name}` before protocol `{}` reaches `end`", function.name, endpoint.protocol), function.span,
            ));
        }
    }
}

fn resolve_endpoint(
    protocols: &BTreeMap<String, Protocol>,
    name: &str,
    role: &str,
) -> Result<(), String> {
    let Some(protocol) = protocols.get(name) else {
        return Err(format!("unknown protocol `{name}`"));
    };
    if protocol.roles.iter().any(|candidate| candidate == role) {
        Ok(())
    } else {
        Err(format!("protocol `{name}` declares no role `{role}`"))
    }
}

fn contains_endpoint(ty: &Type) -> bool {
    match ty {
        Type::ProtocolEndpoint { .. } => true,
        Type::Ref { inner, .. }
        | Type::Slice(inner)
        | Type::Generic { arg: inner, .. }
        | Type::Box(inner)
        | Type::Vec(inner)
        | Type::Option(inner) => contains_endpoint(inner),
        Type::Result(ok, err) | Type::Map(ok, err) => {
            contains_endpoint(ok) || contains_endpoint(err)
        }
        Type::Tuple(items) => items.iter().any(contains_endpoint),
        _ => false,
    }
}

fn error(
    kind: ProtocolErrorKind,
    function: Option<String>,
    endpoint: Option<String>,
    detail: String,
    span: Span,
) -> ProtocolError {
    ProtocolError {
        kind,
        function,
        endpoint,
        detail,
        span,
    }
}
