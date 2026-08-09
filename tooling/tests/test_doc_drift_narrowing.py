#!/usr/bin/env python3
"""
Oracle fixture tests for doc-drift's narrowing mechanisms (RFC-16 §5.1).

Same convention as test_doc_drift.py: a throwaway git repo per test, the gate
run via subprocess, and assertions against HAND-AUTHORED expectations about
BEHAVIOUR — which edits must drift a doc and which must not — rather than
against any hash the tool prints.

The oracle:

  N-1  (anchor narrows):    a governed file carrying
                            `doc:begin(<doc>)` ... `doc:end`; an edit OUTSIDE
                            the region -> CURRENT; an edit INSIDE -> DRIFT.
  N-2  (default unchanged): the same file with no anchor -> any edit DRIFTs.
                            This is the no-flag-day property: absent an opt-in,
                            behaviour is exactly as before.
  N-3  (unclosed anchor):   `doc:begin` with no `doc:end` -> exit 3, no
                            Traceback. Silently narrowing the pin is the
                            failure this mechanism exists to prevent, so it
                            must be inconclusive rather than "clean".
  N-4  (anchor is per-doc): an anchor naming doc A does not narrow doc B's pin
                            over the same file.
  N-5  (extract narrows):   `pin-extract: <file>=claude-hooks`; adding a
                            THIRD-PARTY hook entry -> CURRENT; changing the
                            repo-owned hook command -> DRIFT. This is the
                            measured false positive from 2026-08-07 turned into
                            a regression test.
  N-6  (unknown extractor): `pin-extract: x=nope` -> exit 3, naming the
                            extractor, no Traceback.
"""

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from test_doc_drift import Fixture  # noqa: E402  (shared fixture harness)

_spec = importlib.util.spec_from_file_location(
    "doc_drift", Path(__file__).resolve().parents[1] / "doc-drift.py"
)
_dd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_dd)


def _pin_for(fx, files, doc):
    """The content digest the doc should carry, via the documented algorithm."""
    return _dd._content_digest(
        fx.path, files, doc, _dd.extract_pin_config(fx.path, doc)
    )


def _doc_with(header_lines):
    return (
        "# Fixture doc\n\n<!--\ntier: 3-component\nstatus: draft\n"
        + "".join(header_lines)
        + "-->\n\nbody\n"
    )


SETTINGS_ONE_HOOK = """{
  "hooks": {
    "PreToolUse": [
      {"matcher": "Write|Edit",
       "hooks": [{"type": "command", "command": "python3 tooling/gate.py"}]}
    ]
  }
}
"""

SETTINGS_PLUS_THIRD_PARTY = """{
  "hooks": {
    "PreToolUse": [
      {"matcher": "Write|Edit",
       "hooks": [{"type": "command", "command": "python3 tooling/gate.py"}]}
    ],
    "SessionStart": [
      {"hooks": [{"type": "command", "command": "day hook session-start"}]}
    ]
  }
}
"""

SETTINGS_CHANGED_OWN_HOOK = """{
  "hooks": {
    "PreToolUse": [
      {"matcher": "Write|Edit",
       "hooks": [{"type": "command", "command": "python3 tooling/other.py"}]}
    ]
  }
}
"""


class DocDriftNarrowingTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def _anchored_repo(self, name, doc=".design/widget.md"):
        fx = Fixture(self.tmp / name)
        fx.write_routes([("src/widget.rs", doc)])
        fx.commit(
            "src/widget.rs",
            "outside_before\n// doc:begin(%s)\ninside\n// doc:end\noutside_after\n" % doc,
            "widget v1",
        )
        return fx, doc

    def test_n1_anchor_narrows_to_the_region(self):
        fx, doc = self._anchored_repo("n1")
        fx.write(doc, _doc_with([f"audited-content-sha256: {_pin_for(fx, ['src/widget.rs'], doc)}\n"]))

        outside = fx.path / "src/widget.rs"
        outside.write_text(outside.read_text().replace("outside_after", "CHANGED_OUTSIDE"))
        res = fx.run_gate()
        self.assertEqual(res.returncode, 0, res.stdout + res.stderr)
        self.assertIn("CURRENT", res.stdout)

        outside.write_text(outside.read_text().replace("inside", "CHANGED_INSIDE"))
        res = fx.run_gate()
        self.assertEqual(res.returncode, 1, res.stdout + res.stderr)
        self.assertIn("DRIFT", res.stdout)

    def test_n2_without_an_anchor_the_whole_file_still_counts(self):
        fx = Fixture(self.tmp / "n2")
        doc = ".design/widget.md"
        fx.write_routes([("src/widget.rs", doc)])
        fx.commit("src/widget.rs", "alpha\nbeta\n", "widget v1")
        fx.write(doc, _doc_with([f"audited-content-sha256: {_pin_for(fx, ['src/widget.rs'], doc)}\n"]))

        self.assertEqual(fx.run_gate().returncode, 0)
        (fx.path / "src/widget.rs").write_text("alpha\nGAMMA\n")
        res = fx.run_gate()
        self.assertEqual(res.returncode, 1, res.stdout + res.stderr)
        self.assertIn("DRIFT", res.stdout)

    def test_n3_unclosed_anchor_is_inconclusive(self):
        fx = Fixture(self.tmp / "n3")
        doc = ".design/widget.md"
        fx.write_routes([("src/widget.rs", doc)])
        fx.commit("src/widget.rs", f"// doc:begin({doc})\ninside\n", "widget v1")
        fx.write(doc, _doc_with(["audited-content-sha256: " + "0" * 64 + "\n"]))

        res = fx.run_gate()
        self.assertEqual(res.returncode, 3, res.stdout + res.stderr)
        self.assertNotIn("Traceback", res.stderr)

    def test_n4_an_anchor_is_scoped_to_the_doc_that_names_it(self):
        fx = Fixture(self.tmp / "n4")
        a, b = ".design/a.md", ".design/b.md"
        fx.write_routes([("src/widget.rs", a), ("src/widget.rs", b)])
        fx.commit(
            "src/widget.rs",
            f"outside\n// doc:begin({a})\ninside\n// doc:end\ntail\n",
            "widget v1",
        )
        fx.write(a, _doc_with([f"audited-content-sha256: {_pin_for(fx, ['src/widget.rs'], a)}\n"]))
        fx.write(b, _doc_with([f"audited-content-sha256: {_pin_for(fx, ['src/widget.rs'], b)}\n"]))
        self.assertEqual(fx.run_gate().returncode, 0)

        w = fx.path / "src/widget.rs"
        w.write_text(w.read_text().replace("tail", "TAIL_CHANGED"))
        res = fx.run_gate()
        self.assertEqual(res.returncode, 1, res.stdout + res.stderr)
        # b sees the whole file and drifts; a is narrowed and does not.
        drift_lines = [ln for ln in res.stdout.splitlines() if ln.startswith("DRIFT")]
        self.assertTrue(any(b in ln for ln in drift_lines), res.stdout)
        self.assertFalse(any(a in ln for ln in drift_lines), res.stdout)

    def test_n5_extract_ignores_a_third_partys_hooks(self):
        fx = Fixture(self.tmp / "n5")
        doc = ".design/control-plane.md"
        fx.write_routes([(".claude/settings.json", doc)])
        fx.commit(".claude/settings.json", SETTINGS_ONE_HOOK, "settings v1")
        header = [
            "pin-extract: .claude/settings.json=claude-hooks\n",
            "audited-content-sha256: PLACEHOLDER\n",
        ]
        fx.write(doc, _doc_with(header))
        pin = _pin_for(fx, [".claude/settings.json"], doc)
        fx.write(doc, _doc_with([header[0], f"audited-content-sha256: {pin}\n"]))
        self.assertEqual(fx.run_gate().returncode, 0)

        # A second tool appends its own wiring: must NOT drift.
        fx.write(".claude/settings.json", SETTINGS_PLUS_THIRD_PARTY)
        res = fx.run_gate()
        self.assertEqual(res.returncode, 0, res.stdout + res.stderr)
        self.assertIn("CURRENT", res.stdout)

        # The repo's OWN hook changes: must drift.
        fx.write(".claude/settings.json", SETTINGS_CHANGED_OWN_HOOK)
        res = fx.run_gate()
        self.assertEqual(res.returncode, 1, res.stdout + res.stderr)
        self.assertIn("DRIFT", res.stdout)

    def test_n6_unknown_extractor_is_inconclusive_and_named(self):
        fx = Fixture(self.tmp / "n6")
        doc = ".design/widget.md"
        fx.write_routes([("src/widget.rs", doc)])
        fx.commit("src/widget.rs", "alpha\n", "widget v1")
        fx.write(
            doc,
            _doc_with(
                ["pin-extract: src/widget.rs=nope\n",
                 "audited-content-sha256: " + "0" * 64 + "\n"]
            ),
        )
        res = fx.run_gate()
        self.assertEqual(res.returncode, 3, res.stdout + res.stderr)
        self.assertIn("nope", res.stdout + res.stderr)
        self.assertNotIn("Traceback", res.stderr)


if __name__ == "__main__":
    unittest.main()
