#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DOCS_REF_DIR = REPO_ROOT / "docs" / "operations" / "reference"


def run_cli_help(args: list[str]) -> str:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-dev-atlas", "--", *args],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return proc.stdout.rstrip()


def trim_help_to_usage_and_commands(help_text: str) -> str:
    lines = help_text.splitlines()
    out: list[str] = []
    for line in lines:
        if line.startswith("Options:"):
            break
        out.append(line.rstrip())
    while out and out[-1] == "":
        out.pop()
    return "\n".join(out)


def render_commands_reference() -> str:
    root_help = trim_help_to_usage_and_commands(run_cli_help(["--help"]))
    ops_help = trim_help_to_usage_and_commands(run_cli_help(["ops", "--help"]))
    return "\n".join(
        [
            "# Command Surface Reference",
            "",
            "- Owner: `bijux-atlas-operations`",
            "- Tier: `generated`",
            "- Audience: `operators`",
            "- Source-of-truth: `bijux dev atlas --help`, `bijux dev atlas ops --help`, `makefiles/GENERATED_TARGETS.md`",
            "",
            "## Purpose",
            "",
            "Generated reference for the supported command surface. Narrative docs should link here instead of restating command lists.",
            "",
            "## bijux-dev-atlas",
            "",
            "```text",
            root_help,
            "```",
            "",
            "## bijux-dev-atlas ops",
            "",
            "```text",
            ops_help,
            "```",
            "",
            "## Make Wrapper Surface",
            "",
            "See `makefiles/GENERATED_TARGETS.md` and generated ops surface references. Narrative docs must not duplicate long `make ops-*` command lists.",
            "",
            "## Regenerate",
            "",
            "- `python3 scripts/docs/generate_operations_references.py --write`",
            "",
        ]
    )


def render_ops_surface_reference() -> str:
    surfaces = json.loads((REPO_ROOT / "ops" / "inventory" / "surfaces.json").read_text())
    entrypoints = sorted(surfaces["entrypoints"])
    commands = sorted(surfaces["bijux-dev-atlas_commands"])
    actions = sorted(surfaces["actions"], key=lambda item: item["id"])

    lines = [
        "# Ops Surface Reference",
        "",
        "- Owner: `bijux-atlas-operations`",
        "- Tier: `generated`",
        "- Audience: `operators`",
        "- Source-of-truth: `ops/inventory/surfaces.json`, `ops/_generated.example/control-plane.snapshot.md`",
        "",
        "## Purpose",
        "",
        "Generated ops surface reference derived from inventory surfaces.",
        "",
        "## Entry Points",
        "",
    ]
    lines.extend([f"- `{item}`" for item in entrypoints])
    lines.extend(
        [
            "",
            "## bijux-dev-atlas Commands",
            "",
        ]
    )
    lines.extend([f"- `{item}`" for item in commands])
    lines.extend(
        [
            "",
            "## Actions",
            "",
        ]
    )
    lines.extend([f"- `{repr(item)}`" for item in actions])
    lines.extend(
        [
            "",
            "## See Also",
            "",
            "- `ops/_generated.example/control-plane.snapshot.md` (example generated snapshot)",
            "- `ops/inventory/surfaces.json` (machine truth)",
            "",
        ]
    )
    return "\n".join(lines)


def write_if_changed(path: Path, content: str) -> bool:
    current = path.read_text() if path.exists() else None
    if current == content:
        return False
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate docs/operations reference pages from SSOT inputs.")
    parser.add_argument("--write", action="store_true", help="Write files instead of check-only.")
    args = parser.parse_args()

    targets = {
        DOCS_REF_DIR / "commands.md": render_commands_reference(),
        DOCS_REF_DIR / "ops-surface.md": render_ops_surface_reference(),
    }

    changed: list[str] = []
    for path, content in targets.items():
        if args.write:
            if write_if_changed(path, content):
                changed.append(str(path.relative_to(REPO_ROOT)))
        else:
            existing = path.read_text()
            if existing != content:
                changed.append(str(path.relative_to(REPO_ROOT)))

    if changed:
        print(json.dumps({"status": "drift", "changed": changed}, indent=2))
        return 1
    print(json.dumps({"status": "ok", "generated": [str(p.relative_to(REPO_ROOT)) for p in targets]}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
