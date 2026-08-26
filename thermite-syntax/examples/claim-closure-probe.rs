//! Narrow machine-readable adapter used by the claim-closure executable oracle.

use std::{
    env,
    io::{self, Read},
};

use serde_json::{json, Value};
use thermite_syntax::{tokenize, SyntaxError, TokKind};

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
            json!({
                "kind": error_kind(error),
                "span": [span.start, span.len],
            })
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

fn main() {
    let mode = env::args().nth(1).unwrap_or_default();
    if !matches!(mode.as_str(), "integers" | "tokens") {
        std::process::exit(2);
    }
    let mut source = String::new();
    io::stdin()
        .read_to_string(&mut source)
        .expect("read syntax probe source");
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
