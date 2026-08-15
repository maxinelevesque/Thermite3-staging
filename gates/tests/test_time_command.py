import json
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "gates/time-command.py"


class TimeCommandTests(unittest.TestCase):
    def test_records_success_and_failure_without_masking_status(self) -> None:
        for expected in (0, 7):
            with self.subTest(expected=expected), tempfile.TemporaryDirectory() as tmp:
                output = Path(tmp) / "timing.json"
                result = subprocess.run(
                    [
                        "python3",
                        str(SCRIPT),
                        "--out",
                        str(output),
                        "--label",
                        "sample",
                        "--",
                        "sh",
                        "-c",
                        f"exit {expected}",
                    ],
                    check=False,
                )
                self.assertEqual(result.returncode, expected)
                record = json.loads(output.read_text())
                self.assertEqual(record["schema"], 1)
                self.assertEqual(record["label"], "sample")
                self.assertEqual(record["exit_code"], expected)
                self.assertGreaterEqual(record["elapsed_seconds"], 0)


if __name__ == "__main__":
    unittest.main()
