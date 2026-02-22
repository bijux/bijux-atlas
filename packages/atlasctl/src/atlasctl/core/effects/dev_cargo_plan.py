from __future__ import annotations

from pathlib import Path


def build_dev_cargo_plan(
    *,
    action: str,
    all_tests: bool,
    contracts_tests: bool,
    include_repo_checks: bool,
    include_slow_checks: bool,
    quiet: bool,
    verbose: bool,
    nextest_toml: str,
    rustfmt_config: str,
    deny_config: str,
    profile: str,
    iso_root: Path,
    repo_check_cmd_builder,
) -> list[list[str]]:
    repo_cmd = repo_check_cmd_builder(quiet=quiet, verbose=verbose, include_slow=include_slow_checks)
    if action == "fmt":
        cmds: list[list[str]] = [["cargo", "fmt", "--all", "--", "--check", "--config-path", rustfmt_config]]
        if include_repo_checks:
            cmds.append(repo_cmd)
        return cmds
    if action == "lint":
        cmds = [["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]]
        if include_repo_checks:
            cmds.append(repo_cmd)
        return cmds
    if action == "check":
        cmds = [["cargo", "check", "--workspace", "--all-targets"]]
        if include_repo_checks:
            cmds.append(repo_cmd)
        return cmds
    if action == "test":
        if contracts_tests:
            cmds = [["cargo", "test", "-p", "bijux-atlas-server", "--test", "observability_contract"]]
        else:
            cmd = ["cargo", "nextest", "run", "--workspace", "--all-targets", "--profile", profile, "--config-file", nextest_toml]
            if all_tests:
                cmd.extend(["--run-ignored", "all"])
            cmds = [["cargo", "nextest", "--version"], cmd]
        if include_repo_checks:
            cmds.append(repo_cmd)
        return cmds
    if action == "coverage":
        output = iso_root / "coverage" / "lcov.info"
        cmds = [
            ["cargo", "llvm-cov", "--version"],
            [
                "cargo",
                "llvm-cov",
                "nextest",
                "--workspace",
                "--all-targets",
                "--profile",
                profile,
                "--config-file",
                nextest_toml,
                "--run-ignored",
                "all",
                "--lcov",
                "--output-path",
                str(output),
            ],
            ["cargo", "llvm-cov", "report", "--summary-only"],
        ]
        if include_repo_checks:
            cmds.append(repo_cmd)
        return cmds
    if action == "audit":
        cmds = [["cargo", "+stable", "deny", "--version"], ["cargo", "+stable", "deny", "check", "--config", deny_config]]
        if include_repo_checks:
            cmds.append(repo_cmd)
        return cmds
    return []

