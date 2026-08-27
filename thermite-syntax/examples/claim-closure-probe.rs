//! Narrow machine-readable adapter used by the claim-closure executable oracle.

use std::{
    collections::BTreeSet,
    env,
    io::{self, Read},
};

use serde_json::{json, Value};
use thermite_syntax::{
    addresses_of, parse, tokenize, AddrKind, BinOp, Block, Effect, EffectRow, Expr, HoleContext,
    IndexArg, Item, LoopKind, Pattern, PrimType, SlicePat, Stmt, SyntaxError, TokKind, Type,
    UnaryOp, VariantShape,
};

fn error_kind(error: &SyntaxError) -> &'static str {
    match error {
        SyntaxError::StrayChar { .. } => "StrayChar",
        SyntaxError::UnterminatedString { .. } => "UnterminatedString",
        SyntaxError::Unexpected { .. } => "Unexpected",
        SyntaxError::MissingClause { .. } => "MissingClause",
        SyntaxError::ClauseOrder { .. } => "ClauseOrder",
        SyntaxError::ClauseOrdinalOverflow { .. } => "ClauseOrdinalOverflow",
        SyntaxError::UnexpectedEof { .. } => "UnexpectedEof",
        SyntaxError::ExpressionTooDeep { .. } => "ExpressionTooDeep",
        SyntaxError::BreakContinueOutsideLoop { .. } => "BreakContinueOutsideLoop",
        SyntaxError::HoleOutsideFnBody { .. } => "HoleOutsideFnBody",
        SyntaxError::ProofHoleOutsideProofBlock { .. } => "ProofHoleOutsideProofBlock",
        SyntaxError::BodyHoleInProofBlock { .. } => "BodyHoleInProofBlock",
        SyntaxError::BvTagWithoutShadowPlumbing { .. } => "BvTagWithoutShadowPlumbing",
        SyntaxError::BvTagOnPrecondition { .. } => "BvTagOnPrecondition",
        SyntaxError::BvWidthInvalid { .. } => "BvWidthInvalid",
        SyntaxError::UnknownEffectPrimitive { .. } => "UnknownEffectPrimitive",
    }
}

fn errors_json(errors: &[SyntaxError]) -> Vec<Value> {
    errors
        .iter()
        .map(|error| {
            let span = error.span();
            let mut value = json!({
                "kind": error_kind(error),
                "span": [span.start, span.len],
            });
            if let SyntaxError::MissingClause { item, clause, .. }
            | SyntaxError::ClauseOrder { item, clause, .. } = error
            {
                value["clause"] = json!(clause);
                value["item"] = json!(item);
            }
            if let SyntaxError::HoleOutsideFnBody { number, .. }
            | SyntaxError::ProofHoleOutsideProofBlock { number, .. } = error
            {
                value["number"] = json!(number);
            }
            value
        })
        .collect()
}

fn token_json(token: thermite_syntax::Token) -> Value {
    let span = [token.span.start, token.span.len];
    match token.kind {
        TokKind::Ident(value) => json!({"kind": "Ident", "span": span, "value": value}),
        TokKind::Int { value, raw } => json!({
            "kind": "Int",
            "raw": raw,
            "span": span,
            "value": value.to_string(),
        }),
        TokKind::Bool(value) => json!({"kind": "Bool", "span": span, "value": value}),
        TokKind::Str(value) => json!({"kind": "Str", "span": span, "value": value}),
        TokKind::Hole { number, proof } => json!({
            "kind": "Hole",
            "number": number,
            "proof": proof,
            "span": span,
        }),
        kind => json!({"kind": format!("{kind:?}"), "span": span}),
    }
}

fn binop_text(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "Add",
        BinOp::Sub => "Sub",
        BinOp::Mul => "Mul",
        BinOp::Div => "Div",
        BinOp::Rem => "Rem",
        BinOp::Shl => "Shl",
        BinOp::Shr => "Shr",
        BinOp::BitAnd => "BitAnd",
        BinOp::BitOr => "BitOr",
        BinOp::BitXor => "BitXor",
        BinOp::Eq => "Eq",
        BinOp::Ne => "Ne",
        BinOp::Lt => "Lt",
        BinOp::Le => "Le",
        BinOp::Gt => "Gt",
        BinOp::Ge => "Ge",
        BinOp::And => "And",
        BinOp::Or => "Or",
    }
}

fn unaryop_text(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Not => "Not",
    }
}

fn expr_json(expr: &Expr) -> Value {
    match expr {
        Expr::IntLit { value, raw } => {
            json!({"kind": "Int", "raw": raw, "value": value.to_string()})
        }
        Expr::BoolLit(value) => json!({"kind": "Bool", "value": value}),
        Expr::Path(segments) => json!({"kind": "Path", "segments": segments}),
        Expr::Call { callee, args } => json!({
            "args": args.iter().map(expr_json).collect::<Vec<_>>(),
            "callee": expr_json(callee),
            "kind": "Call",
        }),
        Expr::MethodCall {
            receiver,
            name,
            args,
        } => json!({
            "args": args.iter().map(expr_json).collect::<Vec<_>>(),
            "kind": "MethodCall",
            "name": name,
            "receiver": expr_json(receiver),
        }),
        Expr::Field { receiver, name } => json!({
            "kind": "Field",
            "name": name,
            "receiver": expr_json(receiver),
        }),
        Expr::Binary { op, lhs, rhs } => json!({
            "kind": "Binary",
            "lhs": expr_json(lhs),
            "op": binop_text(*op),
            "rhs": expr_json(rhs),
        }),
        Expr::Unary { op, expr } => {
            json!({"expr": expr_json(expr), "kind": "Unary", "op": unaryop_text(*op)})
        }
        _ => json!({"kind": "Other"}),
    }
}

fn collect_operator_facts(
    expr: &Expr,
    operators: &mut BTreeSet<&'static str>,
    integers: &mut Vec<Value>,
) {
    match expr {
        Expr::IntLit { value, raw } => integers.push(json!({
            "raw": raw,
            "value": value.to_string(),
        })),
        Expr::Binary { op, lhs, rhs } => {
            operators.insert(binop_text(*op));
            collect_operator_facts(lhs, operators, integers);
            collect_operator_facts(rhs, operators, integers);
        }
        Expr::Unary { op, expr } => {
            operators.insert(unaryop_text(*op));
            collect_operator_facts(expr, operators, integers);
        }
        _ => {}
    }
}

fn expr_kind(expr: &Expr) -> &'static str {
    match expr {
        Expr::IntLit { .. } => "IntLit",
        Expr::BoolLit(_) => "BoolLit",
        Expr::Path(_) => "Path",
        Expr::Call { .. } => "Call",
        Expr::MethodCall { .. } => "MethodCall",
        Expr::Field { .. } => "Field",
        Expr::Closure { .. } => "Closure",
        Expr::Match { .. } => "Match",
        Expr::If { .. } => "If",
        Expr::Binary { .. } => "Binary",
        Expr::Unary { .. } => "Unary",
        Expr::Index { .. } => "Index",
        Expr::Cast { .. } => "Cast",
        Expr::Ref { .. } => "Ref",
        Expr::StructLit { .. } => "StructLit",
        Expr::Is { .. } => "Is",
        Expr::Deref(_) => "Deref",
        Expr::StrLit(_) => "StrLit",
        Expr::Tuple(_) => "Tuple",
        Expr::TupleProj { .. } => "TupleProj",
        Expr::Quantifier { .. } => "Quantifier",
    }
}

fn collect_index_expr_kinds(index: &IndexArg, kinds: &mut BTreeSet<&'static str>) {
    match index {
        IndexArg::Single(expr) | IndexArg::RangeTo(expr) | IndexArg::RangeFrom(expr) => {
            collect_expr_kinds(expr, kinds);
        }
        IndexArg::Range(start, end) => {
            collect_expr_kinds(start, kinds);
            collect_expr_kinds(end, kinds);
        }
    }
}

fn collect_block_expr_kinds(block: &Block, kinds: &mut BTreeSet<&'static str>) {
    for statement in &block.stmts {
        match statement {
            Stmt::Let { init, .. } => collect_expr_kinds(init, kinds),
            Stmt::Assign { target, value } => {
                collect_expr_kinds(target, kinds);
                collect_expr_kinds(value, kinds);
            }
            Stmt::Return(value) => {
                if let Some(value) = value {
                    collect_expr_kinds(value, kinds);
                }
            }
            Stmt::If { cond, then, else_ } => {
                collect_expr_kinds(cond, kinds);
                collect_block_expr_kinds(then, kinds);
                if let Some(otherwise) = else_ {
                    collect_block_expr_kinds(otherwise, kinds);
                }
            }
            Stmt::Loop(loop_node) => {
                if let LoopKind::While(cond) = &loop_node.kind {
                    collect_expr_kinds(cond, kinds);
                }
                for invariant in &loop_node.invs {
                    collect_expr_kinds(&invariant.expr, kinds);
                }
                collect_expr_kinds(&loop_node.measures.expr, kinds);
                collect_block_expr_kinds(&loop_node.body, kinds);
            }
            Stmt::Holding { body, .. } => collect_block_expr_kinds(body, kinds),
            Stmt::Expr(expr) => collect_expr_kinds(expr, kinds),
            Stmt::Break | Stmt::Continue => {}
        }
    }
    if let Some(tail) = &block.tail {
        collect_expr_kinds(tail, kinds);
    }
}

fn collect_expr_kinds(expr: &Expr, kinds: &mut BTreeSet<&'static str>) {
    kinds.insert(expr_kind(expr));
    match expr {
        Expr::Call { callee, args } => {
            collect_expr_kinds(callee, kinds);
            for arg in args {
                collect_expr_kinds(arg, kinds);
            }
        }
        Expr::MethodCall { receiver, args, .. } => {
            collect_expr_kinds(receiver, kinds);
            for arg in args {
                collect_expr_kinds(arg, kinds);
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver)
        | Expr::TupleProj { receiver, .. } => collect_expr_kinds(receiver, kinds),
        Expr::Closure { body, .. } => collect_expr_kinds(body, kinds),
        Expr::Match { scrutinee, arms } => {
            collect_expr_kinds(scrutinee, kinds);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_expr_kinds(guard, kinds);
                }
                collect_expr_kinds(&arm.body, kinds);
            }
        }
        Expr::If { cond, then, else_ } => {
            collect_expr_kinds(cond, kinds);
            collect_block_expr_kinds(then, kinds);
            collect_block_expr_kinds(else_, kinds);
        }
        Expr::Binary { lhs, rhs, .. } => {
            collect_expr_kinds(lhs, kinds);
            collect_expr_kinds(rhs, kinds);
        }
        Expr::Index { base, index } => {
            collect_expr_kinds(base, kinds);
            collect_index_expr_kinds(index, kinds);
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                collect_expr_kinds(value, kinds);
            }
        }
        Expr::Is { scrutinee, .. } => collect_expr_kinds(scrutinee, kinds),
        Expr::Tuple(values) => {
            for value in values {
                collect_expr_kinds(value, kinds);
            }
        }
        Expr::Quantifier { domain, body, .. } => {
            collect_expr_kinds(domain, kinds);
            collect_expr_kinds(body, kinds);
        }
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => {}
    }
}

fn type_text(ty: &Type) -> String {
    match ty {
        Type::Prim(PrimType::U8) => "u8".to_string(),
        Type::Prim(PrimType::U16) => "u16".to_string(),
        Type::Prim(PrimType::U32) => "u32".to_string(),
        Type::Prim(PrimType::U64) => "u64".to_string(),
        Type::Prim(PrimType::Usize) => "usize".to_string(),
        Type::Prim(PrimType::Bool) => "bool".to_string(),
        Type::Ref { mutable, inner } => format!(
            "&{}{}",
            if *mutable { "mut " } else { "" },
            type_text(inner)
        ),
        Type::Slice(inner) => format!("[{}]", type_text(inner)),
        Type::Generic { name, arg } => format!("{name}<{}>", type_text(arg)),
        Type::Unit => "()".to_string(),
        Type::Named(name) => name.clone(),
        Type::Box(inner) => format!("Box<{}>", type_text(inner)),
        Type::Vec(inner) => format!("Vec<{}>", type_text(inner)),
        Type::String => "String".to_string(),
        Type::Option(inner) => format!("Option<{}>", type_text(inner)),
        Type::Result(ok, error) => format!("Result<{}, {}>", type_text(ok), type_text(error)),
        Type::Map(key, value) => format!("Map<{}, {}>", type_text(key), type_text(value)),
        Type::Tuple(types) => format!(
            "({})",
            types.iter().map(type_text).collect::<Vec<_>>().join(", ")
        ),
    }
}

fn type_json(ty: &Type) -> Value {
    match ty {
        Type::Prim(prim) => json!({"kind": "Prim", "name": type_text(&Type::Prim(*prim))}),
        Type::Unit => json!({"arity": 0, "kind": "Unit"}),
        Type::Ref { mutable, inner } => json!({
            "inner": type_json(inner),
            "kind": "Ref",
            "mutable": mutable,
        }),
        Type::Slice(inner) => json!({"element": type_json(inner), "kind": "Slice"}),
        Type::Generic { name, arg } => json!({
            "argument": type_json(arg),
            "kind": "Generic",
            "name": name,
        }),
        Type::Named(name) => json!({"kind": "Named", "name": name}),
        Type::Box(inner) => json!({"argument": type_json(inner), "kind": "Box"}),
        Type::Vec(inner) => json!({"element": type_json(inner), "kind": "Vec"}),
        Type::String => json!({"kind": "String"}),
        Type::Option(inner) => json!({"argument": type_json(inner), "kind": "Option"}),
        Type::Result(ok, error) => json!({
            "error": type_json(error),
            "kind": "Result",
            "ok": type_json(ok),
        }),
        Type::Map(key, value) => json!({
            "key": type_json(key),
            "kind": "Map",
            "value": type_json(value),
        }),
        Type::Tuple(elements) => json!({
            "arity": elements.len(),
            "elements": elements.iter().map(type_json).collect::<Vec<_>>(),
            "kind": "Tuple",
        }),
    }
}

fn collect_basis_index(
    index: &IndexArg,
    methods: &mut BTreeSet<String>,
    tuple_arities: &mut BTreeSet<usize>,
    tuple_projections: &mut BTreeSet<usize>,
) {
    match index {
        IndexArg::Single(expr) | IndexArg::RangeTo(expr) | IndexArg::RangeFrom(expr) => {
            collect_basis_expr(expr, methods, tuple_arities, tuple_projections);
        }
        IndexArg::Range(start, end) => {
            collect_basis_expr(start, methods, tuple_arities, tuple_projections);
            collect_basis_expr(end, methods, tuple_arities, tuple_projections);
        }
    }
}

fn collect_basis_block(
    block: &Block,
    methods: &mut BTreeSet<String>,
    tuple_arities: &mut BTreeSet<usize>,
    tuple_projections: &mut BTreeSet<usize>,
) {
    for statement in &block.stmts {
        match statement {
            Stmt::Let { init, .. } => {
                collect_basis_expr(init, methods, tuple_arities, tuple_projections)
            }
            Stmt::Assign { target, value } => {
                collect_basis_expr(target, methods, tuple_arities, tuple_projections);
                collect_basis_expr(value, methods, tuple_arities, tuple_projections);
            }
            Stmt::Return(value) => {
                if let Some(value) = value {
                    collect_basis_expr(value, methods, tuple_arities, tuple_projections);
                }
            }
            Stmt::If { cond, then, else_ } => {
                collect_basis_expr(cond, methods, tuple_arities, tuple_projections);
                collect_basis_block(then, methods, tuple_arities, tuple_projections);
                if let Some(otherwise) = else_ {
                    collect_basis_block(otherwise, methods, tuple_arities, tuple_projections);
                }
            }
            Stmt::Loop(loop_node) => {
                if let LoopKind::While(cond) = &loop_node.kind {
                    collect_basis_expr(cond, methods, tuple_arities, tuple_projections);
                }
                for invariant in &loop_node.invs {
                    collect_basis_expr(&invariant.expr, methods, tuple_arities, tuple_projections);
                }
                collect_basis_expr(
                    &loop_node.measures.expr,
                    methods,
                    tuple_arities,
                    tuple_projections,
                );
                collect_basis_block(&loop_node.body, methods, tuple_arities, tuple_projections);
            }
            Stmt::Holding { body, .. } => {
                collect_basis_block(body, methods, tuple_arities, tuple_projections)
            }
            Stmt::Expr(expr) => collect_basis_expr(expr, methods, tuple_arities, tuple_projections),
            Stmt::Break | Stmt::Continue => {}
        }
    }
    if let Some(tail) = &block.tail {
        collect_basis_expr(tail, methods, tuple_arities, tuple_projections);
    }
}

fn collect_basis_expr(
    expr: &Expr,
    methods: &mut BTreeSet<String>,
    tuple_arities: &mut BTreeSet<usize>,
    tuple_projections: &mut BTreeSet<usize>,
) {
    match expr {
        Expr::Call { callee, args } => {
            collect_basis_expr(callee, methods, tuple_arities, tuple_projections);
            for arg in args {
                collect_basis_expr(arg, methods, tuple_arities, tuple_projections);
            }
        }
        Expr::MethodCall {
            receiver,
            name,
            args,
        } => {
            methods.insert(name.clone());
            collect_basis_expr(receiver, methods, tuple_arities, tuple_projections);
            for arg in args {
                collect_basis_expr(arg, methods, tuple_arities, tuple_projections);
            }
        }
        Expr::Field { receiver, .. }
        | Expr::Unary { expr: receiver, .. }
        | Expr::Cast { expr: receiver, .. }
        | Expr::Ref { expr: receiver, .. }
        | Expr::Deref(receiver) => {
            collect_basis_expr(receiver, methods, tuple_arities, tuple_projections)
        }
        Expr::TupleProj { receiver, index } => {
            tuple_projections.insert(*index);
            collect_basis_expr(receiver, methods, tuple_arities, tuple_projections);
        }
        Expr::Closure { body, .. } => {
            collect_basis_expr(body, methods, tuple_arities, tuple_projections)
        }
        Expr::Match { scrutinee, arms } => {
            collect_basis_expr(scrutinee, methods, tuple_arities, tuple_projections);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_basis_expr(guard, methods, tuple_arities, tuple_projections);
                }
                collect_basis_expr(&arm.body, methods, tuple_arities, tuple_projections);
            }
        }
        Expr::If { cond, then, else_ } => {
            collect_basis_expr(cond, methods, tuple_arities, tuple_projections);
            collect_basis_block(then, methods, tuple_arities, tuple_projections);
            collect_basis_block(else_, methods, tuple_arities, tuple_projections);
        }
        Expr::Binary { lhs, rhs, .. } => {
            collect_basis_expr(lhs, methods, tuple_arities, tuple_projections);
            collect_basis_expr(rhs, methods, tuple_arities, tuple_projections);
        }
        Expr::Index { base, index } => {
            collect_basis_expr(base, methods, tuple_arities, tuple_projections);
            collect_basis_index(index, methods, tuple_arities, tuple_projections);
        }
        Expr::StructLit { fields, .. } => {
            for (_, value) in fields {
                collect_basis_expr(value, methods, tuple_arities, tuple_projections);
            }
        }
        Expr::Is { scrutinee, .. } => {
            collect_basis_expr(scrutinee, methods, tuple_arities, tuple_projections)
        }
        Expr::Tuple(values) => {
            tuple_arities.insert(values.len());
            for value in values {
                collect_basis_expr(value, methods, tuple_arities, tuple_projections);
            }
        }
        Expr::Quantifier { domain, body, .. } => {
            collect_basis_expr(domain, methods, tuple_arities, tuple_projections);
            collect_basis_expr(body, methods, tuple_arities, tuple_projections);
        }
        Expr::IntLit { .. } | Expr::BoolLit(_) | Expr::Path(_) | Expr::StrLit(_) => {}
    }
}

fn basis_item_json(item: &Item) -> Value {
    match item {
        Item::Fn(function) => {
            let mut expression_kinds = BTreeSet::new();
            let mut methods = BTreeSet::new();
            let mut tuple_arities = BTreeSet::new();
            let mut tuple_projections = BTreeSet::new();
            collect_expr_kinds(&function.contract.requires.expr, &mut expression_kinds);
            collect_basis_expr(
                &function.contract.requires.expr,
                &mut methods,
                &mut tuple_arities,
                &mut tuple_projections,
            );
            for clause in &function.contract.ensures {
                collect_expr_kinds(&clause.expr, &mut expression_kinds);
                collect_basis_expr(
                    &clause.expr,
                    &mut methods,
                    &mut tuple_arities,
                    &mut tuple_projections,
                );
            }
            if let Some(measures) = &function.measures {
                collect_expr_kinds(&measures.expr, &mut expression_kinds);
                collect_basis_expr(
                    &measures.expr,
                    &mut methods,
                    &mut tuple_arities,
                    &mut tuple_projections,
                );
            }
            if let Some(body) = &function.body {
                collect_block_expr_kinds(body, &mut expression_kinds);
                collect_basis_block(
                    body,
                    &mut methods,
                    &mut tuple_arities,
                    &mut tuple_projections,
                );
            }
            json!({
                "expression_kinds": expression_kinds,
                "kind": "Fn",
                "measures": function.measures.as_ref().map(|clause| &clause.text),
                "methods": methods,
                "name": function.name,
                "params": function.params.iter().map(|param| json!({
                    "name": param.name,
                    "type": type_json(&param.ty),
                })).collect::<Vec<_>>(),
                "ret": type_json(&function.ret),
                "tuple_expression_arities": tuple_arities,
                "tuple_projections": tuple_projections,
            })
        }
        _ => json!({"kind": "Other", "name": item.name()}),
    }
}

fn effect_text(effect: &Effect) -> String {
    match effect {
        Effect::Read(path) => format!("read({path})"),
        Effect::Write(path) => format!("write({path})"),
        Effect::Net(path) => format!("net({path})"),
        Effect::Owns(lock) => format!("owns({lock})"),
        Effect::Alloc => "alloc".to_string(),
        Effect::Time => "time".to_string(),
        Effect::Rand => "rand".to_string(),
        Effect::Panic => "panic".to_string(),
        Effect::Diverge => "diverge".to_string(),
        Effect::Term => "term".to_string(),
    }
}

fn pattern_kind(pattern: &Pattern) -> &'static str {
    match pattern {
        Pattern::Wildcard => "Wildcard",
        Pattern::Literal(_) => "Literal",
        Pattern::Binding(_) => "Binding",
        Pattern::Slice(_) => "Slice",
        Pattern::Enum { .. } => "Enum",
        Pattern::Struct { .. } => "Struct",
        Pattern::Or(_) => "Or",
    }
}

fn collect_pattern_kinds(pattern: &Pattern, kinds: &mut BTreeSet<&'static str>) {
    kinds.insert(pattern_kind(pattern));
    match pattern {
        Pattern::Slice(parts) => {
            for part in parts {
                if let SlicePat::Pat(pattern) = part {
                    collect_pattern_kinds(pattern, kinds);
                }
            }
        }
        Pattern::Enum { fields, .. } | Pattern::Or(fields) => {
            for field in fields {
                collect_pattern_kinds(field, kinds);
            }
        }
        Pattern::Struct { fields, .. } => {
            for (_, field) in fields {
                collect_pattern_kinds(field, kinds);
            }
        }
        Pattern::Wildcard | Pattern::Literal(_) | Pattern::Binding(_) => {}
    }
}

fn collect_clause_pattern_kinds(expr: &Expr, kinds: &mut BTreeSet<&'static str>) {
    match expr {
        Expr::Match { arms, .. } => {
            for arm in arms {
                collect_pattern_kinds(&arm.pattern, kinds);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            collect_clause_pattern_kinds(lhs, kinds);
            collect_clause_pattern_kinds(rhs, kinds);
        }
        _ => {}
    }
}

fn pattern_json(pattern: &Pattern) -> Value {
    match pattern {
        Pattern::Wildcard => json!({"kind": "Wildcard"}),
        Pattern::Literal(expr) => json!({"kind": "Literal", "value": expr_json(expr)}),
        Pattern::Binding(name) => json!({"kind": "Binding", "name": name}),
        Pattern::Slice(parts) => json!({
            "kind": "Slice",
            "parts": parts.iter().map(|part| match part {
                SlicePat::Pat(pattern) => pattern_json(pattern),
                SlicePat::Rest(name) => json!({"kind": "Rest", "name": name}),
            }).collect::<Vec<_>>(),
        }),
        Pattern::Enum { path, fields } => json!({
            "fields": fields.iter().map(pattern_json).collect::<Vec<_>>(),
            "kind": "Enum",
            "path": path,
        }),
        Pattern::Struct { path, fields, rest } => json!({
            "fields": fields.iter().map(|(name, pattern)| json!({
                "name": name,
                "pattern": pattern_json(pattern),
            })).collect::<Vec<_>>(),
            "kind": "Struct",
            "path": path,
            "rest": rest,
        }),
        Pattern::Or(patterns) => json!({
            "alternatives": patterns.iter().map(pattern_json).collect::<Vec<_>>(),
            "kind": "Or",
        }),
    }
}

fn direct_match_patterns(block: &Block) -> Vec<Value> {
    match block.tail.as_deref() {
        Some(Expr::Match { arms, .. }) => {
            arms.iter().map(|arm| pattern_json(&arm.pattern)).collect()
        }
        _ => Vec::new(),
    }
}

fn variant_shape_json(shape: &VariantShape) -> Value {
    match shape {
        VariantShape::Unit => json!({"kind": "Unit"}),
        VariantShape::Tuple(types) => json!({
            "kind": "Tuple",
            "types": types.iter().map(type_text).collect::<Vec<_>>(),
        }),
        VariantShape::Struct(fields) => json!({
            "fields": fields.iter().map(|field| json!({
                "name": field.name,
                "type": type_text(&field.ty),
            })).collect::<Vec<_>>(),
            "kind": "Struct",
        }),
    }
}

fn adt_item_json(item: &Item) -> Value {
    match item {
        Item::Struct(item) => json!({
            "fields": item.fields.iter().map(|field| json!({
                "name": field.name,
                "type": type_text(&field.ty),
            })).collect::<Vec<_>>(),
            "keeps": item.keeps.as_ref().map(|clause| &clause.text),
            "kind": "Struct",
            "name": item.name,
        }),
        Item::Enum(item) => json!({
            "kind": "Enum",
            "name": item.name,
            "variants": item.variants.iter().map(|variant| json!({
                "name": variant.name,
                "shape": variant_shape_json(&variant.shape),
            })).collect::<Vec<_>>(),
        }),
        Item::Fn(function) => {
            let mut expression_kinds = BTreeSet::new();
            collect_expr_kinds(&function.contract.requires.expr, &mut expression_kinds);
            for clause in &function.contract.ensures {
                collect_expr_kinds(&clause.expr, &mut expression_kinds);
            }
            if let Some(body) = &function.body {
                collect_block_expr_kinds(body, &mut expression_kinds);
            }
            json!({
                "expression_kinds": expression_kinds,
                "kind": "Fn",
                "match_patterns": function.body.as_ref().map(direct_match_patterns).unwrap_or_default(),
                "name": function.name,
                "params": function.params.iter().map(|param| type_text(&param.ty)).collect::<Vec<_>>(),
                "ret": type_text(&function.ret),
            })
        }
        Item::SpecFn(function) => {
            let mut expression_kinds = BTreeSet::new();
            collect_expr_kinds(&function.measures.expr, &mut expression_kinds);
            collect_block_expr_kinds(&function.body, &mut expression_kinds);
            json!({
                "expression_kinds": expression_kinds,
                "kind": "SpecFn",
                "match_patterns": direct_match_patterns(&function.body),
                "name": function.name,
                "params": function.params.iter().map(|param| type_text(&param.ty)).collect::<Vec<_>>(),
                "ret": type_text(&function.ret),
            })
        }
        _ => json!({"kind": "Other", "name": item.name()}),
    }
}

fn collect_loop_facts(block: &Block, loops: &mut Vec<Value>) {
    for statement in &block.stmts {
        match statement {
            Stmt::Loop(loop_node) => {
                loops.push(json!({
                    "has_dec": true,
                    "inv_count": loop_node.invs.len(),
                    "surface_keyword": loop_node.kind.surface_keyword(),
                }));
                collect_loop_facts(&loop_node.body, loops);
            }
            Stmt::If { then, else_, .. } => {
                collect_loop_facts(then, loops);
                if let Some(otherwise) = else_ {
                    collect_loop_facts(otherwise, loops);
                }
            }
            _ => {}
        }
    }
}

fn stmt_kind(statement: &Stmt) -> &'static str {
    match statement {
        Stmt::Let { .. } => "Let",
        Stmt::Assign { .. } => "Assign",
        Stmt::Return(_) => "Return",
        Stmt::If { .. } => "If",
        Stmt::Loop(_) => "Loop",
        Stmt::Holding { .. } => "Holding",
        Stmt::Break => "Break",
        Stmt::Continue => "Continue",
        Stmt::Expr(_) => "Expr",
    }
}

fn collect_stmt_kinds(block: &Block, kinds: &mut BTreeSet<&'static str>) {
    for statement in &block.stmts {
        kinds.insert(stmt_kind(statement));
        match statement {
            Stmt::If { then, else_, .. } => {
                collect_stmt_kinds(then, kinds);
                if let Some(otherwise) = else_ {
                    collect_stmt_kinds(otherwise, kinds);
                }
            }
            Stmt::Loop(loop_node) => collect_stmt_kinds(&loop_node.body, kinds),
            Stmt::Holding { body, .. } => collect_stmt_kinds(body, kinds),
            Stmt::Let { .. }
            | Stmt::Assign { .. }
            | Stmt::Return(_)
            | Stmt::Break
            | Stmt::Continue
            | Stmt::Expr(_) => {}
        }
    }
}

fn fidelity_item_json(item: &Item) -> Value {
    match item {
        Item::Fn(function) => {
            let mut loops = Vec::new();
            if let Some(body) = &function.body {
                collect_loop_facts(body, &mut loops);
            }
            json!({
                "ens_count": function.contract.ensures.len(),
                "fx": match function.contract.effects { EffectRow::Pure => "pure", EffectRow::Set(_) => "set" },
                "kind": "fn",
                "loops": loops,
                "name": function.name,
                "params": function.params.iter().map(|param| json!({"name": param.name, "type": type_text(&param.ty)})).collect::<Vec<_>>(),
                "req_count": 1,
                "ret": type_text(&function.ret),
            })
        }
        Item::SpecFn(function) => json!({
            "has_dec": true,
            "kind": "spec fn",
            "name": function.name,
            "params": function.params.iter().map(|param| json!({"name": param.name, "type": type_text(&param.ty)})).collect::<Vec<_>>(),
            "ret": type_text(&function.ret),
        }),
        _ => json!({"kind": "other", "name": item.name()}),
    }
}

fn address_kind_text(kind: AddrKind) -> &'static str {
    match kind {
        AddrKind::Fn => "fn",
        AddrKind::SpecFn => "spec fn",
        AddrKind::Loop => "loop",
        AddrKind::Inv => "keeps",
        AddrKind::Dec => "measures",
        AddrKind::Hole => "hole",
        AddrKind::Forge => "forge",
        AddrKind::ProofHole => "proof hole",
    }
}

fn main() {
    let mode = env::args().nth(1).unwrap_or_default();
    if !matches!(
        mode.as_str(),
        "ast-basis-types"
            | "ast-adts"
            | "ast-expressions"
            | "ast-operators"
            | "ast-statements"
            | "ast-types-spans"
            | "integers"
            | "parse-edges"
            | "parse-expressions"
            | "parse-fidelity"
            | "parse-items"
            | "tokens"
    ) {
        std::process::exit(2);
    }
    let mut source = String::new();
    io::stdin()
        .read_to_string(&mut source)
        .expect("read syntax probe source");
    if matches!(
        mode.as_str(),
        "ast-basis-types"
            | "ast-adts"
            | "ast-expressions"
            | "ast-operators"
            | "ast-statements"
            | "ast-types-spans"
            | "parse-edges"
            | "parse-expressions"
            | "parse-fidelity"
            | "parse-items"
    ) {
        let result = parse(&source);
        if mode == "ast-basis-types" {
            let observation = json!({
                "errors": errors_json(&result.errors),
                "items": result.program.items.iter().map(basis_item_json).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string(&observation).expect("serialize syntax probe observation")
            );
            return;
        }
        if mode == "ast-adts" {
            let observation = json!({
                "errors": errors_json(&result.errors),
                "items": result.program.items.iter().map(adt_item_json).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string(&observation).expect("serialize syntax probe observation")
            );
            return;
        }
        if mode == "ast-types-spans" {
            let functions = result
                .program
                .items
                .iter()
                .filter_map(|item| match item {
                    Item::Fn(function) => {
                        let mut pattern_kinds = BTreeSet::new();
                        for clause in &function.contract.ensures {
                            collect_clause_pattern_kinds(&clause.expr, &mut pattern_kinds);
                        }
                        let effects = match &function.contract.effects {
                            EffectRow::Pure => vec!["pure".to_string()],
                            EffectRow::Set(effects) => effects.iter().map(effect_text).collect(),
                        };
                        Some(json!({
                            "effects": effects,
                            "ensures": function.contract.ensures.iter().map(|clause| json!({
                                "span": [clause.span.start, clause.span.len],
                                "text": clause.text,
                            })).collect::<Vec<_>>(),
                            "name": function.name,
                            "params": function.params.iter().map(|param| type_text(&param.ty)).collect::<Vec<_>>(),
                            "pattern_kinds": pattern_kinds,
                            "requires": {
                                "span": [function.contract.requires.span.start, function.contract.requires.span.len],
                                "text": function.contract.requires.text,
                            },
                            "ret": type_text(&function.ret),
                            "span": [function.span.start, function.span.len],
                        }))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let observation =
                json!({"errors": errors_json(&result.errors), "functions": functions});
            println!(
                "{}",
                serde_json::to_string(&observation).expect("serialize syntax probe observation")
            );
            return;
        }
        if mode == "ast-expressions" {
            let functions = result
                .program
                .items
                .iter()
                .filter_map(|item| match item {
                    Item::Fn(function) => {
                        let mut expression_kinds = BTreeSet::new();
                        collect_expr_kinds(&function.contract.requires.expr, &mut expression_kinds);
                        for clause in &function.contract.ensures {
                            collect_expr_kinds(&clause.expr, &mut expression_kinds);
                        }
                        if let Some(body) = &function.body {
                            collect_block_expr_kinds(body, &mut expression_kinds);
                        }
                        Some(json!({
                            "expression_kinds": expression_kinds,
                            "name": function.name,
                        }))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let observation =
                json!({"errors": errors_json(&result.errors), "functions": functions});
            println!(
                "{}",
                serde_json::to_string(&observation).expect("serialize syntax probe observation")
            );
            return;
        }
        if mode == "ast-statements" {
            let functions = result
                .program
                .items
                .iter()
                .filter_map(|item| match item {
                    Item::Fn(function) => {
                        let mut statement_kinds = BTreeSet::new();
                        if let Some(body) = &function.body {
                            collect_stmt_kinds(body, &mut statement_kinds);
                        }
                        Some(json!({
                            "body_tail": function.body.as_ref().and_then(|body| body.tail.as_ref()).is_some(),
                            "contract": {
                                "ensures": function.contract.ensures.len(),
                                "effects": match function.contract.effects { EffectRow::Pure => "pure", EffectRow::Set(_) => "set" },
                                "requires": true,
                            },
                            "name": function.name,
                            "slag": function.slag.as_ref().map(|slag| json!({
                                "owner": slag.owner,
                                "reason": slag.reason,
                                "review": slag.review,
                            })),
                            "statement_kinds": statement_kinds,
                        }))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let observation =
                json!({"errors": errors_json(&result.errors), "functions": functions});
            println!(
                "{}",
                serde_json::to_string(&observation).expect("serialize syntax probe observation")
            );
            return;
        }
        if mode == "ast-operators" {
            let functions = result
                .program
                .items
                .iter()
                .filter_map(|item| match item {
                    Item::Fn(function) => {
                        let mut operators = BTreeSet::new();
                        let mut integers = Vec::new();
                        for clause in &function.contract.ensures {
                            collect_operator_facts(&clause.expr, &mut operators, &mut integers);
                        }
                        Some(json!({
                            "integers": integers,
                            "name": function.name,
                            "operators": operators,
                        }))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let observation =
                json!({"errors": errors_json(&result.errors), "functions": functions});
            println!(
                "{}",
                serde_json::to_string(&observation).expect("serialize syntax probe observation")
            );
            return;
        }
        if mode == "parse-edges" {
            let functions = result
                .program
                .items
                .iter()
                .filter_map(|item| match item {
                    Item::Fn(function) => Some(json!({
                        "body": function.body.is_some(),
                        "boundary_target": function.boundary.as_ref().map(|boundary| &boundary.target),
                        "holes": function.holes.iter().map(|hole| json!({
                            "context": match hole.context { HoleContext::Body => "Body", HoleContext::Proof => "Proof" },
                            "number": hole.number,
                        })).collect::<Vec<_>>(),
                        "name": function.name,
                    })),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let observation =
                json!({"errors": errors_json(&result.errors), "functions": functions});
            println!(
                "{}",
                serde_json::to_string(&observation).expect("serialize syntax probe observation")
            );
            return;
        }
        if mode == "parse-fidelity" {
            let addresses = addresses_of(&result.program)
                .into_iter()
                .map(|entry| {
                    json!({
                        "addr": entry.addr,
                        "kind": address_kind_text(entry.kind),
                        "surface_keyword": entry.surface_keyword,
                        "text": entry.text,
                    })
                })
                .collect::<Vec<_>>();
            let observation = json!({
                "addresses": addresses,
                "errors": errors_json(&result.errors),
                "items": result.program.items.iter().map(fidelity_item_json).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string(&observation).expect("serialize syntax probe observation")
            );
            return;
        }
        if mode == "parse-expressions" {
            let expressions = result
                .program
                .items
                .iter()
                .filter_map(|item| match item {
                    Item::Fn(function) => Some(json!({
                        "ensures": function
                            .contract
                            .ensures
                            .iter()
                            .map(|clause| expr_json(&clause.expr))
                            .collect::<Vec<_>>(),
                        "name": function.name,
                    })),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let observation =
                json!({"errors": errors_json(&result.errors), "functions": expressions});
            println!(
                "{}",
                serde_json::to_string(&observation).expect("serialize syntax probe observation")
            );
            return;
        }
        let items: Vec<Value> = result
            .program
            .items
            .iter()
            .map(|item| {
                let kind = match item {
                    Item::Fn(_) => "Fn",
                    Item::SpecFn(_) => "SpecFn",
                    Item::Struct(_) => "Struct",
                    Item::Enum(_) => "Enum",
                    Item::Forge(_) => "Forge",
                    Item::EffectDecl(_) => "EffectDecl",
                    Item::SharedDecl(_) => "SharedDecl",
                    Item::Concurrent(_) => "Concurrent",
                    Item::LockDecl(_) => "LockDecl",
                };
                json!({"kind": kind, "name": item.name()})
            })
            .collect();
        let observation = json!({"errors": errors_json(&result.errors), "items": items});
        println!(
            "{}",
            serde_json::to_string(&observation).expect("serialize syntax probe observation")
        );
        return;
    }
    let (tokens, errors) = tokenize(&source);
    let errors = errors_json(&errors);
    let observation = if mode == "integers" {
        let integers: Vec<Value> = tokens
            .into_iter()
            .filter_map(|token| match token.kind {
                TokKind::Int { value, raw } => Some(json!({
                    "raw": raw,
                    "span": [token.span.start, token.span.len],
                    "value": value.to_string(),
                })),
                _ => None,
            })
            .collect();
        json!({"errors": errors, "integers": integers})
    } else {
        json!({
            "errors": errors,
            "tokens": tokens.into_iter().map(token_json).collect::<Vec<_>>(),
        })
    };
    println!(
        "{}",
        serde_json::to_string(&observation).expect("serialize syntax probe observation")
    );
}
