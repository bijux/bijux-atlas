#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):\s*(.*)$")
MAKE_CALL_RE = re.compile(r"\$\(MAKE\)\s+([^#]+)")


def parse_make_targets(makefiles_dir: Path) -> dict[str, list[str]]:
    graph: dict[str, list[str]] = {}
    for mk in sorted(makefiles_dir.glob("*.mk")):
        text = mk.read_text(encoding="utf-8")
        current_target: str | None = None
        for line in text.splitlines():
            if line.startswith(".") or line.startswith("#"):
                continue
            m = TARGET_RE.match(line)
            if m:
                target, deps = m.group(1), m.group(2)
                if target.startswith("."):
                    continue
                current_target = target
                deps = deps.split("#", 1)[0].strip()
                if deps:
                    graph[target] = [d for d in deps.split() if d and "$" not in d and not d.startswith("|")]
                else:
                    graph.setdefault(target, [])
                continue

            if current_target is None or not line.startswith("\t"):
                continue

            call = MAKE_CALL_RE.search(line)
            if not call:
                continue
            for token in call.group(1).split():
                token = token.strip(");")
                if token.startswith("-") or "=" in token or token in {
                    "&&",
                    ";",
                    "\\",
                    "if",
                    "then",
                    "else",
                    "fi",
                    "for",
                    "do",
                    "done",
                }:
                    continue
                graph.setdefault(current_target, [])
                if token not in graph[current_target]:
                    graph[current_target].append(token)
    return graph


def render_tree(graph: dict[str, list[str]], root: str, prefix: str = "", seen: set[str] | None = None) -> list[str]:
    if seen is None:
        seen = set()
    lines = [f"{prefix}{root}"]
    if root in seen:
        lines[-1] += " (cycle)"
        return lines
    seen = set(seen)
    seen.add(root)
    deps = graph.get(root, [])
    for i, dep in enumerate(deps):
        branch = "└─ " if i == len(deps) - 1 else "├─ "
        child_lines = render_tree(graph, dep, prefix + branch, seen)
        lines.extend(child_lines)
    return lines
