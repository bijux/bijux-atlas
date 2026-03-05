"""Ecosystem integration example with pandas."""

from atlas_client import AtlasClient, ClientConfig, QueryRequest


def load_dataframe():
    import pandas as pd

    client = AtlasClient(ClientConfig(base_url="http://localhost:8080"))
    page = client.query(QueryRequest(dataset="genes", fields=["gene_id", "symbol"], limit=100))
    return pd.DataFrame(page.items)


if __name__ == "__main__":
    df = load_dataframe()
    print(df.head())
