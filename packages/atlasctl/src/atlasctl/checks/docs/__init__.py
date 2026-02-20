from __future__ import annotations

from ..repo.legacy_native import check_docs_no_ops_generated_run_paths, check_docs_scripts_references
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("docs/no-scripts-path-refs", "docs", 800, check_docs_scripts_references),
    CheckDef("docs/no-ops-generated-run-path-refs", "docs", 800, check_docs_no_ops_generated_run_paths),
)
