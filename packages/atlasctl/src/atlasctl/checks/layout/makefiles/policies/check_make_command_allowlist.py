#!/usr/bin/env python3
# Purpose: enforce make recipe command prefixes against a repository allowlist.
# Inputs: Makefile + makefiles/*.mk + configs/layout/make-command-allowlist.txt.
# Outputs: non-zero on disallowed recipe command prefixes.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
ALLOWLIST = ROOT / "configs" / "layout" / "make-command-allowlist.txt"
MK_FILES = [ROOT / "Makefile", *sorted((ROOT / "makefiles").glob("*.mk"))]

SKIP_PREFIXES = ("if ", "for ", "while ", "case ", "{ ", "(", "then", "else", "fi", "do", "done")
SKIP_TOKENS = {"\\", "-u", "-n", "-c", "-", "exit", "trap", "done;", "then", "fi;", "do"}


def _load_allowlist() -> list[str]:
    return [
        ln.strip()
        for ln in ALLOWLIST.read_text(encoding="utf-8").splitlines()
        if ln.strip() and not ln.lstrip().startswith("#")
    ]


def _first_token(cmd: str) -> str:
    line = cmd.strip()
    while True:
        m = re.match(r"^[A-Za-z_][A-Za-z0-9_]*=(?:\"[^\"]*\"|'[^']*'|[^\s]+)\s+", line)
        if not m:
            break
        line = line[m.end() :].lstrip()
    if not line:
        return ""
    return line.split()[0]


def main() -> int:
    if not ALLOWLIST.exists():
        print(f"missing allowlist: {ALLOWLIST.relative_to(ROOT)}", file=sys.stderr)
        return 1
    allow = _load_allowlist()
    violations: list[str] = []
    for mk in MK_FILES:
        continued = False
        phony_block = False
        for idx, raw in enumerate(mk.read_text(encoding="utf-8").splitlines(), start=1):
            if not raw.startswith("\t"):
                continued = False
                phony_block = raw.strip().startswith(".PHONY:")
                continue
            if phony_block:
                phony_block = raw.rstrip().endswith("\\")
                continue
            if continued:
                continued = raw.rstrip().endswith("\\")
                continue
            cmd = raw.lstrip()[1:].strip() if raw.lstrip().startswith("@") else raw.strip()
            continued = raw.rstrip().endswith("\\")
            if not cmd or cmd.startswith("#") or cmd.startswith("-"):
                continue
            if "$$(" in cmd or "|" in cmd:
                continue
            if ":?" in cmd:
                continue
            if any(cmd.startswith(prefix) for prefix in SKIP_PREFIXES):
                continue
            tok = _first_token(cmd)
            if not tok:
                continue
            if tok in SKIP_TOKENS:
                continue
            if tok.startswith("./"):
                continue
            if tok.startswith('"') or tok.startswith("'"):
                continue
            if tok.startswith("$(") or tok.startswith("$${") or tok.startswith('"$('):
                continue
            if not re.fullmatch(r"[A-Za-z0-9_.+-]+", tok):
                continue
            if any(tok == item or tok.startswith(item) for item in allow):
                continue
            violations.append(f"{mk.relative_to(ROOT)}:{idx}: disallowed recipe command `{tok}`")

    if violations:
        print("make command allowlist check failed:", file=sys.stderr)
        for item in violations[:200]:
            print(f"- {item}", file=sys.stderr)
        return 1

    print("make command allowlist check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
