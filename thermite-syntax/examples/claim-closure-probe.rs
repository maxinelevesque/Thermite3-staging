//! Narrow machine-readable adapter used by the claim-closure executable oracle.

use std::io::{self, Read};

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

fn main() {
    let mut source = String::new();
    io::stdin()
        .read_to_string(&mut source)
        .expect("read syntax probe source");
    let (tokens, errors) = tokenize(&source);
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
    let errors: Vec<Value> = errors
        .iter()
        .map(|error| {
            let span = error.span();
            json!({
                "kind": error_kind(error),
                "span": [span.start, span.len],
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string(&json!({"errors": errors, "integers": integers}))
            .expect("serialize syntax probe observation")
    );
}
