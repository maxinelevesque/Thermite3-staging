//! Unit tests for compile-time effect-row subsumption
//! (`.design/lower/effect-subsumption.md` AC-1..AC-6). Expected values are
//! hand-derived from §4.1 + the design doc's lattice/subsumption rule
//! (R-CHAR-3 — never copied from the checker's own output).
//!
//! The corpus is entirely `fx pure` (the accept baseline, AC-2); the reject
//! cases (AC-4) are crafted fixtures whose AST is built directly in Rust so the
//! test does not depend on the parser parsing effectful `fx` rows. (The parser
//! does parse effectful rows — see `parser_parses_effectful_rows` — but the
//! checker operates on the AST regardless of how it was built, so the fixtures
//! are authoritative.) `unwrap` is fine here — `tests/` is not gated.

use thermite_lower::{analyze_effects, check_effects, subsumes, LowerError};
use thermite_syntax::ast::{
    Block, Clause, Contract, Effect, EffectRow, Expr, FnItem, Item, MatchArm, Pattern, PrimType,
    Program, SharedDeclItem, Type,
};
use thermite_syntax::lexer::Span;

// ---------------------------------------------------------------------------
// AST construction helpers (build effectful fixtures directly — AC-3/AC-4).
// ---------------------------------------------------------------------------

fn span() -> Span {
    Span::new(0, 1)
}

fn true_clause() -> Clause {
    Clause {
        expr: Expr::BoolLit(true),
        text: "true".to_string(),
        span: span(),
        bv: None,
    }
}

/// A `fn` named `name` with effect row `fx` whose body is `{ <calls>; }`,
/// one bare-expression `Call` per callee name in `calls`. This is the minimal
/// caller→callee call-graph fixture the checker walks (REQ-3).
fn fn_calling(name: &str, effects: EffectRow, calls: &[&str]) -> Item {
    let is_effectful_leaf = calls.is_empty() && effects != EffectRow::Pure;
    let stmts = calls
        .iter()
        .map(|callee| {
            thermite_syntax::ast::Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Path(vec![(*callee).to_string()])),
                args: vec![],
            })
        })
        .collect();
    Item::Fn(FnItem {
        slag: None,
        boundary: None,
        name: name.to_string(),
        params: vec![],
        ret: Type::Unit,
        contract: Contract {
            requires: true_clause(),
            ensures: vec![true_clause()],
            effects,
            interference: None,
        },
        measures: None,
        body: (!is_effectful_leaf).then_some(Block { stmts, tail: None }),
        holes: Vec::new(),
        refinements: Vec::new(),
        span: span(),
    })
}

fn boundary(name: &str, effects: EffectRow) -> Item {
    let Item::Fn(mut function) = fn_calling(name, effects, &[]) else {
        unreachable!()
    };
    function.body = None;
    Item::Fn(function)
}

fn shared(name: &str) -> Item {
    Item::SharedDecl(SharedDeclItem {
        name: name.to_string(),
        ty: Type::Prim(PrimType::U8),
        span: span(),
    })
}

fn pure() -> EffectRow {
    EffectRow::Pure
}

fn set(effects: Vec<Effect>) -> EffectRow {
    EffectRow::Set(effects)
}

fn all_atoms() -> EffectRow {
    set(vec![
        Effect::Read("x".to_string().into()),
        Effect::Write("y".to_string().into()),
        Effect::Net("d".to_string().into()),
        Effect::Alloc,
        Effect::Time,
        Effect::Rand,
        Effect::Panic,
        Effect::Diverge,
    ])
}

// ---------------------------------------------------------------------------
// AC-1: the lattice + subsumption unit law (hand-derived bools).
// ---------------------------------------------------------------------------

#[test]
fn lattice_law_reflexive() {
    // Reflexive: subsumes(R, R) for every row (a set is a subset of itself).
    let rows = vec![
        pure(),
        set(vec![Effect::Alloc]),
        set(vec![
            Effect::Read("x".to_string().into()),
            Effect::Write("y".to_string().into()),
        ]),
        all_atoms(),
    ];
    for r in &rows {
        assert!(subsumes(r, r), "subsumption must be reflexive for {r:?}");
    }
}

#[test]
fn lattice_law_pure_subsumes_only_pure() {
    // subsumes(Pure, R) iff R == Pure. Hand-derived: Pure is the bottom {} —
    // it subsumes only the empty set.
    assert!(subsumes(&pure(), &pure()), "pure subsumes pure");
    assert!(
        !subsumes(&pure(), &set(vec![Effect::Alloc])),
        "pure must NOT subsume {{alloc}}"
    );
    assert!(
        !subsumes(&pure(), &set(vec![Effect::Panic])),
        "pure must NOT subsume {{panic}}"
    );
    assert!(
        !subsumes(&pure(), &all_atoms()),
        "pure must NOT subsume the top row"
    );
    // An empty Set is extensionally Pure ({}), so pure subsumes it (hand-derived).
    assert!(
        subsumes(&pure(), &set(vec![])),
        "pure subsumes the empty Set (extensionally {{}})"
    );
}

#[test]
fn lattice_law_top_subsumes_everything() {
    // subsumes(all_atoms, R) for every R (the top of the powerset lattice).
    let rows = vec![
        pure(),
        set(vec![Effect::Alloc]),
        set(vec![Effect::Read("x".to_string().into())]),
        set(vec![Effect::Panic, Effect::Diverge]),
        all_atoms(),
    ];
    for r in &rows {
        assert!(subsumes(&all_atoms(), r), "top must subsume {r:?}");
    }
}

#[test]
fn lattice_law_table() {
    // Table-driven: (caller, callee, expected) hand-derived from subset inclusion
    // at the atom-kind level (OQ-1: Write(_) subsumes any Write(_)).
    let cases: Vec<(EffectRow, EffectRow, bool)> = vec![
        // superset subsumes subset
        (
            set(vec![
                Effect::Read("x".to_string().into()),
                Effect::Write("y".to_string().into()),
            ]),
            set(vec![Effect::Read("x".to_string().into())]),
            true,
        ),
        // subset does not subsume superset
        (
            set(vec![Effect::Read("x".to_string().into())]),
            set(vec![
                Effect::Read("x".to_string().into()),
                Effect::Net("d".to_string().into()),
            ]),
            false,
        ),
        // atom-kind level: Write("a") caller subsumes Write("b") callee (OQ-1)
        (
            set(vec![Effect::Write("a".to_string().into())]),
            set(vec![Effect::Write("b".to_string().into())]),
            true,
        ),
        // disjoint single atoms
        (set(vec![Effect::Alloc]), set(vec![Effect::Panic]), false),
        // equal singletons subsume
        (set(vec![Effect::Time]), set(vec![Effect::Time]), true),
    ];
    for (caller, callee, expected) in cases {
        assert_eq!(
            subsumes(&caller, &callee),
            expected,
            "subsumes({caller:?}, {callee:?}) should be {expected}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-2: corpus accepts (both corpus programs are `fx pure`).
// ---------------------------------------------------------------------------

fn parse_corpus(name: &str) -> Program {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("conformance")
        .join(format!("{name}.th"));
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read corpus {name}.th: {e}"));
    let parsed = thermite_syntax::parse(&src);
    assert!(
        parsed.errors.is_empty(),
        "corpus {name}.th must parse clean: {:?}",
        parsed.errors
    );
    parsed.program
}

#[test]
fn corpus_accepts() {
    // Both corpus programs are `fx pure` and call only the pure spec_sum /
    // combinators ⇒ check_effects returns Ok(()) (hand-derived from the corpus).
    for name in ["sum", "binary_search"] {
        let program = parse_corpus(name);
        assert_eq!(
            check_effects(&program),
            Ok(()),
            "corpus {name}.th (fx pure) must be accepted"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-3: crafted accept cases.
// ---------------------------------------------------------------------------

#[test]
fn crafted_accepts() {
    // {alloc} caller calling {alloc} callee — Ok.
    let prog = Program {
        items: vec![
            fn_calling("caller", set(vec![Effect::Alloc]), &["callee"]),
            fn_calling("callee", set(vec![Effect::Alloc]), &[]),
        ],
    };
    assert_eq!(
        check_effects(&prog),
        Ok(()),
        "{{alloc}} -> {{alloc}} accepts"
    );

    // {read(x), write(y)} caller calling {read(x)} callee — Ok (superset).
    let prog = Program {
        items: vec![
            shared("x"),
            shared("y"),
            fn_calling(
                "caller",
                set(vec![
                    Effect::Read("x".to_string().into()),
                    Effect::Write("y".to_string().into()),
                ]),
                &["callee"],
            ),
            fn_calling(
                "callee",
                set(vec![Effect::Read("x".to_string().into())]),
                &[],
            ),
        ],
    };
    assert_eq!(
        check_effects(&prog),
        Ok(()),
        "{{read,write}} -> {{read}} accepts"
    );

    // any (pure) caller calling a `spec fn` — Ok (spec fns are pure).
    let spec = Item::SpecFn(thermite_syntax::ast::SpecFnItem {
        name: "spec_helper".to_string(),
        params: vec![],
        ret: Type::Prim(PrimType::Bool),
        measures: true_clause(),
        body: Block {
            stmts: vec![],
            tail: Some(Box::new(Expr::BoolLit(true))),
        },
        span: span(),
    });
    let prog = Program {
        items: vec![fn_calling("caller", pure(), &["spec_helper"]), spec],
    };
    assert_eq!(
        check_effects(&prog),
        Ok(()),
        "pure -> spec fn accepts (spec fns are pure)"
    );

    // pure caller calling a combinator (`sorted`) — Ok (combinators are pure).
    let prog = Program {
        items: vec![fn_calling("caller", pure(), &["sorted"])],
    };
    assert_eq!(
        check_effects(&prog),
        Ok(()),
        "pure -> combinator accepts (combinators are pure)"
    );
}

// ---------------------------------------------------------------------------
// AC-4: crafted reject cases — the `missing` atom set (hand-derived).
// ---------------------------------------------------------------------------

fn single_error(prog: &Program) -> LowerError {
    match check_effects(prog) {
        Err(mut errs) => {
            assert_eq!(
                errs.len(),
                1,
                "expected exactly one violation, got {errs:?}"
            );
            errs.remove(0)
        }
        Ok(()) => panic!("expected a subsumption violation, got Ok"),
    }
}

#[test]
fn reject_pure_calling_alloc() {
    // pure caller calling {alloc} callee → missing: [Alloc].
    let prog = Program {
        items: vec![
            fn_calling("caller", pure(), &["callee"]),
            fn_calling("callee", set(vec![Effect::Alloc]), &[]),
        ],
    };
    match single_error(&prog) {
        LowerError::EffectNotSubsumed {
            caller,
            callee,
            missing,
            ..
        } => {
            assert_eq!(caller, "caller");
            assert_eq!(callee, "inferred transitive footprint");
            assert_eq!(
                missing,
                vec![Effect::Alloc],
                "missing must be exactly [Alloc]"
            );
        }
        other => panic!("expected EffectNotSubsumed, got {other:?}"),
    }
}

#[test]
fn reject_read_calling_read_net() {
    // {read(x)} caller calling {read(x), net(d)} callee → missing: [Net].
    let prog = Program {
        items: vec![
            fn_calling(
                "caller",
                set(vec![Effect::Read("x".to_string().into())]),
                &["callee"],
            ),
            fn_calling(
                "callee",
                set(vec![
                    Effect::Read("x".to_string().into()),
                    Effect::Net("d".to_string().into()),
                ]),
                &[],
            ),
        ],
    };
    match single_error(&prog) {
        LowerError::EffectNotSubsumed { missing, .. } => {
            assert_eq!(
                missing,
                vec![Effect::Net("d".into())],
                "missing must be exactly [Net]"
            );
        }
        other => panic!("expected EffectNotSubsumed, got {other:?}"),
    }
}

#[test]
fn missing_net_diagnostic_names_basis_entry_and_frame() {
    let prog = Program {
        items: vec![
            fn_calling("caller", pure(), &["callee"]),
            fn_calling(
                "callee",
                set(vec![Effect::Net("d".to_string().into())]),
                &[],
            ),
        ],
    };
    let error = single_error(&prog);
    match &error {
        LowerError::EffectNotSubsumed { missing, .. } => {
            assert_eq!(missing, &vec![Effect::Net("d".into())]);
        }
        other => panic!("expected EffectNotSubsumed, got {other:?}"),
    }

    let diagnostic = error.to_string();
    assert!(diagnostic.contains("net (basis: Combination"));
    assert!(diagnostic.contains("State"));
    assert!(diagnostic.contains("Io"));
    assert!(diagnostic.contains("frame: Conjunction"));
    assert!(diagnostic.contains("MayModifyOnly"));
    assert!(diagnostic.contains("UnconstrainedValueNoRegion"));
}

#[test]
fn reject_pure_calling_panic() {
    // pure caller calling {panic} callee → missing: [Panic].
    let prog = Program {
        items: vec![
            fn_calling("caller", pure(), &["callee"]),
            fn_calling("callee", set(vec![Effect::Panic]), &[]),
        ],
    };
    match single_error(&prog) {
        LowerError::EffectNotSubsumed { missing, .. } => {
            assert_eq!(
                missing,
                vec![Effect::Panic],
                "missing must be exactly [Panic]"
            );
        }
        other => panic!("expected EffectNotSubsumed, got {other:?}"),
    }
}

#[test]
fn reject_accumulates_all_violations() {
    // RFC-9 reports one normalized missing footprint per caller after
    // propagation, rather than duplicating errors at individual call sites.
    let prog = Program {
        items: vec![
            fn_calling("caller", pure(), &["a", "b"]),
            fn_calling("a", set(vec![Effect::Alloc]), &[]),
            fn_calling("b", set(vec![Effect::Panic]), &[]),
        ],
    };
    match check_effects(&prog) {
        Err(errs) => match &errs[..] {
            [LowerError::EffectNotSubsumed { missing, .. }] => assert_eq!(
                missing,
                &vec![Effect::Alloc, Effect::Panic],
                "both transitive operations must be reported"
            ),
            other => panic!("expected one normalized footprint error, got {other:?}"),
        },
        Ok(()) => panic!("expected two violations"),
    }
}

// ---------------------------------------------------------------------------
// AC-5: no panic; unresolved callee is a no-op; deep nesting is structured.
// ---------------------------------------------------------------------------

#[test]
fn unresolved_callee_is_noop() {
    // A pure caller calling an unknown name (neither fn, spec fn, nor combinator)
    // is a no-op; the #2 validator owns unknown-name rejection (AC-5).
    let prog = Program {
        items: vec![fn_calling("caller", pure(), &["totally_unknown_fn"])],
    };
    assert_eq!(
        check_effects(&prog),
        Ok(()),
        "an unresolved callee must be a no-op, not an error or panic"
    );
}

#[test]
fn empty_program_is_ok() {
    assert_eq!(check_effects(&Program { items: vec![] }), Ok(()));
}

#[test]
fn call_propagation_preserves_region_identity() {
    let program = Program {
        items: vec![
            fn_calling("caller", set(vec![Effect::Write("db".into())]), &["callee"]),
            fn_calling("callee", set(vec![Effect::Write("log".into())]), &[]),
        ],
    };

    match single_error(&program) {
        LowerError::EffectNotSubsumed { missing, .. } => {
            assert_eq!(missing, vec![Effect::Write("log".into())]);
        }
        other => panic!("expected region-sensitive rejection, got {other:?}"),
    }
}

#[test]
fn analysis_reaches_a_fixed_point_through_recursive_calls() {
    let program = Program {
        items: vec![
            fn_calling("a", set(vec![Effect::Alloc]), &["b"]),
            fn_calling("b", set(vec![Effect::Alloc]), &["a", "heap_boundary"]),
            boundary("heap_boundary", set(vec![Effect::Alloc])),
        ],
    };

    let analysis = analyze_effects(&program).expect("declared rows cover the fixed point");
    assert_eq!(
        analysis.footprints["a"],
        std::collections::BTreeSet::from([Effect::Alloc])
    );
    assert_eq!(analysis.footprints["b"], analysis.footprints["a"]);
    assert!(analysis.warnings.is_empty());
}

#[test]
fn analysis_returns_deterministic_excess_warnings() {
    let program = Program {
        items: vec![
            shared("db"),
            fn_calling(
                "overdeclared",
                set(vec![Effect::Write("db".into()), Effect::Alloc]),
                &["heap_boundary"],
            ),
            boundary("heap_boundary", set(vec![Effect::Alloc])),
        ],
    };

    let analysis = analyze_effects(&program).expect("over-conservative rows warn");
    assert_eq!(analysis.warnings.len(), 1);
    assert_eq!(analysis.warnings[0].function, "overdeclared");
    assert_eq!(
        analysis.warnings[0].excess,
        vec![Effect::Write("db".into())]
    );
}

#[test]
fn production_analysis_rejects_concurrent_ancestry_conflict() {
    let parsed = thermite_syntax::parse(
        "struct State { child: u64 }\n\
         shared state: State\n\
         #[boundary(\"ext::left\")] fn left() -> u64 ! write(state) requires nothing ensures result == 0;\n\
         #[boundary(\"ext::right\")] fn right() -> u64 ! read(state.child) requires nothing ensures result == 0;\n\
         concurrent pair { left, right }",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);

    let errors = analyze_effects(&parsed.program).expect_err("overlapping write/read must reject");
    assert!(errors.iter().any(|error| matches!(
        error,
        LowerError::EffectAnalysis { detail, .. }
            if detail.contains("concurrent `pair`")
                && detail.contains("`left`")
                && detail.contains("`right`")
                && detail.contains("state")
    )));
}

#[test]
fn shared_metadata_enables_global_region_resolution_without_concurrency() {
    let parsed = thermite_syntax::parse(
        "shared known: u8\n\
         #[boundary(\"ext::bad\")] fn bad() -> u64 ! read(missing) requires nothing ensures result == 0;",
    );
    assert!(parsed.is_clean(), "parse errors: {:?}", parsed.errors);
    let errors = analyze_effects(&parsed.program).expect_err("unknown root must reject");
    assert!(errors.iter().any(|error| matches!(
        error,
        LowerError::EffectAnalysis { detail, .. }
            if detail.contains("UnknownRegionRoot") && detail.contains("missing")
    )));
}

#[test]
fn direct_effectful_intrinsic_requires_its_canonical_effect() {
    let mut item = fn_calling("grow", pure(), &[]);
    let Item::Fn(function) = &mut item else {
        unreachable!()
    };
    function.body = Some(Block {
        stmts: vec![thermite_syntax::Stmt::Expr(Expr::MethodCall {
            receiver: Box::new(Expr::Path(vec!["values".into()])),
            name: "push".into(),
            args: vec![Expr::IntLit {
                value: 1,
                raw: "1".into(),
            }],
        })],
        tail: None,
    });

    let errors = analyze_effects(&Program { items: vec![item] })
        .expect_err("push has the canonical allocation footprint");
    assert!(errors.iter().any(|error| matches!(
        error,
        LowerError::EffectNotSubsumed { missing, .. } if missing == &vec![Effect::Alloc]
    )));
}

#[test]
fn owned_literal_and_collection_constructor_infer_allocation() {
    for expression in [
        Expr::StrLit("owned".into()),
        Expr::Call {
            callee: Box::new(Expr::Path(vec!["Vec".into(), "new".into()])),
            args: vec![],
        },
        Expr::Call {
            callee: Box::new(Expr::Path(vec!["String".into(), "from_byte".into()])),
            args: vec![Expr::IntLit {
                value: 65,
                raw: "65".into(),
            }],
        },
        Expr::Call {
            callee: Box::new(Expr::Path(vec!["Box".into(), "new".into()])),
            args: vec![Expr::IntLit {
                value: 1,
                raw: "1".into(),
            }],
        },
    ] {
        let mut item = fn_calling("construct", pure(), &[]);
        let Item::Fn(function) = &mut item else {
            unreachable!()
        };
        function.body = Some(Block {
            stmts: vec![thermite_syntax::Stmt::Expr(expression)],
            tail: None,
        });
        let errors = analyze_effects(&Program { items: vec![item] })
            .expect_err("owned construction requires alloc");
        assert!(matches!(
            &errors[..],
            [LowerError::EffectNotSubsumed { missing, .. }]
                if missing == &vec![Effect::Alloc]
        ));
    }
}

#[test]
fn deeply_nested_body_returns_result_not_panic() {
    // Build a body whose tail is a deeply-nested Ref chain (well past
    // MAX_WALK_DEPTH=256) and assert check_effects returns a Result (a TooDeep
    // error), never overflowing the native stack (AC-5).
    let mut expr = Expr::Path(vec!["x".to_string()]);
    for _ in 0..2000 {
        expr = Expr::Ref {
            mutable: false,
            expr: Box::new(expr),
        };
    }
    let item = Item::Fn(FnItem {
        slag: None,
        boundary: None,
        name: "deep".to_string(),
        params: vec![],
        ret: Type::Unit,
        contract: Contract {
            requires: true_clause(),
            ensures: vec![true_clause()],
            effects: pure(),
            interference: None,
        },
        measures: None,
        body: Some(Block {
            stmts: vec![],
            tail: Some(Box::new(expr)),
        }),
        holes: Vec::new(),
        refinements: Vec::new(),
        span: span(),
    });
    let prog = Program { items: vec![item] };
    match check_effects(&prog) {
        Err(errs) => assert!(
            errs.iter().any(|e| matches!(e, LowerError::TooDeep { .. })),
            "deep nesting must surface as TooDeep, got {errs:?}"
        ),
        Ok(()) => panic!("expected a TooDeep structured error for a 2000-deep body"),
    }
}

#[test]
fn deeply_nested_match_guard_returns_too_deep_not_stack_overflow() {
    let mut guard = Expr::Path(vec!["x".to_string()]);
    for _ in 0..2000 {
        guard = Expr::Ref {
            mutable: false,
            expr: Box::new(guard),
        };
    }
    let mut item = fn_calling("deep_guard", pure(), &[]);
    let Item::Fn(function) = &mut item else {
        unreachable!()
    };
    function.body = Some(Block {
        stmts: vec![],
        tail: Some(Box::new(Expr::Match {
            scrutinee: Box::new(Expr::IntLit {
                value: 0,
                raw: "0".into(),
            }),
            arms: vec![MatchArm {
                pattern: Pattern::Wildcard,
                guard: Some(guard),
                body: Expr::IntLit {
                    value: 0,
                    raw: "0".into(),
                },
            }],
        })),
    });
    let errors = analyze_effects(&Program { items: vec![item] })
        .expect_err("deep guard must fail with a structured bound");
    assert!(errors
        .iter()
        .any(|error| matches!(error, LowerError::TooDeep { limit, .. } if *limit == 256)));
}

// ---------------------------------------------------------------------------
// Observation: the parser does parse effectful `fx` rows (not a blocker).
// ---------------------------------------------------------------------------

#[test]
fn parser_parses_effectful_rows() {
    // Confirms the thermite-syntax parser accepts a non-pure `fx` row (so the
    // checker is not blocked on a parser gap; the fixtures above build AST
    // directly only for hermeticity, not out of necessity). Hand-derived
    // expected row: {alloc}.
    let src = "fn f() -> ()\n  ! alloc
  requires true\n  ensures true\n{\n}\n";
    let parsed = thermite_syntax::parse(src);
    assert!(
        parsed.errors.is_empty(),
        "parser must accept `fx alloc`: {:?}",
        parsed.errors
    );
    match &parsed.program.items[0] {
        Item::Fn(f) => assert_eq!(f.contract.effects, set(vec![Effect::Alloc])),
        other => panic!("expected a fn item, got {other:?}"),
    }
}
