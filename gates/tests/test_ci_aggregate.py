import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "gates/ci-aggregate.py"


class AggregateTests(unittest.TestCase):
    def run_aggregate(self, *results: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", str(SCRIPT), "G", *results],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_all_success_passes(self) -> None:
        self.assertEqual(self.run_aggregate("success", "success").returncode, 0)

    def test_failure_fails(self) -> None:
        self.assertEqual(self.run_aggregate("failure").returncode, 1)

    def test_cancelled_fails(self) -> None:
        self.assertEqual(self.run_aggregate("cancelled").returncode, 1)

    def test_skipped_fails(self) -> None:
        self.assertEqual(self.run_aggregate("skipped").returncode, 1)


if __name__ == "__main__":
    unittest.main()
