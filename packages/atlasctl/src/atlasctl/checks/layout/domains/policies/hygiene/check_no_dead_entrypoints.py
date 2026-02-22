#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

from atlasctl.core.exec import run

ROOT = Path(__file__).resolve().parents[6]
DOCS = ROOT / "docs"
WORKFLOWS = ROOT / ".github" / "workflows"

SCRIPT_RE = re.compile(r"`(?:\./)?((?:scripts|ops)/[A-Za-z0-9_./-]+\.(?:sh|py))`")
DOC_MAKE_RE = re.compile(r"`make\s+([A-Za-z0-9_./-]+)\b[^`]*`")
SHELL_MAKE_RE = re.compile(r"(?:^|[;&|]\s*|\s)(?:\$\((?:MAKE)\)|make)\s+([^\n#]+)")


def load_make_targets() -> set[str]:
    proc = run(["make", "-qp"], cwd=ROOT, text=True, capture_output=True, check=False)
    targets: set[str] = set()
    for line in proc.stdout.splitlines():
        if ":" not in line or line.startswith("\t") or line.startswith("#"):
            continue
        name = line.split(":", 1)[0].strip()
        if not name or any(ch in name for ch in " %$()"):
            continue
        targets.add(name)
    return targets


def scan_files() -> list[Path]:
    files = sorted(
        p
        for p in DOCS.rglob("*.md")
        if "docs/_generated/" not in p.as_posix() and "docs/_lint/" not in p.as_posix()
    )
    files.extend(sorted(WORKFLOWS.glob("*.yml")))
    return files


def parse_shell_make_targets(line: str) -> list[str]:
    found: list[str] = []
    for match in SHELL_MAKE_RE.finditer(line):
        tail = match.group(1).strip()
        if not tail:
            continue
        tokens = tail.split()
        for token in tokens:
            if token.startswith("-"):
                continue
            if "=" in token:
                continue
            if "$" in token or "(" in token or ")" in token:
                continue
            if token in {"&&", "||", ";", "\\"}:
                continue
            if re.fullmatch(r"[A-Za-z0-9_./-]+", token):
                found.append(token)
            break
    return found


def main() -> int:
    targets = load_make_targets()
    errs: list[str] = []
    for path in scan_files():
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for i, line in enumerate(text.splitlines(), start=1):
            for m in SCRIPT_RE.finditer(line):
                script = m.group(1)
                if not (ROOT / script).exists():
                    errs.append(f"{rel}:{i}: missing referenced script `{script}`")

            if path.suffix == ".md":
                for m in DOC_MAKE_RE.finditer(line):
                    target = m.group(1).strip()
                    if target and target not in targets:
                        errs.append(f"{rel}:{i}: missing referenced make target `{target}`")
            elif path.suffix in {".yml", ".yaml"} and "run:" in line:
                for target in parse_shell_make_targets(line):
                    if target not in targets:
                        errs.append(f"{rel}:{i}: missing referenced make target `{target}`")

    if errs:
        print("dead entrypoint check failed:", file=sys.stderr)
        for err in errs[:200]:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("dead entrypoint check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
