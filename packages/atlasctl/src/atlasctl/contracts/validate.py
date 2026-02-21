"""Compatibility shim for `atlasctl.contracts.validate`."""

from .schema.validate import validate, validate_file

__all__ = ["validate", "validate_file"]
