# Client SDK Examples (Pinned OpenAPI v1)

OpenAPI source of truth (pinned):

- `configs/openapi/v1/openapi.generated.json`

## curl

```sh
BASE="http://127.0.0.1:8080"

curl -fsS "$BASE/v1/datasets"
curl -fsS "$BASE/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=5"
curl -fsS "$BASE/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-100"
```

## Python (requests)

```python
import requests

BASE = "http://127.0.0.1:8080"
params = {
    "release": "110",
    "species": "homo_sapiens",
    "assembly": "GRCh38",
    "limit": 5,
}

resp = requests.get(f"{BASE}/v1/genes", params=params, timeout=10)
resp.raise_for_status()
print(resp.headers.get("X-Atlas-Dataset-Hash"))
print(resp.headers.get("X-Atlas-Release"))
print(resp.json())
```

## Generate Typed Client from Pinned Spec

Example with `openapi-python-client`:

```sh
openapi-python-client generate --path configs/openapi/v1/openapi.generated.json --meta none
```

Keep generated clients tied to the checked-in v1 snapshot to avoid drift.
