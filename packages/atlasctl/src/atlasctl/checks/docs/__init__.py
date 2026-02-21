from __future__ import annotations

from ..repo.native import check_docs_no_ops_generated_run_paths, check_docs_scripts_references
from ..base import CheckDef
from .integrity import (
    check_docs_index_complete,
    check_docs_links_exist,
    check_no_package_root_markdown_except_readme,
)

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("docs.no_scripts_path_refs", "docs", "forbid scripts/ path references in docs", 800, check_docs_scripts_references, fix_hint="Update docs references to atlasctl commands."),
    CheckDef("docs.no_ops_generated_run_path_refs", "docs", "forbid runtime ops generated path references in docs", 800, check_docs_no_ops_generated_run_paths, fix_hint="Use stable docs or generated index links."),
    CheckDef("docs.package_root_markdown", "docs", "forbid markdown files at package root except README", 400, check_no_package_root_markdown_except_readme, fix_hint="Move package root markdown docs into packages/atlasctl/docs/."),
    CheckDef("docs.links_exist", "docs", "validate docs markdown link targets exist", 1200, check_docs_links_exist, fix_hint="Fix broken relative links in packages/atlasctl/docs."),
    CheckDef("docs.index_complete", "docs", "validate docs/index.md covers all docs files", 600, check_docs_index_complete, fix_hint="Add missing docs entries to packages/atlasctl/docs/index.md."),
)
