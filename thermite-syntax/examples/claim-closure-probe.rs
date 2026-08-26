//! Narrow machine-readable adapter used by the claim-closure executable oracle.

use std::{
    env,
    io::{self, Read},
};

use serde_json::{json, Value};
use thermite_syntax::{
    addresses_of, parse, tokenize, AddrKind, BinOp, Block, EffectRow, Expr, Item, PrimType, Stmt,
    SyntaxError, TokKind, Type, UnaryOp,
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
        Expr::Binary { op, lhs, rhs } => {
            let op = match op {
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
            };
            json!({"kind": "Binary", "lhs": expr_json(lhs), "op": op, "rhs": expr_json(rhs)})
        }
        Expr::Unary { op, expr } => {
            let op = match op {
                UnaryOp::Not => "Not",
            };
            json!({"expr": expr_json(expr), "kind": "Unary", "op": op})
        }
        _ => json!({"kind": "Other"}),
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
        "integers" | "parse-expressions" | "parse-fidelity" | "parse-items" | "tokens"
    ) {
        std::process::exit(2);
    }
    let mut source = String::new();
    io::stdin()
        .read_to_string(&mut source)
        .expect("read syntax probe source");
    if matches!(
        mode.as_str(),
        "parse-expressions" | "parse-fidelity" | "parse-items"
    ) {
        let result = parse(&source);
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
