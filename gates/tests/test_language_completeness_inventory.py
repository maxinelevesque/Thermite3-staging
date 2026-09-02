import importlib.util
import hashlib
import tempfile
import unittest
from pathlib import Path

GATE = Path(__file__).resolve().parents[1] / "language-completeness-inventory.py"
SPEC = importlib.util.spec_from_file_location("language_inventory", GATE)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class AstInventoryTests(unittest.TestCase):
    ROOT = GATE.parents[1]

    def test_extracts_all_variant_shapes_and_ignores_comments(self):
        source = """
        // pub enum Fake { Nope }
        pub enum Item {
            Unit,
            Tuple(Vec<(A, B)>),
            Struct { field: Vec<T> },
        }
        """
        self.assertEqual(
            MODULE.enum_variants(source),
            {"Item::Unit", "Item::Tuple", "Item::Struct"},
        )

    def test_new_variant_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "ast.rs").write_text("pub enum Item { Old, New }", encoding="utf-8")
            (root / "evidence.rs").write_text("evidence", encoding="utf-8")
            (root / "claim.md").write_text("anchor", encoding="utf-8")
            claim_sha = hashlib.sha256(b"anchor").hexdigest()
            profiles = "\n".join(f'{stage} = "supported"' for stage in MODULE.STAGES)
            increments = "\n".join(
                f'[[rfc3_increment]]\nid = "R2-{n}"\nstatus = "partial"\nevidence = ["evidence.rs"]'
                for n in range(1, 10)
            )
            inventory = f'''ast_source = "ast.rs"
[[profile]]
id = "all"
{profiles}
evidence = ["evidence.rs"]
[[construct]]
id = "Item::Old"
profile = "all"
[[claim]]
id = "C-1"
source = "claim.md"
source_sha256 = "{claim_sha}"
anchor = "anchor"
evidence = ["evidence.rs"]
{increments}
'''
            path = root / "inventory.toml"
            path.write_text(inventory, encoding="utf-8")
            errors = MODULE.check(root, path)
            self.assertTrue(any("Item::New" in error for error in errors), errors)

    def test_checked_support_matrix_is_generated_from_ledger(self):
        data = MODULE.tomllib.loads(
            (self.ROOT / MODULE.INVENTORY).read_text(encoding="utf-8")
        )
        self.assertEqual(
            (self.ROOT / MODULE.MATRIX).read_text(encoding="utf-8"),
            MODULE.matrix_text(data),
        )
        self.assertEqual(len(MODULE.support_matrix(data)["constructs"]), 128)
        self.assertEqual(len(MODULE.support_matrix(data)["claims"]), 11)

    def test_claim_without_stage_profile_fails_closed(self):
        text = (self.ROOT / MODULE.INVENTORY).read_text(encoding="utf-8")
        text = text.replace('profile = "current-language"\n', "", 1)
        with tempfile.NamedTemporaryFile("w", suffix=".toml", delete=False) as handle:
            handle.write(text)
            path = Path(handle.name)
        try:
            errors = MODULE.check(self.ROOT, path)
            self.assertTrue(any("missing stage profile" in error for error in errors), errors)
        finally:
            path.unlink()

    def test_authoritative_source_population_cannot_silently_shrink(self):
        text = (self.ROOT / MODULE.INVENTORY).read_text(encoding="utf-8")
        text = text.replace('"docs/verification.md",', '"docs/does-not-exist.md",')
        with tempfile.NamedTemporaryFile("w", suffix=".toml", delete=False) as handle:
            handle.write(text)
            path = Path(handle.name)
        try:
            errors = MODULE.check(self.ROOT, path)
            self.assertTrue(any("matches nothing" in error for error in errors), errors)
        finally:
            path.unlink()

    def test_open_gap_requires_disposition_specific_evidence(self):
        text = (self.ROOT / MODULE.INVENTORY).read_text(encoding="utf-8")
        text += '''

[[gap]]
id = "GAP-MISSING-REVIEW-ISSUE-FIXTURE"
status = "open"
stages = ["certification"]
counterexample = "A fixture semantic counterexample."
claimed = "A fixture claim."
observed = "A fixture observation."
trust_consequence = "A fixture trust consequence."
disposition = "completeness_review"
evidence = []
'''
        with tempfile.NamedTemporaryFile("w", suffix=".toml", delete=False) as handle:
            handle.write(text)
            path = Path(handle.name)
        try:
            errors = MODULE.check(self.ROOT, path)
            self.assertTrue(any("requires a GitHub issue URL" in error for error in errors), errors)
        finally:
            path.unlink()


if __name__ == "__main__":
    unittest.main()
