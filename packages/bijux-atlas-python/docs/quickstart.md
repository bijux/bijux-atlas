# Python Client Quickstart

## Install

```bash
python -m pip install -e packages/bijux-atlas-python
```

## Query

```python
from bijux_atlas import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
page = client.query(QueryRequest(dataset="genes", limit=5))
print(page.items)
```
