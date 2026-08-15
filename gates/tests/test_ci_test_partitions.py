import importlib.util
import sys
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).parents[1] / "ci-test-partitions.py"
SPEC = importlib.util.spec_from_file_location("ci_test_partitions", SCRIPT)
PARTITIONS = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = PARTITIONS
SPEC.loader.exec_module(PARTITIONS)


class PartitionTests(unittest.TestCase):
    def sample_tests(self, count=12):
        return [PARTITIONS.TestId("pkg::bin", f"test_{index:02}") for index in range(count)]

    def manifest(self):
        return {
            "schema": 1,
            "bucket_count": 13,
            "explicit_bucket_count": 12,
            "catch_all_bucket": 13,
        }

    def test_lpt_allocation_is_deterministic_and_complete(self):
        tests = self.sample_tests()
        timings = {test: float(index + 1) for index, test in enumerate(tests)}
        forward = PARTITIONS.allocate(tests, timings)
        reverse = PARTITIONS.allocate(list(reversed(tests)), timings)
        self.assertEqual(forward, reverse)
        self.assertEqual({item.test for item in forward}, set(tests))
        self.assertTrue(all(1 <= item.bucket <= 12 for item in forward))

    def test_duplicate_assignment_fails_closed(self):
        tests = self.sample_tests(2)
        assignments = [
            PARTITIONS.Assignment(tests[0], 1.0, 1),
            PARTITIONS.Assignment(tests[0], 1.0, 2),
            PARTITIONS.Assignment(tests[1], 1.0, 3),
        ]
        errors = PARTITIONS.validate_manifest(self.manifest(), assignments, tests)
        self.assertTrue(any("duplicate assignments" in error for error in errors))

    def test_deleted_assignment_enters_catch_all_and_fails_review_gate(self):
        tests = self.sample_tests(2)
        assignments = [PARTITIONS.Assignment(tests[0], 1.0, 1)]
        errors = PARTITIONS.validate_manifest(self.manifest(), assignments, tests)
        self.assertTrue(any("unreviewed catch-all" in error for error in errors))
        explicit = {item.test for item in assignments}
        self.assertEqual(sorted(set(tests) - explicit), [tests[1]])

    def test_renamed_assignment_is_both_stale_and_unreviewed(self):
        tests = self.sample_tests(1)
        stale = PARTITIONS.TestId("pkg::bin", "old_name")
        assignments = [PARTITIONS.Assignment(stale, 1.0, 1)]
        errors = PARTITIONS.validate_manifest(self.manifest(), assignments, tests)
        self.assertTrue(any("stale assignments" in error for error in errors))
        self.assertTrue(any("unreviewed catch-all" in error for error in errors))

    def test_filter_expression_qualifies_binary_and_exact_test(self):
        expression = PARTITIONS.filter_expression(
            [PARTITIONS.TestId("pkg::one", "same"), PARTITIONS.TestId("pkg::two", "same")]
        )
        self.assertIn("binary_id(=pkg::one)", expression)
        self.assertIn("binary_id(=pkg::two)", expression)
        self.assertEqual(expression.count("test(=same)"), 2)

    def test_empty_catch_all_still_has_a_valid_nextest_filter(self):
        self.assertEqual(PARTITIONS.filter_expression([]), "not all()")

    def test_inventory_disables_ci_forced_color(self):
        completed = PARTITIONS.subprocess.CompletedProcess(
            args=[], returncode=0, stdout="pkg::bin test_name\n", stderr=""
        )
        with mock.patch.object(PARTITIONS.subprocess, "run", return_value=completed) as run:
            self.assertEqual(
                PARTITIONS.inventory(Path("/repo")),
                [PARTITIONS.TestId("pkg::bin", "test_name")],
            )
        command = run.call_args.args[0]
        self.assertEqual(command[command.index("--color") + 1], "never")

    def test_simulation_uses_longest_test_as_lower_bound(self):
        test = PARTITIONS.TestId("pkg::bin", "elephant")
        rows = PARTITIONS.simulation([PARTITIONS.Assignment(test, 319.485, 1)])
        self.assertEqual(rows[0][4], 319.485)


if __name__ == "__main__":
    unittest.main()
