#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
CONFIGS = ROOT / "configs"
ALLOWED_INTERNAL_PREFIXES = (
    "configs/_schemas/",
    "configs/_meta/",
    "configs/meta/",
    "configs/layout/",
    "configs/repo/",
    "configs/docs/.vale/styles/",
    "configs/ops/pins/",
)
ALLOWED_INTERNAL_FILES = {
    "configs/docs/depth-budget.json",
    "configs/ops/policies/ops-smoke-budget.json",
    "configs/ops/public-make-targets.json",
    "configs/perf/critical_queries_explain_snapshot.json",
    "configs/policy/ops-smoke-budget-relaxations.json",
    "configs/contracts/inventory-budgets.schema.json",
    "configs/contracts/inventory-configs.schema.json",
    "configs/contracts/inventory-contracts.schema.json",
    "configs/contracts/inventory-make.schema.json",
    "configs/contracts/inventory-ops.schema.json",
    "configs/contracts/inventory-owners.schema.json",
    "configs/contracts/inventory-schemas.schema.json",
    "configs/contracts/make-contracts-check-output.schema.json",
}
EXTS = {".json", ".yaml", ".yml", ".toml"}


def load_corpus() -> str:
    parts: list[str] = []
    roots = [ROOT / "docs", ROOT / "makefiles", ROOT / "scripts", ROOT / "ops", ROOT / "configs"]
    for base in roots:
        for p in sorted(base.rglob("*")):
            if not p.is_file():
                continue
            rel = p.relative_to(ROOT).as_posix()
            if rel.startswith(("artifacts/", "ops/_artifacts/", "ops/_evidence/", "docs/_generated/")):
                continue
            if "__pycache__" in rel:
                continue
            if p.suffix not in {".md", ".mk", ".sh", ".py", ".json", ".yaml", ".yml", ".toml", ".ini"}:
                continue
            parts.append(p.read_text(encoding="utf-8", errors="ignore"))
    return "\n".join(parts)


def main() -> int:
    corpus = load_corpus()
    errors: list[str] = []
    for p in sorted(CONFIGS.rglob("*")):
        if not p.is_file() or p.suffix not in EXTS:
            continue
        rel = p.relative_to(ROOT).as_posix()
        if any(rel.startswith(prefix) for prefix in ALLOWED_INTERNAL_PREFIXES):
            continue
        if rel in ALLOWED_INTERNAL_FILES:
            continue
        if re.search(rf"\b{re.escape(rel)}\b", corpus):
            continue
        errors.append(f"orphan config file (not referenced by docs/contracts): {rel}")

    if errors:
        print("orphan config file check failed:", file=sys.stderr)
        for err in errors[:200]:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("orphan config file check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
