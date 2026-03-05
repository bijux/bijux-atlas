from __future__ import annotations

import time
import unittest

from atlas_client.query import QueryRequest


class PerformanceTests(unittest.TestCase):
    def test_payload_encoding_budget(self) -> None:
        start = time.perf_counter()
        for _ in range(20_000):
            QueryRequest(dataset="genes", filters={"chromosome": "1"}, limit=100).to_payload()
        elapsed = time.perf_counter() - start
        self.assertLess(elapsed, 1.5)


if __name__ == "__main__":
    unittest.main()
