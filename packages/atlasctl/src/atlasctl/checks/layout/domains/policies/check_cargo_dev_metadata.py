#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
DEV_MK = ROOT / "makefiles" / "dev.mk"

TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)


def main() -> int:
    text = DEV_MK.read_text(encoding="utf-8")
    targets = [t for t in TARGET_RE.findall(text) if not t.startswith(".")]
    legacy = {
        "dev-fmt",
        "dev-lint",
        "dev-test",
        "dev-coverage",
        "internal/dev/fmt",
        "internal/dev/lint",
        "internal/dev/test",
        "internal/dev/audit",
        "internal/dev/coverage",
        "internal/dev/ci",
        "ci-core",
    }
    errors: list[str] = [f"legacy dev-* target still present in dev.mk: {t}" for t in targets if t in legacy]
    recipe_by_target: dict[str, str] = {}
    current: str | None = None
    for raw in text.splitlines():
        m = TARGET_RE.match(raw)
        if m:
            current = m.group(1)
            continue
        if current and raw.startswith("\t"):
            recipe_by_target[current] = raw.strip()

    if "fmt" not in targets:
        errors.append("missing required dev.mk target: fmt")
    required = {"fmt", "lint", "test", "test-all", "coverage", "check", "atlasctl-check"}
    for target in sorted(required):
        if target not in targets:
            errors.append(f"missing required dev.mk target: {target}")
    if recipe_by_target.get("atlasctl-check") != "@./bin/atlasctl check run --group repo":
        errors.append("atlasctl-check must reference canonical repo checks gate (`@./bin/atlasctl check run --group repo`)")

    if errors:
        print("dev wrapper metadata check failed", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("dev wrapper metadata check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
