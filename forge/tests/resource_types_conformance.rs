//! RFC-11 production certification anchor.

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
        .join("resource_types.th")
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
fn resource_program_receives_only_a_resource_aware_certificate() {
    assert!(
        verus_present(),
        "Verus is required: RFC-11 certificate conformance must fail closed when its producer is unavailable"
    );
    let output = Command::new(forge_bin())
        .arg("check")
        .arg(fixture())
        .arg("--json")
        .output()
        .expect("spawn forge");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let certificates: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!(
            "RFC-11 check must emit certificate JSON: {error}\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert_eq!(output.status.code(), Some(0), "{certificates:#}");
    let discard = certificates
        .as_array()
        .unwrap()
        .iter()
        .find(|certificate| certificate["item"] == "discard")
        .expect("discard certificate");
    assert_eq!(discard["level"], "L3");
    let resource = &discard["resource_flow"];
    assert_eq!(resource["verdict"], "accepted");
    assert_eq!(resource["formal_replay"]["verdict"], "kernel_accepted");
    assert_eq!(
        resource["residual_trust"],
        serde_json::json!([
            "parser",
            "type_provenance_resolution",
            "witness_extraction",
            "executable_target_behavior"
        ])
    );
    let regions = resource["forgets"][0]["regions"].as_array().unwrap();
    assert!(regions.contains(&Value::from("heap")));
    assert!(regions.contains(&Value::from("device.port")));
}
