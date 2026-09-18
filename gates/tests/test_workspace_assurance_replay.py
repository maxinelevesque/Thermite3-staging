import json
import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class WorkspaceAssuranceReplayTest(unittest.TestCase):
    def test_matrix_and_generated_lean_are_current(self) -> None:
        subprocess.run(
            [sys.executable, "gates/workspace-assurance-replay.py"],
            cwd=ROOT,
            check=True,
        )

    def test_every_denominator_dimension_has_a_hostile_case(self) -> None:
        data = json.loads((ROOT / "gates/workspace-assurance-replay.json").read_text())
        ids = {case["id"] for case in data["case"]}
        self.assertTrue(
            {
                "missing-package-report",
                "missing-target-report",
                "feature-cell-misbound",
                "platform-cell-misbound",
                "generated-source-omitted",
                "planned-item-omitted",
                "duplicate-matrix-cell",
            }.issubset(ids)
        )

    def test_workspace_authority_never_reads_archival_levels(self) -> None:
        source = (ROOT / "forge/src/workspace_assurance.rs").read_text()
        authority = source.split("#[cfg(test)]", 1)[0]
        self.assertNotIn("manifest::Level", authority)
        self.assertNotIn("Level::L", authority)


if __name__ == "__main__":
    unittest.main()
