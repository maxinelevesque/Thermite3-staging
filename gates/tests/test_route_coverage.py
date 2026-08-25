#!/usr/bin/env python3
"""Oracle tests for gates/route-coverage.py.

Each test asserts on the reported message rather than only the exit code. A
gate that fails for the wrong reason sends the next reader to the wrong place,
and an exit-code-only test cannot tell the two apart.

Fixtures are real git repositories: the gate enumerates the TRACKED tree, so a
file that exists but was never added must be invisible to it.
"""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


GATE = Path(__file__).resolve().parents[1] / "route-coverage.py"

GIT_ENV = {
    "GIT_AUTHOR_NAME": "t",
    "GIT_AUTHOR_EMAIL": "t@e",
    "GIT_COMMITTER_NAME": "t",
    "GIT_COMMITTER_EMAIL": "t@e",
    "PATH": "/usr/bin:/bin:/usr/local/bin",
    "HOME": "/nonexistent",  # no user gitconfig
}


class Fixture:
    def __init__(self, root: Path):
        self.root = root
        self.git("init", "-q")
        (root / "gates").mkdir()
        (root / ".design").mkdir()

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

    def routes(self, body: str):
        self.file("gates/routes.toml", body)

    def run(self):
        return subprocess.run(
            [sys.executable, str(GATE), "--root", str(self.root)],
            capture_output=True,
            text=True,
        )


ROUTE = """\
[[route]]
crate_pattern = "{pattern}"
design = "{design}"
reference = {reference}
conformance_ops = []
{extra}
"""


def route(pattern, design=".design/d.md", reference="[]", extra=""):
    return ROUTE.format(
        pattern=pattern, design=design, reference=reference, extra=extra
    )


class RouteCoverageTests(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.fx = Fixture(Path(self._tmp.name))
        self.fx.file(".design/d.md")

    def tearDown(self):
        self._tmp.cleanup()

    def test_resolving_route_passes(self):
        self.fx.file("forge/src/check.rs")
        self.fx.routes(route("forge/src/check.rs"))
        r = self.fx.run()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("ok", r.stdout)

    def test_dead_route_is_named(self):
        """A pattern matching nothing is the g4.sh failure: the file moved
        and the route silently points at nothing."""
        self.fx.file("forge/src/check.rs")
        self.fx.routes(
            route("forge/src/check.rs") + route("scripts/g4-*")
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("DEAD-ROUTE scripts/g4-*", r.stdout)
        self.assertIn("unbuilt = true", r.stdout)

    def test_unbuilt_route_is_exempt(self):
        self.fx.file("forge/src/check.rs")
        self.fx.routes(
            route("forge/src/check.rs")
            + route("forge/src/session.rs", extra="unbuilt = true")
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_stale_unbuilt_flag_is_named(self):
        """When the file lands, the exemption itself must be retired."""
        self.fx.file("forge/src/check.rs")
        self.fx.routes(
            route("forge/src/check.rs", extra="unbuilt = true")
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("STALE-UNBUILT forge/src/check.rs", r.stdout)
        self.assertIn("drop the flag", r.stdout)

    def test_untracked_target_is_dead(self):
        """The gate reads the tracked tree: a file on disk but never added
        does not satisfy a route."""
        self.fx.file("forge/src/check.rs")
        p = self.fx.root / "forge/src/ghost.rs"
        p.write_text("x\n", encoding="utf-8")  # exists, NOT added
        self.fx.routes(
            route("forge/src/check.rs") + route("forge/src/ghost.rs")
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("DEAD-ROUTE forge/src/ghost.rs", r.stdout)

    def test_missing_design_is_named(self):
        self.fx.file("forge/src/check.rs")
        self.fx.routes(route("forge/src/check.rs", design=".design/gone.md"))
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("MISSING-DESIGN .design/gone.md", r.stdout)

    def test_missing_reference_is_named(self):
        self.fx.file("forge/src/check.rs")
        self.fx.routes(
            route("forge/src/check.rs", reference='["conformance/gone"]')
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("MISSING-REFERENCE conformance/gone", r.stdout)

    def test_reference_satisfied_by_directory_prefix(self):
        self.fx.file("forge/src/check.rs")
        self.fx.file("conformance/parse/case1.th")
        self.fx.routes(
            route("forge/src/check.rs", reference='["conformance/parse"]')
        )
        r = self.fx.run()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_unrouted_gated_file_is_named(self):
        """The other direction: a gated .rs file no route reaches. The edit
        hook would block it (R-XLATE-2); this surfaces it before an edit."""
        self.fx.file("forge/src/check.rs")
        self.fx.file("forge/src/orphan.rs")
        self.fx.routes(route("forge/src/check.rs"))
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("UNROUTED forge/src/orphan.rs", r.stdout)
        self.assertIn("add a route", r.stdout)

    def test_ungated_files_are_not_swept(self):
        """Only the spec-discipline predicate is swept: non-.rs files,
        excluded crates, and files outside src/ carry no route obligation."""
        self.fx.file("forge/src/check.rs")
        self.fx.file("gates/g4.sh")
        self.fx.file("thermite-test-utils/src/lib.rs")
        self.fx.file("forge/tests/oracle.rs")
        self.fx.routes(route("forge/src/check.rs"))
        r = self.fx.run()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_glob_route_covers_gated_files(self):
        self.fx.file("forge/src/a.rs")
        self.fx.file("forge/src/b.rs")
        self.fx.routes(route("forge/src/*.rs"))
        r = self.fx.run()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    # --- the burn-down list -------------------------------------------

    def test_burndown_entry_suppresses_unrouted(self):
        self.fx.file("forge/src/check.rs")
        self.fx.file("forge/src/orphan.rs")
        self.fx.file(
            "gates/route-coverage-burndown.txt",
            "# measured debt\nforge/src/orphan.rs\n",
        )
        self.fx.routes(route("forge/src/check.rs"))
        r = self.fx.run()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("1 on the burn-down list", r.stdout)

    def test_burndown_entry_that_became_routed_is_stale(self):
        """The list only shrinks: a routed file must leave it."""
        self.fx.file("forge/src/check.rs")
        self.fx.file(
            "gates/route-coverage-burndown.txt", "forge/src/check.rs\n"
        )
        self.fx.routes(route("forge/src/check.rs"))
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("STALE-KNOWN-UNROUTED forge/src/check.rs", r.stdout)
        self.assertIn("now routed", r.stdout)

    def test_burndown_entry_that_left_the_tree_is_stale(self):
        self.fx.file("forge/src/check.rs")
        self.fx.file(
            "gates/route-coverage-burndown.txt", "forge/src/gone.rs\n"
        )
        self.fx.routes(route("forge/src/check.rs"))
        r = self.fx.run()
        self.assertEqual(r.returncode, 1)
        self.assertIn("STALE-KNOWN-UNROUTED forge/src/gone.rs", r.stdout)
        self.assertIn("left the tree", r.stdout)

    # --- environment --------------------------------------------------

    def test_empty_route_table_is_inconclusive(self):
        self.fx.routes("# no routes\n")
        r = self.fx.run()
        self.assertEqual(r.returncode, 3)
        self.assertIn("INCONCLUSIVE", r.stderr)

    def test_missing_route_table_is_inconclusive(self):
        r = self.fx.run()
        self.assertEqual(r.returncode, 3)
        self.assertIn("INCONCLUSIVE", r.stderr)

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
