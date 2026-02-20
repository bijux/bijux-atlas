from __future__ import annotations

from ..repo.legacy_native import check_docs_no_ops_generated_run_paths, check_docs_scripts_references
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("docs.no_scripts_path_refs", "docs", "forbid scripts/ path references in docs", 800, check_docs_scripts_references, fix_hint="Update docs references to atlasctl commands."),
    CheckDef("docs.no_ops_generated_run_path_refs", "docs", "forbid runtime ops generated path references in docs", 800, check_docs_no_ops_generated_run_paths, fix_hint="Use stable docs or generated index links."),
)
