from __future__ import annotations

import argparse
import json
from pathlib import Path

from ..core.context import RunContext

DEFAULT_OUT_DIR = Path("docs/_generated")


def _repo_files(repo_root: Path, pattern: str) -> list[str]:
    return sorted(str(p.relative_to(repo_root)) for p in repo_root.glob(pattern) if p.is_file())


def collect_make(repo_root: Path) -> dict[str, object]:
    data = json.loads((repo_root / "configs/make/public-targets.json").read_text(encoding="utf-8"))
    targets = data.get("public_targets", [])
    return {
        "kind": "make",
        "targets": [
            {
                "name": t.get("name"),
                "description": t.get("description", ""),
                "area": t.get("area", ""),
                "lanes": t.get("lanes", []),
            }
            for t in targets
            if isinstance(t, dict)
        ],
    }


def collect_ops(repo_root: Path) -> dict[str, object]:
    data = json.loads((repo_root / "configs/ops/public-surface.json").read_text(encoding="utf-8"))
    return {
        "kind": "ops",
        "make_targets": sorted(data.get("make_targets", [])),
        "ops_run_commands": sorted(data.get("ops_run_commands", [])),
        "core_targets": sorted(data.get("core_targets", [])),
    }


def collect_configs(repo_root: Path) -> dict[str, object]:
    files = []
    for p in sorted((repo_root / "configs").rglob("*")):
        if p.is_file() and p.suffix in {".json", ".yaml", ".yml", ".toml", ".md", ".txt"}:
            files.append(str(p.relative_to(repo_root)))
    return {"kind": "configs", "files": files}


def collect_schemas(repo_root: Path) -> dict[str, object]:
    schemas = _repo_files(repo_root, "configs/_schemas/**/*.json") + _repo_files(repo_root, "ops/_schemas/**/*.json")
    return {"kind": "schemas", "files": sorted(set(schemas))}


def collect_owners(repo_root: Path) -> dict[str, object]:
    owners: dict[str, dict[str, object]] = {}
    for rel in ("configs/meta/ownership.json", "configs/_meta/ownership.json", "makefiles/ownership.json"):
        p = repo_root / rel
        if not p.exists():
            continue
        payload = json.loads(p.read_text(encoding="utf-8"))
        if isinstance(payload, dict):
            owners[rel] = payload
    return {"kind": "owners", "sources": owners}


def collect_contracts(repo_root: Path) -> dict[str, object]:
    contracts = sorted(str(p.relative_to(repo_root)) for p in repo_root.rglob("CONTRACT.md") if p.is_file())
    schemas = sorted(
        set(
            _repo_files(repo_root, "configs/contracts/*.schema.json")
            + _repo_files(repo_root, "ops/_schemas/**/*.schema.json")
        )
    )
    return {"kind": "contracts", "contract_files": contracts, "schema_files": schemas}


def collect_budgets(repo_root: Path) -> dict[str, object]:
    make_targets = collect_make(repo_root)["targets"]
    scripts_commands = [p.name for p in (repo_root / "scripts/bin").glob("*") if p.is_file()]
    ops_areas = [p.name for p in (repo_root / "ops").iterdir() if p.is_dir() and not p.name.startswith("_")]
    counts = {
        "public_make_targets": len(make_targets),
        "scripts_commands": len(scripts_commands),
        "ops_areas": len(ops_areas),
    }
    return {
        "kind": "budgets",
        "counts": counts,
        "scripts_commands": sorted(scripts_commands),
        "ops_areas": sorted(ops_areas),
    }


def collect_scripts_migration(repo_root: Path) -> dict[str, object]:
    script_root = repo_root / "scripts"
    entries: list[dict[str, object]] = []
    bin_commands: list[dict[str, str]] = []
    area_indexes: list[dict[str, str]] = []
    categories = {
        "library_helper": ("lib/", "areas/python/", "areas/tools/", "areas/internal/"),
        "report_emitter": ("report", "score", "summary", "bundle", "evidence", "artifact"),
        "gate_runner": ("check_", "lint", "validate", "drift", "contract"),
        "ops_orchestrator": ("stack", "k8s", "ops", "smoke", "drill", "up.sh", "down.sh"),
        "docs_generator": ("generate_", "docs", "mkdocs", "nav", "index"),
        "config_validator": ("config", "schema", "env", "versions"),
        "policy_checker": ("policy", "relaxation", "allow", "bypass"),
        "make_integration": ("make", "target", "help", "graph", "catalog"),
    }
    for p in sorted(script_root.rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(repo_root).as_posix()
        if "__pycache__/" in rel:
            continue
        if rel.startswith("scripts/bin/"):
            name = p.name
            mapped = name
            if name.startswith("bijux-atlas-"):
                mapped = f"atlasctl {name.removeprefix('bijux-atlas-').replace('-', ' ')}"
            elif name == "run_drill.sh":
                mapped = "atlasctl ops drill"
            elif name in {"make_explain", "make_graph", "render_public_help"}:
                mapped = f"atlasctl make {name.replace('make_', '').replace('render_public_help', 'help')}"
            elif name in {"isolate", "require-isolate"}:
                mapped = "atlasctl gates run"
            bin_commands.append(
                {
                    "script": rel,
                    "command": name,
                    "mapped_cli": mapped,
                }
            )
        if rel.startswith("scripts/areas/") and rel.endswith("/INDEX.md"):
            area = rel.split("/")[2]
            mapped = f"atlasctl check {area}"
            if area in {"internal", "tools", "python"}:
                mapped = "atlasctl run <module>"
            if area in {"ops", "k8s", "stack", "e2e", "obs", "load", "datasets"}:
                mapped = f"atlasctl ops {area}"
            area_indexes.append(
                {
                    "index": rel,
                    "area": area,
                    "mapped_cli_group": mapped,
                }
            )
        ext = p.suffix
        is_executable = bool(p.stat().st_mode & 0o111)
        lower = rel.lower()
        tags: list[str] = []
        for tag, needles in categories.items():
            if any(needle in lower for needle in needles):
                tags.append(tag)
        entries.append(
            {
                "path": rel,
                "extension": ext,
                "is_executable": is_executable,
                "tags": sorted(set(tags)),
            }
        )
    return {
        "kind": "scripts-migration",
        "entries": entries,
        "bin_commands": sorted(bin_commands, key=lambda x: x["script"]),
        "area_indexes": sorted(area_indexes, key=lambda x: x["index"]),
    }


def collect_legacy_scripts(repo_root: Path) -> dict[str, object]:
    root = repo_root / "scripts"
    files: list[str] = []
    if root.exists():
        files = sorted(
            p.relative_to(repo_root).as_posix()
            for p in root.rglob("*")
            if p.is_file() and "__pycache__" not in p.parts
        )
    return {"kind": "legacy-scripts", "files": files, "count": len(files)}


def collect_commands(_repo_root: Path) -> dict[str, object]:
    from ..domain_cmd import registry as command_registry

    commands = [
        {
            "name": c.name,
            "help": c.help_text,
            "stable": bool(c.stable),
        }
        for c in sorted(command_registry(), key=lambda item: item.name)
    ]
    return {"kind": "commands", "commands": commands, "count": len(commands)}


TOUCHED_PATHS: dict[str, list[str]] = {
    "check": ["makefiles/", "configs/", ".github/workflows/"],
    "docs": ["docs/", "mkdocs.yml", "docs/_generated/"],
    "configs": ["configs/", "docs/_generated/config*"],
    "ops": ["ops/", "artifacts/evidence/"],
    "make": ["makefiles/", "docs/development/make-targets.md"],
    "report": ["artifacts/evidence/", "ops/_generated_committed/"],
    "gates": ["configs/gates/lanes.json", "artifacts/evidence/"],
}


def collect_touched_paths(command: str) -> dict[str, object]:
    return {
        "kind": "touched-paths",
        "command": command,
        "paths": sorted(TOUCHED_PATHS.get(command, [])),
    }


def render_md(payload: dict[str, object]) -> str:
    kind = str(payload["kind"])
    lines = [f"# {kind.title()} Inventory", "", "Generated by `bijux-atlas inventory`.", ""]
    if kind == "make":
        lines += ["| Target | Area | Description |", "|---|---|---|"]
        for row in payload.get("targets", []):
            lines.append(f"| `{row['name']}` | `{row['area']}` | {row['description']} |")
    elif kind == "ops":
        lines.append("## Make Targets")
        lines.append("")
        for t in payload.get("make_targets", []):
            lines.append(f"- `{t}`")
        lines.append("")
        lines.append("## Run Commands")
        lines.append("")
        for t in payload.get("ops_run_commands", []):
            lines.append(f"- `{t}`")
    elif kind in {"configs", "schemas"}:
        for f in payload.get("files", []):
            lines.append(f"- `{f}`")
    elif kind == "owners":
        for src, data in payload.get("sources", {}).items():
            lines.append(f"## `{src}`")
            lines.append("")
            lines.append(f"- entries: `{len(data)}`")
            lines.append("")
    elif kind == "contracts":
        lines.append("## CONTRACT.md")
        lines.append("")
        for f in payload.get("contract_files", []):
            lines.append(f"- `{f}`")
        lines.append("")
        lines.append("## Schemas")
        lines.append("")
        for f in payload.get("schema_files", []):
            lines.append(f"- `{f}`")
    elif kind == "budgets":
        counts = payload.get("counts", {})
        for k, v in counts.items():
            lines.append(f"- `{k}`: `{v}`")
    elif kind == "scripts-migration":
        lines.append("## scripts/bin command mapping")
        lines.append("")
        lines += ["| Script | Legacy Command | Mapped CLI |", "|---|---|---|"]
        for row in payload.get("bin_commands", []):
            lines.append(f"| `{row['script']}` | `{row['command']}` | `{row['mapped_cli']}` |")
        lines.append("")
        lines.append("## scripts/areas index mapping")
        lines.append("")
        lines += ["| Area Index | Area | Mapped CLI Group |", "|---|---|---|"]
        for row in payload.get("area_indexes", []):
            lines.append(f"| `{row['index']}` | `{row['area']}` | `{row['mapped_cli_group']}` |")
        lines.append("")
        lines.append("## scripts tree inventory")
        lines.append("")
        lines += ["| Path | Ext | Executable | Tags |", "|---|---|---|---|"]
        for row in payload.get("entries", []):
            tags = ", ".join(row.get("tags", []))
            lines.append(f"| `{row['path']}` | `{row['extension']}` | `{row['is_executable']}` | {tags} |")
    elif kind == "legacy-scripts":
        lines.append(f"- total: `{payload.get('count', 0)}`")
        lines.append("")
        for row in payload.get("files", []):
            lines.append(f"- `{row}`")
    elif kind == "commands":
        lines.append(f"- total: `{payload.get('count', 0)}`")
        lines.append("")
        lines += ["| Command | Stable | Help |", "|---|---|---|"]
        for row in payload.get("commands", []):
            lines.append(f"| `{row['name']}` | `{row['stable']}` | {row['help']} |")
    elif kind == "touched-paths":
        lines.append(f"- command: `{payload.get('command', '')}`")
        lines.append("")
        for row in payload.get("paths", []):
            lines.append(f"- `{row}`")
    return "\n".join(lines) + "\n"


def outputs_for(kind: str) -> tuple[Path, Path]:
    mapping = {
        "make": (Path("make-targets.json"), Path("make-targets.md")),
        "ops": (Path("ops-surface.json"), Path("ops-surface.md")),
        "configs": (Path("configs-surface.json"), Path("configs-surface.md")),
        "schemas": (Path("schema-index.json"), Path("schema-index.md")),
        "owners": (Path("ownership.json"), Path("ownership.md")),
        "contracts": (Path("contracts-index.json"), Path("contracts-index.md")),
        "budgets": (Path("inventory-budgets.json"), Path("inventory-budgets.md")),
        "scripts-migration": (Path("scripts-migration.json"), Path("scripts-migration.md")),
        "legacy-scripts": (Path("legacy-scripts.json"), Path("legacy-scripts.md")),
        "commands": (Path("commands.json"), Path("commands.md")),
        "touched-paths": (Path("touched-paths.json"), Path("touched-paths.md")),
    }
    return mapping[kind]


def validate_json_schema(repo_root: Path, kind: str, payload: dict[str, object]) -> None:
    import jsonschema

    schema = json.loads((repo_root / f"configs/contracts/inventory-{kind}.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)


def _budget_check(repo_root: Path, payload: dict[str, object]) -> list[str]:
    budget_cfg = repo_root / "configs/layout/inventory-budgets.json"
    data = json.loads(budget_cfg.read_text(encoding="utf-8")) if budget_cfg.exists() else {"max": {}}
    maxes = data.get("max", {})
    counts = payload.get("counts", {})
    errs: list[str] = []
    for key, max_v in maxes.items():
        cur = int(counts.get(key, 0))
        if cur > int(max_v):
            errs.append(f"{key} budget exceeded: {cur} > {max_v}")
    return errs


def _emit(repo_root: Path, out_dir: Path, kind: str, payload: dict[str, object], fmt: str, dry_run: bool) -> None:
    json_name, md_name = outputs_for(kind)
    validate_json_schema(repo_root, kind, payload)
    if fmt in {"json", "both"}:
        if dry_run:
            print(json.dumps(payload, sort_keys=True))
        else:
            (out_dir / json_name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if fmt in {"md", "both"}:
        text = render_md(payload)
        if dry_run:
            print(text, end="")
        else:
            (out_dir / md_name).write_text(text, encoding="utf-8")


def run_inventory(
    ctx: RunContext,
    category: str,
    fmt: str,
    out_dir: str | None,
    dry_run: bool,
    check: bool,
    command: str | None = None,
) -> int:
    repo_root = ctx.repo_root
    out = (repo_root / (out_dir or str(DEFAULT_OUT_DIR))).resolve()
    if not dry_run:
        out.mkdir(parents=True, exist_ok=True)

    collectors = {
        "make": collect_make,
        "ops": collect_ops,
        "configs": collect_configs,
        "schemas": collect_schemas,
        "owners": collect_owners,
        "contracts": collect_contracts,
        "budgets": collect_budgets,
        "scripts-migration": collect_scripts_migration,
        "legacy-scripts": collect_legacy_scripts,
        "commands": collect_commands,
    }

    if category == "touched-paths":
        if not command:
            print("inventory touched-paths requires --command")
            return 2
        payload = collect_touched_paths(command)
        _emit(repo_root, out, "touched-paths", payload, fmt, dry_run)
        return 0

    categories = list(collectors.keys()) if category == "all" else [category]
    for kind in categories:
        payload = collectors[kind](repo_root)
        _emit(repo_root, out, kind, payload, fmt, dry_run)
        if kind == "budgets" and check:
            errs = _budget_check(repo_root, payload)
            if errs:
                for e in errs:
                    print(e)
                return 1

    if not dry_run and category == "all" and fmt in {"md", "both"}:
        index = out / "INDEX.md"
        index.write_text(
            "\n".join(
                [
                    "# Generated Docs Index",
                    "",
                    "Generated by `make inventory` / `bijux-atlas inventory all`.",
                    "",
                    "## Files",
                    "- `make-targets.md`",
                    "- `ops-surface.md`",
                    "- `configs-surface.md`",
                    "- `schema-index.md`",
                    "- `ownership.md`",
                    "- `contracts-index.md`",
                    "- `inventory-budgets.md`",
                    "",
                    "## Update",
                    "- `make inventory`",
                    "- `make verify-inventory`",
                ]
            )
            + "\n",
            encoding="utf-8",
        )

    return 0


def configure_inventory_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("inventory", help="generate inventory docs and JSON from SSOT")
    p.add_argument(
        "category",
        choices=[
            "all",
            "make",
            "ops",
            "configs",
            "schemas",
            "owners",
            "contracts",
            "budgets",
            "scripts-migration",
            "legacy-scripts",
            "commands",
            "touched-paths",
        ],
    )
    p.add_argument("--format", choices=["json", "md", "both"], default="md")
    p.add_argument("--out-dir", default=str(DEFAULT_OUT_DIR))
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--check", action="store_true", help="enforce budgets for budgets category")
    p.add_argument("--command", help="command id for touched-paths category")
