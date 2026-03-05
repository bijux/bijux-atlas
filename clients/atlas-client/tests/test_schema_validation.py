from __future__ import annotations

import json
import pathlib
import unittest


def _validate_simple(config: dict[str, object]) -> None:
    base_url = config.get("base_url")
    if not isinstance(base_url, str) or not base_url.startswith(("http://", "https://")):
        raise ValueError("base_url invalid")

    timeout = config.get("timeout_seconds", 10.0)
    if not isinstance(timeout, (int, float)) or timeout <= 0:
        raise ValueError("timeout_seconds invalid")


class ConfigSchemaValidationTests(unittest.TestCase):
    def test_schema_fixture_present(self) -> None:
        fixture = pathlib.Path(__file__).parent / "fixtures" / "client-config.schema.json"
        schema = json.loads(fixture.read_text(encoding="utf-8"))
        self.assertEqual(schema["title"], "atlas-client-config")

    def test_validate_valid_config(self) -> None:
        _validate_simple({"base_url": "http://localhost:8080", "timeout_seconds": 3.0})

    def test_validate_invalid_config(self) -> None:
        with self.assertRaises(ValueError):
            _validate_simple({"base_url": "localhost:8080"})


if __name__ == "__main__":
    unittest.main()
