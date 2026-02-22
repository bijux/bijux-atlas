from __future__ import annotations

import json
import re
import shutil
from importlib import import_module
from datetime import date, datetime, timezone
from pathlib import Path

from atlasctl.core.exec import run
from atlasctl.core.runtime.paths import write_text_file

def check_script_tool_guards(repo_root: Path) -> tuple[int, list[str]]:
    tool_re = re.compile(r"\b(kubectl|helm|kind|k6)\b")
    guards = ("check_tool_versions.py", "ops_version_guard", "packages/atlasctl/src/atlasctl/observability/contracts/governance/check_tool_versions.py")
    errors: list[str] = []
    for scan_dir in (repo_root / "scripts/bin", repo_root / "scripts/check", repo_root / "scripts/ci"):
        if not scan_dir.exists():
            continue
        for path in sorted(scan_dir.rglob("*.sh")):
            rel = path.relative_to(repo_root).as_posix()
            text = path.read_text(encoding="utf-8", errors="ignore")
            if not tool_re.search(text):
                continue
            if any(g in text for g in guards):
                continue
            errors.append(rel)
    return (0 if not errors else 1), errors

def check_script_shim_expiry(repo_root: Path) -> tuple[int, list[str]]:
    cfg = repo_root / "configs/layout/script-shim-expiries.json"
    data = json.loads(cfg.read_text(encoding="utf-8"))
    shims = data.get("shims", [])
    known = {entry["path"] for entry in shims if isinstance(entry, dict) and "path" in entry}
    errors: list[str] = []
    max_active = int(data.get("max_active_shims", 9999))
    shim_paths: list[str] = []
    for base in (repo_root / "scripts/bin", repo_root / "bin"):
        if not base.exists():
            continue
        for path in sorted(base.glob("*")):
            if not path.is_file() or path.name == "atlasctl":
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            if "DEPRECATED:" not in text:
                continue
            rel = path.relative_to(repo_root).as_posix()
            shim_paths.append(rel)
            if rel not in known:
                errors.append(f"shim missing expiry metadata: {rel}")
    if len(shim_paths) > max_active:
        errors.append(f"shim budget exceeded: active={len(shim_paths)} max_active_shims={max_active}")
    today = date.today()
    for row in shims:
        rel = row.get("path", "")
        if not rel:
            errors.append("shim metadata missing path")
            continue
        if not str(row.get("replacement", "")).strip():
            errors.append(f"shim metadata missing replacement command: {rel}")
        if not str(row.get("migration_doc", "")).strip():
            errors.append(f"shim metadata missing migration_doc: {rel}")
        path = repo_root / rel
        if not path.exists():
            errors.append(f"shim metadata points to missing file: {rel}")
            continue
        exp = date.fromisoformat(str(row.get("expires_on", "")))
        if exp < today:
            errors.append(f"shim expired: {rel} expired_on={exp.isoformat()}")
    return (0 if not errors else 1), errors

def check_script_shims_minimal(repo_root: Path) -> tuple[int, list[str]]:
    cfg = repo_root / "configs/layout/script-shim-expiries.json"
    payload = json.loads(cfg.read_text(encoding="utf-8"))
    errors: list[str] = []
    for row in payload.get("shims", []):
        if not isinstance(row, dict):
            continue
        rel = str(row.get("path", ""))
        if not rel:
            continue
        path = repo_root / rel
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        lines = [ln.strip() for ln in text.splitlines() if ln.strip()]
        if not lines or not lines[0].startswith("#!/usr/bin/env sh"):
            errors.append(f"{rel}: shim must use portable sh shebang")
        if "DEPRECATED:" not in text:
            errors.append(f"{rel}: missing DEPRECATED warning banner")
        if "docs/development/tooling/atlasctl.md" not in text:
            errors.append(f"{rel}: missing migration doc link")
        if "exec " not in text:
            errors.append(f"{rel}: missing exec passthrough")
        if any(tok in text for tok in ("tee ", "mktemp", "touch ", "cat > ", "printf > ", "echo > ")):
            errors.append(f"{rel}: shim must not write artifacts/files")
        if "set -x" in text or "uname" in text or "if [ \"$OSTYPE\"" in text:
            errors.append(f"{rel}: shim must be deterministic and OS-neutral")
        non_comment = [ln for ln in lines if not ln.startswith("#")]
        if len(non_comment) > 2:
            errors.append(f"{rel}: shim must stay minimal (echo + exec only)")
    return (0 if not errors else 1), errors

def check_venv_location_policy(repo_root: Path) -> tuple[int, list[str]]:
    allowed_prefixes = ("artifacts/atlasctl/",)
    proc = run(
        ["git", "ls-files", "--others", "--cached", "--exclude-standard"],
        cwd=repo_root,
        text=True,
        capture_output=True,
    )
    paths = [p.strip() for p in proc.stdout.splitlines() if p.strip()]
    violations: list[str] = []
    for rel in paths:
        if ".venv" not in Path(rel).parts:
            continue
        if any(rel.startswith(prefix) for prefix in allowed_prefixes):
            continue
        violations.append(rel)
    root_venv = repo_root / ".venv"
    if root_venv.exists():
        violations.append(".venv")
    return (0 if not violations else 1), violations

def check_python_runtime_artifacts(repo_root: Path, *, fix: bool = False) -> tuple[int, list[str]]:
    allowed_prefix = (repo_root / "artifacts").resolve()

    def allowed(path: Path) -> bool:
        resolved = path.resolve()
        return resolved == allowed_prefix or allowed_prefix in resolved.parents

    violations: list[str] = []
    paths_to_remove: list[Path] = []
    forbidden_dirs = {".venv", ".ruff_cache", ".pytest_cache", ".mypy_cache", "__pycache__", ".hypothesis"}
    for path in repo_root.rglob("*"):
        if path.is_dir() and path.name in forbidden_dirs and not allowed(path):
            if ".git" in path.parts:
                continue
            violations.append(f"forbidden dir outside artifacts: {path.relative_to(repo_root)}")
            paths_to_remove.append(path)
    for path in repo_root.rglob("*.pyc"):
        if not allowed(path):
            if ".git" in path.parts:
                continue
            violations.append(f"forbidden pyc outside artifacts: {path.relative_to(repo_root)}")
            paths_to_remove.append(path)
    tracked = run(["git", "ls-files"], cwd=repo_root, text=True, capture_output=True)
    for rel in tracked.stdout.splitlines():
        if fnmatch(rel, "*.pyc"):
            violations.append(f"tracked pyc file: {rel}")
    if violations and fix:
        for path in sorted(set(paths_to_remove), key=lambda p: len(p.parts), reverse=True):
            if path.is_dir():
                shutil.rmtree(path, ignore_errors=True)
            elif path.is_file():
                path.unlink(missing_ok=True)
        return 0, [f"python runtime artifact policy auto-fixed ({len(paths_to_remove)} paths)"]
    return (0 if not violations else 1), violations

def check_repo_script_boundaries(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    scripts_files = [p for p in _git_ls_files(repo_root, ["scripts/**"]) if not p.endswith(".md")]
    for rel in scripts_files:
        if _find_python_migration_exception(repo_root, "scripts_dir", rel, "") is None:
            errors.append(f"scripts directory transition is closed; file must move under packages/: {rel}")

    exec_proc = run(
        ["git", "ls-files", "--stage", "*.py"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    for line in exec_proc.stdout.splitlines():
        if not line.strip():
            continue
        mode, _obj, stage_path = line.split(maxsplit=2)
        _stage, rel = stage_path.split("\t", 1)
        if mode != "100755":
            continue
        if rel.startswith("packages/") or "/tests/" in rel:
            continue
        if _find_python_migration_exception(repo_root, "executable_python", rel, rel) is None:
            errors.append(f"executable python outside packages/: {rel}")

    for rel in _git_ls_files(repo_root, ["*.sh"]):
        if rel.startswith("docker/") or rel.startswith("packages/"):
            continue
        if _find_python_migration_exception(repo_root, "shell_location", rel, "") is None:
            errors.append(f"shell script outside docker/ or packages/: {rel}")
    return (0 if not errors else 1), errors

def check_atlas_scripts_cli_contract(repo_root: Path) -> tuple[int, list[str]]:
    cli = repo_root / "bin/atlasctl"
    pyproject = repo_root / "packages/atlasctl/pyproject.toml"
    expected_version = ""
    for ln in pyproject.read_text(encoding="utf-8").splitlines():
        stripped = ln.strip()
        if stripped.startswith("version = "):
            expected_version = stripped.split("=", 1)[1].strip().strip('"').strip("'")
            break
    errs: list[str] = []
    h1 = run([str(cli), "--help"], cwd=repo_root, text=True, capture_output=True)
    h2 = run([str(cli), "--help"], cwd=repo_root, text=True, capture_output=True)
    if h1.returncode != 0 or h2.returncode != 0:
        errs.append("atlasctl --help must exit 0")
    if h1.stdout != h2.stdout:
        errs.append("atlasctl --help output is not deterministic")
    v = run([str(cli), "--version"], cwd=repo_root, text=True, capture_output=True)
    if v.returncode != 0:
        errs.append("atlasctl --version must exit 0")
    else:
        out = (v.stdout or v.stderr).strip()
        if expected_version and expected_version not in out:
            errs.append(f"atlasctl version mismatch: expected {expected_version}, got `{out}`")
    return (0 if not errs else 1), errs

def check_atlasctl_boundaries(repo_root: Path) -> tuple[int, list[str]]:
    from ...layout.boundary_check import check_boundaries

    violations = check_boundaries(repo_root)
    errors = [f"{v.file}:{v.line} disallowed import {v.source} -> {v.target}" for v in violations]
    return (0 if not errors else 1), errors

def generate_scripts_sbom(repo_root: Path, lock_rel: str, out_rel: str) -> tuple[int, list[str]]:
    lock = repo_root / lock_rel
    lines = [ln.strip() for ln in lock.read_text(encoding="utf-8").splitlines() if ln.strip() and not ln.startswith("#")]
    packages = []
    for item in lines:
        name, version = item.split("==", 1)
        packages.append({"name": name, "version": version, "purl": f"pkg:pypi/{name}@{version}"})
    payload = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source_lock": lock.as_posix(),
        "package_count": len(packages),
        "packages": packages,
    }
    out = repo_root / out_rel
    write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0, [out_rel]

_NATIVE_LINT = import_module("atlasctl.checks.repo.native_lint")
check_effects_lint = _NATIVE_LINT.check_effects_lint
check_naming_intent_lint = _NATIVE_LINT.check_naming_intent_lint
check_root_bin_shims = _NATIVE_LINT.check_root_bin_shims
