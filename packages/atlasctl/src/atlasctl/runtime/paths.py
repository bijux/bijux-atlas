"""Canonical runtime paths import surface.

Delegates to `atlasctl.core.runtime.paths` during runtime path consolidation migration.
"""
from __future__ import annotations

from atlasctl.core.runtime.paths import evidence_root_path
from atlasctl.core.runtime.paths import run_dir_root_path
from atlasctl.core.runtime.paths import scripts_artifact_root_path
from atlasctl.core.runtime.paths import write_text_file

__all__ = ("evidence_root_path", "scripts_artifact_root_path", "run_dir_root_path", "write_text_file")
