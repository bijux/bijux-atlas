"""Compatibility shim for `atlasctl.contracts.catalog`."""

from .schema.catalog import CatalogEntry, catalog_path, lint_catalog, list_catalog_entries, load_catalog, schema_path_for

__all__ = [
    "CatalogEntry",
    "catalog_path",
    "lint_catalog",
    "list_catalog_entries",
    "load_catalog",
    "schema_path_for",
]
