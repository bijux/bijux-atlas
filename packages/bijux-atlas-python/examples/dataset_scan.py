# Purpose: demonstrate Atlas Python client usage for a specific scenario.
# Expected output: successful query or streaming rows for dataset `genes`.

"""Dataset scan example."""

import os

from bijux_atlas import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url=os.getenv("BIJUX_ATLAS_URL", "http://127.0.0.1:8080")))
page = client.query(
    QueryRequest(dataset="genes", fields=["gene_id", "name"], limit=100)
)
for row in page.items:
    print(row)
