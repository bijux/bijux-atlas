from __future__ import annotations

from pathlib import Path

from atlasctl.checks.tools.repo_domain.contracts.naming_hygiene import (
    check_command_module_cli_intent,
    check_contracts_namespace_purpose,
    check_layout_domain_alias_cleanup,
    check_no_wildcard_exports,
    check_public_api_doc_exists,
    check_single_registry_module,
    check_single_runner_module,
)


def test_naming_hygiene_checks_pass_repo() -> None:
    repo_root = Path(__file__).resolve().parents[4]
    for check in (
        check_single_registry_module,
        check_single_runner_module,
        check_command_module_cli_intent,
        check_no_wildcard_exports,
        check_contracts_namespace_purpose,
        check_layout_domain_alias_cleanup,
        check_public_api_doc_exists,
    ):
        code, errors = check(repo_root)
        assert code == 0, errors
