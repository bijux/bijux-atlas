"""Compatibility shim for `atlasctl.contracts.generators`."""

from .schema.generators import (
    generate_chart_schema,
    generate_contract_artifacts,
    generate_openapi,
    generate_schema_catalog,
    generate_schema_samples,
)

__all__ = [
    "generate_chart_schema",
    "generate_contract_artifacts",
    "generate_openapi",
    "generate_schema_catalog",
    "generate_schema_samples",
]
