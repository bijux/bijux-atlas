# Purpose: demonstrate Atlas Python client usage for a specific scenario.
# Expected output: successful query or streaming rows for dataset `genes`.

"""Pagination example."""

import os

from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url=os.getenv("ATLAS_BASE_URL", "http://127.0.0.1:8080")))
request = QueryRequest(dataset="genes", fields=["gene_id"], limit=10)

page = client.query(request)
print("first", len(page.items), page.next_token)

if page.next_token:
    second = client.query(
        QueryRequest(dataset="genes", fields=["gene_id"], limit=10, page_token=page.next_token)
    )
    print("second", len(second.items), second.next_token)
