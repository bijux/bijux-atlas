"""Compatibility shim for `atlasctl.contracts.output`."""

from .output.json_output import validate_json_output

__all__ = ["validate_json_output"]
