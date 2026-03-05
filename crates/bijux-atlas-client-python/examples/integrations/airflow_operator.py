# Purpose: demonstrate Atlas Python client usage for a specific scenario.
# Expected output: successful query or streaming rows for dataset `genes`.

"""Ecosystem integration example with Airflow style callable."""

import os

from atlas_client import AtlasClient, ClientConfig, QueryRequest


def fetch_genes_for_workflow(**_: object) -> list[dict[str, object]]:
    client = AtlasClient(
        ClientConfig(base_url=os.getenv("ATLAS_BASE_URL", "http://atlas-runtime.default.svc:8080"))
    )
    page = client.query(QueryRequest(dataset="genes", limit=50))
    return page.items
