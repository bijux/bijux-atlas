# Purpose: demonstrate Atlas Python client usage for a specific scenario.
# Expected output: successful query or streaming rows for dataset `genes`.

"""Streaming results example."""

import os

from bijux_atlas import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url=os.getenv("BIJUX_ATLAS_URL", "http://127.0.0.1:8080")))
request = QueryRequest(dataset="genes", fields=["gene_id"], limit=50)
for row in client.stream_query(request):
    print(row)
