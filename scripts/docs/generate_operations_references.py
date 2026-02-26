#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

import yaml
import tomli


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


def render_tools_reference() -> str:
    data = tomli.loads((REPO_ROOT / "ops" / "inventory" / "tools.toml").read_text())
    tools = sorted(data.get("tools", []), key=lambda t: t["name"])
    lines = [
        "# Tools Reference",
        "",
        "- Owner: `bijux-atlas-operations`",
        "- Tier: `generated`",
        "- Audience: `operators`",
        "- Source-of-truth: `ops/inventory/tools.toml`",
        "",
        "## Tools",
        "",
        "| Tool | Required | Probe Args | Version Regex |",
        "| --- | --- | --- | --- |",
    ]
    for tool in tools:
        probe = " ".join(tool.get("probe_argv", []))
        lines.append(
            f"| `{tool['name']}` | `{str(tool.get('required', False)).lower()}` | `{probe}` | `{tool.get('version_regex','')}` |"
        )
    lines.extend(
        [
            "",
            "## Regenerate",
            "",
            "- `python3 scripts/docs/generate_operations_references.py --write`",
            "",
        ]
    )
    return "\n".join(lines)


def render_toolchain_reference() -> str:
    data = json.loads((REPO_ROOT / "ops" / "inventory" / "toolchain.json").read_text())
    tools_map = data.get("tools", {}) or {}
    tools = sorted(tools_map.items())
    images = sorted((data.get("images") or {}).items())
    actions_map = data.get("github_actions", {}) or {}
    actions = sorted(actions_map.items())
    lines = [
        "# Toolchain Reference",
        "",
        "- Owner: `bijux-atlas-operations`",
        "- Tier: `generated`",
        "- Audience: `operators`",
        "- Source-of-truth: `ops/inventory/toolchain.json`",
        "",
        "## Tools",
        "",
        "| Tool | Required | Probe Args |",
        "| --- | --- | --- |",
    ]
    for name, tool in tools:
        lines.append(f"| `{name}` | `{tool.get('required', False)}` | `{' '.join(tool.get('probe_argv', []))}` |")
    lines.extend(["", "## Images", "", "| Image Key | Reference |", "| --- | --- |"])
    for key, value in images:
        lines.append(f"| `{key}` | `{value}` |")
    lines.extend(["", "## GitHub Actions Pins", "", "| Action | Ref | SHA |", "| --- | --- | --- |"])
    for action_name, action in actions:
        lines.append(f"| `{action_name}` | `{action.get('ref','')}` | `{action.get('sha','')}` |")
    lines.append("")
    return "\n".join(lines)


def render_pins_reference() -> str:
    data = yaml.safe_load((REPO_ROOT / "ops" / "inventory" / "pins.yaml").read_text())
    rows: list[tuple[str, str, str]] = []
    for section, value in data.items():
        if isinstance(value, dict):
            for key in sorted(value):
                rows.append((section, key, str(value[key])))
        elif isinstance(value, list):
            for idx, item in enumerate(value):
                rows.append((section, str(idx), str(item)))
        else:
            rows.append(("root", section, str(value)))
    lines = [
        "# Pins Reference",
        "",
        "- Owner: `bijux-atlas-operations`",
        "- Tier: `generated`",
        "- Audience: `operators`",
        "- Source-of-truth: `ops/inventory/pins.yaml`",
        "",
        "## Pins",
        "",
        "| Section | Key | Value |",
        "| --- | --- | --- |",
    ]
    for section, key, value in rows:
        lines.append(f"| `{section}` | `{key}` | `{value}` |")
    lines.append("")
    return "\n".join(lines)


def render_gates_reference() -> str:
    data = json.loads((REPO_ROOT / "ops" / "inventory" / "gates.json").read_text())
    gates = sorted(data.get("gates", []), key=lambda g: g["id"])
    lines = [
        "# Gates Reference",
        "",
        "- Owner: `bijux-atlas-operations`",
        "- Tier: `generated`",
        "- Audience: `operators`",
        "- Source-of-truth: `ops/inventory/gates.json`",
        "",
        "## Gates",
        "",
        "| Gate ID | Category | Action ID | Description |",
        "| --- | --- | --- | --- |",
    ]
    for gate in gates:
        lines.append(
            f"| `{gate['id']}` | `{gate.get('category','')}` | `{gate.get('action_id','')}` | {gate.get('description','')} |"
        )
    lines.append("")
    return "\n".join(lines)


def render_drills_reference() -> str:
    data = json.loads((REPO_ROOT / "ops" / "inventory" / "drills.json").read_text())
    drills_raw = data.get("drills", [])
    drills = sorted(d["id"] if isinstance(d, dict) else str(d) for d in drills_raw)
    lines = [
        "# Drills Reference",
        "",
        "- Owner: `bijux-atlas-operations`",
        "- Tier: `generated`",
        "- Audience: `operators`",
        "- Source-of-truth: `ops/inventory/drills.json`",
        "",
        "## Drills",
        "",
    ]
    lines.extend([f"- `{drill}`" for drill in drills])
    lines.append("")
    return "\n".join(lines)


def render_schema_index_reference() -> str:
    return "\n".join(
        [
            "# Schema Index Reference",
            "",
            "- Owner: `bijux-atlas-operations`",
            "- Tier: `generated`",
            "- Audience: `operators`",
            "- Source-of-truth: `ops/schema/generated/schema-index.md`",
            "",
            "## Canonical Source",
            "",
            "- `ops/schema/generated/schema-index.md` is the authoritative generated schema index.",
            "- This page is a docs-site reference pointer to avoid duplicating the schema table.",
            "",
        ]
    )


def render_evidence_model_reference() -> str:
    levels = json.loads((REPO_ROOT / "ops" / "schema" / "report" / "evidence-levels.schema.json").read_text())
    bundle = json.loads((REPO_ROOT / "ops" / "schema" / "report" / "release-evidence-bundle.schema.json").read_text())
    title = levels.get("title", "")
    _ = bundle  # used as existence/parse validation input
    return "\n".join(
        [
            "# Evidence Model Reference",
            "",
            "- Owner: `bijux-atlas-operations`",
            "- Tier: `generated`",
            "- Audience: `operators`",
            "- Source-of-truth: `ops/schema/report/evidence-levels.schema.json`, `ops/schema/report/release-evidence-bundle.schema.json`",
            "",
            "## Canonical Schemas",
            "",
            "- `ops/schema/report/evidence-levels.schema.json`",
            "- `ops/schema/report/release-evidence-bundle.schema.json`",
            "",
            "## Notes",
            "",
            f"- evidence-levels schema title: `{title}`",
            "",
        ]
    )


def render_what_breaks_reference() -> str:
    data = json.loads((REPO_ROOT / "ops" / "_generated.example" / "what-breaks-if-removed-report.json").read_text())
    targets = data.get("targets", [])
    lines = [
        "# What Breaks If Removed Reference",
        "",
        "- Owner: `bijux-atlas-operations`",
        "- Tier: `generated`",
        "- Audience: `operators`",
        "- Source-of-truth: `ops/_generated.example/what-breaks-if-removed-report.json`",
        "",
        "## Removal Impact Targets",
        "",
        "| Path | Impact | Consumers |",
        "| --- | --- | --- |",
    ]
    for t in targets:
        consumers = ", ".join(t.get("consumers", []))
        lines.append(f"| `{t.get('path','')}` | `{t.get('impact','')}` | `{consumers}` |")
    lines.append("")
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
        DOCS_REF_DIR / "tools.md": render_tools_reference(),
        DOCS_REF_DIR / "toolchain.md": render_toolchain_reference(),
        DOCS_REF_DIR / "pins.md": render_pins_reference(),
        DOCS_REF_DIR / "gates.md": render_gates_reference(),
        DOCS_REF_DIR / "drills.md": render_drills_reference(),
        DOCS_REF_DIR / "schema-index.md": render_schema_index_reference(),
        DOCS_REF_DIR / "evidence-model.md": render_evidence_model_reference(),
        DOCS_REF_DIR / "what-breaks-if-removed.md": render_what_breaks_reference(),
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
