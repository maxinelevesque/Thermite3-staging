#!/usr/bin/env python3
import importlib.util
import hashlib
import json
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

GATE = Path(__file__).resolve().parents[1] / "claim-closure-author.py"
SPEC = importlib.util.spec_from_file_location("claim_closure_author", GATE)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class ClaimClosureAuthorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / ".design/reqs").mkdir(parents=True)
        (self.root / "gates").mkdir()
        (self.root / MODULE.REGISTRY).write_text(
            '''
schema_version = 1
[[requirement]]
id = "REQ-B"
status = "shipped"
[[requirement]]
id = "REQ-A"
status = "shipped"
[[requirement]]
id = "REQ-FUTURE"
status = "not_started"
'''.lstrip(),
            encoding="utf-8",
        )
        (self.root / MODULE.LEDGER).write_text(
            'version = 1\n\n[[item]]\nid = "CR-X"\n', encoding="utf-8"
        )
        self.previous_size = MODULE.BASELINE_SIZE
        self.previous_review_size = MODULE.REVIEW.BASELINE_SHIPPED_COUNT
        MODULE.BASELINE_SIZE = 2
        MODULE.REVIEW.BASELINE_SHIPPED_COUNT = 2

    def tearDown(self):
        MODULE.BASELINE_SIZE = self.previous_size
        MODULE.REVIEW.BASELINE_SHIPPED_COUNT = self.previous_review_size
        self.tmp.cleanup()

    def test_freeze_is_sorted_exact_and_keeps_staging_version(self):
        MODULE.freeze_baseline(self.root)

        text = (self.root / MODULE.LEDGER).read_text(encoding="utf-8")
        self.assertTrue(text.startswith("version = 1\n"))
        self.assertLess(text.index('"REQ-A"'), text.index('"REQ-B"'))
        self.assertEqual(MODULE.check_baseline(self.root), [])

    def test_freeze_refuses_an_unexpected_population(self):
        MODULE.BASELINE_SIZE = 3
        with self.assertRaisesRegex(ValueError, "refusing to freeze 2 shipped IDs"):
            MODULE.freeze_baseline(self.root)

    def test_exact_population_draft_includes_live_addition_at_materialization(self):
        MODULE.BASELINE_SIZE = 1
        MODULE.REVIEW.BASELINE_SHIPPED_COUNT = 1
        summary = "The closed modes are alpha and beta."
        (self.root / MODULE.REGISTRY).write_text(
            f'''
# A comment preamble is part of the live registry format.
schema_version = 1
[[requirement]]
id = "REQ-A"
status = "shipped"
summary = "{summary}"
[[requirement]]
id = "REQ-LIVE"
status = "shipped"
summary = "A live shipped addition uses the same closed modes."
'''.lstrip(),
            encoding="utf-8",
        )
        (self.root / MODULE.LEDGER).write_text(
            'version = 1\nbaseline_shipped_ids = ["REQ-A"]\n',
            encoding="utf-8",
        )
        (self.root / MODULE.REVIEW.INVENTORY).write_text(
            "version = 1\n", encoding="utf-8"
        )
        (self.root / "modes.txt").write_text("mode=alpha\nmode=beta\n", encoding="utf-8")
        (self.root / "live-modes.txt").write_text(
            "mode=gamma\nmode=delta\n", encoding="utf-8"
        )
        (self.root / "gates/completeness-review.py").write_text(
            "# bound verifier\n", encoding="utf-8"
        )
        (self.root / "gates/claim_closure_schema.py").write_text(
            "# bound schema\n", encoding="utf-8"
        )
        draft_dir = self.root / MODULE.DRAFT_DIR
        draft_dir.mkdir()
        draft = {
            "version": 1,
            "slice_id": "modes",
            "entries": [
                {
                    "requirement_id": "REQ-A",
                    "witness_id": "W-MODES",
                    "claim": {
                        "kind": "exact_population",
                        "subject": r"regex:modes.txt#^mode=([a-z_]+)$",
                        "expected": ["alpha", "beta"],
                        "reviewed_summary_sha256": hashlib.sha256(
                            summary.encode("utf-8")
                        ).hexdigest(),
                    },
                    "closure": {
                        "population_semantics": "closed_enum",
                        "artifacts": [
                            "modes.txt#mode=alpha",
                            "gates/completeness-review.py",
                            "gates/claim_closure_schema.py",
                        ],
                        "extractor": {
                            "kind": "regex",
                            "path": "modes.txt",
                            "pattern": r"^mode=([a-z_]+)$",
                        },
                        "counterfeit": [
                            {"name": "addition"},
                            {"name": "duplication"},
                            {"name": "omission"},
                            {"name": "substitution"},
                        ],
                    },
                }
            ],
        }
        live_entry = json.loads(json.dumps(draft["entries"][0]))
        live_entry["requirement_id"] = "REQ-LIVE"
        live_entry["claim"]["subject"] = (
            r"regex:live-modes.txt#^mode=([a-z_]+)$"
        )
        live_entry["claim"]["expected"] = ["delta", "gamma"]
        live_entry["claim"]["reviewed_summary_sha256"] = hashlib.sha256(
            b"A live shipped addition uses the same closed modes."
        ).hexdigest()
        live_entry["closure"]["artifacts"][0] = "live-modes.txt#mode=gamma"
        live_entry["closure"]["extractor"]["path"] = "live-modes.txt"
        draft["entries"].append(live_entry)
        (draft_dir / "modes.json").write_text(
            json.dumps(draft, indent=2) + "\n", encoding="utf-8"
        )

        authored, problems = MODULE.check_drafts(self.root)

        self.assertEqual(problems, [])
        self.assertEqual(len(authored), 2)
        self.assertRegex(authored[0]["closure"]["receipt"], r"^[0-9a-f]{64}$")
        registry_sha = hashlib.sha256(
            (self.root / MODULE.REGISTRY).read_bytes()
        ).hexdigest()
        (self.root / MODULE.INVENTORY).write_text(
            "version = 1\n"
            "[[claim]]\n"
            'id = "CLAIM-REGISTRY-SHIPPED-EVIDENCE"\n'
            f'source = "{MODULE.REGISTRY}"\n'
            f'source_sha256 = "{registry_sha}"\n',
            encoding="utf-8",
        )
        self.assertEqual(MODULE.coordinated_materialize(self.root), 2)
        registry = tomllib.loads((self.root / MODULE.REGISTRY).read_text())
        ledger = tomllib.loads((self.root / MODULE.LEDGER).read_text())
        inventory = tomllib.loads((self.root / MODULE.INVENTORY).read_text())
        self.assertEqual(registry["schema_version"], 2)
        self.assertEqual(registry["requirement"][0]["claim"]["expected"], ["alpha", "beta"])
        self.assertEqual(ledger["version"], 2)
        self.assertEqual(ledger["witness"][0]["members"], ["REQ-A", "REQ-LIVE"])
        self.assertEqual(
            inventory["claim"][0]["source_sha256"],
            hashlib.sha256((self.root / MODULE.REGISTRY).read_bytes()).hexdigest(),
        )

        self.assertEqual(MODULE.coordinated_materialize(self.root), 2)
        refreshed_registry = tomllib.loads((self.root / MODULE.REGISTRY).read_text())
        refreshed_ledger = tomllib.loads((self.root / MODULE.LEDGER).read_text())
        self.assertEqual(refreshed_registry["schema_version"], 2)
        self.assertEqual(len(refreshed_ledger["closure"]), 2)

    def test_executable_draft_round_trips_through_authoritative_gate(self):
        MODULE.BASELINE_SIZE = 1
        MODULE.REVIEW.BASELINE_SHIPPED_COUNT = 1
        summary = "The JSON oracle is accepted and its semantic neighbour is rejected."
        (self.root / MODULE.REGISTRY).write_text(
            f'''
schema_version = 1
[[requirement]]
id = "REQ-A"
status = "shipped"
owner = "implementation.rs"
summary = "{summary}"
'''.lstrip(),
            encoding="utf-8",
        )
        (self.root / MODULE.LEDGER).write_text(
            'version = 1\nbaseline_shipped_ids = ["REQ-A"]\n', encoding="utf-8"
        )
        (self.root / MODULE.REVIEW.INVENTORY).write_text(
            "version = 1\n", encoding="utf-8"
        )
        (self.root / "gates/completeness-review.py").write_text(
            "# bound verifier\n", encoding="utf-8"
        )
        (self.root / "gates/claim_closure_schema.py").write_text(
            "# bound schema\n", encoding="utf-8"
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
        (self.root / "implementation.rs").write_text(
            "// implementation under test\n", encoding="utf-8"
        )
        draft_dir = self.root / MODULE.DRAFT_DIR
        draft_dir.mkdir()
        draft = {
            "version": 1,
            "slice_id": "executable",
            "entries": [
                {
                    "requirement_id": "REQ-A",
                    "witness_id": "W-EXECUTABLE",
                    "claim": {
                        "kind": "executable_discriminator",
                        "subject": "oracle:oracle.json",
                        "expected": ["accepted"],
                        "reviewed_summary_sha256": hashlib.sha256(
                            summary.encode("utf-8")
                        ).hexdigest(),
                    },
                    "closure": {
                        "verifier": [sys.executable, "verify.py"],
                        "tool_version_argv": [sys.executable, "--version"],
                        "oracle": "oracle.json",
                        "artifacts": [
                            "oracle.json",
                            "verify.py",
                            "implementation.rs",
                            "gates/completeness-review.py",
                            "gates/claim_closure_schema.py",
                        ],
                        "counterfeit": [
                            {
                                "name": "semantic-neighbour",
                                "mutation": "replace_text",
                                "from": '"good"',
                                "to": '"evil"',
                                "expected_exit": 7,
                            }
                        ],
                    },
                }
            ],
        }
        (draft_dir / "executable.json").write_text(
            json.dumps(draft, indent=2) + "\n", encoding="utf-8"
        )

        unbound = json.loads(json.dumps(draft))
        unbound["entries"][0]["closure"]["artifacts"].remove("implementation.rs")
        (draft_dir / "executable.json").write_text(
            json.dumps(unbound, indent=2) + "\n", encoding="utf-8"
        )
        _, unbound_problems = MODULE.check_drafts(self.root)
        self.assertTrue(
            any("requirement owner must be content-bound" in value for value in unbound_problems)
        )
        (draft_dir / "executable.json").write_text(
            json.dumps(draft, indent=2) + "\n", encoding="utf-8"
        )

        authored, problems = MODULE.check_drafts(self.root)

        self.assertEqual(problems, [])
        self.assertEqual(len(authored), 1)
        MODULE.materialize(self.root, authored)
        self.assertEqual(MODULE.REVIEW.check(self.root), [])

        sharded = []
        for shard_index in range(3):
            results, selected, groups, shard_problems = MODULE.check_draft_shard(
                self.root, shard_index, 3
            )
            self.assertEqual(shard_problems, [])
            self.assertEqual(len(results), selected)
            self.assertLessEqual(groups, selected)
            sharded.extend(result["requirement_id"] for result in results)
        self.assertEqual(sharded, ["REQ-A"])

    def test_draft_shards_are_stable_complete_and_keep_shared_oracles_together(self):
        shared = {
            "claim": {"kind": "executable_discriminator"},
            "closure": {"verifier": ["python", "verify.py"], "oracle": "same.json"},
        }
        entries = [
            {**shared, "requirement_id": "REQ-A"},
            {**shared, "requirement_id": "REQ-B"},
            {
                "requirement_id": "REQ-C",
                "claim": {"kind": "executable_discriminator"},
                "closure": {
                    "verifier": ["python", "verify.py"],
                    "oracle": "other.json",
                },
            },
            {
                "requirement_id": "REQ-D",
                "claim": {"kind": "formal_theorem"},
                "closure": {},
            },
            {
                "requirement_id": "REQ-E",
                "claim": {"kind": "formal_theorem"},
                "closure": {},
            },
        ]

        assignments = [MODULE.draft_shard(entry, 8) for entry in entries]

        self.assertEqual(assignments[0], assignments[1])
        self.assertEqual(assignments[3], assignments[4])
        self.assertEqual(assignments, [MODULE.draft_shard(entry, 8) for entry in entries])
        covered = [
            index
            for shard_index in range(8)
            for index, assignment in enumerate(assignments)
            if assignment == shard_index
        ]
        self.assertEqual(sorted(covered), list(range(len(entries))))

    def test_parse_shard_spec_is_one_based_and_fail_closed(self):
        self.assertEqual(MODULE.parse_shard_spec("1/8"), (0, 8))
        self.assertEqual(MODULE.parse_shard_spec("8/8"), (7, 8))
        for value in ("0/8", "9/8", "1/0", "1", "1/2/3", "a/b"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                MODULE.parse_shard_spec(value)

    def test_shard_refuses_an_incomplete_global_draft_population(self):
        authored, selected, groups, problems = MODULE.check_draft_shard(
            self.root, 0, 2
        )

        self.assertEqual(authored, [])
        self.assertEqual(selected, 0)
        self.assertEqual(groups, 0)
        self.assertTrue(any("draft population is missing" in value for value in problems))


if __name__ == "__main__":
    unittest.main()
