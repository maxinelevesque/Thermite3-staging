#!/usr/bin/env python3
"""Oracle tests for gates/paths-exist.py.

The three planted breaks mirror the three measured CI failures from the
RFC-18 layout move (RFC-18 §4): a shell gate invoking a moved script, a Rust
test reading a moved path literal, and a stale ``include_str!``. A gate that
cannot re-catch the measured failures is a witness that cannot fail.
"""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


GATE = Path(__file__).resolve().parents[1] / "paths-exist.py"

GIT_ENV = {
    "GIT_AUTHOR_NAME": "t",
    "GIT_AUTHOR_EMAIL": "t@e",
    "GIT_COMMITTER_NAME": "t",
    "GIT_COMMITTER_EMAIL": "t@e",
    "PATH": "/usr/bin:/bin:/usr/local/bin",
    "HOME": "/nonexistent",
}


class Fixture:
    def __init__(self, root: Path):
        self.root = root
        self.git("init", "-q")

    def git(self, *a):
        subprocess.run(
            ["git", "-c", "commit.gpgsign=false", *a],
            cwd=self.root,
            check=True,
            capture_output=True,
            env=GIT_ENV,
        )

    def file(self, rel: str, content: str = "x\n"):
        p = self.root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8")
        self.git("add", rel)
        return p

    def run(self):
        return subprocess.run(
            [sys.executable, str(GATE), "--root", str(self.root)],
            capture_output=True,
            text=True,
        )


class PathsExistTests(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.fx = Fixture(Path(self._tmp.name))

    def tearDown(self):
        self._tmp.cleanup()

    # --- the three measured breaks ------------------------------------

    def test_shell_gate_invoking_moved_script_fails(self):
        """gates/g3.sh called scripts/lean-axiom-probe.sh after the move:
        exit 127 in CI, invisible to the local suite."""
        self.fx.file("gates/g3.sh", "bash scripts/lean-axiom-probe.sh\n")
        r = self.fx.run()
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("MISSING-PATH scripts/lean-axiom-probe.sh", r.stdout)
        self.assertIn("gates/g3.sh", r.stdout)

    def test_rust_path_literal_to_moved_file_fails(self):
        """A Rust test read scripts/audit.sh by path and got NotFound."""
        self.fx.file(
            "forge/tests/audit_oracle.rs",
            'fn t() { let p = "scripts/audit.sh"; }\n',
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("MISSING-PATH scripts/audit.sh", r.stdout)

    def test_stale_include_str_fails(self):
        """forge/src/epr_reconstruct.rs include_str! is a COMPILE-TIME read;
        one stale path failed four CI jobs as a block."""
        self.fx.file(
            "forge/src/epr.rs",
            'const P: &str = include_str!("../pins/g4.txt");\n',
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("MISSING-PATH", r.stdout)
        self.assertIn("pins/g4.txt", r.stdout)

    # --- resolution ----------------------------------------------------

    def test_resolving_references_pass(self):
        self.fx.file("gates/probe.sh", "echo hi\n")
        self.fx.file("gates/g3.sh", "bash gates/probe.sh\n")
        self.fx.file("Makefile", "audit:\n\t@bash gates/g3.sh\n")
        self.fx.file(
            ".github/workflows/ci.yml",
            "jobs:\n  x:\n    steps:\n      - run: bash gates/g3.sh\n",
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("ok", r.stdout)

    def test_include_str_resolves_against_the_file(self):
        self.fx.file("forge/pins/g4.txt", "pin\n")
        self.fx.file(
            "forge/src/epr.rs",
            'const P: &str = include_str!("../pins/g4.txt");\n',
        )
        self.assertEqual(self.fx.run().returncode, 0)

    def test_crate_relative_literal_resolves(self):
        """Cargo tests run from the manifest dir, so forge/tests/x.rs may
        name tests/golden/... meaning forge/tests/golden/...."""
        self.fx.file("forge/tests/golden/sum.verus.rs", "golden\n")
        self.fx.file(
            "forge/tests/lower.rs",
            'fn t() { let p = "tests/golden/sum.verus.rs"; }\n',
        )
        self.assertEqual(self.fx.run().returncode, 0)

    def test_glob_reference_resolves_against_tracked_files(self):
        self.fx.file("docs/v2/program.md", "frozen\n")
        self.fx.file(
            ".github/workflows/ci.yml",
            "on:\n  push:\n    paths:\n      - docs/**\n",
        )
        self.assertEqual(self.fx.run().returncode, 0)

    def test_dead_glob_reference_fails(self):
        self.fx.file(
            ".github/workflows/ci.yml",
            "on:\n  push:\n    paths:\n      - scripts/g4-*\n",
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("MISSING-PATH scripts/g4-*", r.stdout)

    # --- deliberate blindness -------------------------------------------

    def test_comments_are_not_references(self):
        """A historical path in a comment is the frozen-record convention."""
        self.fx.file(
            "gates/g3.sh",
            "# used to live at scripts/g3-gate.sh\necho ok\n",
        )
        self.fx.file(
            "Makefile",
            "# see tooling/doc-drift.py for history\nall:\n\t@true\n",
        )
        self.assertEqual(self.fx.run().returncode, 0)

    def test_prose_slash_in_rust_string_is_not_a_reference(self):
        """'install lean/elan' is a sentence; only a literal that IS a path
        is checked."""
        self.fx.file(
            "forge/src/msg.rs",
            'const M: &str = "install lean/elan to exercise the suite";\n',
        )
        self.assertEqual(self.fx.run().returncode, 0)

    def test_dev_null_is_not_a_tree_reference(self):
        self.fx.file("gates/g3.sh", "run_thing 2>/dev/null\n")
        self.assertEqual(self.fx.run().returncode, 0)

    def test_variable_prefixed_reference_is_still_checked(self):
        """$tmp_dir/gates/x.py references gates/x.py — the doc-drift-ci
        Makefile shape."""
        self.fx.file(
            "Makefile",
            "check:\n\t@python \"$$tmp_dir/gates/doc-drift.py\"\n",
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("MISSING-PATH gates/doc-drift.py", r.stdout)

    def test_gitignored_generated_path_is_skipped(self):
        """lean/.lake/build in a CI cache block: generated, absence normal."""
        self.fx.file(".gitignore", "lean/.lake/\n")
        self.fx.file("lean/Main.lean", "def x := 1\n")
        self.fx.file(
            ".github/workflows/ci.yml",
            "jobs:\n  x:\n    steps:\n      - run: ls lean/.lake/build\n",
        )
        self.assertEqual(self.fx.run().returncode, 0)

    def test_test_fixture_paths_are_excluded(self):
        self.fx.file(
            "gates/tests/test_thing.py",
            'DOC = ".design/widget.md"\n',
        )
        self.assertEqual(self.fx.run().returncode, 0)

    def test_python_docstring_prose_is_not_a_reference(self):
        """A gate's docstring quotes historical paths as prose; only a
        literal that IS a path is checked. This is the self-scan case: the
        two gates flagged their own docstrings the moment they were
        tracked."""
        self.fx.file(
            "gates/history.py",
            '"""Once upon a time scripts/audit.sh lived here."""\n'
            "X = 1\n",
        )
        self.assertEqual(self.fx.run().returncode, 0)

    def test_python_path_literal_is_checked(self):
        """doc-drift's ROUTES_RELPATH shape: a config literal that IS a
        path must resolve — it was one of RFC-18 §3.3's named couplings."""
        self.fx.file(
            "gates/reader.py",
            'ROUTES_RELPATH = "gates/routes.toml"\n',
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("MISSING-PATH gates/routes.toml", r.stdout)

    def test_python_bare_prefix_literal_is_config(self):
        self.fx.file(
            "gates/config.py",
            'PREFIXES = ("gates/", "scripts/", "tooling/")\n',
        )
        self.assertEqual(self.fx.run().returncode, 0)

    # --- environment ----------------------------------------------------

    def test_outside_a_repo_is_inconclusive(self):
        with tempfile.TemporaryDirectory() as bare:
            r = subprocess.run(
                [sys.executable, str(GATE), "--root", bare],
                capture_output=True,
                text=True,
            )
        self.assertEqual(r.returncode, 3)
        self.assertIn("INCONCLUSIVE", r.stderr)


if __name__ == "__main__":
    unittest.main()
