"""Focused unit checks for config and request encoding."""
# test_scope: unit

from __future__ import annotations

import unittest
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


if __name__ == "__main__":
    unittest.main()
