import importlib.util
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
