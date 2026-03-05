from __future__ import annotations

import pathlib
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]


class TutorialScenarioTests(unittest.TestCase):
    def test_tutorial_cli_workflow(self) -> None:
        script = ROOT / "tutorials/scripts/tutorial_cli_workflow.sh"
        result = subprocess.run([str(script)], cwd=ROOT, capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("tutorial CLI workflow completed", result.stdout)

    def test_dataset_packaging(self) -> None:
        script = ROOT / "tutorials/scripts/package_example_datasets.sh"
        result = subprocess.run([str(script)], cwd=ROOT, capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        out = ROOT / "tutorials/evidence/dataset-packages/atlas-example-minimal.tar.gz"
        self.assertTrue(out.exists())


if __name__ == "__main__":
    unittest.main()
