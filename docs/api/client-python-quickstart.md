---
title: Python Client Quickstart
audience: user
type: guide
stability: experimental
owner: api-contracts
last_reviewed: 2026-03-05
tags:
  - api
  - python
  - sdk
---

# Python Client Quickstart

```bash
python -m pip install -e packages/bijux-atlas-python
```

```python
from atlas_client import AtlasClient, ClientConfig, QueryRequest

client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
page = client.query(QueryRequest(dataset="genes", limit=5))
print(page.items)
```
