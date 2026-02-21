from __future__ import annotations

from .legacy_native import (
    check_docs_no_ops_generated_run_paths,
    check_duplicate_script_names,
    check_forbidden_top_dirs,
    check_no_executable_python_outside_packages,
    check_no_direct_bash_invocations,
    check_no_direct_python_invocations,
    check_no_adhoc_python,
    check_no_ops_generated_placeholder,
    check_no_xtask_refs,
    check_ops_examples_immutable,
    check_ops_generated_tracked,
    check_tracked_timestamp_paths,
)
from ..base import CheckDef
from .legacy_guard import check_legacy_package_quarantine
from .module_size import check_module_size
from .cwd_usage import check_no_path_cwd_usage
from .command_contracts import (
    check_command_help_docs_drift,
    check_command_metadata_contract,
    check_no_duplicate_command_names,
)
from .scripts_dir import check_scripts_dir_absent
from .public_api import check_public_api_exports
from .type_coverage import check_type_coverage
from .dependencies import check_dependency_declarations
from .reachability import check_repo_check_modules_registered
from .pyproject_contracts import (
    check_console_script_entry,
    check_pyproject_no_duplicate_tool_config,
    check_pyproject_required_blocks,
)

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("repo.forbidden_top_dirs", "repo", "forbid top-level forbidden directories", 500, check_forbidden_top_dirs, fix_hint="Remove forbidden root directories."),
    CheckDef("repo.docs_no_ops_generated_refs", "repo", "disallow docs refs to ops generated runtime paths", 800, check_docs_no_ops_generated_run_paths, fix_hint="Replace generated runtime paths with canonical docs references."),
    CheckDef("repo.no_xtask_refs", "repo", "forbid xtask references", 1000, check_no_xtask_refs, fix_hint="Remove xtask references and use atlasctl workflows."),
    CheckDef("repo.no_exec_python_outside_packages", "repo", "forbid executable python outside package boundaries", 1500, check_no_executable_python_outside_packages, fix_hint="Move script into package module or remove executable bit."),
    CheckDef("repo.no_direct_python_invocations", "repo", "forbid direct python script calls in docs/makefiles", 1000, check_no_direct_python_invocations, fix_hint="Use atlasctl command entrypoints instead of python path/to/script.py."),
    CheckDef("repo.no_direct_bash_invocations", "repo", "forbid direct bash script calls in docs/makefiles", 1000, check_no_direct_bash_invocations, fix_hint="Use atlasctl commands instead of bash scripts/... invocations."),
    CheckDef("repo.no_adhoc_python", "repo", "forbid ad-hoc python files outside package boundaries", 1200, check_no_adhoc_python, fix_hint="Migrate ad-hoc python files into package modules."),
    CheckDef("repo.no_tracked_ops_generated", "repo", "ensure ops/_generated has no tracked files", 1000, check_ops_generated_tracked, fix_hint="Untrack generated files and add to ignore policy."),
    CheckDef("repo.no_ops_generated_placeholder", "repo", "forbid placeholder generated dirs", 400, check_no_ops_generated_placeholder, fix_hint="Remove placeholder generated files/directories."),
    CheckDef("repo.ops_examples_immutable", "repo", "enforce immutability of ops examples", 800, check_ops_examples_immutable, fix_hint="Restore example fixtures to committed canonical content."),
    CheckDef("repo.no_tracked_timestamp_paths", "repo", "forbid timestamp-like tracked paths", 1000, check_tracked_timestamp_paths, fix_hint="Remove timestamped tracked files/dirs."),
    CheckDef("repo.duplicate_script_names", "repo", "forbid duplicate script stem names", 1200, check_duplicate_script_names, fix_hint="Rename colliding script names."),
    CheckDef("repo.no_scripts_dir", "repo", "forbid legacy root scripts dir", 250, check_scripts_dir_absent, fix_hint="Migrate scripts into atlasctl package commands."),
    CheckDef("repo.legacy_quarantine", "repo", "quarantine legacy package growth", 250, check_legacy_package_quarantine, fix_hint="Do not add new modules under atlasctl/legacy."),
    CheckDef("repo.module_size", "repo", "enforce module size budget", 400, check_module_size, fix_hint="Split oversized modules into focused submodules."),
    CheckDef("repo.no_path_cwd_usage", "repo", "forbid Path.cwd usage outside core/repo_root.py", 400, check_no_path_cwd_usage, fix_hint="Use ctx.repo_root or core.repo_root helpers."),
    CheckDef("repo.command_metadata_contract", "repo", "ensure command metadata includes touches/tools", 400, check_command_metadata_contract, fix_hint="Add touches/tools metadata in cli registry."),
    CheckDef("repo.no_duplicate_command_names", "repo", "ensure command names are unique", 300, check_no_duplicate_command_names, fix_hint="Rename duplicate command/alias entries."),
    CheckDef("repo.command_help_docs_drift", "repo", "check command help/docs drift", 500, check_command_help_docs_drift, fix_hint="Regenerate docs/_generated/cli.md from current CLI surface."),
    CheckDef("repo.public_api_exports", "repo", "enforce docs/public-api.md coverage for __all__ exports", 300, check_public_api_exports, fix_hint="Document exported symbols in docs/public-api.md or remove them from __all__."),
    CheckDef("repo.type_coverage", "repo", "enforce minimum type coverage in core/contracts", 600, check_type_coverage, fix_hint="Add function annotations in core/contracts until the threshold is met."),
    CheckDef("repo.dependency_declarations", "repo", "ensure pyproject dependency declarations match imports", 600, check_dependency_declarations, fix_hint="Add missing dependencies or remove unused declarations."),
    CheckDef("repo.check_module_reachability", "repo", "ensure repo check modules are imported and reachable via registry", 300, check_repo_check_modules_registered, fix_hint="Import new repo check modules in checks/repo/__init__.py."),
    CheckDef("repo.pyproject_required_blocks", "repo", "ensure pyproject contains required project and tool config blocks", 300, check_pyproject_required_blocks, fix_hint="Add required [project]/[tool.*] blocks to packages/atlasctl/pyproject.toml."),
    CheckDef("repo.pyproject_no_duplicate_tool_config", "repo", "forbid duplicate tool config files beside pyproject", 300, check_pyproject_no_duplicate_tool_config, fix_hint="Remove duplicated tool config files and keep pyproject as SSOT."),
    CheckDef("repo.console_script_entry", "repo", "ensure atlasctl console script entry exists and points to callable target", 300, check_console_script_entry, fix_hint="Set [project.scripts] atlasctl = \"atlasctl.cli.main:main\" and ensure target is importable."),
)
