"""Focused unit checks for config and request encoding."""
# test_scope: unit

from __future__ import annotations

import unittest
from unittest import mock
try:
    import pytest
except ImportError:  # pragma: no cover
    pytest = None

from bijux_atlas import ClientConfig, QueryRequest
from bijux_atlas.errors import AtlasConfigError

pytestmark = pytest.mark.unit if pytest is not None else []


class UnitTests(unittest.TestCase):
    def test_reject_invalid_base_url(self) -> None:
        with self.assertRaises(AtlasConfigError):
            ClientConfig(base_url="localhost:8080").validate()

    def test_query_payload(self) -> None:
        payload = QueryRequest(dataset="genes", filters={"chromosome": "1"}, limit=10).to_payload()
        self.assertEqual(payload["dataset"], "genes")
        self.assertEqual(payload["filters"], {"chromosome": "1"})
        self.assertEqual(payload["limit"], 10)

    def test_config_from_env(self) -> None:
        with mock.patch.dict(
            "os.environ",
            {
                "BIJUX_ATLAS_URL": "https://atlas.example",
                "BIJUX_ATLAS_TOKEN": "secret-token",
                "BIJUX_ATLAS_MAX_RETRIES": "3",
            },
            clear=True,
        ):
            cfg = ClientConfig.from_env()
        self.assertEqual(cfg.base_url, "https://atlas.example")
        self.assertEqual(cfg.auth_token, "secret-token")
        self.assertEqual(cfg.max_retries, 3)


if __name__ == "__main__":
    unittest.main()
