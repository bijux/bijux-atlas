"""Integration tests for bijux-atlas against a local HTTP server."""
# test_scope: integration
# requires_env: BIJUX_ATLAS_URL=http://127.0.0.1:8080

from __future__ import annotations

import json
import os
import threading
import unittest
from http.server import BaseHTTPRequestHandler, HTTPServer
from socketserver import ThreadingMixIn
try:
    import pytest
except ImportError:  # pragma: no cover
    pytest = None

from bijux_atlas import AtlasClient, ClientConfig, QueryRequest

pytestmark = pytest.mark.integration if pytest is not None else []


class _ThreadedHTTPServer(ThreadingMixIn, HTTPServer):
    daemon_threads = True


class _QueryHandler(BaseHTTPRequestHandler):
    server_version = "AtlasTestServer/1.0"

    def log_message(self, format: str, *args: object) -> None:  # noqa: A003
        return

    def do_POST(self) -> None:  # noqa: N802
        if self.path != "/v1/query":
            self.send_response(404)
            self.end_headers()
            return

        content_length = int(self.headers.get("Content-Length", "0"))
        payload = json.loads(self.rfile.read(content_length).decode("utf-8"))

        dataset = payload.get("dataset")
        token = payload.get("page_token")

        if dataset != "genes":
            self.send_response(400)
            self.end_headers()
            self.wfile.write(b'{"error":"invalid dataset"}')
            return

        if token == "p2":
            body = {"items": [{"gene_id": "g3"}], "next_page_token": None}
        elif token == "p1":
            body = {"items": [{"gene_id": "g2"}], "next_page_token": "p2"}
        else:
            body = {"items": [{"gene_id": "g1"}], "next_page_token": "p1"}

        encoded = json.dumps(body).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)


class IntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if not os.getenv("BIJUX_ATLAS_URL"):
            cls.server = None
            cls.thread = None
            cls.base_url = ""
            return
        cls.server = _ThreadedHTTPServer(("127.0.0.1", 0), _QueryHandler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        host, port = cls.server.server_address
        cls.base_url = f"http://{host}:{port}"

    @classmethod
    def tearDownClass(cls) -> None:
        if cls.server is None:
            return
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join(timeout=2)

    @unittest.skipUnless(
        os.getenv("BIJUX_ATLAS_URL"),
        "set BIJUX_ATLAS_URL to run integration tests",
    )
    def test_query_single_page(self) -> None:
        client = AtlasClient(ClientConfig(base_url=self.base_url))
        page = client.query(QueryRequest(dataset="genes", limit=1))
        self.assertEqual(page.items, [{"gene_id": "g1"}])
        self.assertEqual(page.next_token, "p1")

    @unittest.skipUnless(
        os.getenv("BIJUX_ATLAS_URL"),
        "set BIJUX_ATLAS_URL to run integration tests",
    )
    def test_stream_query(self) -> None:
        client = AtlasClient(ClientConfig(base_url=self.base_url))
        rows = list(client.stream_query(QueryRequest(dataset="genes", limit=1)))
        self.assertEqual(rows, [{"gene_id": "g1"}, {"gene_id": "g2"}, {"gene_id": "g3"}])


if __name__ == "__main__":
    unittest.main()
