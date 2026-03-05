# Purpose: demonstrate Atlas Python client usage for a specific scenario.
# Expected output: successful query or streaming rows for dataset `genes`.

"""Ecosystem integration example with pandas."""

import os

from atlas_client import AtlasClient, ClientConfig, QueryRequest


def load_dataframe():
    import pandas as pd

    client = AtlasClient(ClientConfig(base_url=os.getenv("ATLAS_BASE_URL", "http://127.0.0.1:8080")))
    page = client.query(QueryRequest(dataset="genes", fields=["gene_id", "symbol"], limit=100))
    return pd.DataFrame(page.items)


if __name__ == "__main__":
    df = load_dataframe()
    print(df.head())
