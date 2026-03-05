"""Streaming results example."""

from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
request = QueryRequest(dataset="genes", fields=["gene_id"], limit=50)
for row in client.stream_query(request):
    print(row)
