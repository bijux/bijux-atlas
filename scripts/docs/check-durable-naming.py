#!/usr/bin/env python3
# owner: docs-governance
# purpose: enforce durable naming rules and forbid temporal/task placeholder naming.
# stability: public
# called-by: make rename-lint, make ci-rename-lint
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FORBIDDEN_PREFIX_RE = re.compile(
    r"^(phase|task|stage|round|iteration|tmp|placeholder|vnext)([-_0-9]|$)",
    re.IGNORECASE,
)
KEBAB_SCRIPT = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*\.(sh|py)$")
KEBAB_DOC = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*\.md$")
ADR_DOC = re.compile(r"^ADR-\d{4}-[a-z0-9-]+\.md$")
SCREAM_DOC = re.compile(r"^[A-Z0-9_]+\.md$")
DOC_EXACT_EXCEPTIONS = {
    "docs/STYLE.md",
    "docs/contracts/README.md",
    "docs/api/compatibility.md",
    "docs/api/deprecation.md",
    "docs/api/v1-surface.md",
}
DOC_NAME_EXCEPTIONS = {"INDEX.md", "CONCEPT_REGISTRY.md", "DEPTH_POLICY.md", "DEPTH_RUBRIC.md"}


def tracked_files() -> list[str]:
    out = subprocess.check_output(["git", "ls-files"], cwd=ROOT, text=True)
    return [line.strip() for line in out.splitlines() if line.strip()]


def is_docs_markdown(path: str) -> bool:
    return path.startswith("docs/") and path.endswith(".md")


def is_public_script(path: str) -> bool:
    if not path.startswith("scripts/public/"):
        return False
    rel = Path(path).relative_to("scripts/public")
    return rel.parent == Path(".") and (path.endswith(".sh") or path.endswith(".py"))


def main() -> int:
    files = tracked_files()
    errors: list[str] = []

    case_map: dict[str, list[str]] = {}
    for path in files:
        case_map.setdefault(path.lower(), []).append(path)

    for lower, variants in sorted(case_map.items()):
        uniq = sorted(set(variants))
        if len(uniq) > 1:
            errors.append(f"case-collision path variants: {uniq}")

    for path in files:
        name = Path(path).name
        stem = Path(path).stem
        if path.startswith("docs/_drafts/"):
            pass
        elif FORBIDDEN_PREFIX_RE.search(stem):
            errors.append(f"forbidden temporal/task token in name: {path}")
        if is_docs_markdown(path):
            name = Path(path).name
            if (
                path not in DOC_EXACT_EXCEPTIONS
                and name not in DOC_NAME_EXCEPTIONS
                and not KEBAB_DOC.match(name)
                and not ADR_DOC.match(name)
                and not (path.startswith("docs/_generated/contracts/") and SCREAM_DOC.match(name))
            ):
                errors.append(f"docs markdown must use kebab-case or approved canonical exception: {path}")
        if is_public_script(path):
            if not KEBAB_SCRIPT.match(name):
                errors.append(f"public scripts must use kebab-case: {path}")

    if errors:
        print("durable naming check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("durable naming check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
