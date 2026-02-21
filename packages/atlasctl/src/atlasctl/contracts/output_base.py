"""Compatibility shim for `atlasctl.contracts.output_base`."""

from .output.base import OUTPUT_BASE_V1, OUTPUT_BASE_V2, build_output_base

__all__ = ["OUTPUT_BASE_V1", "OUTPUT_BASE_V2", "build_output_base"]
