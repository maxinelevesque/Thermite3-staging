import importlib.util
import re
import unittest
from pathlib import Path


def load_gate():
    path = Path(__file__).parents[1] / "assurance-policy-migration-replay.py"
    spec = importlib.util.spec_from_file_location(
        "assurance_policy_migration_replay", path
    )
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class AssurancePolicyMigrationReplayTest(unittest.TestCase):
    def test_policy_migration_replay_is_clean(self):
        gate = load_gate()
        self.assertEqual(gate.validate(Path(__file__).parents[2]), [])

    def test_every_invalid_or_lossy_case_is_policy_skew(self):
        gate = load_gate()
        root = Path(__file__).parents[2]
        import json

        matrix = json.loads(
            (root / gate.MATRIX).read_text(encoding="utf-8")
        )
        compared = [case for case in matrix["case"] if gate.classify(case) == "compared"]
        self.assertEqual([case["id"] for case in compared], ["compatible"])

    def test_policy_migration_never_reads_archival_levels(self):
        root = Path(__file__).parents[2]
        for relative in (
            "forge/src/policy_migration.rs",
            "lean/Thermite/PolicyMigration.lean",
            "lean/Thermite/PolicyMigrationReplay.lean",
            "gates/assurance-policy-migration-replay.py",
        ):
            text = (root / relative).read_text(encoding="utf-8")
            executable = "\n".join(
                line
                for line in text.splitlines()
                if not line.lstrip().startswith(("//!", "/--", "--"))
            )
            self.assertIsNone(
                re.search(r"\bLevel\b|\bL[0-4]\b", executable), relative
            )
