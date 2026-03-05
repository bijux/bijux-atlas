"""Simple query example."""

from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
page = client.query(QueryRequest(dataset="genes", limit=5))
print(page.items)
