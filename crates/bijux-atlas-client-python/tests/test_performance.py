from __future__ import annotations
# test_scope: perf

import time
import unittest

from atlas_client.query import QueryRequest


class PerformanceTests(unittest.TestCase):
    @unittest.skipUnless(
        __import__("os").getenv("ATLAS_CLIENT_RUN_PERF") == "1",
        "set ATLAS_CLIENT_RUN_PERF=1 to run performance tests",
    )
    def test_payload_encoding_budget(self) -> None:
        start = time.perf_counter()
        for _ in range(20_000):
            QueryRequest(dataset="genes", filters={"chromosome": "1"}, limit=100).to_payload()
        elapsed = time.perf_counter() - start
        self.assertLess(elapsed, 1.5)


if __name__ == "__main__":
    unittest.main()
