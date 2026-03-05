# test_scope: unit
from __future__ import annotations

import unittest
try:
    import pytest
except ImportError:  # pragma: no cover
    pytest = None

from bijux_atlas.errors import AtlasApiError, AtlasConfigError, AtlasRetryExhaustedError

pytestmark = pytest.mark.unit if pytest is not None else []


def classify_error(error: Exception) -> str:
    if isinstance(error, AtlasConfigError):
        return "configuration"
    if isinstance(error, AtlasApiError):
        if error.status_code >= 500:
            return "server"
        if error.status_code >= 400:
            return "request"
    if isinstance(error, AtlasRetryExhaustedError):
        return "transport"
    return "unknown"


class ErrorClassificationTests(unittest.TestCase):
    def test_classify_config_error(self) -> None:
        self.assertEqual(classify_error(AtlasConfigError("bad")), "configuration")

    def test_classify_request_error(self) -> None:
        self.assertEqual(classify_error(AtlasApiError(404, "missing")), "request")

    def test_classify_server_error(self) -> None:
        self.assertEqual(classify_error(AtlasApiError(500, "boom")), "server")

    def test_classify_retry_error(self) -> None:
        self.assertEqual(classify_error(AtlasRetryExhaustedError(3)), "transport")


if __name__ == "__main__":
    unittest.main()
