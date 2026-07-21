import os
import unittest
import subprocess

class TestTDLibUpgradeAndTransitionPlan(unittest.TestCase):
    def setUp(self):
        self.root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def test_cmakelists_version(self):
        cmake_path = os.path.join(self.root_dir, "CMakeLists.txt")
        self.assertTrue(os.path.exists(cmake_path), "CMakeLists.txt does not exist")
        with open(cmake_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("find_package(Td 1.8.65 REQUIRED)", content, 
                      "CMakeLists.txt does not require TDLib version 1.8.65")

    def test_portfile_cmake_settings(self):
        portfile_path = os.path.join(self.root_dir, "tools", "vcpkg", "ports", "tdlib", "portfile.cmake")
        self.assertTrue(os.path.exists(portfile_path), "portfile.cmake does not exist")
        with open(portfile_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("set(VERSION 1.8.65)", content, "portfile.cmake VERSION is not 1.8.65")
        self.assertIn("REF 022d60202e446ad1287b9fb68e687c8a0760788b", content, 
                      "portfile.cmake REF commit is incorrect")
        self.assertIn("SHA512 7f6446c2c2937dba8971d8b13b67ae7a0056aa812a9ae55bcbdb7875213421262d09613be1869525b2e3e8c2f4b494b7521d0f36e7257e87f5d0d0fa867f604c", 
                      content, "portfile.cmake SHA512 hash is incorrect")

    def test_vcpkg_control_version(self):
        control_path = os.path.join(self.root_dir, "tools", "vcpkg", "ports", "tdlib", "CONTROL")
        self.assertTrue(os.path.exists(control_path), "CONTROL file does not exist")
        with open(control_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("Version: 1.8.65", content, "CONTROL Version is not 1.8.65")

    def test_dockerfile_commit(self):
        dockerfile_path = os.path.join(self.root_dir, "tools", "docker", "linux-builder", "Dockerfile")
        self.assertTrue(os.path.exists(dockerfile_path), "Dockerfile does not exist")
        with open(dockerfile_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("git reset --hard 022d60202e446ad1287b9fb68e687c8a0760788b", content, 
                      "Dockerfile git reset commit is incorrect")

    def test_agents_doc_exists(self):
        agents_path = os.path.join(self.root_dir, "AGENTS.md")
        self.assertTrue(os.path.exists(agents_path), "AGENTS.md does not exist")
        self.assertGreater(os.path.getsize(agents_path), 0, "AGENTS.md is empty")

    def test_gitignore_ignores_agents(self):
        gitignore_path = os.path.join(self.root_dir, ".gitignore")
        self.assertTrue(os.path.exists(gitignore_path), ".gitignore does not exist")
        with open(gitignore_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("AGENTS.md", content, ".gitignore does not list AGENTS.md")
        
        # Check using git check-ignore if git is available
        try:
            res = subprocess.run(
                ["git", "check-ignore", "AGENTS.md"],
                cwd=self.root_dir,
                capture_output=True,
                text=True
            )
            self.assertEqual(res.returncode, 0, "AGENTS.md is not git-ignored according to git check-ignore")
        except FileNotFoundError:
            # git command is not in PATH, skip git check-ignore validation
            pass

    def test_rust_transition_plan_exists(self):
        plan_path = os.path.join(self.root_dir, "docs", "rust_transition_plan.md")
        self.assertTrue(os.path.exists(plan_path), "rust_transition_plan.md does not exist")
        self.assertGreater(os.path.getsize(plan_path), 0, "rust_transition_plan.md is empty")

if __name__ == "__main__":
    unittest.main()
