from __future__ import annotations

from ..core.base import CheckDef
from ..repo.contracts.pyproject_contracts import (
    check_pyproject_no_duplicate_tool_config,
    check_pyproject_required_blocks,
)

CHECKS: tuple[CheckDef, ...] = (
    CheckDef(
        "configs.pyproject_required_blocks",
        "configs",
        "validate required pyproject blocks",
        800,
        check_pyproject_required_blocks,
        fix_hint="Restore missing [project]/[build-system]/tool config blocks in packages/atlasctl/pyproject.toml.",
    ),
    CheckDef(
        "configs.pyproject_no_duplicate_tool_config",
        "configs",
        "forbid duplicate pyproject tool blocks",
        800,
        check_pyproject_no_duplicate_tool_config,
        fix_hint="Deduplicate tool.atlasctl/tool.pytest sections in packages/atlasctl/pyproject.toml.",
    ),
)
