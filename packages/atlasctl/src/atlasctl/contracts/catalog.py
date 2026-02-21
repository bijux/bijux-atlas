"""Compatibility shim for `atlasctl.contracts.catalog`."""

from .schema.catalog import (
    CatalogEntry,
    catalog_path,
    lint_catalog,
    list_catalog_entries,
    load_catalog,
    schema_path_for,
    write_catalog_deterministic,
    write_schema_readme_deterministic,
    check_schema_readme_sync,
    check_schema_change_release_policy,
)

__all__ = [
    "CatalogEntry",
    "catalog_path",
    "lint_catalog",
    "list_catalog_entries",
    "load_catalog",
    "schema_path_for",
    "write_catalog_deterministic",
    "write_schema_readme_deterministic",
    "check_schema_readme_sync",
    "check_schema_change_release_policy",
]
