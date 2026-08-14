import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class GateSegmentTests(unittest.TestCase):
    def assert_invalid_segment_fails_closed(self, gate: str) -> None:
        result = subprocess.run(
            ["bash", str(ROOT / "gates" / gate), "not-a-segment"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("usage:", result.stderr)

    def test_g3_segment_inventory_and_closed_selector(self) -> None:
        source = (ROOT / "gates/g3.sh").read_text()
        self.assertIn("all|parser-lowering|checked-replay", source)
        self.assertIn("run_parser_lowering", source)
        self.assertIn("run_checked_replay", source)
        self.assert_invalid_segment_fails_closed("g3.sh")

    def test_g4_segment_inventory_and_closed_selector(self) -> None:
        source = (ROOT / "gates/g4.sh").read_text()
        self.assertIn(
            "all|bridge-lean|lrat-cache|release-routing|hygiene", source
        )
        for function in (
            "run_bridge_lean",
            "run_lrat_cache",
            "run_release_routing",
            "run_hygiene",
        ):
            self.assertIn(function, source)
        self.assert_invalid_segment_fails_closed("g4.sh")

    def test_g4_memory_limit_reexec_preserves_segment(self) -> None:
        source = (ROOT / "gates/g4.sh").read_text()
        self.assertIn('bash "$ROOT/gates/g4.sh" "$@"', source)


if __name__ == "__main__":
    unittest.main()
