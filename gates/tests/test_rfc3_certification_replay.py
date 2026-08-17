"""Mutation tests for the RFC-3 generated Rust/Lean replay gate."""

from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "rfc3_certification_replay", ROOT / "gates/rfc3-certification-replay.py"
)
assert SPEC and SPEC.loader
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)


def matrix() -> dict[str, object]:
    return json.loads((ROOT / gate.MATRIX).read_text(encoding="utf-8"))


class ReplayGateTests(unittest.TestCase):
    def test_checked_matrix_and_generated_lean_are_current(self) -> None:
        data = matrix()
        self.assertEqual(gate.validate(ROOT, data), [])
        self.assertEqual(
            (ROOT / gate.LEAN).read_text(encoding="utf-8"), gate.generated(data["case"])
        )

    def test_missing_policy_point_fails_exact_coverage(self) -> None:
        data = matrix()
        for row in data["case"]:
            if row["policy_point"] == "bounded":
                row["policy_point"] = "runtime"
        self.assertTrue(
            any("policy_point coverage mismatch" in error for error in gate.validate(ROOT, data))
        )

    def test_engineer_label_is_not_an_authoritative_field(self) -> None:
        data = matrix()
        generated = gate.generated(data["case"])
        formal = generated.split("structure RawReplayProjection", 1)[1].split(
            "deriving", 1
        )[0]
        self.assertNotIn("engineer_label", formal)
        self.assertIn("engineer_label_is_non_authoritative", generated)
        self.assertIn("engineer_label_formal_substitution_rejected", generated)

    def test_policy_swap_preserving_marginal_coverage_is_rejected(self) -> None:
        data = matrix()
        by_id = {row["id"]: row for row in data["case"]}
        left = by_id["no_claim"]
        right = by_id["complete_solver"]
        left["policy_point"], right["policy_point"] = (
            right["policy_point"], left["policy_point"]
        )
        errors = gate.validate(ROOT, data)
        self.assertTrue(any("canonical family" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
