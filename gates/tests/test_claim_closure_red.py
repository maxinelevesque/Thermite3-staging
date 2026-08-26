#!/usr/bin/env python3
import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GATE = ROOT / "gates/completeness-review.py"
SPEC = importlib.util.spec_from_file_location("claim_closure_review", GATE)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class KnownRedCorpusTests(unittest.TestCase):
    def test_each_semantic_policy_inversion_is_rejected(self):
        verifier = [sys.executable, "gates/claim-closure-red.py"]
        oracle = "gates/fixtures/claim-closure-known-red.json"
        positive = MODULE.run_bound_verifier(ROOT, verifier, oracle)
        self.assertEqual(positive[0], 0)
        mutations = [
            ({
                "name": "unbind-executable-owner",
                "mutation": "replace_text",
                "from": '"executable_owner_is_content_bound": true',
                "to": '"executable_owner_is_content_bound": false',
            }, 11),
            ({
                "name": "launder-raw-provenance",
                "mutation": "replace_text",
                "from": '"raw_provenance_closes_semantic_claim": false',
                "to": '"raw_provenance_closes_semantic_claim": true',
            }, 7),
            ({
                "name": "weaken-shared-membership",
                "mutation": "replace_text",
                "from": '"shared_witness_membership_is_exact": true',
                "to": '"shared_witness_membership_is_exact": false',
            }, 8),
            ({
                "name": "demote-typed-authority",
                "mutation": "replace_text",
                "from": '"typed_claim_is_authoritative": true',
                "to": '"typed_claim_is_authoritative": false',
            }, 9),
        ]
        for mutation, expected_exit in mutations:
            with self.subTest(mutation=mutation["name"]):
                first = MODULE.run_mutated_verifier(ROOT, verifier, oracle, mutation)
                second = MODULE.run_mutated_verifier(ROOT, verifier, oracle, mutation)
                self.assertEqual(first, second)
                self.assertEqual(first[0], expected_exit)
                self.assertNotEqual(first[0], positive[0])
