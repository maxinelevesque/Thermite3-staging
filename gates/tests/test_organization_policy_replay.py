import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class OrganizationPolicyReplayTest(unittest.TestCase):
    def test_generated_matrix_is_current(self):
        subprocess.run(
            [sys.executable, "gates/organization-policy-replay.py"],
            cwd=ROOT,
            check=True,
        )

    def test_organization_authority_never_reads_presentation_fields(self):
        authority = (ROOT / "forge/src/organization_assurance.rs").read_text()
        for forbidden in (
            "EngineerClaim",
            "EngineerStatement",
            "pub evidence_frontier",
            "pub badge",
            "pub percentage",
        ):
            self.assertNotIn(forbidden, authority)


if __name__ == "__main__":
    unittest.main()
