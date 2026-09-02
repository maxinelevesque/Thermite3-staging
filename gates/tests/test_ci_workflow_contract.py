import re
import subprocess
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/ci.yml"
DESIGN = ROOT / ".design/ci-duration-balanced-fanout.md"
MANIFEST = ROOT / "gates/ci-test-partitions.toml"


class CiWorkflowContractTests(unittest.TestCase):
    def workflow(self) -> str:
        return WORKFLOW.read_text(encoding="utf-8")

    def job(self, name: str) -> str:
        source = self.workflow()
        match = re.search(
            rf"(?ms)^  {re.escape(name)}:\n(.*?)(?=^  [a-zA-Z0-9_-]+:\n|\Z)",
            source,
        )
        self.assertIsNotNone(match, f"missing workflow job {name}")
        return match.group(1)

    def test_before_after_report_separates_live_metrics(self) -> None:
        design = DESIGN.read_text(encoding="utf-8")
        for run in ("31811912559", "31837152080", "31843256167"):
            self.assertIn(run, design)
        for dimension in (
            "critical path",
            "aggregate runner time",
            "test-execution spread",
            "queue delay",
            "Non-suite per-test-job time",
            "1,533 tests exactly once",
        ):
            self.assertIn(dimension, design)
        self.assertIn("attempts 1 and 2", design)

    def test_thirteen_duration_buckets_match_the_manifest(self) -> None:
        manifest = tomllib.loads(MANIFEST.read_text(encoding="utf-8"))
        self.assertEqual(manifest["bucket_count"], 13)
        self.assertEqual(manifest["explicit_bucket_count"], 12)
        self.assertEqual(manifest["catch_all_bucket"], 13)
        self.assertIn(
            "bucket: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]",
            self.job("test"),
        )

    def test_test_partitions_restore_every_required_proof_tool(self) -> None:
        job = self.job("test")
        for required in (
            "Restore prepared Lean build",
            "Install Verus",
            '"$VERUS_BIN_PATH" --version',
            "Install pinned CaDiCaL and drat-trim",
            "THERMITE_EPR_CADICAL",
            "THERMITE_EPR_DRAT_TRIM",
        ):
            self.assertIn(required, job)

    def test_prepared_lean_build_includes_rfc11_resource_replay(self) -> None:
        for job_name in ("lean-prepare", "test"):
            self.assertIn("Thermite.ResourceFlow", self.job(job_name))

    def test_gate_fanout_and_stable_aggregates_are_closed(self) -> None:
        self.assertIn(
            "segment: [parser-lowering, checked-replay]", self.job("g3_children")
        )
        self.assertIn(
            "segment: [bridge-lean, lrat-cache, release-routing, hygiene]",
            self.job("g4_children"),
        )
        for aggregate, child in (("g3", "g3_children"), ("g4", "g4_children")):
            job = self.job(aggregate)
            self.assertIn(f"needs: [changes, {child}]", job)
            self.assertIn("if: always()", job)
            self.assertIn(f"needs.{child}.result", job)
            self.assertIn("gates/ci-aggregate.py", job)

    def test_timing_artifacts_are_published_even_on_failure(self) -> None:
        test_job = self.job("test")
        self.assertIn("Publish nextest timing report", test_job)
        self.assertIn("if: always()", test_job)
        self.assertIn("nextest-junit-bucket-${{ matrix.bucket }}", test_job)
        for name, label in (("g3_children", "G3"), ("g4_children", "G4")):
            job = self.job(name)
            self.assertIn("gates/time-command.py", job)
            self.assertIn(f"Publish {label} segment timing", job)
            self.assertIn("if: always()", job)
            self.assertIn("if-no-files-found: error", job)

    def test_ci_optimization_landed_after_rfc10_without_rewriting_it(self) -> None:
        merged_ci_pr = "92310867"
        post_rfc10_staging = "15d362df"
        ancestry = subprocess.run(
            ["git", "merge-base", "--is-ancestor", post_rfc10_staging, merged_ci_pr],
            cwd=ROOT,
            check=False,
        )
        self.assertEqual(ancestry.returncode, 0)
        changed = subprocess.run(
            ["git", "diff", "--name-only", f"{post_rfc10_staging}..{merged_ci_pr}"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertIn(".design/ci-duration-balanced-fanout.md", changed)
        self.assertFalse(any("rfc-0010" in path.lower() for path in changed))


if __name__ == "__main__":
    unittest.main()
