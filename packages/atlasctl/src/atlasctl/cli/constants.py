"""CLI constants and registration tables."""

from __future__ import annotations
from datetime import date

DOMAINS = ("registry", "layout")
NO_NETWORK_FLAG_EXPIRY = date(2026, 12, 31)

CONFIGURE_HOOKS: tuple[tuple[str, str], ...] = (
    ("atlasctl.env.command", "configure_env_parser"),
    ("atlasctl.paths.command", "configure_paths_parser"),
    ("atlasctl.configs.command", "configure_configs_parser"),
    ("atlasctl.contracts.command", "configure_contracts_parser"),
    ("atlasctl.docker.command", "configure_docker_parser"),
    ("atlasctl.ci.command", "configure_ci_parser"),
    ("atlasctl.checks.command", "configure_check_parser"),
    ("atlasctl.deps.command", "configure_deps_parser"),
    ("atlasctl.gen.command", "configure_gen_parser"),
    ("atlasctl.policies.command", "configure_policies_parser"),
    ("atlasctl.repo.command", "configure_repo_parser"),
    ("atlasctl.commands.docs.runtime", "configure_docs_parser"),
    ("atlasctl.make.command", "configure_make_parser"),
    ("atlasctl.migrate.command", "configure_migrate_parser"),
    ("atlasctl.commands.ops.runtime", "configure_ops_parser"),
    ("atlasctl.inventory.command", "configure_inventory_parser"),
    ("atlasctl.lint.command", "configure_lint_parser"),
    ("atlasctl.test_tools.command", "configure_test_parser"),
    ("atlasctl.suite.command", "configure_suite_parser"),
    ("atlasctl.reporting.command", "configure_report_parser"),
    ("atlasctl.commands.compat", "configure_compat_parser"),
    ("atlasctl.commands.legacy_inventory", "configure_legacy_parser"),
    ("atlasctl.orchestrate.command", "configure_orchestrate_parsers"),
    ("atlasctl.gates.command", "configure_gates_parser"),
    ("atlasctl.python_tools.command", "configure_python_parser"),
    ("atlasctl.dev.command", "configure_dev_parser"),
    ("atlasctl.internal.command", "configure_internal_parser"),
)
