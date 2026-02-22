from __future__ import annotations

import json
from pathlib import Path

from ...core.runtime.paths import write_text_file


TARGETS: tuple[str, ...] = (
    "packages/atlasctl/src/atlasctl/checks/REGISTRY.toml",
    "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json",
    "packages/atlasctl/src/atlasctl/registry/checks_catalog.json",
    "packages/atlasctl/tests",
    "packages/atlasctl/docs",
    "docs",
    "configs",
    "makefiles",
)


def _load_map(repo_root: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for rel in ("configs/policy/check-id-migration.json", "configs/policy/target-renames.json"):
        path = repo_root / rel
        if not path.exists():
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        rows = payload.get("check_ids", {})
        if not isinstance(rows, dict):
            continue
        out.update({str(old): str(new) for old, new in rows.items() if str(old).strip() and str(new).strip()})
    return out


def run_refactor_check_ids(repo_root: Path, *, apply: bool) -> tuple[int, list[str]]:
    mapping = _load_map(repo_root)
    touched: list[str] = []
    for target in TARGETS:
        path = repo_root / target
        if not path.exists():
            continue
        files = [path] if path.is_file() else sorted(p for p in path.rglob("*") if p.is_file())
        for file in files:
            if file.suffix in {".pyc", ".png", ".jpg", ".jpeg", ".gif", ".pdf"}:
                continue
            text = file.read_text(encoding="utf-8", errors="ignore")
            updated = text
            for old, new in mapping.items():
                updated = updated.replace(old, new)
            if updated == text:
                continue
            touched.append(file.relative_to(repo_root).as_posix())
            if apply:
                write_text_file(file, updated, encoding="utf-8")
    return 0, sorted(touched)
