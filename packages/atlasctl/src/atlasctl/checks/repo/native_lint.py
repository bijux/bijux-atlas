from __future__ import annotations

import re
from pathlib import Path


def check_root_bin_shims(repo_root: Path) -> tuple[int, list[str]]:
    bin_dir = repo_root / "bin"
    if not bin_dir.exists():
        return 0, []
    max_lines = 30
    allowed = re.compile(
        r"^(#!|# DEPRECATED:|# Migration:|echo \"DEPRECATED: .*\" >&2|set -euo pipefail|set -eu|ROOT=|PYTHONPATH=|\s*exec python3 -m atlasctl\.cli \"\$@\"|exec \".*/bijux-atlas\" make (explain|graph|help) \"\$@\"|\s*$)"
    )
    errors: list[str] = []
    for path in sorted(bin_dir.iterdir()):
        if path.name == "README.md" or not path.is_file():
            continue
        lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        rel = path.relative_to(repo_root).as_posix()
        if len(lines) > max_lines:
            errors.append(f"{rel} exceeds {max_lines} lines")
        for idx, line in enumerate(lines, 1):
            if not allowed.match(line):
                errors.append(f"{rel}:{idx}: non-shim logic is forbidden")
                break
    return (0 if not errors else 1), errors


def check_effects_lint(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    planner_files = [
        repo_root / "crates/bijux-atlas-query/src/planner/mod.rs",
        repo_root / "crates/bijux-atlas-query/src/filters.rs",
        repo_root / "crates/bijux-atlas-query/src/cost.rs",
        repo_root / "crates/bijux-atlas-query/src/limits.rs",
    ]
    forbidden = ("rusqlite", "reqwest", "std::fs", "tokio::net", "std::process")
    for path in planner_files:
        if not path.exists():
            errors.append(f"missing planner file: {path.relative_to(repo_root)}")
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for pat in forbidden:
            if re.search(rf"\b{re.escape(pat)}\b", text):
                errors.append(f"forbidden `{pat}` in {path.relative_to(repo_root)}")
    http_root = repo_root / "crates/bijux-atlas-server/src/http"
    if http_root.exists():
        for path in sorted(http_root.rglob("*.rs")):
            if path.name == "effects_adapters.rs":
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            if re.search(r"std::fs::|use std::fs::|File::open\(", text):
                errors.append(f"raw fs IO forbidden in {path.relative_to(repo_root)}")
            if re.search(
                r"runtime::dataset_cache_manager_(maintenance|storage)|crate::runtime::dataset_cache_manager_(maintenance|storage)",
                text,
            ):
                errors.append(f"http mapping must not import runtime effect internals in {path.relative_to(repo_root)}")
    return (0 if not errors else 1), errors


def check_naming_intent_lint(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in sorted((repo_root / "crates").rglob("*")):
        if not path.is_file():
            continue
        name = path.name
        if name == "helpers.rs" or name.endswith("_helpers.rs"):
            errors.append(path.relative_to(repo_root).as_posix())
    return (0 if not errors else 1), errors
