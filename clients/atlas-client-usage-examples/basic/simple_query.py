from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
print(client.query(QueryRequest(dataset="genes", limit=5)).items)
