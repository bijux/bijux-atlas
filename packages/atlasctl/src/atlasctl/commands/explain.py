"""Command explain metadata powered by CLI registry."""

from __future__ import annotations

from ..cli.registry import command_registry


_DEFAULT = {
    "touches": [],
    "tools": [],
    "note": "no explicit contract entry; inspect command implementation",
}

_EXPLAIN_OVERRIDES: dict[str, dict[str, object]] = {
    "check": {"touches": ["makefiles/", "configs/", ".github/workflows/"], "tools": []},
    "docs": {"touches": ["docs/", "mkdocs.yml", "docs/_generated/"], "tools": ["mkdocs"]},
    "configs": {"touches": ["configs/", "docs/_generated/config*"], "tools": []},
    "ops": {"touches": ["ops/", "artifacts/evidence/"], "tools": ["kubectl", "helm", "k6"]},
    "make": {"touches": ["makefiles/", "docs/development/make-targets.md"], "tools": ["make"]},
    "report": {"touches": ["artifacts/evidence/", "ops/_generated_committed/"], "tools": []},
    "gates": {"touches": ["configs/gates/lanes.json", "artifacts/evidence/"], "tools": ["make"]},
}


def describe_command(name: str) -> dict[str, object]:
    known = {spec.name for spec in command_registry()}
    if name not in known:
        return dict(_DEFAULT)
    if name in _EXPLAIN_OVERRIDES:
        return dict(_EXPLAIN_OVERRIDES[name])
    return {"touches": [], "tools": []}
