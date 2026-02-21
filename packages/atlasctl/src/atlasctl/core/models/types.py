"""Compatibility shim for `atlasctl.core.models.types`."""

from ..model.types import EvidenceConfig, PathConfig, build_evidence_config, build_path_config

__all__ = ["EvidenceConfig", "PathConfig", "build_evidence_config", "build_path_config"]
