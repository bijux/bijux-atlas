from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
for row in client.stream_query(QueryRequest(dataset="genes", limit=100)):
    print(row)
