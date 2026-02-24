from __future__ import annotations

# BYPASS_TEST_OK: reads configs/policy fixtures for policy-check coverage.
import json
from pathlib import Path

from atlasctl.checks.tools.repo_domain.native import (
    check_cli_help_output_deterministic,
    check_committed_generated_hygiene,
    check_generated_dirs_policy,
    check_duplicate_script_names,
    check_json_output_deterministic_policy,
    check_layout_contract,
    check_make_command_allowlist,
    check_make_forbidden_paths,
    check_make_no_direct_python_script_invocations,
    check_make_scripts_references,
    check_no_large_binary_files,
    check_no_direct_env_reads_outside_runtime_env,
    check_no_models_shim_when_model_canonical,
    check_no_os_getcwd_outside_runtime_context,
    check_no_repo_root_path_literals,
    check_single_canonical_runtime_adapters,
    check_no_xtask_refs,
    check_ops_configs_deterministic_newlines,
    check_ops_generated_tracked,
    check_repo_no_python_caches_committed,
    check_script_ownership,
    check_tracked_timestamp_paths,
    check_tmp_paths_outside_artifacts,
)


def test_check_duplicate_script_names_detects_dash_underscore_conflict(tmp_path: Path) -> None:
    script_dir = tmp_path / "scripts"
    script_dir.mkdir()
    (script_dir / "a_b.py").write_text("print('x')\n", encoding="utf-8")
    (script_dir / "a-b.sh").write_text("#!/usr/bin/env sh\n", encoding="utf-8")
    code, errors = check_duplicate_script_names(tmp_path)
    assert code == 1
    assert errors


def test_check_script_ownership_passes_for_mapped_paths(tmp_path: Path) -> None:
    meta = tmp_path / "configs/meta"
    meta.mkdir(parents=True)
    (tmp_path / "scripts/bin").mkdir(parents=True)
    (tmp_path / "scripts/bin/tool.sh").write_text("#!/usr/bin/env sh\n", encoding="utf-8")
    ownership = {"paths": {"scripts/bin": "platform"}}
    (meta / "ownership.json").write_text(json.dumps(ownership), encoding="utf-8")
    code, errors = check_script_ownership(tmp_path)
    assert code == 0
    assert errors == []


def test_check_no_xtask_refs_flags_non_adr_mentions(tmp_path: Path) -> None:
    (tmp_path / "docs").mkdir()
    (tmp_path / "docs/page.md").write_text("use xtask command\n", encoding="utf-8")
    code, errors = check_no_xtask_refs(tmp_path)
    assert code == 1
    assert "docs/page.md" in errors


def test_check_make_forbidden_paths_blocks_tools_and_xtask(tmp_path: Path) -> None:
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "Makefile").write_text("all:\n\t@echo ok\n", encoding="utf-8")
    (tmp_path / "makefiles/test.mk").write_text("x:\n\t@python3 tools/foo.py\n", encoding="utf-8")
    code, errors = check_make_forbidden_paths(tmp_path)
    assert code == 1
    assert errors


def test_check_make_no_direct_python_script_invocations_flags_direct_calls(tmp_path: Path) -> None:
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "configs/layout").mkdir(parents=True)
    (tmp_path / "configs/layout/python-migration-exceptions.json").write_text('{"exceptions":[]}', encoding="utf-8")
    (tmp_path / "Makefile").write_text("all:\n\t@python3 tools/check.py\n", encoding="utf-8")
    code, errors = check_make_no_direct_python_script_invocations(tmp_path)
    assert code == 1
    assert errors


def test_check_ops_generated_tracked_flags_tracked_entries(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "atlasctl.checks.tools.repo_domain.native.modules.repo_checks_make_and_layout._git_ls_files",
        lambda _repo_root, _spec: ["ops/_generated/run-1/report.json"],
    )
    code, errors = check_ops_generated_tracked(tmp_path)
    assert code == 1
    assert "ops/_generated" in errors[0]


def test_check_tracked_timestamp_paths_flags_timestamp_segments(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "atlasctl.checks.tools.repo_domain.native.modules.repo_checks_make_and_layout._git_ls_files",
        lambda _repo_root, _spec: ["artifacts/evidence/2026-02-20/report.json", "docs/index.md"],
    )
    code, errors = check_tracked_timestamp_paths(tmp_path)
    assert code == 1
    assert "2026-02-20" in errors[0]


def test_check_committed_generated_hygiene_flags_logs_and_timestamps(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "atlasctl.checks.tools.repo_domain.native.modules.repo_checks_make_and_layout._git_ls_files",
        lambda _repo_root, _spec: [
            "docs/_generated/2026-02-20/index.md",
            "ops/_generated.example/run.log",
        ],
    )
    code, errors = check_committed_generated_hygiene(tmp_path)
    assert code == 1
    assert len(errors) == 2


def test_check_repo_no_python_caches_committed_flags_python_cache_artifacts(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "atlasctl.checks.tools.repo_domain.native.modules.repo_checks_make_and_layout._git_ls_files",
        lambda _repo_root, _spec: [
            "packages/atlasctl/tests/__pycache__/conftest.cpython-311.pyc",
            "packages/atlasctl/.pytest_cache/README.md",
            "packages/atlasctl/src/atlasctl/__init__.py",
        ],
    )
    code, errors = check_repo_no_python_caches_committed(tmp_path)
    assert code == 1
    assert len(errors) == 2


def test_check_repo_no_python_caches_committed_passes_when_none_found(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "atlasctl.checks.tools.repo_domain.native.modules.repo_checks_make_and_layout._git_ls_files",
        lambda _repo_root, _spec: ["packages/atlasctl/src/atlasctl/__init__.py"],
    )
    code, errors = check_repo_no_python_caches_committed(tmp_path)
    assert code == 0
    assert errors == []


def test_check_tmp_paths_outside_artifacts_flags_tracked_tmp(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "atlasctl.checks.tools.repo_domain.native.modules.repo_checks_make_and_layout._git_ls_files",
        lambda _repo_root, _spec: ["tmp/cache.txt", "artifacts/tmp/cache.txt", "configs/app.toml"],
    )
    code, errors = check_tmp_paths_outside_artifacts(tmp_path)
    assert code == 1
    assert errors == ["tracked tmp path outside artifacts/: tmp/cache.txt"]


def test_check_generated_dirs_policy_flags_noncanonical_generated_dir(tmp_path: Path) -> None:
    (tmp_path / "docs/_generated").mkdir(parents=True)
    (tmp_path / "packages/foo/_generated").mkdir(parents=True)
    code, errors = check_generated_dirs_policy(tmp_path)
    assert code == 1
    assert errors == ["disallowed generated dir: packages/foo/_generated"]


def test_check_ops_configs_deterministic_newlines_flags_crlf_and_trailing_ws(monkeypatch, tmp_path: Path) -> None:
    path = tmp_path / "ops/example.txt"
    path.parent.mkdir(parents=True)
    path.write_bytes(b"ok  \r\nnext\r\n")
    monkeypatch.setattr(
        "atlasctl.checks.tools.repo_domain.native.modules.repo_checks_make_and_layout._git_ls_files",
        lambda _repo_root, _spec: ["ops/example.txt"],
    )
    code, errors = check_ops_configs_deterministic_newlines(tmp_path)
    assert code == 1
    assert any("CRLF" in err for err in errors)
    assert any("trailing whitespace" in err for err in errors)


def test_check_no_large_binary_files_flags_large_binary(monkeypatch, tmp_path: Path) -> None:
    blob = tmp_path / "ops/blob.bin"
    blob.parent.mkdir(parents=True)
    blob.write_bytes(b"\0" * (5 * 1024 * 1024 + 10))
    monkeypatch.setattr(
        "atlasctl.checks.tools.repo_domain.native.modules.repo_checks_make_and_layout._git_ls_files",
        lambda _repo_root, _spec: ["ops/blob.bin"],
    )
    code, errors = check_no_large_binary_files(tmp_path)
    assert code == 1
    assert "blob.bin" in errors[0]


def test_check_no_models_shim_when_model_canonical_flags_core_models_package(tmp_path: Path) -> None:
    path = tmp_path / "packages/atlasctl/src/atlasctl/core/models"
    path.mkdir(parents=True)
    code, errors = check_no_models_shim_when_model_canonical(tmp_path)
    assert code == 1
    assert any("core/models" in e for e in errors)


def test_check_single_canonical_runtime_adapters_flags_duplicate_exec_module(tmp_path: Path) -> None:
    src = tmp_path / "packages/atlasctl/src/atlasctl"
    (src / "core").mkdir(parents=True)
    (src / "core/exec.py").write_text("x=1\n", encoding="utf-8")
    (src / "foo").mkdir()
    (src / "foo/exec.py").write_text("x=2\n", encoding="utf-8")
    code, errors = check_single_canonical_runtime_adapters(tmp_path)
    assert code == 1
    assert any("foo/exec.py" in e for e in errors)


def test_check_make_command_allowlist_passes_for_allowlisted_command(tmp_path: Path) -> None:
    (tmp_path / "configs/layout").mkdir(parents=True)
    (tmp_path / "configs/layout/make-command-allowlist.txt").write_text("echo\n", encoding="utf-8")
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "Makefile").write_text("all:\n\t@echo ok\n", encoding="utf-8")
    code, errors = check_make_command_allowlist(tmp_path)
    assert code == 0
    assert errors == []


def test_check_layout_contract_flags_unexpected_root_entry(tmp_path: Path) -> None:
    (tmp_path / "configs/repo").mkdir(parents=True)
    (tmp_path / ".github/workflows").mkdir(parents=True)
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "configs/repo/surfaces.json").write_text(
        json.dumps(
            {
                "allowed_root_dirs": [".github", "configs", "makefiles"],
                "allowed_root_files": ["README.md"],
            }
        ),
        encoding="utf-8",
    )
    (tmp_path / "configs/repo/root-files-allowlist.txt").write_text("README.md\n", encoding="utf-8")
    (tmp_path / "configs/repo/symlink-allowlist.json").write_text(
        json.dumps({"root": {}, "non_root": {}}),
        encoding="utf-8",
    )
    (tmp_path / ".github/workflows/ci.yml").write_text("- run: make check\n", encoding="utf-8")
    (tmp_path / "makefiles/root.mk").write_text("check:\n\t@echo ok\n", encoding="utf-8")
    (tmp_path / "README.md").write_text("x\n", encoding="utf-8")
    (tmp_path / "unexpected").mkdir()
    code, errors = check_layout_contract(tmp_path)
    assert code == 1
    assert any("unexpected root directory: unexpected" in e for e in errors)


def test_check_layout_contract_passes_minimal_repo(tmp_path: Path) -> None:
    (tmp_path / "configs/repo").mkdir(parents=True)
    (tmp_path / ".github/workflows").mkdir(parents=True)
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "configs/repo/surfaces.json").write_text(
        json.dumps(
            {
                "allowed_root_dirs": [".github", "configs", "makefiles"],
                "allowed_root_files": ["README.md"],
            }
        ),
        encoding="utf-8",
    )
    (tmp_path / "configs/repo/root-files-allowlist.txt").write_text("README.md\n", encoding="utf-8")
    (tmp_path / "configs/repo/symlink-allowlist.json").write_text(
        json.dumps({"root": {}, "non_root": {}}),
        encoding="utf-8",
    )
    (tmp_path / ".github/workflows/ci.yml").write_text("- run: make check\n", encoding="utf-8")
    (tmp_path / "makefiles/root.mk").write_text("check:\n\t@echo ok\n", encoding="utf-8")
    (tmp_path / "README.md").write_text("x\n", encoding="utf-8")
    code, errors = check_layout_contract(tmp_path)
    assert code == 0
    assert errors == []


def test_check_make_scripts_references_flags_unapproved_scripts_path(tmp_path: Path) -> None:
    (tmp_path / "configs/layout").mkdir(parents=True)
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "configs/layout/make-scripts-reference-exceptions.json").write_text(
        json.dumps({"exceptions": []}),
        encoding="utf-8",
    )
    (tmp_path / "Makefile").write_text("x:\n\t@python3 scripts/check/foo.py\n", encoding="utf-8")
    code, errors = check_make_scripts_references(tmp_path)
    assert code == 1
    assert any("scripts/" in e for e in errors)


def test_check_cli_help_output_deterministic_passes() -> None:
    code, errors = check_cli_help_output_deterministic(Path("."))
    assert code == 0
    assert errors == []


def test_check_json_output_deterministic_policy_flags_unsorted_json_dumps(tmp_path: Path) -> None:
    path = tmp_path / "packages/atlasctl/src/atlasctl/foo.py"
    path.parent.mkdir(parents=True)
    path.write_text('import json\nx=json.dumps({"b":1})\n', encoding="utf-8")
    code, errors = check_json_output_deterministic_policy(tmp_path)
    assert code == 1
    assert any("sort_keys=True" in e for e in errors)


def test_runtime_access_guard_checks_flag_non_allowlisted_usage(tmp_path: Path) -> None:
    src = tmp_path / "packages/atlasctl/src/atlasctl/foo"
    src.mkdir(parents=True)
    f = src / "bar.py"
    f.write_text(
        'import os\nfrom pathlib import Path\nx=os.environ.get("X")\ny=os.getcwd()\nz=Path("configs/x.json")\n',
        encoding="utf-8",
    )
    (tmp_path / "configs/policy").mkdir(parents=True)
    (tmp_path / "configs/policy/runtime-access-legacy-allowlist.json").write_text(
        json.dumps({"getcwd": [], "direct_env": [], "repo_root_path_literals": []}),
        encoding="utf-8",
    )
    code_env, errors_env = check_no_direct_env_reads_outside_runtime_env(tmp_path)
    code_cwd, errors_cwd = check_no_os_getcwd_outside_runtime_context(tmp_path)
    code_path, errors_path = check_no_repo_root_path_literals(tmp_path)
    assert code_env == 1 and any("direct env access" in e for e in errors_env)
    assert code_cwd == 1 and any("os.getcwd()" in e for e in errors_cwd)
    assert code_path == 1 and any("repo-root path literals" in e for e in errors_path)
