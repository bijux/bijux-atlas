from __future__ import annotations

import argparse
import json
from pathlib import Path

from .ops_runtime_parser import configure_ops_parser


def _ops_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="atlasctl")
    sub = parser.add_subparsers(dest="cmd")
    configure_ops_parser(sub)
    return parser


def _walk_subcommands(prefix: list[str], parser: argparse.ArgumentParser) -> list[list[str]]:
    nested = next((a for a in parser._actions if isinstance(a, argparse._SubParsersAction)), None)
    if not nested or not nested.choices:
        return [prefix]
    rows: list[list[str]] = []
    for name, child in sorted(nested.choices.items()):
        rows.extend(_walk_subcommands([*prefix, name], child))
    return rows


def list_ops_actions() -> list[str]:
    parser = _ops_parser()
    ops = next(a for a in parser._actions if isinstance(a, argparse._SubParsersAction) and "ops" in a.choices)
    ops_parser = ops.choices["ops"]
    ops_sub = next(a for a in ops_parser._actions if isinstance(a, argparse._SubParsersAction))
    actions: set[str] = set()
    for name, p in sorted(ops_sub.choices.items()):
        for path in _walk_subcommands([name], p):
            if len(path) == 1:
                actions.add(path[0])
            else:
                actions.add(".".join(path))
    return sorted(actions)


def action_registry() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for item in list_ops_actions():
        parts = item.split(".")
        if len(parts) == 1:
            action_id = f"ops.root.{parts[0]}"
            argv = [parts[0]]
            domain = "root"
        else:
            action_id = "ops." + ".".join(parts)
            argv = parts
            domain = parts[0]
        rows.append({"id": action_id, "domain": domain, "command": ["atlasctl", "ops", *argv], "argv": argv})
    rows.sort(key=lambda r: str(r["id"]))
    return rows


def action_argv(action_id: str) -> list[str] | None:
    for row in action_registry():
        if row["id"] == action_id:
            return [str(x) for x in row["argv"]]
    return None


def inventory_payload() -> dict[str, object]:
    actions = list_ops_actions()
    registry = action_registry()
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "ops-actions-inventory",
        "status": "ok",
        "total": len(actions),
        "items": actions,
        "action_ids": [str(row["id"]) for row in registry],
        "actions": registry,
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
