from __future__ import annotations

import json
from pathlib import Path

from .ops_runtime_parser import configure_ops_parser
import argparse


def list_ops_actions() -> list[str]:
    parser = argparse.ArgumentParser(prog="atlasctl")
    sub = parser.add_subparsers(dest="cmd")
    configure_ops_parser(sub)
    ops = next(a for a in parser._actions if isinstance(a, argparse._SubParsersAction) and "ops" in a.choices)
    ops_parser = ops.choices["ops"]
    ops_sub = next(a for a in ops_parser._actions if isinstance(a, argparse._SubParsersAction))
    actions: set[str] = set()
    for name, p in ops_sub.choices.items():
        nested = next((a for a in p._actions if isinstance(a, argparse._SubParsersAction)), None)
        if nested and nested.choices:
            for subname in nested.choices.keys():
                actions.add(f"{name}.{subname}")
        else:
            actions.add(name)
    return sorted(actions)


def inventory_payload() -> dict[str, object]:
    actions = list_ops_actions()
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "ops-actions-inventory",
        "status": "ok",
        "total": len(actions),
        "items": actions,
    }


def render_ops_actions_doc(repo_root: Path) -> str:
    payload = inventory_payload()
    lines = [
        "# Ops Actions",
        "",
        "Generated from `atlasctl ops --list-actions --json`.",
        "",
        f"- total: {payload['total']}",
        "",
        "## Actions",
        "",
    ]
    for action in payload["items"]:
        lines.append(f"- `{action}`")
    lines.append("")
    return "\n".join(lines)


def write_ops_actions_doc(repo_root: Path) -> Path:
    out = repo_root / "docs" / "_generated" / "ops-actions.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(render_ops_actions_doc(repo_root), encoding="utf-8")
    return out


def main() -> int:
    print(json.dumps(inventory_payload(), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
