# Atlas Python client examples

## Basic

- `simple_query.py`: query a small page from the `genes` dataset.
- `filtered_query.py`: apply filter predicates on `genes` rows.
- `dataset_scan.py`: scan selected fields from `genes`.
- `pagination.py`: request multiple pages with `page_token`.
- `streaming_results.py`: stream rows from `genes`.

## Integrations

- `integrations/pandas_pipeline.py`: load query results into a pandas dataframe.
- `integrations/airflow_operator.py`: fetch `genes` rows from an Airflow-style callable.

## Usage walkthrough

- `usage/basic/simple_query.py`: minimal end-to-end usage.
- `usage/advanced/streaming_pipeline.py`: larger streaming usage sample.
