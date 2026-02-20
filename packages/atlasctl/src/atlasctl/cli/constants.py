"""CLI constants and registration tables."""

from __future__ import annotations

DOMAINS = ("registry", "layout")

CONFIGURE_HOOKS: tuple[tuple[str, str], ...] = (
    ("atlasctl.env.command", "configure_env_parser"),
    ("atlasctl.paths.command", "configure_paths_parser"),
    ("atlasctl.configs.command", "configure_configs_parser"),
    ("atlasctl.contracts.command", "configure_contracts_parser"),
    ("atlasctl.docker.command", "configure_docker_parser"),
    ("atlasctl.ci.command", "configure_ci_parser"),
    ("atlasctl.checks.command", "configure_check_parser"),
    ("atlasctl.gen.command", "configure_gen_parser"),
    ("atlasctl.policies.command", "configure_policies_parser"),
    ("atlasctl.docs.command", "configure_docs_parser"),
    ("atlasctl.make.command", "configure_make_parser"),
    ("atlasctl.migrate.command", "configure_migrate_parser"),
    ("atlasctl.ops.command", "configure_ops_parser"),
    ("atlasctl.inventory.command", "configure_inventory_parser"),
    ("atlasctl.lint.command", "configure_lint_parser"),
    ("atlasctl.reporting.command", "configure_report_parser"),
    ("atlasctl.compat.command", "configure_compat_parser"),
    ("atlasctl.legacy.command", "configure_legacy_parser"),
    ("atlasctl.orchestrate.command", "configure_orchestrate_parsers"),
    ("atlasctl.gates.command", "configure_gates_parser"),
    ("atlasctl.python_tools.command", "configure_python_parser"),
)
