from __future__ import annotations

from ...check.native import check_docs_scripts_references
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("docs/no-scripts-path-refs", "docs", 800, check_docs_scripts_references),
)
