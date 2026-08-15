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


if __name__ == "__main__":
    unittest.main()
