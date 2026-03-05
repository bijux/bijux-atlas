"""Filtered query example."""

from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
page = client.query(
    QueryRequest(
        dataset="genes",
        filters={"chromosome": "1", "biotype": "protein_coding"},
        fields=["gene_id", "symbol"],
        limit=25,
    )
)
print(page.items)
