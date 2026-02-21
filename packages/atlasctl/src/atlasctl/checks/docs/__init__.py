from __future__ import annotations

from ..repo.native import check_docs_no_ops_generated_run_paths, check_docs_scripts_references
from ..base import CheckDef
from .integrity import (
    check_command_group_docs_pages,
    check_docs_registry_command_drift,
    check_docs_index_complete,
    check_docs_links_exist,
    check_migration_docs_not_stale,
    check_no_package_root_markdown_except_readme,
    check_stable_command_examples_in_group_docs,
)

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("docs.no_scripts_path_refs", "docs", "forbid scripts/ path references in docs", 800, check_docs_scripts_references, fix_hint="Update docs references to atlasctl commands."),
    CheckDef("docs.no_ops_generated_run_path_refs", "docs", "forbid runtime ops generated path references in docs", 800, check_docs_no_ops_generated_run_paths, fix_hint="Use stable docs or generated index links."),
    CheckDef("docs.package_root_markdown", "docs", "forbid markdown files at package root except README", 400, check_no_package_root_markdown_except_readme, fix_hint="Move package root markdown docs into packages/atlasctl/docs/."),
    CheckDef("docs.links_exist", "docs", "validate docs markdown link targets exist", 1200, check_docs_links_exist, fix_hint="Fix broken relative links in packages/atlasctl/docs."),
    CheckDef("docs.index_complete", "docs", "validate docs/index.md covers all docs files", 600, check_docs_index_complete, fix_hint="Add missing docs entries to packages/atlasctl/docs/index.md."),
    CheckDef("docs.command_group_pages", "docs", "require command-group docs pages with examples section", 600, check_command_group_docs_pages, fix_hint="Add docs/commands/groups/<group>.md pages with ## Examples."),
    CheckDef("docs.registry_command_drift", "docs", "forbid docs references to unknown atlasctl commands", 600, check_docs_registry_command_drift, fix_hint="Fix stale command references to match cli surface registry."),
    CheckDef("docs.stable_command_examples", "docs", "require stable commands to have examples in group docs", 600, check_stable_command_examples_in_group_docs, fix_hint="Add command entries and atlasctl examples to docs/commands/groups/*.md."),
    CheckDef("docs.migration_not_stale", "docs", "forbid stale migration wording once removals are active", 600, check_migration_docs_not_stale, fix_hint="Remove stale parallel-legacy wording from docs/migration pages."),
)
