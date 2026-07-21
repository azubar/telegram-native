import os
import unittest
import subprocess

class TestTDLibUpgradeAndTransitionPlan(unittest.TestCase):
    def setUp(self):
        self.root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def test_cargo_toml(self):
        cargo_path = os.path.join(self.root_dir, "Cargo.toml")
        self.assertTrue(os.path.exists(cargo_path), "Cargo.toml does not exist")
        with open(cargo_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn('name = "telegram_native"', content)
        self.assertIn('version = "1.8.65"', content)

    def test_release_workflow(self):
        workflow_path = os.path.join(self.root_dir, ".github", "workflows", "release.yml")
        self.assertTrue(os.path.exists(workflow_path), "release.yml workflow does not exist")

    def test_usage_1c_doc(self):
        usage_path = os.path.join(self.root_dir, "docs", "usage_1c.md")
        self.assertTrue(os.path.exists(usage_path), "usage_1c.md does not exist")

    def test_manifest_xml(self):
        manifest_path = os.path.join(self.root_dir, "MANIFEST.xml")
        self.assertTrue(os.path.exists(manifest_path), "MANIFEST.xml does not exist")

    def test_rust_transition_plan_exists(self):
        plan_path = os.path.join(self.root_dir, "docs", "rust_transition_plan.md")
        self.assertTrue(os.path.exists(plan_path), "rust_transition_plan.md does not exist")
        self.assertGreater(os.path.getsize(plan_path), 0, "rust_transition_plan.md is empty")

if __name__ == "__main__":
    unittest.main()
