import os

from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url=os.getenv("ATLAS_BASE_URL", "http://127.0.0.1:8080")))
print(client.query(QueryRequest(dataset="genes", limit=5)).items)
