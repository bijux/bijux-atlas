from __future__ import annotations

from pathlib import Path
from typing import Any

from .readers import load_json_catalog


def load_suites_catalog(repo_root: Path) -> dict[str, Any]:
    return load_json_catalog(repo_root, "packages/atlasctl/src/atlasctl/registry/suites_catalog.json")


def load_checks_catalog(repo_root: Path) -> dict[str, Any]:
    return load_json_catalog(repo_root, "packages/atlasctl/src/atlasctl/registry/checks_catalog.json")


def load_ops_tasks_catalog(repo_root: Path) -> dict[str, Any]:
    return load_json_catalog(repo_root, "packages/atlasctl/src/atlasctl/registry/ops_tasks_catalog.json")
