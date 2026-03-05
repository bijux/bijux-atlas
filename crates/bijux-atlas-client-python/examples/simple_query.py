# Purpose: demonstrate Atlas Python client usage for a specific scenario.
# Expected output: successful query or streaming rows for dataset `genes`.

"""Simple query example."""

import os

from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url=os.getenv("ATLAS_BASE_URL", "http://127.0.0.1:8080")))
page = client.query(QueryRequest(dataset="genes", limit=5))
print(page.items)
