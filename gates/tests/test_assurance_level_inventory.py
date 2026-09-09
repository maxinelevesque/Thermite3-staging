import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GATE = ROOT / "gates/assurance-level-inventory.py"
SPEC = importlib.util.spec_from_file_location("assurance_level_inventory", GATE)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class AssuranceLevelInventoryTests(unittest.TestCase):
    def test_scanner_ignores_comments_literals_and_cfg_test_items(self):
        source = '''
// Level::L3 is prose.
const LABEL: &str = "Level::L2";
fn display(level: Level) -> Level { level }
fn describe(cert: &Certificate) -> Level { cert.compatibility_level() }
fn headline(row: &Row) -> Level { row.level }
#[cfg(test)]
mod tests {
    fn fixture() { let _ = Level::L4; let _ = cert.compatibility_level(); }
}
'''
        code = MODULE.code_only(source)
        lines = code.splitlines()
        excluded = MODULE.cfg_test_lines(lines)
        visible = [
            signal
            for line_number, line in enumerate(lines, 1)
            if line_number not in excluded
            for signal, _match in MODULE.level_signals(line)
        ]
        self.assertEqual(
            visible,
            [
                "level_token",
                "level_token",
                "level_token",
                "value_access",
                "level_token",
                "value_access",
            ],
        )

    def test_new_sites_require_review_and_authority_decisions_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "forge/src").mkdir(parents=True)
            (root / "gates").mkdir()
            (root / "forge/src/lib.rs").write_text(
                "pub fn render(level: Level) -> Level { row.level }\n",
                encoding="utf-8",
            )
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(
                ["git", "-C", str(root), "add", "forge/src/lib.rs"], check=True
            )

            write = subprocess.run(
                [sys.executable, str(GATE), "--root", str(root), "--write"],
                capture_output=True,
                text=True,
            )
            self.assertEqual(write.returncode, 0, write.stderr)
            unreviewed = subprocess.run(
                [sys.executable, str(GATE), "--root", str(root), "--check"],
                capture_output=True,
                text=True,
            )
            self.assertEqual(unreviewed.returncode, 1)
            self.assertIn("unclassified", unreviewed.stderr)

            baseline = root / MODULE.BASELINE
            value = json.loads(baseline.read_text(encoding="utf-8"))
            for site in value["sites"]:
                site["category"] = "authority_decision"
            baseline.write_text(
                json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
            # Re-render once so the generated summary agrees with the reviewed sites.
            subprocess.run(
                [sys.executable, str(GATE), "--root", str(root), "--write"], check=True
            )
            decision = subprocess.run(
                [sys.executable, str(GATE), "--root", str(root), "--check"],
                capture_output=True,
                text=True,
            )
            self.assertEqual(decision.returncode, 1)
            self.assertIn("production authority decisions remain", decision.stderr)


if __name__ == "__main__":
    unittest.main()
