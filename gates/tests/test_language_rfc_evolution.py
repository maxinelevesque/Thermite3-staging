#!/usr/bin/env python3
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

GATE = Path(__file__).resolve().parents[1] / "language-rfc-evolution.py"


class EvolutionGateTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / ".design/rfcs").mkdir(parents=True)
        (self.root / "gates").mkdir()
        (self.root / "evidence").mkdir()
        (self.root / "evidence/all.txt").write_text(
            "classifier inclusion matrix negative compatibility counterexample preservation",
            encoding="utf-8",
        )

    def tearDown(self):
        self.tmp.cleanup()

    def rfc(self, mode="tracked"):
        (self.root / ".design/rfcs/0021-fixture.md").write_text(
            f"---\nrfc: 21\ntitle: Fixture\nstatus: draft\n"
            f"language-evolution: {mode}\n---\n",
            encoding="utf-8",
        )

    def manifest(self, body=""):
        (self.root / "gates/language-rfc-evolution.toml").write_text(
            "version = 1\nbaseline_rfc = []\n" + textwrap.dedent(body),
            encoding="utf-8",
        )

    def run_gate(self):
        return subprocess.run(
            [sys.executable, str(GATE), "--root", str(self.root)],
            text=True, capture_output=True,
        )

    def expansion(self, omit=None):
        fields = {
            "classifier": "evidence/all.txt#classifier",
            "inclusion": "evidence/all.txt#inclusion",
            "support_matrix": "evidence/all.txt#matrix",
            "negative_witness": "evidence/all.txt#negative",
        }
        fields.pop(omit, None)
        lines = [
            "[[evolution]]", 'rfc = "0021-fixture.md"', 'fragment = "core"',
            'change = "expand"',
        ] + [f'{key} = "{value}"' for key, value in fields.items()]
        self.manifest("\n".join(lines) + "\n")

    def test_complete_expansion_passes(self):
        self.rfc(); self.expansion()
        self.assertEqual(self.run_gate().returncode, 0)

    def test_expansion_fails_for_each_omitted_artifact(self):
        self.rfc()
        for field in ("classifier", "inclusion", "support_matrix", "negative_witness"):
            with self.subTest(field=field):
                self.expansion(field)
                result = self.run_gate()
                self.assertEqual(result.returncode, 1)
                self.assertIn(field, result.stderr)

    def test_narrowing_cannot_claim_ordinary_inclusion(self):
        self.rfc()
        self.manifest('''
            [[evolution]]
            rfc = "0021-fixture.md"
            fragment = "core"
            change = "narrow"
            classifier = "evidence/all.txt#classifier"
            inclusion = "evidence/all.txt#inclusion"
            compatibility_break = "evidence/all.txt#compatibility"
            counterexample = "evidence/all.txt#counterexample"
            support_matrix = "evidence/all.txt#matrix"
            negative_witness = "evidence/all.txt#negative"
        ''')
        result = self.run_gate()
        self.assertEqual(result.returncode, 1)
        self.assertIn("cannot claim ordinary inclusion", result.stderr)

    def test_new_rfc_must_declare_evolution_mode(self):
        self.rfc(mode="missing")
        path = self.root / ".design/rfcs/0021-fixture.md"
        path.write_text(path.read_text().replace("language-evolution: missing\n", ""))
        self.manifest()
        result = self.run_gate()
        self.assertEqual(result.returncode, 1)
        self.assertIn("must declare language-evolution", result.stderr)


if __name__ == "__main__":
    unittest.main()
