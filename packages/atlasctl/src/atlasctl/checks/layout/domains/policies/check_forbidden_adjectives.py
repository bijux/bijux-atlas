#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

from ......core.process import run_command

ROOT = Path(__file__).resolve().parents[8]
ALLOWLIST = ROOT / "configs" / "policy" / "forbidden-adjectives-allowlist.txt"
TERMS = ("elite", "refgrade", "gold")
PATTERN = re.compile(r"\b(?:elite|refgrade|gold)\b", re.IGNORECASE)
TEXT_EXTS = {
    ".md",
    ".txt",
    ".rst",
    ".json",
    ".yaml",
    ".yml",
    ".toml",
    ".mk",
    ".py",
    ".sh",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".rs",
}


def _allow_rules() -> list[str]:
    if not ALLOWLIST.exists():
        return []
    return [
        line.strip()
        for line in ALLOWLIST.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    ]


def _is_allowed(rel: str, rules: list[str]) -> bool:
    return any(rel == rule or rel.startswith(rule) for rule in rules)


def _tracked_files(diff_only: bool) -> list[str]:
    cmd = ["git", "diff", "--name-only", "--diff-filter=ACMRTUXB", "HEAD"] if diff_only else ["git", "ls-files"]
    proc = run_command(cmd, cwd=ROOT)
    if proc.code != 0:
        return []
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def main() -> int:
    diff_only = "--all-files" not in sys.argv[1:]
    rules = _allow_rules()
    errors: list[str] = []
    for rel in _tracked_files(diff_only=diff_only):
        path = ROOT / rel
        if path.suffix.lower() not in TEXT_EXTS:
            continue
        if not path.exists():
            continue
        if _is_allowed(rel, rules):
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if PATTERN.search(line):
                errors.append(f"{rel}:{lineno}: forbidden adjective match in `{line.strip()}`")
    if errors:
        print("forbidden adjective check failed", file=sys.stderr)
        print(f"terms={', '.join(TERMS)}", file=sys.stderr)
        for err in errors[:200]:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("forbidden adjective check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
