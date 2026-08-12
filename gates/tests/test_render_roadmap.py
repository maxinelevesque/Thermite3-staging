"""Regression tests for the derived roadmap renderer."""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("render_roadmap", ROOT / "dev/render-roadmap.py")
roadmap = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(roadmap)


class RoadmapRendererTests(unittest.TestCase):
    def test_thermite3_guide_covers_rfc_7_through_13(self):
        rfcs, problem = roadmap.load_rfc_index(ROOT)
        self.assertIsNone(problem)
        guides, problem = roadmap.load_rfc_guides(ROOT, rfcs)
        self.assertIsNone(problem)
        self.assertEqual([guide["rfc"] for guide in guides], list(range(7, 14)))

    def test_guide_rejects_duplicate_or_missing_entries(self):
        rfcs = [{"rfc": str(n), "title": f"RFC {n}", "status": "draft", "file": f"{n}.md"}
                for n in range(7, 14)]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "dev").mkdir()
            data = json.loads((ROOT / "dev/roadmap-rfcs.json").read_text(encoding="utf-8"))
            data["guides"][-1]["rfc"] = 12
            (root / "dev/roadmap-rfcs.json").write_text(json.dumps(data), encoding="utf-8")
            guides, problem = roadmap.load_rfc_guides(root, rfcs)
        self.assertIsNone(guides)
        self.assertIn("duplicate", problem)

    def test_doc_drift_treats_unexpected_exit_as_inconclusive(self):
        with mock.patch.object(roadmap, "_run", return_value=(2, "", "broken invocation")):
            current, drifted, problem = roadmap.load_doc_drift(ROOT)
        self.assertEqual((current, drifted), ([], []))
        self.assertIn("exit 2", problem)

    def test_meter_clamps_invalid_progress_width(self):
        self.assertIn("width:100%", roadmap.meter("scope", 3, 2))
        self.assertIn("width:0%", roadmap.meter("scope", -1, 2))

    def test_rfc_cards_escape_content_and_use_source_links(self):
        guide = {
            "rfc": 9,
            "summary": "checked <effects>",
            "features": ["A & B"],
            "example_label": "Example",
            "example": "x < y",
            "index": {"title": "Rows", "status": "draft", "file": "0009.md"},
        }
        rendered = roadmap.render_rfc_guides(
            [guide],
            [{"status": "shipped", "contributors": [".design/rfcs/0009.md"]}],
            "https://example.test/repo/blob/deadbeef",
        )
        self.assertIn("https://example.test/repo/blob/deadbeef/.design/rfcs/0009.md", rendered)
        self.assertIn("checked &lt;effects&gt;", rendered)
        self.assertIn("A &amp; B", rendered)
        self.assertIn("1/1 registered requirements shipped", rendered)

    def test_origin_ssh_url_becomes_browser_url(self):
        with mock.patch.object(
            roadmap, "_run", return_value=(0, "git@github.com:owner/repo.git\n", "")
        ):
            self.assertEqual(roadmap.repository_web_url(ROOT), "https://github.com/owner/repo")


if __name__ == "__main__":
    unittest.main()
