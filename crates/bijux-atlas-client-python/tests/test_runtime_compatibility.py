# test_scope: unit
from __future__ import annotations

import json
import pathlib
import unittest


class RuntimeCompatibilityTests(unittest.TestCase):
    def test_runtime_compatibility_matrix_has_supported_versions(self) -> None:
        fixture = pathlib.Path(__file__).parent / "fixtures" / "runtime-compatibility.json"
        data = json.loads(fixture.read_text(encoding="utf-8"))
        self.assertEqual(data["python_sdk"], "atlas-client")
        self.assertIn("0.1.x", data["supported_runtime_versions"])


if __name__ == "__main__":
    unittest.main()
