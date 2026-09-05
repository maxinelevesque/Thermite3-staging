//! RFC-13 production certification anchor.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn forge_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_forge"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("conformance")
        .join("protocol_types.th")
}

fn verus_present() -> bool {
    if let Ok(path) = std::env::var("VERUS_BIN") {
        if Path::new(&path).exists() {
            return true;
        }
    }
    Command::new("which")
        .arg("verus")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
        || std::env::var("HOME")
            .map(|home| PathBuf::from(home).join(".local/bin/verus").exists())
            .unwrap_or(false)
}

#[test]
fn protocol_program_certifies_only_after_formal_replay() {
    assert!(verus_present(), "Verus is required for RFC-13 conformance");
    let output = Command::new(forge_bin())
        .arg("check")
        .arg(fixture())
        .arg("--mutation-floor")
        .arg("0")
        .arg("--json")
        .output()
        .expect("spawn forge");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let certificates: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!(
            "RFC-13 check must emit certificate JSON: {error}\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert_eq!(output.status.code(), Some(0), "{certificates:#}");
    let rows = certificates.as_array().expect("certificate array");
    assert_eq!(rows.len(), 3);
    let functions = rows
        .iter()
        .filter(|row| matches!(row["item"].as_str(), Some("provider" | "user")))
        .collect::<Vec<_>>();
    assert_eq!(functions.len(), 2);
    assert!(functions.iter().all(|row| row["level"] == "L3"));
    assert!(functions
        .iter()
        .all(|row| row["effects"] == serde_json::json!(["blocks"])));
}
