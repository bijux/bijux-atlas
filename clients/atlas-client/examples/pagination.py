"""Pagination example."""

from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
request = QueryRequest(dataset="genes", fields=["gene_id"], limit=10)

page = client.query(request)
print("first", len(page.items), page.next_token)

if page.next_token:
    second = client.query(
        QueryRequest(dataset="genes", fields=["gene_id"], limit=10, page_token=page.next_token)
    )
    print("second", len(second.items), second.next_token)
