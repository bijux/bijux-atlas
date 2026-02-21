"""CLI constants and registration tables."""

from __future__ import annotations
from datetime import date

DOMAINS = ("registry", "layout")
NO_NETWORK_FLAG_EXPIRY = date(2026, 12, 31)

CONFIGURE_HOOKS: tuple[tuple[str, str], ...] = (
    ("atlasctl.commands.dev.env.command", "configure_env_parser"),
    ("atlasctl.commands.dev.paths.command", "configure_paths_parser"),
    ("atlasctl.commands.configs.command", "configure_configs_parser"),
    ("atlasctl.contracts.command", "configure_contracts_parser"),
    ("atlasctl.commands.ops.docker.command", "configure_docker_parser"),
    ("atlasctl.commands.dev.ci.command", "configure_ci_parser"),
    ("atlasctl.checks.command", "configure_check_parser"),
    ("atlasctl.checks.command", "configure_checks_parser"),
    ("atlasctl.commands.dev.deps.command", "configure_deps_parser"),
    ("atlasctl.commands.dev.gen.command", "configure_gen_parser"),
    ("atlasctl.commands.policies.runtime.command", "configure_policies_parser"),
    ("atlasctl.commands.dev.repo.command", "configure_repo_parser"),
    ("atlasctl.commands.docs.runtime", "configure_docs_parser"),
    ("atlasctl.commands.dev.make.command", "configure_make_parser"),
    ("atlasctl.commands.ops.runtime", "configure_ops_parser"),
    ("atlasctl.commands.dev.inventory.command", "configure_inventory_parser"),
    ("atlasctl.commands.policies.lint.command", "configure_lint_parser"),
    ("atlasctl.commands.internal.test_tools.command", "configure_test_parser"),
    ("atlasctl.suite.command", "configure_suite_parser"),
    ("atlasctl.reporting.command", "configure_report_parser"),
    ("atlasctl.reporting.command", "configure_reporting_parser"),
    ("atlasctl.commands.ops.orchestrate.command", "configure_orchestrate_parsers"),
    ("atlasctl.commands.policies.gates.command", "configure_gates_parser"),
    ("atlasctl.commands.dev.python_tools.command", "configure_python_parser"),
    ("atlasctl.commands.dev.install", "configure_install_parser"),
    ("atlasctl.commands.dev.release", "configure_release_parser"),
    ("atlasctl.commands.dev.command", "configure_dev_parser"),
    ("atlasctl.commands.internal.command", "configure_internal_parser"),
)
