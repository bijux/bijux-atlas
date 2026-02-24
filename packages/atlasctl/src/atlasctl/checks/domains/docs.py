from __future__ import annotations

from ..model import CheckDef
from ..tools.docs_integrity import (
    check_command_group_docs_pages,
    check_docs_check_id_drift,
    check_docs_ci_lane_mapping,
    check_docs_index_complete,
    check_docs_links_exist,
    check_docs_lint_style,
    check_docs_nav_references_exist,
    check_docs_new_command_workflow,
    check_docs_no_legacy_cli_invocation,
    check_docs_no_orphans,
    check_docs_no_placeholder_release_docs,
    check_docs_ownership_metadata,
    check_docs_registry_command_drift,
    check_docs_registry_indexes,
    check_migration_docs_not_stale,
    check_no_package_root_markdown_except_readme,
    check_stable_command_examples_in_group_docs,
)
from ..tools.repo_domain.native import check_docs_no_ops_generated_run_paths, check_docs_scripts_references

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("docs.no_scripts_path_refs", "docs", "forbid scripts/ path references in docs", 800, check_docs_scripts_references, fix_hint="Update docs references to atlasctl commands."),
    CheckDef("docs.no_ops_generated_run_path_refs", "docs", "forbid runtime ops generated path references in docs", 800, check_docs_no_ops_generated_run_paths, fix_hint="Use stable docs or generated index links."),
    CheckDef("docs.package_root_markdown", "docs", "forbid markdown files at package root except README", 400, check_no_package_root_markdown_except_readme, fix_hint="Move package root markdown docs into packages/atlasctl/docs/."),
    CheckDef("docs.links_exist", "docs", "validate docs markdown link targets exist", 1200, check_docs_links_exist, fix_hint="Fix broken relative links in packages/atlasctl/docs."),
    CheckDef("docs.index_complete", "docs", "validate docs/index.md covers all docs files", 600, check_docs_index_complete, fix_hint="Add missing docs entries to packages/atlasctl/docs/index.md."),
    CheckDef("docs.command_group_pages", "docs", "require command-group docs pages with examples section", 600, check_command_group_docs_pages, fix_hint="Add docs/commands/groups/<group>.md pages with ## Examples."),
    CheckDef("docs.registry_command_drift", "docs", "forbid docs references to unknown atlasctl commands", 600, check_docs_registry_command_drift, fix_hint="Fix stale command references to match cli surface registry."),
    CheckDef("docs.check_id_drift", "docs", "forbid docs references to unknown check ids", 600, check_docs_check_id_drift, fix_hint="Update check:id references to registered check ids."),
    CheckDef("docs.stable_command_examples", "docs", "require stable commands to have examples in group docs", 600, check_stable_command_examples_in_group_docs, fix_hint="Add command entries and atlasctl examples to docs/commands/groups/*.md."),
    CheckDef("docs.migration_not_stale", "docs", "forbid stale migration wording once removals are active", 600, check_migration_docs_not_stale, fix_hint="Remove stale parallel-legacy wording from docs/migration pages."),
    CheckDef("docs.nav_references_exist", "docs", "validate mkdocs nav references existing docs pages", 600, check_docs_nav_references_exist, fix_hint="Fix missing docs files referenced in mkdocs.yml nav."),
    CheckDef("docs.no_orphans", "docs", "forbid orphan docs files outside allowed generated/meta paths", 600, check_docs_no_orphans, fix_hint="Link docs pages from docs/index.md or other docs pages."),
    CheckDef("docs.no_legacy_cli_invocation", "docs", "forbid legacy atlasctl invocation patterns in docs", 600, check_docs_no_legacy_cli_invocation, fix_hint="Use `./bin/atlasctl ...` consistently in all docs examples."),
    CheckDef("docs.registry_indexes", "docs", "require registry-generated command/check/suite index pages to be in sync", 600, check_docs_registry_indexes, fix_hint="Run `atlasctl docs generate-registry-indexes --report text`."),
    CheckDef("docs.ci_lane_mapping", "docs", "require CI workflow lane mapping doc and referenced jobs/workflows", 600, check_docs_ci_lane_mapping, fix_hint="Update packages/atlasctl/docs/control-plane/ci-lane-mapping.md and CI workflow job names together."),
    CheckDef("docs.new_command_workflow", "docs", "require docs/tests updates when command registry changes", 600, check_docs_new_command_workflow, fix_hint="Update docs index, pyproject, and tests for command-surface changes."),
    CheckDef("docs.ownership_metadata", "docs", "require docs ownership metadata for major docs areas", 600, check_docs_ownership_metadata, fix_hint="Update packages/atlasctl/docs/_meta/owners.json with major area owners."),
    CheckDef("docs.lint_style", "docs", "enforce docs style lint policy", 600, check_docs_lint_style, fix_hint="Fix long lines, invalid headings, and TODO/TBD placeholders."),
    CheckDef("docs.no_placeholder_release_docs", "docs", "forbid placeholder docs paths in release docs tree", 600, check_docs_no_placeholder_release_docs, fix_hint="Remove placeholder docs files or move drafts under docs/_drafts/."),
)


def register() -> tuple[CheckDef, ...]:
    return CHECKS


__all__ = ["CHECKS", "register"]
