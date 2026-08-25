//! The live cert-oracle for `forge check` (`goal.md` verification model (B);
//! `.design/forge/check.md` AC-1/AC-2/AC-3). It drives the built `forge` binary
//! (`.design/forge/cli.md` Verification: "a CLI integration test drives the built
//! `forge` binary") with `check --json`, parses the emitted certificate JSON, and
//! asserts its deterministic fields match the golden `conformance/<name>.cert.json`
//! — `item`, `level`, `effects`, `slag` — under the forward-declaration contract
//! (`conformance/README.md`): `contract_quality.*` and `solver_time_ms` are not
//! asserted (R-CHAR-3 — expected values trace to the golden cert, never to forge's
//! own output).
//!
//! Driving the binary (rather than calling a library API) keeps `forge` a pure
//! `bin` crate (no `lib.rs`) and exercises the real REQ-4/REQ-5 stream + exit-code
//! surface end to end.
//!
//! These checks run verus. If verus is absent they skip with a logged note (mirroring
//! `thermite-lower/tests/lower_conformance.rs`'s Option-resolve + eprintln-skip)
//! — never panic on a missing solver. `tests/` is not anti-pattern-gated, so
//! `unwrap`/`expect` are fine here.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("conformance")
}

/// Path to the freshly built `forge` binary (cargo sets this for integration
/// tests).
fn forge_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_forge"))
}

/// `true` iff verus can be located (`VERUS_BIN`, then PATH, then
/// `~/.local/bin/verus`) — mirrors `lower_conformance.rs`. Skips with a logged note otherwise.
fn verus_present() -> bool {
    if let Ok(p) = std::env::var("VERUS_BIN") {
        if Path::new(&p).exists() {
            return true;
        }
    }
    if let Ok(out) = Command::new("which").arg("verus").output() {
        if out.status.success() && !String::from_utf8_lossy(&out.stdout).trim().is_empty() {
            return true;
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if PathBuf::from(home).join(".local/bin/verus").exists() {
            return true;
        }
    }
    false
}

/// Run `forge check <file> --json`, returning (exit_code, parsed JSON array of
/// certificates). stdout under `--json` must be a single JSON document (AC-2).
fn run_check_json(file: &Path) -> (Option<i32>, Vec<Value>) {
    let out = Command::new(forge_bin())
        .arg("check")
        .arg(file)
        .arg("--json")
        .output()
        .unwrap_or_else(|e| panic!("spawn forge: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let value: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!(
            "forge --json stdout must be one JSON document: {e}\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    let arr = value
        .as_array()
        .unwrap_or_else(|| panic!("forge --json must emit a JSON array of certs: {value}"))
        .clone();
    (out.status.code(), arr)
}

fn run_check_json_with_env(file: &Path, key: &str, value: &str) -> (Option<i32>, Vec<Value>) {
    let out = Command::new(forge_bin())
        .arg("check")
        .arg(file)
        .arg("--json")
        .env("THERMITE_EPR_CACHE_DISABLE", "1")
        .env(key, value)
        .output()
        .unwrap_or_else(|e| panic!("spawn forge: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let value: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!(
            "forge --json stdout must be one JSON document: {e}\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (
        out.status.code(),
        value.as_array().expect("certificate array").clone(),
    )
}

/// The golden certificate JSON for `<name>` (the external oracle, R-CHAR-3).
fn golden_cert(name: &str) -> Value {
    let path = corpus_dir().join(format!("{name}.cert.json"));
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read golden cert {}: {e}", path.display()));
    serde_json::from_str(&src).unwrap_or_else(|e| panic!("parse golden cert {name}: {e}"))
}

fn find_cert(certs: &[Value], item: &str) -> Value {
    certs
        .iter()
        .find(|c| c.get("item").and_then(|v| v.as_str()) == Some(item))
        .unwrap_or_else(|| panic!("no certificate for item `{item}` in {certs:?}"))
        .clone()
}

// ---- AC-1: sum → L3, deterministic fields == golden -----------------------

#[test]
fn sum_cert_matches_golden_deterministic_subset() {
    if !verus_present() {
        eprintln!(
            "SKIP: verus not available — `forge check sum.th` cert-oracle not run \
             (set VERUS_BIN or install verus on PATH)."
        );
        return;
    }
    let (code, certs) = run_check_json(&corpus_dir().join("sum.th"));
    assert_eq!(code, Some(0), "a fully verified sum must exit 0");
    let sum = find_cert(&certs, "sum");
    let golden = golden_cert("sum");

    // The deterministic subset must match the golden oracle; not
    // contract_quality.* / solver_time_ms (forward-declared / non-det).
    assert_eq!(sum["item"], golden["item"]);
    assert_eq!(sum["item"], Value::from("sum"));
    assert_eq!(sum["level"], Value::from("L3"), "sum must verify L3");
    assert_eq!(sum["level"], golden["level"]);
    assert_eq!(sum["effects"], golden["effects"]);
    assert_eq!(sum["effects"], serde_json::json!(["pure"]));
    assert_eq!(sum["slag"], golden["slag"]);
    assert_eq!(sum["slag"], Value::from(false));

    // Per-obligation list: present and all discharged.
    let obs = sum["obligations"]
        .as_array()
        .expect("obligations array present");
    assert!(!obs.is_empty(), "discharged cert carries a non-empty list");
    assert!(
        obs.iter()
            .all(|o| o.get("status").and_then(|s| s.as_str()) == Some("discharged")),
        "every sum obligation discharged: {obs:?}"
    );
}

// ---- AC-2: binary_search → L3 (level only; no golden cert yet) ------------

#[test]
fn binary_search_is_l3() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — `forge check binary_search.th` not run.");
        return;
    }
    let (code, certs) = run_check_json(&corpus_dir().join("binary_search.th"));
    assert_eq!(code, Some(0));
    let bs = find_cert(&certs, "binary_search");
    assert_eq!(
        bs["level"],
        Value::from("L3"),
        "binary_search must verify L3"
    );
    assert_eq!(bs["effects"], serde_json::json!(["pure"]));
}

// ---- AC-3: broken contract → reported non-L3 + counterexample -------------

#[test]
fn broken_contract_is_reported_failure_with_counterexample() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — broken-contract cert not run.");
        return;
    }
    // `ens result == x + 2` but the body returns `x + 1`: parses, validates,
    // effect-checks, and lowers — only the SMT proof fails. Written to a
    // temp `.th` fixture (no committed broken corpus entry needed).
    let fixture = std::env::temp_dir().join(format!("forge_broken_{}.th", std::process::id()));
    std::fs::write(
        &fixture,
        "fn add_one(x: u64) -> u64\n  ! pure
  requires x < 1000\n  ensures result == x + 2\n{\n  x + 1\n}\n",
    )
    .expect("write broken fixture");

    let (code, certs) = run_check_json(&fixture);
    let _ = std::fs::remove_file(&fixture);

    // A reported verification failure: nonzero exit (the verification-failure
    // code, not the environment code), but a valid cert document on stdout.
    assert_eq!(
        code,
        Some(1),
        "a reported verification failure exits with the verification-failure code"
    );
    let cert = find_cert(&certs, "add_one");
    assert_ne!(
        cert["level"],
        Value::from("L3"),
        "a false ens must NOT certify L3"
    );
    let obs = cert["obligations"].as_array().expect("obligations present");
    let failures: Vec<&Value> = obs
        .iter()
        .filter(|o| o.get("status").and_then(|s| s.as_str()) == Some("failed"))
        .collect();
    assert!(
        !failures.is_empty(),
        "the cert must carry a per-obligation failure (the counterexample): {obs:?}"
    );
    // The failure names the obligation + carries a source location / diagnostic
    // (§5.1 "counterexamples, not adjectives"), never a bare adjective.
    let f = failures[0];
    let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        !name.is_empty() && name != "verification failed",
        "failure must name the obligation, not a bare adjective: {f:?}"
    );
    assert!(
        f.get("location").is_some() || f.get("diagnostic").is_some(),
        "failure must carry a source location or diagnostic witness: {f:?}"
    );
}

// ---- C7 (#100): the external cert oracle for option_result + parse_u64 -----

/// Assert the deterministic stable-subset of `<corpus>`'s `<item>` certificate
/// matches its golden `conformance/<cert_stem>.cert.json` (R-CHAR-3): `item`,
/// `level`, `contract_quality.tautology`, `contract_quality.vacuous_precondition`,
/// `effects`, `slag`. Not `contract_quality.mutants_killed` / `solver_time_ms`
/// (tool-computed / non-det — `oracle_subset`, §5.3). `level` must be `L3`. The
/// golden cert is keyed on the corpus stem (one `.cert.json` per `.th`), so a
/// multi-item corpus has a single golden cert naming one representative `item`.
fn assert_stable_subset_matches_golden(corpus: &str, cert_stem: &str, item: &str) {
    let (code, certs) = run_check_json(&corpus_dir().join(corpus));
    assert_eq!(code, Some(0), "{corpus} must verify (exit 0)");
    let cert = find_cert(&certs, item);
    let golden = golden_cert(cert_stem);

    assert_eq!(
        cert["item"], golden["item"],
        "{item}: item must match golden"
    );
    assert_eq!(cert["item"], Value::from(item));
    assert_eq!(
        cert["level"], golden["level"],
        "{item}: level must match golden"
    );
    assert_eq!(cert["level"], Value::from("L3"), "{item} must verify L3");
    assert_eq!(
        cert["contract_quality"]["tautology"], golden["contract_quality"]["tautology"],
        "{item}: tautology must match golden"
    );
    assert_eq!(
        cert["contract_quality"]["vacuous_precondition"],
        golden["contract_quality"]["vacuous_precondition"],
        "{item}: vacuous_precondition must match golden"
    );
    assert_eq!(
        cert["effects"], golden["effects"],
        "{item}: effects must match golden"
    );
    assert_eq!(
        cert["slag"], golden["slag"],
        "{item}: slag must match golden"
    );
}

/// C7 / `.design/basis/09-option-result.md` AC-4 (#100): `parse_valid` certifies L3
/// against the committed `conformance/parse_u64.cert.json` oracle. A valid in-range
/// digit string proves `result is Some` via parse_u64's strengthened contract.
#[test]
fn parse_valid_cert_matches_golden_deterministic_subset() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — parse_u64.th cert-oracle not run.");
        return;
    }
    assert_stable_subset_matches_golden("parse_u64.th", "parse_u64", "parse_valid");
}

/// C7 / `.design/basis/09-option-result.md` AC-1 (#100): `make_some` certifies L3
/// against the committed `conformance/option_result.cert.json` oracle (built-in
/// `Some` construction + payload-in-contract). The sibling L3 items (`small`,
/// `ok_seven`, `checked`) are covered in `option_result_conformance.rs`.
#[test]
fn make_some_cert_matches_golden_deterministic_subset() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — option_result.th cert-oracle not run.");
        return;
    }
    assert_stable_subset_matches_golden("option_result.th", "option_result", "make_some");
}

/// C12 / `.design/basis/13-map.md` AC-3 (#124): `map_kv.th::has_key` certifies L3
/// against the committed `conformance/map_kv.cert.json` oracle (the R-CHAR-3
/// hand-derived cert — `has_key -> bool ens result == m.contains_key(k)`, L3 pure,
/// non-vacuous, mutation-strong). `contains_key` in `BUILTIN_METHODS` admits the
/// §4.2-caged accessor; the lowerer maps spec-position `contains_key` to the TMap
/// wrapper's `spec_contains_key`. The stable subset (item/level/tautology/
/// vacuous_precondition/effects/slag) must match the committed oracle; the
/// insert-then-get round trip and absent→None cases are pinned at the verus
/// codegen-grounding level in `map_conformance.rs`.
///
/// Unlike the single-L3-item corpora above, `map_kv.th` is multi-item and its
/// thin runnable-core fns (`build_one`/`demo`/`lookup_absent`) carry the §7-partial
/// caveat the oracle's `note` documents (a `Map`-return has no scoreable scalar-zero
/// mutant; a `None`-only contract is the #101 partial class) — they certify below
/// L3, so the whole-file exit is not 0. We assert the `has_key` cert's stable subset
/// directly (the mutation-strong L3 anchor), mirroring `map_conformance.rs::ac3`,
/// rather than the whole-file-exit-0 helper.
#[test]
fn has_key_cert_matches_golden_deterministic_subset() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — map_kv.th cert-oracle not run.");
        return;
    }
    let (_code, certs) = run_check_json(&corpus_dir().join("map_kv.th"));
    let cert = find_cert(&certs, "has_key");
    let golden = golden_cert("map_kv");

    assert_eq!(
        cert["item"], golden["item"],
        "has_key: item must match golden"
    );
    assert_eq!(cert["item"], Value::from("has_key"));
    assert_eq!(
        cert["level"], golden["level"],
        "has_key: level must match golden"
    );
    assert_eq!(cert["level"], Value::from("L3"), "has_key must verify L3");
    assert_eq!(
        cert["contract_quality"]["tautology"], golden["contract_quality"]["tautology"],
        "has_key: tautology must match golden"
    );
    assert_eq!(
        cert["contract_quality"]["vacuous_precondition"],
        golden["contract_quality"]["vacuous_precondition"],
        "has_key: vacuous_precondition must match golden"
    );
    assert_eq!(
        cert["effects"], golden["effects"],
        "has_key: effects must match golden"
    );
    assert_eq!(cert["effects"], serde_json::json!(["pure"]));
    assert_eq!(
        cert["slag"], golden["slag"],
        "has_key: slag must match golden"
    );
}

#[test]
fn rfc10_shared_state_certifies_through_the_production_route() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — RFC-10 production-route anchor not run.");
        return;
    }
    let (code, certs) = run_check_json(&corpus_dir().join("shared_state_rfc10.th"));
    assert_eq!(code, Some(0), "the RFC-10 anchor must certify end to end");
    let cert = find_cert(&certs, "read_state");
    assert_eq!(cert["level"], Value::from("L3"));
    assert_eq!(
        cert["effects"],
        serde_json::json!(["owns(gate)", "read(state.n)"])
    );
}

#[test]
fn generated_rfc10_positions_reach_provider_free_forge_check() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — generated RFC-10 Forge matrix not run.");
        return;
    }
    let mut payload =
        "if flag { holding gate { state.n; 1 } } else { holding gate { state.n; 2 } }".to_string();
    let mut loop_payload = "if true { holding gate { state.n; flag } } else { flag }".to_string();
    // The lowerer matrix separately uses depth 24. Keep the provider-free cells
    // below the production Lean replay's 60-second operational timeout.
    for _ in 0..8 {
        payload = format!("if true {{ {payload} }} else {{ 0 }}");
        loop_payload = format!("if true {{ {loop_payload} }} else {{ flag }}");
    }
    let positions = [
        ("initializer", format!("let x: u64 = {payload}; x")),
        (
            "assignment_value",
            format!("let mut x: u64 = 0; x = {payload}; x"),
        ),
        ("return_value", format!("return {payload};")),
        ("tail", payload.clone()),
        (
            "if_condition",
            format!("if {payload} == 1 {{ 1 }} else {{ 2 }}"),
        ),
        (
            "match_scrutinee",
            format!("match {payload} {{ 1 => 1, _ => 2 }}"),
        ),
        (
            "match_guard",
            format!("match 0 {{ _ if {payload} == 1 => 1, _ => 2 }}"),
        ),
        ("call_argument", format!("id({payload})")),
        ("tuple_element", format!("({payload}, 0).0")),
        (
            "loop_test",
            format!(
                "let mut out: u64 = 2; while {loop_payload} keeps (flag && (out == 1 || out == 2)) || (!flag && out == 2) measures 1 as u64 {{ out = 1; break; }} out"
            ),
        ),
    ];
    for (name, body) in positions {
        let source = format!(
            "struct State {{ n: u64 }} keeps n < 10\n\
             shared state: State\n\
             lock gate guards state\n\
             fn id(x: u64) -> u64 ! pure requires true ensures result == x {{ x }}\n\
             fn probe_{name}(flag: bool) -> u64 ! owns(gate), read(state.n)\n\
             requires true ensures (flag && result == 1) || (!flag && result == 2) {{ {body} }}\n"
        );
        let fixture = std::env::temp_dir().join(format!(
            "forge_rfc10_matrix_{}_{name}.th",
            std::process::id()
        ));
        std::fs::write(&fixture, source).expect("write RFC-10 matrix fixture");
        let l2 = Command::new(forge_bin())
            .arg("check")
            .arg(&fixture)
            .args(["--level", "l2"])
            .output()
            .expect("spawn Forge L2 RFC-10 matrix cell");
        assert_eq!(l2.status.code(), Some(2), "probe_{name}");
        assert!(
            String::from_utf8_lossy(&l2.stderr).contains("RFC-10 shared-state L2 Kani harness"),
            "probe_{name}: {}",
            String::from_utf8_lossy(&l2.stderr)
        );
        let (code, certs) = run_check_json(&fixture);
        let _ = std::fs::remove_file(&fixture);
        let cert = find_cert(&certs, &format!("probe_{name}"));
        if name == "loop_test" {
            assert_eq!(code, Some(1), "probe_{name}: {certs:?}");
            assert_eq!(cert["level"], Value::from("L0"), "probe_{name}");
            assert!(
                cert["obligations"].as_array().is_some_and(|rows| rows
                    .iter()
                    .any(|row| { row["diagnostic"] == "error: postcondition not satisfied" })),
                "loop-test proof failure must remain explicit: {cert:?}"
            );
        } else {
            assert_eq!(code, Some(0), "probe_{name} must certify: {certs:?}");
            assert_eq!(cert["level"], Value::from("L3"), "probe_{name}");
        }
    }
}

#[test]
fn generated_rfc10_invariant_breaks_fail_provider_free_forge_check() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — RFC-10 invariant-break matrix not run.");
        return;
    }
    let payload = "if true { holding gate { state.n = 10; state.n } 0 } else { 0 }";
    let positions = [
        ("initializer", format!("let x: u64 = {payload}; x")),
        (
            "assignment_value",
            format!("let mut x: u64 = 0; x = {payload}; x"),
        ),
        ("return_value", format!("return {payload};")),
        ("tail", payload.to_string()),
        (
            "if_condition",
            format!("if {payload} == 0 {{ 0 }} else {{ 1 }}"),
        ),
        ("match_scrutinee", format!("match {payload} {{ _ => 0 }}")),
        (
            "match_guard",
            format!("match 0 {{ _ if {payload} == 0 => 0, _ => 0 }}"),
        ),
        ("call_argument", format!("id({payload})")),
        ("tuple_element", format!("({payload}, 0).0")),
        (
            "loop_test",
            format!("while {payload} == 0 keeps true measures 1 as u64 {{ break; }} 0"),
        ),
    ];
    for (name, body) in positions {
        let source = format!(
            "struct State {{ n: u64 }} keeps n < 10\n\
             shared state: State\n\
             lock gate guards state\n\
             fn id(x: u64) -> u64 ! pure requires true ensures result == x {{ x }}\n\
             fn break_invariant_{name}() -> u64 ! owns(gate), read(state.n), write(state.n)\n\
             requires true ensures result < 10 {{ {body} }}\n"
        );
        let fixture = std::env::temp_dir().join(format!(
            "forge_rfc10_invariant_break_{}_{name}.th",
            std::process::id()
        ));
        std::fs::write(&fixture, source).expect("write RFC-10 invariant-break fixture");
        let (code, certs) = run_check_json(&fixture);
        let _ = std::fs::remove_file(&fixture);
        let cert = find_cert(&certs, &format!("break_invariant_{name}"));
        assert_eq!(code, Some(1), "break_invariant_{name}: {certs:?}");
        assert_eq!(cert["level"], Value::from("L0"), "break_invariant_{name}");
        assert!(
            cert["obligations"]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|row| {
                    row["diagnostic"]
                        .as_str()
                        .is_some_and(|diagnostic| diagnostic.contains("precondition not satisfied"))
                })),
            "invariant-breaking close must remain an explicit proof failure: {cert:?}"
        );
    }
}

#[test]
fn explicit_l2_cleanly_refuses_rfc10_without_losing_metadata() {
    let out = Command::new(forge_bin())
        .arg("check")
        .arg(corpus_dir().join("shared_state_rfc10.th"))
        .args(["--level", "l2"])
        .output()
        .expect("spawn forge");
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("RFC-10 shared-state L2 Kani harness"),
        "{stderr}"
    );
    assert!(!stderr.contains("unknown lock"), "{stderr}");
}

#[test]
fn every_root_corpus_item_preserves_its_frozen_certification_level() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — full corpus certification sweep not run.");
        return;
    }
    let mut baseline: serde_json::Map<String, Value> = serde_json::from_str(
        &std::fs::read_to_string(corpus_dir().join("forge-corpus-baseline.json")).unwrap(),
    )
    .unwrap();
    let provenance = baseline
        .remove("__provenance")
        .expect("baseline provenance");
    assert_eq!(provenance["kind"], "post-fix-round freeze");
    for (file, expected) in baseline {
        let (_code, certs) = run_check_json(&corpus_dir().join(&file));
        let expected = expected.as_object().unwrap();
        assert_eq!(
            certs.len(),
            expected.len(),
            "certificate count drifted for {file}"
        );
        for (item, expectation) in expected {
            let cert = find_cert(&certs, item);
            let (level, cause) = match expectation {
                Value::String(_) => (expectation.clone(), None),
                Value::Object(fields) => (
                    fields.get("level").expect("baseline level").clone(),
                    fields.get("cause").cloned(),
                ),
                other => panic!("invalid baseline entry for {file}::{item}: {other}"),
            };
            if cert["level"] == level {
                if let Some(cause) = cause {
                    assert_eq!(
                        cert.pointer("/reject/cause").unwrap_or(&Value::Null),
                        &cause,
                        "reject cause drifted for {file}::{item}"
                    );
                }
                continue;
            }
            let environment_only = expectation
                .as_object()
                .and_then(|fields| fields.get("fallback_level"))
                == Some(&cert["level"]);
            assert!(
                environment_only,
                "certification level drifted for {file}::{item}: expected {level}, got {}",
                cert["level"]
            );
        }
    }
}

#[test]
fn unavailable_epr_tools_preserve_the_clean_l3_base() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — EPR fallback routes not run.");
        return;
    }
    let fixture = corpus_dir().join("string_demo.th");
    for variable in ["THERMITE_EPR_CADICAL", "THERMITE_EPR_DRAT_TRIM"] {
        let (code, certs) = run_check_json_with_env(&fixture, variable, "/nonexistent");
        let cert = find_cert(&certs, "greeting_len");
        assert_eq!(code, Some(0), "{variable} absence must preserve success");
        assert_eq!(
            cert["level"],
            Value::from("L3"),
            "{variable} absence must preserve the independently proved L3"
        );
        assert!(cert["reject"].is_null(), "{variable}: {cert}");
    }
}

#[test]
fn shared_state_in_a_contract_is_a_structured_front_door_rejection() {
    let fixture = std::env::temp_dir().join(format!(
        "thermite-shared-contract-{}.th",
        std::process::id()
    ));
    std::fs::write(
        &fixture,
        "struct State { n: u64 } keeps n < 10
         shared state: State
         lock gate guards state
         fn read_state() -> u64 ! owns(gate), read(state.n)
         requires state.n == 0 ensures result < 10
         { holding gate { state.n } }",
    )
    .expect("write fixture");
    let out = Command::new(forge_bin())
        .arg("check")
        .arg(&fixture)
        .output()
        .expect("spawn forge");
    std::fs::remove_file(&fixture).expect("remove fixture");
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("shared-state root `state`"), "{stderr}");
    assert!(stderr.contains("`requires`"), "{stderr}");
    assert!(!stderr.contains("E0425"), "{stderr}");
    assert!(!stderr.contains("vacuity harness"), "{stderr}");
}

// ---- AC-2 (stream discipline) + AC-1: usage error exits non-zero ----------

#[test]
fn missing_file_is_usage_error_nonzero() {
    // No verus needed: arg parsing fails before the pipeline.
    let out = Command::new(forge_bin())
        .arg("check")
        .output()
        .expect("spawn forge");
    assert_ne!(out.status.code(), Some(0), "missing <file> must not exit 0");
    assert!(
        out.stdout.is_empty(),
        "a usage error writes nothing to stdout (diagnostics go to stderr)"
    );
    assert!(
        !out.stderr.is_empty(),
        "a usage error writes a diagnostic to stderr"
    );
}

#[test]
fn explicit_l2_empty_certificate_array_is_not_success() {
    let fixture = std::env::temp_dir().join(format!("thermite-l2-empty-{}.th", std::process::id()));
    std::fs::write(&fixture, "spec fn only(x: u64) -> u64 measures x { x }\n").unwrap();
    let out = Command::new(forge_bin())
        .arg("check")
        .arg(&fixture)
        .args(["--level", "l2", "--json"])
        .output()
        .expect("spawn forge");
    let _ = std::fs::remove_file(&fixture);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "[]",
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "no bounded-check certificate is not successful L2 verification"
    );
}
