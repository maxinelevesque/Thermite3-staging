"""Mutation tests for the AssurancePolicyV2 Rust/Lean replay gate."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "assurance_v2_replay", ROOT / "gates/assurance-v2-replay.py"
)
assert SPEC and SPEC.loader
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)


def matrix() -> dict[str, object]:
    return json.loads((ROOT / gate.MATRIX).read_text(encoding="utf-8"))


def issuers() -> dict[str, object]:
    return json.loads((ROOT / gate.ISSUERS).read_text(encoding="utf-8"))


class AssuranceV2ReplayGateTests(unittest.TestCase):
    def test_checked_matrix_and_generated_lean_are_current(self) -> None:
        data = matrix()
        self.assertEqual(gate.validate(ROOT, data), [])
        self.assertEqual(
            (ROOT / gate.LEAN).read_text(encoding="utf-8"),
            gate.generated(data["pair"]),
        )
        self.assertEqual(gate.validate_issuers(ROOT, issuers()), [])

    def test_omitted_family_fails_closed_signature(self) -> None:
        data = matrix()
        data["families"].remove("lean_complete")
        self.assertTrue(any("six-family" in error for error in gate.validate(ROOT, data)))

    def test_changed_pair_law_fails_exact_replay(self) -> None:
        data = matrix()
        row = next(
            row
            for row in data["pair"]
            if row["left"] == "solver_complete" and row["right"] == "lean_empirical"
        )
        row["lower_bound_frontier"] = []
        self.assertTrue(any("6x6" in error for error in gate.validate(ROOT, data)))

    def test_matrix_enumerates_constructor_families_not_parameter_values(self) -> None:
        data = matrix()
        self.assertEqual(len(data["families"]), 6)
        self.assertEqual(len(data["pair"]), 36)
        serialized = json.dumps(data)
        self.assertNotIn("fixture-run", serialized)
        self.assertNotIn("through_bound", serialized)
        self.assertNotIn("semantics_version", serialized)

    def test_issuer_omission_and_unclassified_predecessor_fail(self) -> None:
        data = issuers()
        data["family"] = data["family"][:-1]
        self.assertTrue(any("six-family" in error for error in gate.validate_issuers(ROOT, data)))

        data = issuers()
        data["legacy_relation"].append(
            {"carrier": "FuturePolicy", "classification": "silently_assumed_compatible"}
        )
        self.assertTrue(any("predecessor" in error for error in gate.validate_issuers(ROOT, data)))

    def test_wrong_family_existing_symbol_and_duplicate_issuer_fail(self) -> None:
        data = issuers()
        runtime = data["family"][0]["issuers"][0]
        runtime["symbol"] = "assemble_certificate"
        self.assertTrue(any("bidirectional" in error for error in gate.validate_issuers(ROOT, data)))

        data = issuers()
        data["family"][0]["issuers"].append(dict(data["family"][0]["issuers"][0]))
        self.assertTrue(any("bidirectional" in error for error in gate.validate_issuers(ROOT, data)))

    def test_unregistered_source_markers_fail_both_inventories(self) -> None:
        source_issuers, source_predecessors, source_characterizations = (
            gate.source_extension_markers(ROOT)
        )
        with patch.object(
            gate,
            "source_extension_markers",
            return_value=(
                source_issuers + [("runtime", "forge/src/check.rs", "future_issuer")],
                source_predecessors,
                source_characterizations,
            ),
        ):
            self.assertTrue(
                any("bidirectional" in error for error in gate.validate_issuers(ROOT, issuers()))
            )
        with patch.object(
            gate,
            "source_extension_markers",
            return_value=(
                source_issuers,
                source_predecessors + [("FuturePolicy", "unknown")],
                source_characterizations,
            ),
        ):
            self.assertTrue(
                any("predecessor inventory" in error for error in gate.validate_issuers(ROOT, issuers()))
            )

    def test_unrelated_existing_test_cannot_replace_characterization(self) -> None:
        data = issuers()
        runtime = data["family"][0]["issuers"][0]
        runtime["characterization_test"] = (
            "parseable_failure_is_reported_cert_with_counterexample"
        )
        self.assertTrue(
            any(
                "CHARACTERIZATION" in error
                for error in gate.validate_issuers(ROOT, data)
            )
        )

    def test_source_marker_must_be_adjacent_to_its_declaration(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "forge/src"
            lean = root / "lean/Thermite"
            source.mkdir(parents=True)
            lean.mkdir(parents=True)
            (source / "bad.rs").write_text(
                "// ASSURANCE_V2_ISSUER runtime real_issuer\n"
                "// displaced\n// displaced\n// displaced\n// displaced\n"
                "fn real_issuer() {}\n",
                encoding="utf-8",
            )
            self.assertTrue(
                any(
                    "adjacent" in error
                    for error in gate.source_marker_adjacency_errors(root)
                )
            )


if __name__ == "__main__":
    unittest.main()
