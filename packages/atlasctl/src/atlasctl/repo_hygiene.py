from __future__ import annotations

import json
import shutil
from dataclasses import dataclass
from pathlib import Path
from subprocess import run


_CACHE_DIR_NAMES = {"__pycache__", ".pytest_cache", ".mypy_cache", ".ruff_cache"}
_SIZE_LIMIT_MB = 5
_ALLOWLISTED_OPS_ARTIFACT_EXAMPLES: set[str] = set()
_GENERATED_PATH_PREFIXES = (
    "configs/_generated/",
    "docs/_generated/",
    "ops/_generated_committed/",
    "ops/_generated.example/",
)
_TIMESTAMP_KEYS = ("generated_at", "timestamp", "created_at", "updated_at")
_FORBIDDEN_OPS_LOCAL_WRITE_TOKENS = ("ops/_evidence/", "ops/_artifacts/")


@dataclass(frozen=True)
class RepoHygieneResult:
    status: str
    checks: dict[str, list[str]]


def _git_ls_files(repo_root: Path, pathspecs: list[str]) -> list[str]:
    proc = run(["git", "ls-files", "--", *pathspecs], cwd=repo_root, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        return []
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def _find_dirs(root: Path, name: str) -> list[Path]:
    if not root.exists():
        return []
    return sorted(path for path in root.rglob(name) if path.is_dir())


def _find_files(root: Path, pattern: str) -> list[Path]:
    return sorted(path for path in root.rglob(pattern) if path.is_file())


def _allowed_symlink_targets(repo_root: Path) -> set[str]:
    cfg = json.loads((repo_root / "configs/repo/symlink-allowlist.json").read_text(encoding="utf-8"))
    allowed = set()
    for rel in cfg.get("root", {}).keys():
        allowed.add(rel)
    for rel in cfg.get("non_root", {}).keys():
        allowed.add(rel)
    return allowed


def _tracked_files_case_conflicts(repo_root: Path) -> list[str]:
    by_lower: dict[str, list[str]] = {}
    for rel in _git_ls_files(repo_root, ["."]):
        key = rel.lower()
        by_lower.setdefault(key, []).append(rel)
    conflicts = []
    for rows in by_lower.values():
        uniq = sorted(set(rows))
        if len(uniq) > 1:
            conflicts.append(", ".join(uniq))
    return sorted(conflicts)


def _tracked_symlink_violations(repo_root: Path) -> list[str]:
    allowed = _allowed_symlink_targets(repo_root)
    violations: list[str] = []
    for rel in _git_ls_files(repo_root, ["."]):
        path = repo_root / rel
        if not path.is_symlink():
            continue
        if rel not in allowed:
            violations.append(rel)
    return sorted(violations)


def _tracked_large_file_violations(repo_root: Path, size_limit_mb: int) -> list[str]:
    limit = size_limit_mb * 1024 * 1024
    violations: list[str] = []
    for rel in _git_ls_files(repo_root, ["."]):
        path = repo_root / rel
        if not path.is_file():
            continue
        try:
            size = path.stat().st_size
        except OSError:
            continue
        if size > limit:
            violations.append(f"{rel} ({size} bytes)")
    return sorted(violations)


def _tracked_json_yaml_timestamp_violations(repo_root: Path) -> list[str]:
    violations: list[str] = []
    tracked = _git_ls_files(repo_root, ["."])
    for rel in tracked:
        if not rel.endswith((".json", ".yaml", ".yml")):
            continue
        if not rel.startswith(_GENERATED_PATH_PREFIXES):
            continue
        text = (repo_root / rel).read_text(encoding="utf-8", errors="ignore").lower()
        if any(key in text for key in _TIMESTAMP_KEYS):
            violations.append(rel)
    return sorted(violations)


def _tracked_ops_artifact_violations(repo_root: Path) -> list[str]:
    tracked = _git_ls_files(repo_root, ["ops/_artifacts"])
    violations = []
    for rel in tracked:
        if rel in _ALLOWLISTED_OPS_ARTIFACT_EXAMPLES:
            continue
        violations.append(rel)
    return sorted(violations)


def run_repo_hygiene_checks(repo_root: Path) -> RepoHygieneResult:
    cache_dirs_packages = [p.relative_to(repo_root).as_posix() for p in _find_dirs(repo_root / "packages", "__pycache__")]
    pyc_all = [p.relative_to(repo_root).as_posix() for p in _find_files(repo_root, "*.pyc")]
    pytest_cache_all = [p.relative_to(repo_root).as_posix() for p in _find_dirs(repo_root, ".pytest_cache")]
    ops_evidence_tracked = _git_ls_files(repo_root, ["ops/_evidence"])
    ops_artifacts_tracked = _tracked_ops_artifact_violations(repo_root)
    ops_generated_tracked = _git_ls_files(repo_root, ["ops/_generated"])
    configs_generated_tracked = _git_ls_files(repo_root, ["configs/_generated"])
    config_checksums = repo_root / "configs/_generated/checksums.json"
    configs_generated_without_checksums = []
    if configs_generated_tracked and not config_checksums.exists():
        configs_generated_without_checksums = ["configs/_generated/checksums.json missing"]
    duplicate_case_paths = _tracked_files_case_conflicts(repo_root)
    symlink_violations = _tracked_symlink_violations(repo_root)
    large_file_violations = _tracked_large_file_violations(repo_root, _SIZE_LIMIT_MB)
    generated_timestamp_violations = _tracked_json_yaml_timestamp_violations(repo_root)
    ops_local_write_token_violations: list[str] = []
    commands_root = repo_root / "packages/atlasctl/src/atlasctl/commands"
    if commands_root.exists():
        for path in sorted(commands_root.rglob("*.py")):
            rel = path.relative_to(repo_root).as_posix()
            text = path.read_text(encoding="utf-8", errors="ignore")
            if any(token in text for token in _FORBIDDEN_OPS_LOCAL_WRITE_TOKENS):
                ops_local_write_token_violations.append(rel)

    checks = {
        "no_pycache_under_packages": cache_dirs_packages,
        "no_pyc_in_repo": pyc_all,
        "no_pytest_cache_in_repo": pytest_cache_all,
        "no_tracked_ops_evidence": ops_evidence_tracked,
        "no_tracked_ops_artifacts_except_examples": ops_artifacts_tracked,
        "no_tracked_ops_generated": ops_generated_tracked,
        "configs_generated_requires_checksums": configs_generated_without_checksums,
        "no_case_conflicting_paths": duplicate_case_paths,
        "symlinks_match_allowlist": symlink_violations,
        "no_tracked_files_larger_than_5mb": large_file_violations,
        "no_timestamps_in_generated_json_yaml": generated_timestamp_violations,
        "no_ops_local_write_paths_in_commands": ops_local_write_token_violations,
    }
    failed = any(rows for rows in checks.values())
    return RepoHygieneResult(status="error" if failed else "ok", checks=checks)


def apply_hygiene_fixes(repo_root: Path) -> dict[str, object]:
    removed: list[str] = []
    for name in _CACHE_DIR_NAMES:
        for path in _find_dirs(repo_root, name):
            try:
                shutil.rmtree(path)
                removed.append(path.relative_to(repo_root).as_posix())
            except OSError:
                continue
    for path in _find_files(repo_root, "*.pyc"):
        try:
            path.unlink()
            removed.append(path.relative_to(repo_root).as_posix())
        except OSError:
            continue
    for path in _find_files(repo_root, "*.pyo"):
        try:
            path.unlink()
            removed.append(path.relative_to(repo_root).as_posix())
        except OSError:
            continue
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "fix-hygiene",
        "status": "ok",
        "removed": sorted(set(removed)),
    }
