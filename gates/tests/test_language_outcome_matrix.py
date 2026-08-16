import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

GATE = Path(__file__).resolve().parents[1] / "language-outcome-matrix.py"
SPEC = importlib.util.spec_from_file_location("language_outcome_matrix", GATE)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


def complete_rows():
    stages = sorted(MODULE.STAGES)
    outcomes = sorted(MODULE.OUTCOMES)
    rows = []
    for index, outcome in enumerate(outcomes):
        row = {
            "id": f"case-{outcome}",
            "stage": stages[index % len(stages)],
            "program": f"program-{outcome}",
            "expected": outcome,
        }
        if outcome != "success":
            row["facts"] = {outcome: True}
        rows.append(row)
    covered = {row["stage"] for row in rows}
    for stage in sorted(MODULE.STAGES - covered):
        rows.append({
            "id": f"stage-{stage}", "stage": stage,
            "program": f"program-{stage}", "expected": "success",
        })
    return rows


class OutcomeMatrixTests(unittest.TestCase):
    def write_and_check(self, rows):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "matrix.json"
            path.write_text(json.dumps({"version": 1, "case": rows}), encoding="utf-8")
            root = Path(__file__).resolve().parents[2]
            return MODULE.check(root, path)

    def test_complete_matrix_passes(self):
        self.assertEqual(self.write_and_check(complete_rows()), [])

    def test_missing_stage_fails_closed(self):
        rows = [row for row in complete_rows() if row["stage"] != "validator"]
        errors = self.write_and_check(rows)
        self.assertTrue(any("validator" in error for error in errors), errors)

    def test_missing_outcome_fails_closed(self):
        rows = [row for row in complete_rows() if row["expected"] != "resource_exhausted"]
        errors = self.write_and_check(rows)
        self.assertTrue(any("resource_exhausted" in error for error in errors), errors)

    def test_mismatched_terminal_fact_fails(self):
        rows = complete_rows()
        row = next(row for row in rows if row["expected"] == "tool_unavailable")
        row["facts"] = {"unsupported_language": True}
        errors = self.write_and_check(rows)
        self.assertTrue(any("sole terminal fact" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main()
