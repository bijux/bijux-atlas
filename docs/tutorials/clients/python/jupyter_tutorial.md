# Atlas Python Jupyter Tutorial

## Setup

```bash
python -m pip install -e crates/bijux-atlas-python
python -m pip install jupyter
jupyter notebook crates/bijux-atlas-python/notebooks/simple_query.ipynb
```

## Walkthrough

1. Configure `AtlasClient` with your runtime base URL.
2. Run a simple query against `genes` dataset.
3. Add filters and field selection.
4. Iterate over paginated results.

## Verification

Use the notebook with a local runtime and verify returned payload shape against `POST /v1/query` contract.
