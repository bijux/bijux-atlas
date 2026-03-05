# Purpose: demonstrate Atlas Python client usage for a specific scenario.
# Expected output: successful query or streaming rows for dataset `genes`.

"""Filtered query example."""

import os

from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url=os.getenv("ATLAS_BASE_URL", "http://127.0.0.1:8080")))
page = client.query(
    QueryRequest(
        dataset="genes",
        filters={"chromosome": "1", "biotype": "protein_coding"},
        fields=["gene_id", "symbol"],
        limit=25,
    )
)
print(page.items)
