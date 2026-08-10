//! Migrate Thermite 2 clause syntax to Thermite 3, driven by the pinned front
//! end rather than by pattern matching.
//!
//! The rewrite is span-directed: `parse` supplies the item boundaries and
//! decides whether the text is Thermite at all, `tokenize` supplies every offset
//! inside them. Nothing outside a spliced span is touched, so comments, blank
//! lines and alignment survive byte for byte — which is what the RFC asks for
//! and what a pretty-printer would not give.
//!
//!     req P   ->  requires P        ens P  ->  ensures P
//!     inv P   ->  keeps P           dec E  ->  measures E
//!     fx  E   ->  ! E, moved to the head of the contract
//!
//! Two facts make this exact where matching was not. A clause keyword is a
//! reserved token, so `TokKind::Req` is a clause and an identifier spelled `req`
//! cannot be one. And the effect row is a closed grammar with no brace in it
//! (`parse_effect_row`), so the row ends at the first token that cannot continue
//! it and the body's `{` is whatever follows — the contract/body boundary that
//! no regex could locate.
//!
//! Modes:
//!     thmig gate   < src     exit 0 if `src` parses clean as Thermite, 3 if not
//!     thmig to-v3  < src     the migrated source on stdout
//!     thmig edits  < src     one line per edit, for inspection

use std::io::{Read, Write};
use thermite_syntax::{parse, tokenize, Span, TokKind, Token};

/// One text replacement: `[start, start+len)` becomes `text`.
#[derive(Debug, Clone)]
struct Edit {
    start: usize,
    len: usize,
    text: String,
    note: &'static str,
}

fn renamed(k: &TokKind) -> Option<&'static str> {
    match k {
        TokKind::Req => Some("requires"),
        TokKind::Ens => Some("ensures"),
        TokKind::Inv => Some("keeps"),
        TokKind::Dec => Some("measures"),
        _ => None,
    }
}

fn is_clause_kw(k: &TokKind) -> bool {
    matches!(
        k,
        TokKind::Req | TokKind::Ens | TokKind::Inv | TokKind::Dec
    )
}

/// End offset of the effect row introduced by the `fx` at token index `i`.
///
/// `parse_effect_row` accepts `pure`, or a comma list of atoms each optionally
/// taking one parenthesised argument. Nothing else, and no brace — so the row
/// ends at the first token that cannot continue it.
fn row_end(toks: &[Token], mut i: usize) -> usize {
    let mut end = toks[i].span.start;
    loop {
        if i >= toks.len() {
            return end;
        }
        match &toks[i].kind {
            TokKind::Ident(_) | TokKind::Pure => {
                end = toks[i].span.start + toks[i].span.len;
                i += 1;
                if i < toks.len() && matches!(toks[i].kind, TokKind::LParen) {
                    let mut depth = 0usize;
                    while i < toks.len() {
                        match toks[i].kind {
                            TokKind::LParen => depth += 1,
                            TokKind::RParen => {
                                depth -= 1;
                                if depth == 0 {
                                    end = toks[i].span.start + toks[i].span.len;
                                    i += 1;
                                    break;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                }
                if i < toks.len() && matches!(toks[i].kind, TokKind::Comma) {
                    i += 1;
                    continue;
                }
                return end;
            }
            _ => return end,
        }
    }
}

/// Extend a row deletion so that a row occupying a line of its own takes the
/// line with it, rather than leaving a blank one behind.
fn widen_deletion(src: &str, start: usize, end: usize) -> (usize, usize) {
    let b = src.as_bytes();
    let mut s = start;
    while s > 0 && matches!(b[s - 1], b' ' | b'\t') {
        s -= 1;
    }
    let alone_before = s == 0 || b[s - 1] == b'\n';
    let mut e = end;
    while e < b.len() && matches!(b[e], b' ' | b'\t') {
        e += 1;
    }
    let alone_after = e >= b.len() || b[e] == b'\n';
    if alone_before && alone_after && e < b.len() {
        e += 1; // take the newline
        (s, e)
    } else {
        // keep the leading gap eaten so `fx pure` does not leave a double space
        (s, end)
    }
}

fn edits_for(src: &str, toks: &[Token], lo: usize, hi: usize) -> Vec<Edit> {
    let inside: Vec<&Token> = toks
        .iter()
        .filter(|t| t.span.start >= lo && t.span.start < hi)
        .collect();
    let mut out = Vec::new();

    // Every clause keyword in the item renames in place — including the ones on
    // a loop inside the body, which need the rename and no reordering.
    for t in &inside {
        if let Some(new) = renamed(&t.kind) {
            out.push(Edit {
                start: t.span.start,
                len: t.span.len,
                text: new.to_string(),
                note: "rename",
            });
        }
    }

    // The effect row moves to the head of the contract.
    if let Some(k) = inside.iter().position(|t| matches!(t.kind, TokKind::Fx)) {
        let fx: Span = inside[k].span;
        let global = toks.iter().position(|t| t.span.start == fx.start).unwrap();
        let end = row_end(toks, global + 1);
        let atoms = src[fx.start + fx.len..end].trim().to_string();

        // The contract head: the first clause keyword before the row.
        let head = inside
            .iter()
            .find(|t| is_clause_kw(&t.kind) && t.span.start < fx.start)
            .map(|t| t.span.start)
            .unwrap_or(fx.start);

        let (del_s, del_e) = widen_deletion(src, fx.start, end);
        out.push(Edit {
            start: del_s,
            len: del_e - del_s,
            text: String::new(),
            note: "row removed",
        });

        if head != fx.start {
            let line_start = src[..head].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let head_alone = src[line_start..head].trim().is_empty();
            let indent: String = src[line_start..head].to_string();
            let text = if head_alone {
                format!("! {atoms}\n{indent}")
            } else {
                format!("! {atoms} ")
            };
            out.push(Edit {
                start: head,
                len: 0,
                text,
                note: "row inserted",
            });
        }
    }
    out
}

fn edits(src: &str) -> (Vec<Edit>, usize, usize) {
    let (toks, _) = tokenize(src);
    let r = parse(src);
    let mut out = Vec::new();
    for item in &r.program.items {
        let sp = item_span(item);
        out.extend(edits_for(src, &toks, sp.start, sp.start + sp.len));
    }
    out.sort_by_key(|e| (e.start, e.len));
    (out, r.program.items.len(), r.errors.len())
}

fn item_span(item: &thermite_syntax::Item) -> Span {
    use thermite_syntax::{ForgeItem, Item};
    match item {
        Item::Fn(f) => f.span,
        Item::SpecFn(s) => s.span,
        Item::Struct(s) => s.span,
        Item::Enum(e) => e.span,
        Item::Forge(f) => match f {
            ForgeItem::PropFn(p) => p.span,
            ForgeItem::Lemma(l) => l.span,
            ForgeItem::Proof(p) => p.span,
            ForgeItem::Witness(w) => w.span,
        },
    }
}

/// Splice. Overlapping edits are a bug, not something to skip quietly: a
/// dropped edit is silent text loss, which is the one outcome worth aborting on.
fn apply(src: &str, edits: &[Edit]) -> Result<String, String> {
    let mut out = String::with_capacity(src.len() + 64);
    let mut pos = 0usize;
    for e in edits {
        if e.start < pos {
            return Err(format!(
                "overlapping edits at {} (already at {}): {:?}",
                e.start, pos, e
            ));
        }
        out.push_str(&src[pos..e.start]);
        out.push_str(&e.text);
        pos = e.start + e.len;
    }
    out.push_str(&src[pos..]);
    Ok(out)
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "to-v3".to_string());
    let mut src = String::new();
    std::io::stdin().read_to_string(&mut src).unwrap();

    match mode.as_str() {
        "gate" => {
            let r = parse(&src);
            let items = r.program.items.len();
            eprintln!("items={} errors={}", items, r.errors.len());
            for e in r.errors.iter().take(3) {
                eprintln!("  {e:?}");
            }
            std::process::exit(if r.errors.is_empty() && items > 0 { 0 } else { 3 });
        }
        "count" => {
            // Clause sites straight from the lexer: a `req` token is a clause
            // and an identifier spelled `req` is not, which is the ambiguity a
            // textual count cannot resolve.
            let (toks, _) = tokenize(&src);
            let (mut req, mut ens, mut fx, mut inv, mut dec) = (0, 0, 0, 0, 0);
            for t in &toks {
                match t.kind {
                    TokKind::Req => req += 1,
                    TokKind::Ens => ens += 1,
                    TokKind::Fx => fx += 1,
                    TokKind::Inv => inv += 1,
                    TokKind::Dec => dec += 1,
                    _ => {}
                }
            }
            let r = parse(&src);
            // Trivial clauses, from the parser's verbatim clause text.
            let (mut contracts, mut reqtrue, mut enstrue) = (0, 0, 0);
            for it in &r.program.items {
                if let thermite_syntax::Item::Fn(f) = it {
                    contracts += 1;
                    if f.contract.req.text.trim() == "true" { reqtrue += 1; }
                    for e in &f.contract.ens {
                        if e.text.trim() == "true" { enstrue += 1; }
                    }
                }
            }
            println!("req {req}\nens {ens}\nfx {fx}\ninv {inv}\ndec {dec}\nitems {}\ncontracts {contracts}\nreqtrue {reqtrue}\nenstrue {enstrue}", r.program.items.len());
        }
        "edits" => {
            let (es, items, errs) = edits(&src);
            eprintln!("items={items} errors={errs}");
            for e in es {
                println!("{:6} +{:<3} {:<12} {:?}", e.start, e.len, e.note, e.text);
            }
        }
        _ => {
            let (es, _items, errs) = edits(&src);
            if errs > 0 {
                eprintln!("declined: {errs} parse error(s)");
                std::process::exit(3);
            }
            match apply(&src, &es) {
                Ok(out) => std::io::stdout().write_all(out.as_bytes()).unwrap(),
                Err(m) => {
                    eprintln!("error: {m}");
                    std::process::exit(4);
                }
            }
        }
    }
}
