#!/usr/bin/env python3
import unittest

from gates.claim_closure_schema import (
    claim_expectation_problem,
    claim_subject_problem,
)


class ClaimClosureSchemaTests(unittest.TestCase):
    def test_formal_subject_requires_repo_relative_lean_declaration(self):
        self.assertIsNone(
            claim_subject_problem(
                "formal_theorem", "lean/Thermite/Demo.lean#Thermite.Demo.sound"
            )
        )
        for subject in (
            "/tmp/Demo.lean#Demo.sound",
            "../Demo.lean#Demo.sound",
            "lean/Demo.txt#Demo.sound",
            "lean/Demo.lean",
        ):
            self.assertIsNotNone(claim_subject_problem("formal_theorem", subject))

    def test_executable_subject_requires_repo_relative_json_oracle(self):
        self.assertIsNone(
            claim_subject_problem("executable_discriminator", "oracle:gates/oracle.json")
        )
        for subject in (
            "oracle:/tmp/oracle.json",
            "oracle:../oracle.json",
            "oracle:gates/oracle.txt",
            "gates/oracle.json",
        ):
            self.assertIsNotNone(
                claim_subject_problem("executable_discriminator", subject)
            )

    def test_exact_population_requires_one_whole_line_capture(self):
        self.assertIsNone(
            claim_subject_problem(
                "exact_population", r"regex:modes.txt#^mode=([a-z_]+)$"
            )
        )
        for subject in (
            r"regex:../modes.txt#^mode=([a-z_]+)$",
            r"regex:modes.txt#mode=([a-z_]+)",
            r"regex:modes.txt#^mode=[a-z_]+$",
            r"regex:modes.txt#^mode=([a-z_]+):(.*)$",
        ):
            self.assertIsNotNone(claim_subject_problem("exact_population", subject))

    def test_exact_population_requires_distinct_multi_member_expectation(self):
        self.assertIsNotNone(claim_expectation_problem("exact_population", ["one"]))
        self.assertIsNotNone(
            claim_expectation_problem("exact_population", ["same", "same"])
        )
        self.assertIsNone(
            claim_expectation_problem("exact_population", ["alpha", "beta"])
        )


if __name__ == "__main__":
    unittest.main()
