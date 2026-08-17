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
        formal = generated.split("structure FormalReplayProjection", 1)[1].split(
            "deriving", 1
        )[0]
        self.assertNotIn("engineer_label", formal)
        self.assertIn("engineer_label_is_non_authoritative", generated)
        self.assertIn("engineer_label_formal_substitution_rejected", generated)


if __name__ == "__main__":
    unittest.main()
