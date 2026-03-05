# test_scope: unit
from __future__ import annotations

import unittest
try:
    import pytest
except ImportError:  # pragma: no cover
    pytest = None

from bijux_atlas.retry import RetryPolicy

pytestmark = pytest.mark.unit if pytest is not None else []


class RetryPolicyTests(unittest.TestCase):
    def test_retries_server_error(self) -> None:
        policy = RetryPolicy(max_retries=2, backoff_seconds=0.01)
        self.assertTrue(policy.should_retry(0, 500, None))
        self.assertFalse(policy.should_retry(2, 500, None))

    def test_retries_transport_error(self) -> None:
        policy = RetryPolicy(max_retries=1, backoff_seconds=0.01)
        self.assertTrue(policy.should_retry(0, None, RuntimeError("io")))

    def test_backoff_increases_linearly(self) -> None:
        policy = RetryPolicy(max_retries=3, backoff_seconds=0.2)
        self.assertEqual(policy.backoff_delay(0), 0.2)
        self.assertEqual(policy.backoff_delay(1), 0.4)


if __name__ == "__main__":
    unittest.main()
