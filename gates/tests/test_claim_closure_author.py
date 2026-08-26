#!/usr/bin/env python3
import importlib.util
import tempfile
import unittest
from pathlib import Path

GATE = Path(__file__).resolve().parents[1] / "claim-closure-author.py"
SPEC = importlib.util.spec_from_file_location("claim_closure_author", GATE)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class ClaimClosureAuthorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / ".design/reqs").mkdir(parents=True)
        (self.root / "gates").mkdir()
        (self.root / MODULE.REGISTRY).write_text(
            '''
schema_version = 1
[[requirement]]
id = "REQ-B"
status = "shipped"
[[requirement]]
id = "REQ-A"
status = "shipped"
[[requirement]]
id = "REQ-FUTURE"
status = "not_started"
'''.lstrip(),
            encoding="utf-8",
        )
        (self.root / MODULE.LEDGER).write_text(
            'version = 1\n\n[[item]]\nid = "CR-X"\n', encoding="utf-8"
        )
        self.previous_size = MODULE.BASELINE_SIZE
        MODULE.BASELINE_SIZE = 2

    def tearDown(self):
        MODULE.BASELINE_SIZE = self.previous_size
        self.tmp.cleanup()

    def test_freeze_is_sorted_exact_and_keeps_staging_version(self):
        MODULE.freeze_baseline(self.root)

        text = (self.root / MODULE.LEDGER).read_text(encoding="utf-8")
        self.assertTrue(text.startswith("version = 1\n"))
        self.assertLess(text.index('"REQ-A"'), text.index('"REQ-B"'))
        self.assertEqual(MODULE.check_baseline(self.root), [])

    def test_freeze_refuses_an_unexpected_population(self):
        MODULE.BASELINE_SIZE = 3
        with self.assertRaisesRegex(ValueError, "refusing to freeze 2 shipped IDs"):
            MODULE.freeze_baseline(self.root)


if __name__ == "__main__":
    unittest.main()
