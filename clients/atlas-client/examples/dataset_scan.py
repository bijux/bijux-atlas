"""Dataset scan example."""

from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
page = client.query(
    QueryRequest(dataset="genes", fields=["gene_id", "name"], limit=100)
)
for row in page.items:
    print(row)
