#!/usr/bin/env python3
"""Oracle tests for doc-drift's `code-normalized` extractor (RFC-16 §5.1).

Same convention as test_doc_drift_narrowing.py: hand-authored expectations
about BEHAVIOUR — which edits must drift a doc pinned through the extractor
and which must not. The must-not cases are the measured 2026-08-07 event:
one commit deleted an orphaned comment and let rustfmt re-wrap an export
list, and four documents drifted with no claim in any of them affected.

  C-1  (re-wrap is silent):     rustfmt re-wrapping a one-line export list to
                                multi-line (indentation + trailing comma)
                                must NOT drift.
  C-2  (comment edit is silent): deleting or editing a comment must NOT
                                drift. This blindness is deliberate and named
                                in the extractor's docstring.
  C-3  (token change fires):    a name entering the export list MUST drift.
  C-4  (string content fires):  a rename passing through a format string is
                                semantic (the RFC-6 `.ens#` case) and MUST
                                drift — string literals are never normalized.
  C-5  (string formatting is significant): whitespace INSIDE a string is
                                content, not formatting.
  C-6  (non-UTF-8 is exit 3):   an undecodable governed file is
                                INCONCLUSIVE, never a silent pass.
"""

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from test_doc_drift import Fixture, _git  # noqa: E402  (shared harness)

_spec = importlib.util.spec_from_file_location(
    "doc_drift", Path(__file__).resolve().parents[1] / "doc-drift.py"
)
_dd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_dd)


DOC = ".design/widget.md"


def _doc_text():
    return (
        "# Fixture doc\n\n<!--\ntier: 3-component\nstatus: draft\n"
        f"pin-extract: src/widget.rs=code-normalized\n"
        "-->\n\nbody\n"
    )


ONE_LINE_EXPORTS = (
    "pub use ast::{Contract, FnItem, LoopNode, SpecFnItem};\n"
    "// orphaned note about a removed arm\n"
    "pub fn answer() -> u32 { 41 + 1 }\n"
)

# The same tokens after the measured event: comment deleted, list re-wrapped
# by rustfmt with indentation and a trailing comma.
REWRAPPED_EXPORTS = (
    "pub use ast::{\n"
    "    Contract,\n"
    "    FnItem,\n"
    "    LoopNode,\n"
    "    SpecFnItem,\n"
    "};\n"
    "pub fn answer() -> u32 {\n"
    "    41 + 1\n"
    "}\n"
)


class CodeNormalizedExtractorTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def _repo(self, name, source):
        fx = Fixture(self.tmp / name)
        fx.write_routes([("src/widget.rs", DOC)])
        fx.commit("src/widget.rs", source, "widget v1")
        pin = _dd._content_digest(
            fx.path, ["src/widget.rs"], DOC,
            {"src/widget.rs": "code-normalized"},
        )
        fx.commit(
            DOC,
            _doc_text().replace(
                "status: draft\n",
                f"status: draft\naudited-content-sha256: {pin}\n",
            ),
            "doc pinned",
        )
        return fx

    def _rewrite(self, fx, source, msg):
        fx.commit("src/widget.rs", source, msg)

    def test_c1_rustfmt_rewrap_is_silent(self):
        fx = self._repo("c1", ONE_LINE_EXPORTS)
        self._rewrite(fx, REWRAPPED_EXPORTS, "fmt + comment removal")
        res = fx.run_gate()
        out = res.stdout + res.stderr
        self.assertEqual(res.returncode, 0, out)
        self.assertIn("CURRENT", out)

    def test_c2_comment_only_edit_is_silent(self):
        fx = self._repo("c2", ONE_LINE_EXPORTS)
        self._rewrite(
            fx,
            ONE_LINE_EXPORTS.replace(
                "// orphaned note about a removed arm",
                "/* a different\n   block comment */",
            ),
            "comment churn",
        )
        res = fx.run_gate()
        self.assertEqual(res.returncode, 0, res.stdout + res.stderr)

    def test_c3_export_change_fires(self):
        fx = self._repo("c3", ONE_LINE_EXPORTS)
        self._rewrite(
            fx,
            ONE_LINE_EXPORTS.replace("SpecFnItem};", "SpecFnItem, StructItem};"),
            "a name enters the export list",
        )
        res = fx.run_gate()
        out = res.stdout + res.stderr
        self.assertEqual(res.returncode, 1, out)
        self.assertIn("DRIFT", out)

    def test_c4_rename_through_format_string_fires(self):
        """The RFC-6 `.ens#` case: clause names built as data are semantic."""
        src = 'pub fn seg() -> String { format!("{}.ens#{}", a, b) }\n'
        fx = self._repo("c4", src)
        self._rewrite(
            fx,
            src.replace(".ens#", ".ensures#"),
            "rename passes through a format string",
        )
        res = fx.run_gate()
        out = res.stdout + res.stderr
        self.assertEqual(res.returncode, 1, out)
        self.assertIn("DRIFT", out)

    def test_c5_whitespace_inside_string_is_content(self):
        src = 'pub const BANNER: &str = "a  b";\n'
        fx = self._repo("c5", src)
        self._rewrite(fx, src.replace('"a  b"', '"a b"'), "string edit")
        res = fx.run_gate()
        self.assertEqual(res.returncode, 1, res.stdout + res.stderr)

    def test_c5b_char_literal_quote_does_not_derail(self):
        """A '\"' char literal must not open a phantom string that swallows
        the code after it."""
        src = "pub fn q() -> char { '\"' }\npub fn answer() -> u32 { 42 }\n"
        fx = self._repo("c5b", src)
        self._rewrite(
            fx,
            src.replace("42", "43"),
            "semantic change after the char literal",
        )
        res = fx.run_gate()
        self.assertEqual(res.returncode, 1, res.stdout + res.stderr)

    def test_c6_non_utf8_is_inconclusive(self):
        fx = Fixture(self.tmp / "c6")
        fx.write_routes([("src/widget.rs", DOC)])
        (fx.path / "src").mkdir(parents=True, exist_ok=True)
        (fx.path / "src/widget.rs").write_bytes(b"\xff\xfe not utf8")
        _git(fx.path, "add", "src/widget.rs", env=fx.env)
        _git(fx.path, "commit", "-q", "-m", "binary widget", env=fx.env)
        fx.commit(
            DOC,
            _doc_text().replace(
                "status: draft\n",
                "status: draft\n"
                "audited-content-sha256: "
                + "0" * 64 + "\n",
            ),
            "doc pinned",
        )
        res = fx.run_gate()
        out = res.stdout + res.stderr
        self.assertEqual(res.returncode, 3, out)
        self.assertNotIn("Traceback", out)


if __name__ == "__main__":
    unittest.main()
