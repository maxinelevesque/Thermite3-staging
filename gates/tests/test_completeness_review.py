#!/usr/bin/env python3
import hashlib
import importlib.util
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path
from unittest import mock

GATE = Path(__file__).resolve().parents[1] / "completeness-review.py"
SPEC = importlib.util.spec_from_file_location("completeness_review", GATE)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class ReviewTrackTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "gates").mkdir()
        (self.root / MODULE.GATE).write_text("# fixture gate\n", encoding="utf-8")
        (self.root / MODULE.SCHEMA).write_text("# fixture schema\n", encoding="utf-8")
        (self.root / ".design/reqs").mkdir(parents=True)
        (self.root / "proof.py").write_text(
            "def closure_counterfeit(): pass\ndef closure_witness(): pass\n",
            encoding="utf-8",
        )
        self.previous_baseline_count = MODULE.BASELINE_SHIPPED_COUNT
        MODULE.BASELINE_SHIPPED_COUNT = 1

    def tearDown(self):
        MODULE.BASELINE_SHIPPED_COUNT = self.previous_baseline_count
        self.tmp.cleanup()

    @staticmethod
    def open_gap():
        return '''
            [[gap]]
            id = "GAP-X"
            status = "open"
            disposition = "completeness_review"
            issue = "https://github.com/o/r/issues/1"
        '''

    @staticmethod
    def open_item():
        return '''
            [[item]]
            id = "CR-X"
            gap = "GAP-X"
            issue = "https://github.com/o/r/issues/1"
            status = "open"
            closure_evidence = []
        '''

    def write(
        self,
        gap: str,
        item: str,
        *,
        include_closure: bool = True,
        witness_members: str = '"REQ-X"',
        stale_receipt: bool = False,
    ):
        claim = {
            "kind": "exact_population",
            "subject": r"regex:proof.py#^def ([a-z_]+)\(\): pass$",
            "expected": ["closure_counterfeit", "closure_witness"],
        }
        claim_sha = MODULE.claim_digest("REQ-X", claim)
        assert claim_sha is not None
        summary_sha = hashlib.sha256(b"Fixture closure.").hexdigest()
        (self.root / ".design/reqs/registry.toml").write_text(
            textwrap.dedent(
                f'''
                schema_version = 2

                [[requirement]]
                id = "REQ-X"
                title = "Fixture closure"
                owner = "proof.py"
                status = "shipped"
                scope = "tooling"
                summary = "Fixture closure."
                claim = {{ kind = "exact_population", subject = 'regex:proof.py#^def ([a-z_]+)\\(\\): pass$', expected = ["closure_counterfeit", "closure_witness"], reviewed_summary_sha256 = "{summary_sha}" }}
                generated_to = ["status"]
                '''
            ).lstrip(),
            encoding="utf-8",
        )

        counterfeits = [
            {"name": "addition"},
            {"name": "duplication"},
            {"name": "omission"},
            {"name": "substitution"},
        ]
        closure = {
            "requirement_id": "REQ-X",
            "witness_id": "W-X",
            "mechanism": "exact_population",
            "population_semantics": "closed_case_set",
            "claim_digest": claim_sha,
            "verifier": ["builtin:exact_population"],
            "verifier_version": MODULE.gate_version(self.root),
            "artifacts": ["proof.py#closure_witness", "gates/completeness-review.py", "gates/claim_closure_schema.py"],
            "expected": ["closure_counterfeit", "closure_witness"],
            "extractor": {
                "kind": "regex",
                "path": "proof.py",
                "pattern": r"^def ([a-z_]+)\(\): pass$",
            },
            "counterfeit": counterfeits,
        }
        observed = ["closure_counterfeit", "closure_witness"]
        discriminator = MODULE.discriminator_digest(self.root, closure, observed)
        assert discriminator is not None
        closure["discriminator"] = discriminator
        receipt = MODULE.closure_receipt(self.root, closure, observed=observed)
        assert receipt is not None
        if stale_receipt:
            receipt = "0" * 64

        closure_toml = ""
        if include_closure:
            closure_toml = textwrap.dedent(
                f'''
                [[closure]]
                requirement_id = "REQ-X"
                witness_id = "W-X"
                mechanism = "exact_population"
                population_semantics = "closed_case_set"
                discriminator = "{discriminator}"
                claim_digest = "{claim_sha}"
                verifier = ["builtin:exact_population"]
                verifier_version = "{MODULE.gate_version(self.root)}"
                artifacts = ["proof.py#closure_witness", "gates/completeness-review.py", "gates/claim_closure_schema.py"]
                expected = ["closure_counterfeit", "closure_witness"]
                extractor = {{ kind = "regex", path = "proof.py", pattern = '^def ([a-z_]+)\\(\\): pass$' }}
                counterfeit = [{{ name = "addition" }}, {{ name = "duplication" }}, {{ name = "omission" }}, {{ name = "substitution" }}]
                receipt = "{receipt}"
                '''
            )

        (self.root / MODULE.INVENTORY).write_text(
            "version = 1\n" + textwrap.dedent(gap), encoding="utf-8"
        )
        (self.root / MODULE.BACKLOG).write_text(
            "version = 2\n"
            'baseline_shipped_ids = ["REQ-X"]\n'
            + textwrap.dedent(item)
            + textwrap.dedent(
                f'''
                [[witness]]
                id = "W-X"
                mechanism = "exact_population"
                members = [{witness_members}]
                '''
            )
            + closure_toml,
            encoding="utf-8",
        )

    def write_executable(self, *, counterfeit_field: str = ""):
        (self.root / "implementation.rs").write_text(
            "// implementation under test\n", encoding="utf-8"
        )
        (self.root / "oracle.json").write_text(
            '{"verdict":"good"}\n', encoding="utf-8"
        )
        (self.root / "verify.py").write_text(
            "import json, pathlib, sys\n"
            "value = json.loads(pathlib.Path(sys.argv[1]).read_text())\n"
            "sys.exit(0 if value == {'verdict': 'good'} else 7)\n",
            encoding="utf-8",
        )
        claim = {
            "kind": "executable_discriminator",
            "subject": "oracle:oracle.json",
            "expected": ["accepted"],
        }
        claim_sha = MODULE.claim_digest("REQ-X", claim)
        assert claim_sha is not None
        summary_sha = hashlib.sha256(b"Fixture closure.").hexdigest()
        (self.root / ".design/reqs/registry.toml").write_text(
            textwrap.dedent(
                f'''
                schema_version = 2
                [[requirement]]
                id = "REQ-X"
                title = "Fixture closure"
                owner = "implementation.rs"
                status = "shipped"
                scope = "tooling"
                summary = "Fixture closure."
                claim = {{ kind = "executable_discriminator", subject = "oracle:oracle.json", expected = ["accepted"], reviewed_summary_sha256 = "{summary_sha}" }}
                generated_to = ["status"]
                '''
            ).lstrip(),
            encoding="utf-8",
        )
        verifier = [sys.executable, "verify.py"]
        version_argv = [sys.executable, "--version"]
        tool_version = MODULE.command_version(self.root, version_argv)
        closure = {
            "requirement_id": "REQ-X",
            "witness_id": "W-X",
            "mechanism": "executable_discriminator",
            "claim_digest": claim_sha,
            "verifier": verifier,
            "verifier_version": MODULE.gate_version(self.root),
            "tool_version_argv": version_argv,
            "tool_version": tool_version,
            "oracle": "oracle.json",
            "artifacts": [
                "oracle.json",
                "verify.py",
                "implementation.rs",
                "gates/completeness-review.py",
                "gates/claim_closure_schema.py",
            ],
            "expected": ["accepted"],
            "counterfeit": [
                {
                    "name": "replace-positive-oracle",
                    "mutation": "replace_text",
                    "from": '"good"',
                    "to": '"evil"',
                    "expected_exit": 7,
                }
            ],
        }
        _, observation_digest = MODULE.run_bound_verifier(
            self.root, verifier, "oracle.json"
        )
        observed = {"result": ["accepted"], "output_digest": observation_digest}
        discriminator = MODULE.discriminator_digest(self.root, closure, observed)
        assert discriminator is not None
        closure["discriminator"] = discriminator
        receipt = MODULE.closure_receipt(self.root, closure, observed=observed)
        assert receipt is not None
        (self.root / MODULE.INVENTORY).write_text(
            "version = 1\n" + textwrap.dedent(self.open_gap()), encoding="utf-8"
        )
        (self.root / MODULE.BACKLOG).write_text(
            textwrap.dedent(
                f'''
                version = 2
                baseline_shipped_ids = ["REQ-X"]
                {self.open_item()}
                [[witness]]
                id = "W-X"
                mechanism = "executable_discriminator"
                members = ["REQ-X"]

                [[closure]]
                requirement_id = "REQ-X"
                witness_id = "W-X"
                mechanism = "executable_discriminator"
                discriminator = "{discriminator}"
                claim_digest = "{claim_sha}"
                verifier = ["{sys.executable}", "verify.py"]
                verifier_version = "{MODULE.gate_version(self.root)}"
                tool_version_argv = ["{sys.executable}", "--version"]
                tool_version = "{tool_version}"
                oracle = "oracle.json"
                artifacts = ["oracle.json", "verify.py", "implementation.rs", "gates/completeness-review.py", "gates/claim_closure_schema.py"]
                expected = ["accepted"]
                counterfeit = [{{ name = "replace-positive-oracle", mutation = "replace_text", from = '\"good\"', to = '\"evil\"', expected_exit = 7{counterfeit_field} }}]
                receipt = "{receipt}"
                '''
            ).lstrip(),
            encoding="utf-8",
        )

    def test_open_gap_and_backlog_agree_with_complete_closure(self):
        self.write(self.open_gap(), self.open_item())
        self.assertEqual(MODULE.check(self.root), [])

    def test_missing_and_orphan_items_fail_both_directions(self):
        self.write(self.open_gap(), "")
        self.assertTrue(any("expected exactly one" in e for e in MODULE.check(self.root)))
        self.write("", self.open_item())
        self.assertTrue(any("orphan" in e for e in MODULE.check(self.root)))

    def test_closing_item_while_gap_open_fails(self):
        closed = self.open_item().replace('status = "open"', 'status = "closed"').replace(
            "closure_evidence = []", 'closure_evidence = ["proof.py#closure_witness"]'
        )
        self.write(self.open_gap(), closed)
        self.assertTrue(any("open gap requires an open" in e for e in MODULE.check(self.root)))

    def test_resolved_gap_requires_closed_item(self):
        gap = '''
            [[gap]]
            id = "GAP-X"
            status = "resolved"
            review_item = "CR-X"
        '''
        self.write(gap, self.open_item())
        self.assertTrue(any("resolved gap requires a closed" in e for e in MODULE.check(self.root)))

    def test_closed_item_without_resolving_evidence_fails(self):
        gap = '''
            [[gap]]
            id = "GAP-X"
            status = "resolved"
            review_item = "CR-X"
        '''
        closed = self.open_item().replace('status = "open"', 'status = "closed"')
        self.write(gap, closed)
        self.assertTrue(any("requires executable/formal" in e for e in MODULE.check(self.root)))

    def test_missing_shipped_closure_fails(self):
        self.write(self.open_gap(), self.open_item(), include_closure=False)
        self.assertTrue(any("lack closure" in e for e in MODULE.check(self.root)))

    def test_shared_witness_membership_is_exact(self):
        self.write(
            self.open_gap(),
            self.open_item(),
            witness_members='"REQ-X", "REQ-EXTRA"',
        )
        self.assertTrue(any("declared members differ" in e for e in MODULE.check(self.root)))

    def test_stale_receipt_fails(self):
        self.write(self.open_gap(), self.open_item(), stale_receipt=True)
        self.assertTrue(any("receipt is missing or stale" in e for e in MODULE.check(self.root)))

    def test_markdown_audit_pin_is_not_semantic_evidence(self):
        path = self.root / "design.md"
        path.write_text(
            "audited-content-sha256: " + "1" * 64 + " (first pin)\nBody.\n",
            encoding="utf-8",
        )
        first = MODULE.artifact_digest(self.root, "design.md")
        path.write_text(
            "audited-content-sha256: " + "2" * 64 + " (refreshed pin)\nBody.\n",
            encoding="utf-8",
        )
        self.assertEqual(first, MODULE.artifact_digest(self.root, "design.md"))
        path.write_text(
            "audited-content-sha256: " + "2" * 64 + " (refreshed pin)\nChanged.\n",
            encoding="utf-8",
        )
        self.assertNotEqual(first, MODULE.artifact_digest(self.root, "design.md"))

    def test_executable_discriminator_runs_same_verifier_on_mutated_oracle(self):
        self.write_executable()
        self.assertEqual(MODULE.check(self.root), [])

    def test_executable_discriminator_binds_requirement_owner(self):
        self.write_executable()
        path = self.root / MODULE.BACKLOG
        path.write_text(
            path.read_text(encoding="utf-8").replace(
                '"implementation.rs", ',
                "",
            ),
            encoding="utf-8",
        )

        problems = MODULE.check(self.root)

        self.assertTrue(
            any("requirement owner must be content-bound" in value for value in problems)
        )

    def test_executable_discriminator_rejects_second_counterfeit_verifier(self):
        self.write_executable(counterfeit_field=', verifier = ["false"]')
        self.assertTrue(
            any("counterfeit contains unknown field" in e for e in MODULE.check(self.root))
        )

    def test_executable_discriminator_rejects_byte_corruption_counterfeit(self):
        self.write_executable()
        path = self.root / MODULE.BACKLOG
        path.write_text(
            path.read_text(encoding="utf-8").replace(
                'mutation = "replace_text"',
                'mutation = "flip_byte"',
            ),
            encoding="utf-8",
        )

        problems = MODULE.check(self.root)

        self.assertTrue(any("semantic replace_text" in e for e in problems))

    def test_constant_nonzero_verifier_cannot_launder_oracle_and_counterfeit(self):
        self.write_executable()
        (self.root / "verify.py").write_text(
            "import sys\nsys.exit(7)\n", encoding="utf-8"
        )

        problems = MODULE.check(self.root)

        self.assertTrue(any("positive oracle must exit 0" in e for e in problems))
        self.assertTrue(any("counterfeit did not discriminate" in e for e in problems))

    def test_exact_population_observation_comes_from_bound_artifact(self):
        self.write(self.open_gap(), self.open_item())
        (self.root / "proof.py").write_text("def different(): pass\n", encoding="utf-8")
        self.assertTrue(
            any("extracted population differs" in e for e in MODULE.check(self.root))
        )

    def test_v1_staging_cannot_predeclare_v2_claims_or_closures(self):
        self.write(self.open_gap(), self.open_item())
        ledger = self.root / MODULE.BACKLOG
        ledger.write_text(
            ledger.read_text(encoding="utf-8").replace("version = 2", "version = 1", 1),
            encoding="utf-8",
        )
        registry = self.root / ".design/reqs/registry.toml"
        registry.write_text(
            registry.read_text(encoding="utf-8").replace(
                "schema_version = 2", "schema_version = 1", 1
            ),
            encoding="utf-8",
        )

        problems = MODULE.check(self.root)

        self.assertTrue(any("must not predeclare v2 witnesses" in e for e in problems))
        self.assertTrue(any("must not predeclare v2 typed claims" in e for e in problems))

    def test_formal_probe_binds_the_kernel_reported_type(self):
        module_path = self.root / "lean/Thermite/Demo.lean"
        module_path.parent.mkdir(parents=True)
        module_path.write_text("theorem t : True := by trivial\n", encoding="utf-8")
        version = mock.Mock(returncode=0, stdout="Lake test (Lean test)\n", stderr="")
        first_lean = mock.Mock(
            returncode=0,
            stdout=(
                "Thermite.Demo.t : True\n"
                "'Thermite.Demo.t' does not depend on any axioms\n"
            ),
            stderr="",
        )
        second_lean = mock.Mock(
            returncode=0,
            stdout=(
                "Thermite.Demo.t : False\n"
                "'Thermite.Demo.t' does not depend on any axioms\n"
            ),
            stderr="",
        )
        with mock.patch.object(MODULE.shutil, "which", return_value="/test/lake"), mock.patch.object(
            MODULE.subprocess, "run", side_effect=[version, first_lean, version, second_lean]
        ) as run:
            first, _ = MODULE.probe_lean_theorem(
                self.root, "lean/Thermite/Demo.lean#Thermite.Demo.t"
            )
            second, _ = MODULE.probe_lean_theorem(
                self.root, "lean/Thermite/Demo.lean#Thermite.Demo.t"
            )

        self.assertIsNotNone(first)
        self.assertIsNotNone(second)
        self.assertNotEqual(first[1], second[1])
        self.assertTrue(first[1].startswith("type_sha256:"))
        self.assertEqual(run.call_args_list[0].kwargs["cwd"], self.root / "lean")
        self.assertEqual(run.call_args_list[2].kwargs["cwd"], self.root / "lean")


if __name__ == "__main__":
    unittest.main()
