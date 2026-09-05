//! RFC-13 protocol checker conformance against the hand-derived oracle.

use std::path::PathBuf;

use serde::Deserialize;
use thermite_spec::check_protocols;

#[derive(Debug, Deserialize)]
struct Oracle {
    accept: Vec<AcceptCase>,
    reject: Vec<RejectCase>,
}

#[derive(Debug, Deserialize)]
struct AcceptCase {
    name: String,
    program: String,
}

#[derive(Debug, Deserialize)]
struct RejectCase {
    name: String,
    expected_kind: String,
    program: String,
}

fn oracle() -> Oracle {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../conformance/protocol-types/cases.json");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&source)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

#[test]
fn accepted_protocol_cases_complete() {
    for case in oracle().accept {
        let parsed = thermite_syntax::parse(&case.program);
        assert!(
            parsed.is_clean(),
            "{} parse: {:?}",
            case.name,
            parsed.errors
        );
        check_protocols(&parsed.program)
            .unwrap_or_else(|errors| panic!("{} rejected: {errors:?}", case.name));
    }
}

#[test]
fn rejected_protocol_cases_have_the_expected_structured_kind() {
    for case in oracle().reject {
        let parsed = thermite_syntax::parse(&case.program);
        assert!(
            parsed.is_clean(),
            "{} parse: {:?}",
            case.name,
            parsed.errors
        );
        let errors = match check_protocols(&parsed.program) {
            Ok(_) => panic!("{} unexpectedly accepted", case.name),
            Err(errors) => errors,
        };
        let kinds = errors
            .iter()
            .map(|error| format!("{:?}", error.kind))
            .collect::<Vec<_>>();
        assert!(
            kinds.contains(&case.expected_kind),
            "{}: expected {}, got {kinds:?}",
            case.name,
            case.expected_kind
        );
    }
}
