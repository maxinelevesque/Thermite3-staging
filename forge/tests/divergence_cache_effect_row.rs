//! `forge/tests/divergence_cache_effect_row.rs` — the effect row is absent from
//! the proof-cache key, so a cache HIT can serve a certificate whose `effects`
//! contradict the source's declared row.
//!
//! This pins a divergence from `.design/forge/proof-cache.md` REQ-2 (the
//! soundness-completeness invariant): *"a cache HIT returns the SAME verdict a
//! fresh verus run would, by construction."* The certificate's `effects` field
//! is the third element of `Certificate::oracle_subset` (`manifest.rs`), i.e. a
//! DETERMINISTIC field a hit must agree with a fresh verify on — REQ-2's own
//! carve-out says only `cached` "cannot be an oracle field", so `effects` is in.
//!
//! `cache::cache_key` hashes the item's LOWERED verus source (REQ-1a). The
//! bookkeeping-only effect labels (`read`/`write`/`net`/`alloc`/`time`/`rand`/
//! `panic`/`term`) do not survive lowering — they change no proof obligation —
//! so they never reach the key. `diverge` is the sole exception: it changes the
//! termination obligation, hence the lowered source, hence the key.
//!
//! Measured at staging `b79b4005`: two programs identical but for the row
//! collide, and the second is served the first's certificate. The realistic
//! shape is the ordinary edit-and-recheck loop — widen a row in place, re-run
//! `forge check`, and read a stale `effects` list back.
//!
//! Expected verdicts trace to the design doc, never copied from forge's own
//! output (`goal.md` R-CHAR-3). `tests/` is not anti-pattern-gated, so
//! `unwrap`/`expect` are fine here.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

/// A fixed verus version pin so the key is stable across runs (mirrors
/// `cache_conformance.rs`).
const PINNED_VERUS_VERSION: &str = "verus-test-pin-0.0.1";

fn forge_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_forge"))
}

/// `true` iff verus can be located (`VERUS_BIN`, then PATH, then
/// `~/.local/bin/verus`) — mirrors `cache_conformance.rs`.
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

fn unique_dir(tag: &str) -> PathBuf {
    static C: AtomicU64 = AtomicU64::new(0);
    let n = C.fetch_add(1, Ordering::Relaxed);
    let p = std::env::temp_dir().join(format!("forge_rowdiv_{}_{}_{}", tag, std::process::id(), n));
    let _ = std::fs::remove_dir_all(&p);
    p
}

fn write_fixture(tag: &str, body: &str) -> PathBuf {
    static C: AtomicU64 = AtomicU64::new(0);
    let n = C.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "forge_rowfix_{}_{}_{}.th",
        tag,
        std::process::id(),
        n
    ));
    std::fs::write(&path, body).expect("write fixture");
    path
}

fn run_check(file: &Path, cache_dir: &Path) -> (Option<i32>, Vec<Value>) {
    let mut cmd = Command::new(forge_bin());
    cmd.arg("check").arg(file).arg("--json");
    cmd.env("FORGE_CACHE_DIR", cache_dir);
    cmd.env("VERUS_VERSION", PINNED_VERUS_VERSION);
    let out = cmd.output().unwrap_or_else(|e| panic!("spawn forge: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let value: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!(
            "forge --json stdout must be one JSON document: {e}\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (
        out.status.code(),
        value
            .as_array()
            .unwrap_or_else(|| panic!("forge --json must emit a cert array: {value}"))
            .clone(),
    )
}

fn cert_for<'a>(certs: &'a [Value], item: &str) -> &'a Value {
    certs
        .iter()
        .find(|c| c.get("item").and_then(|v| v.as_str()) == Some(item))
        .unwrap_or_else(|| panic!("no certificate for item `{item}` in {certs:?}"))
}

/// The same verifiable item under a caller-chosen effect row. Identical name,
/// signature, contract and body — the row is the ONLY difference, which is what
/// makes the pair collide.
fn program_with_row(row: &str) -> String {
    format!("fn row_item(x: u64) -> u64\n  ! {row}\n  requires x < 1000\n  ensures result == x\n{{\n  x\n}}\n")
}

/// REQ-2, stated as a test: a HIT must equal a FRESH VERIFY on the oracle
/// fields. We compute both readings of the SAME source — one against a cache
/// already populated by a differently-rowed twin, one against a virgin cache —
/// and require they agree.
#[test]
fn cache_hit_effects_must_equal_a_fresh_verify() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — proof-cache effect-row divergence not run.");
        return;
    }

    let shared = unique_dir("shared");
    let virgin = unique_dir("virgin");

    // Populate `shared` with the twin that declares a write effect.
    let writey = write_fixture("writey", &program_with_row("write(log)"));
    let (code_w, certs_w) = run_check(&writey, &shared);
    assert_eq!(code_w, Some(0), "the write-rowed twin must certify");
    assert_eq!(
        cert_for(&certs_w, "row_item")["effects"],
        Value::from(vec![Value::from("write(log)")]),
        "a fresh verify reports the row the source declares",
    );

    // The same item, now declaring `! pure`. Identical in every other byte.
    let purey = write_fixture("purey", &program_with_row("pure"));

    // Reading A: against the cache the twin populated.
    let (_, certs_hit) = run_check(&purey, &shared);
    let hit = cert_for(&certs_hit, "row_item");

    // Reading B: against a virgin cache — this is the fresh verify REQ-2 names.
    let (_, certs_fresh) = run_check(&purey, &virgin);
    let fresh = cert_for(&certs_fresh, "row_item");

    // The fresh verify is the ground truth, and it must say what the source says.
    assert_eq!(
        fresh["effects"],
        Value::from(vec![Value::from("pure")]),
        "a fresh verify of `! pure` reports [pure]",
    );

    // REQ-2: the hit must agree with it. This is the divergence.
    assert_eq!(
        hit["effects"], fresh["effects"],
        "REQ-2 (a hit equals a fresh verify): the cached certificate reports \
         effects {:?} for a source whose declared row is `! pure`, because the \
         effect row is not part of the cache key",
        hit["effects"],
    );

    // The certificate's headline must not drift either.
    assert_eq!(hit["level"], fresh["level"], "level must agree too");
}

/// The realistic shape: ONE file, edited in place to widen its row, re-checked
/// against the same project cache. This is the edit-and-recheck loop, and it is
/// how an author or an agent actually meets the defect.
#[test]
fn editing_only_the_row_in_place_must_change_the_reported_effects() {
    if !verus_present() {
        eprintln!("SKIP: verus not available — in-place row-edit divergence not run.");
        return;
    }

    let cache = unique_dir("inplace");
    let file = write_fixture("inplace", &program_with_row("pure"));

    let (_, before) = run_check(&file, &cache);
    assert_eq!(
        cert_for(&before, "row_item")["effects"],
        Value::from(vec![Value::from("pure")]),
        "as authored, the row is `! pure`",
    );

    // Widen the row in place. Nothing else about the file changes.
    std::fs::write(&file, program_with_row("write(log)")).expect("rewrite fixture");

    let (_, after) = run_check(&file, &cache);
    assert_eq!(
        cert_for(&after, "row_item")["effects"],
        Value::from(vec![Value::from("write(log)")]),
        "after widening the row in place, the certificate must report the \
         declared row — not the one cached under the pre-edit key",
    );
}
