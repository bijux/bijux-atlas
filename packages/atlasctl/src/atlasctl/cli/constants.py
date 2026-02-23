"""CLI constants and registration tables."""

from __future__ import annotations
from datetime import date

DOMAINS = ("layout",)
NO_NETWORK_FLAG_EXPIRY = date(2026, 12, 31)

CONFIGURE_HOOKS: tuple[tuple[str, str], ...] = (
    ("atlasctl.commands.dev.env.command", "configure_env_parser"),
    ("atlasctl.commands.dev.paths.command", "configure_paths_parser"),
    ("atlasctl.commands.configs.command", "configure_configs_parser"),
    ("atlasctl.contracts.command", "configure_contracts_parser"),
    ("atlasctl.commands.docker.command", "configure_docker_parser"),
    ("atlasctl.commands.ci.command", "configure_ci_parser"),
    ("atlasctl.commands.check.command", "configure_check_parser"),
    ("atlasctl.commands.check.command", "configure_checks_parser"),
    ("atlasctl.commands.registry.command", "configure_registry_parser"),
    ("atlasctl.commands.dev.deps.command", "configure_deps_parser"),
    ("atlasctl.commands.dev.gen.command", "configure_gen_parser"),
    ("atlasctl.commands.policies.command", "configure_policies_parser"),
    ("atlasctl.commands.policies.command", "configure_policy_parser"),
    ("atlasctl.commands.dev.repo.command", "configure_repo_parser"),
    ("atlasctl.commands.docs.command", "configure_docs_parser"),
    ("atlasctl.commands.dev.make.command", "configure_make_parser"),
    ("atlasctl.commands.ops.command", "configure_ops_parser"),
    ("atlasctl.commands.product.command", "configure_product_parser"),
    ("atlasctl.commands.packages.command", "configure_packages_parser"),
    ("atlasctl.commands.dev.inventory.command", "configure_inventory_parser"),
    ("atlasctl.commands.policies.lint.command", "configure_lint_parser"),
    ("atlasctl.commands.internal.test_tools.command", "configure_test_parser"),
    ("atlasctl.suite.command", "configure_suite_parser"),
    ("atlasctl.reporting.command", "configure_report_parser"),
    ("atlasctl.reporting.command", "configure_reporting_parser"),
    ("atlasctl.commands.ops.orchestrate.command", "configure_orchestrate_parsers"),
    ("atlasctl.commands.policies.gates.command", "configure_gates_parser"),
    ("atlasctl.commands.policies.gates.command", "configure_gate_parser"),
    ("atlasctl.commands.dev.python_tools.command", "configure_python_parser"),
    ("atlasctl.commands.dev.install", "configure_install_parser"),
    ("atlasctl.commands.dev.release", "configure_release_parser"),
    ("atlasctl.commands.dev.command", "configure_dev_parser"),
    ("atlasctl.commands.owners", "configure_owners_parser"),
    ("atlasctl.commands.dev.approvals", "configure_approvals_parser"),
    ("atlasctl.commands.dev.migrate", "configure_migrate_parser"),
    ("atlasctl.commands.internal.contributing", "configure_contributing_parser"),
    ("atlasctl.commands.internal.command", "configure_internal_parser"),
)
