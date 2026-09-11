import importlib.util
import re
import unittest
from pathlib import Path


def load_gate():
    path = Path(__file__).parents[1] / "assurance-transport-replay.py"
    spec = importlib.util.spec_from_file_location("assurance_transport_replay", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class AssuranceTransportReplayTest(unittest.TestCase):
    def test_assurance_transport_replay_is_clean(self):
        gate = load_gate()
        self.assertEqual(gate.validate(Path(__file__).parents[2]), [])

    def test_transport_decisions_do_not_read_archival_levels(self):
        root = Path(__file__).parents[2]
        for relative in (
            "forge/src/assurance_transport.rs",
            "lean/Thermite/AssuranceTransport.lean",
            "lean/Thermite/AssuranceTransportReplay.lean",
            "gates/assurance-transport-replay.py",
        ):
            text = (root / relative).read_text(encoding="utf-8")
            executable = "\n".join(
                line
                for line in text.splitlines()
                if not line.lstrip().startswith(("//!", "/--", "--"))
            )
            self.assertIsNone(
                re.search(r"\bLevel\b|\bL[0-4]\b", executable), relative
            )
