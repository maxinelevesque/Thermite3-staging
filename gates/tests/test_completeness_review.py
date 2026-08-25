#!/usr/bin/env python3
import importlib.util
import tempfile
import textwrap
import unittest
from pathlib import Path

GATE = Path(__file__).resolve().parents[1] / "completeness-review.py"
SPEC = importlib.util.spec_from_file_location("completeness_review", GATE)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class ReviewTrackTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "gates").mkdir()
        (self.root / "proof.py").write_text("def closure_witness(): pass\n", encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, gap: str, item: str):
        (self.root / MODULE.INVENTORY).write_text(
            "version = 1\n" + textwrap.dedent(gap), encoding="utf-8"
        )
        (self.root / MODULE.BACKLOG).write_text(
            "version = 1\n" + textwrap.dedent(item), encoding="utf-8"
        )

    @staticmethod
    def open_gap():
        return '''
            [[gap]]
            id = "GAP-X"
            status = "open"
            disposition = "completeness_review"
            issue = "https://github.com/o/r/issues/1"
        '''

    @staticmethod
    def open_item():
        return '''
            [[item]]
            id = "CR-X"
            gap = "GAP-X"
            issue = "https://github.com/o/r/issues/1"
            status = "open"
            closure_evidence = []
        '''

    def test_open_gap_and_backlog_agree(self):
        self.write(self.open_gap(), self.open_item())
        self.assertEqual(MODULE.check(self.root), [])

    def test_missing_and_orphan_items_fail_both_directions(self):
        self.write(self.open_gap(), "")
        self.assertTrue(any("expected exactly one" in e for e in MODULE.check(self.root)))
        self.write("", self.open_item())
        self.assertTrue(any("orphan" in e for e in MODULE.check(self.root)))

    def test_closing_item_while_gap_open_fails(self):
        closed = self.open_item().replace('status = "open"', 'status = "closed"').replace(
            "closure_evidence = []", 'closure_evidence = ["proof.py#closure_witness"]'
        )
        self.write(self.open_gap(), closed)
        self.assertTrue(any("open gap requires an open" in e for e in MODULE.check(self.root)))

    def test_resolved_gap_requires_closed_item(self):
        gap = '''
            [[gap]]
            id = "GAP-X"
            status = "resolved"
            review_item = "CR-X"
        '''
        self.write(gap, self.open_item())
        self.assertTrue(any("resolved gap requires a closed" in e for e in MODULE.check(self.root)))

    def test_closed_item_without_resolving_evidence_fails(self):
        gap = '''
            [[gap]]
            id = "GAP-X"
            status = "resolved"
            review_item = "CR-X"
        '''
        closed = self.open_item().replace('status = "open"', 'status = "closed"')
        self.write(gap, closed)
        self.assertTrue(any("requires executable/formal" in e for e in MODULE.check(self.root)))


if __name__ == "__main__":
    unittest.main()
