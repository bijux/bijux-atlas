"""Ecosystem integration example with Airflow style callable."""

from atlas_client import AtlasClient, ClientConfig, QueryRequest


def fetch_genes_for_workflow(**_: object) -> list[dict[str, object]]:
    client = AtlasClient(ClientConfig(base_url="http://atlas-runtime.default.svc:8080"))
    page = client.query(QueryRequest(dataset="genes", limit=50))
    return page.items
