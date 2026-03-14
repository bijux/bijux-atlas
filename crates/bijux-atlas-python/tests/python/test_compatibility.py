# test_scope: unit
from __future__ import annotations

import unittest
import warnings
from unittest import mock

try:
    import pytest
except ImportError:  # pragma: no cover
    pytest = None

from bijux_atlas import AtlasClient, ClientConfig
from bijux_atlas.http import HttpTransport

pytestmark = pytest.mark.unit if pytest is not None else []


class CompatibilityTests(unittest.TestCase):
    def test_discover_runtime_falls_back_to_health(self) -> None:
        client = AtlasClient(ClientConfig(base_url="https://atlas.example"))
        with mock.patch.object(
            HttpTransport,
            "get_json",
            side_effect=[RuntimeError("no /version"), {"status": "ok", "runtime_version": "0.1.5"}],
        ):
            payload = client.discover_runtime_info()
        self.assertEqual(payload["discovery_endpoint"], "/health")
        self.assertEqual(payload["runtime_version"], "0.1.5")

    def test_check_compatibility_supported_runtime(self) -> None:
        client = AtlasClient(ClientConfig(base_url="https://atlas.example"))
        with mock.patch.object(client, "discover_runtime_info", return_value={"version": "0.1.8"}):
            result = client.check_compatibility()
        self.assertTrue(result["is_supported"])
        self.assertEqual(result["runtime_version"], "0.1.8")

    def test_check_compatibility_warns_for_unsupported_runtime(self) -> None:
        client = AtlasClient(ClientConfig(base_url="https://atlas.example"))
        with mock.patch.object(client, "discover_runtime_info", return_value={"version": "2.0.0"}):
            with warnings.catch_warnings(record=True) as caught:
                warnings.simplefilter("always")
                result = client.check_compatibility()
        self.assertFalse(result["is_supported"])
        self.assertTrue(any("outside supported ranges" in str(item.message) for item in caught))
