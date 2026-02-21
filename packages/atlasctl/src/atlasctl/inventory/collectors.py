from __future__ import annotations

import json
from pathlib import Path


def repo_files(repo_root: Path, pattern: str) -> list[str]:
    return sorted(str(path.relative_to(repo_root)) for path in repo_root.glob(pattern) if path.is_file())


def collect_make(repo_root: Path) -> dict[str, object]:
    data = json.loads((repo_root / "configs/make/public-targets.json").read_text(encoding="utf-8"))
    targets = data.get("public_targets", [])
    return {
        "kind": "make",
        "targets": [
            {
                "name": target.get("name"),
                "description": target.get("description", ""),
                "area": target.get("area", ""),
                "lanes": target.get("lanes", []),
            }
            for target in targets
            if isinstance(target, dict)
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
    files: list[str] = []
    for path in sorted((repo_root / "configs").rglob("*")):
        if path.is_file() and path.suffix in {".json", ".yaml", ".yml", ".toml", ".md", ".txt"}:
            files.append(str(path.relative_to(repo_root)))
    return {"kind": "configs", "files": files}


def collect_schemas(repo_root: Path) -> dict[str, object]:
    schemas = repo_files(repo_root, "configs/_schemas/**/*.json") + repo_files(repo_root, "ops/_schemas/**/*.json")
    return {"kind": "schemas", "files": sorted(set(schemas))}


def collect_owners(repo_root: Path) -> dict[str, object]:
    owners: dict[str, dict[str, object]] = {}
    for rel in ("configs/meta/ownership.json", "configs/_meta/ownership.json", "makefiles/ownership.json"):
        path = repo_root / rel
        if not path.exists():
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        if isinstance(payload, dict):
            owners[rel] = payload
    return {"kind": "owners", "sources": owners}


def collect_contracts(repo_root: Path) -> dict[str, object]:
    contracts = sorted(str(path.relative_to(repo_root)) for path in repo_root.rglob("CONTRACT.md") if path.is_file())
    schemas = sorted(set(repo_files(repo_root, "configs/contracts/*.schema.json") + repo_files(repo_root, "ops/_schemas/**/*.schema.json")))
    return {"kind": "contracts", "contract_files": contracts, "schema_files": schemas}


def collect_budgets(repo_root: Path) -> dict[str, object]:
    make_targets = collect_make(repo_root)["targets"]
    scripts_commands = [path.name for path in (repo_root / "scripts/bin").glob("*") if path.is_file()]
    ops_areas = [path.name for path in (repo_root / "ops").iterdir() if path.is_dir() and not path.name.startswith("_")]
    return {
        "kind": "budgets",
        "counts": {
            "public_make_targets": len(make_targets),
            "scripts_commands": len(scripts_commands),
            "ops_areas": len(ops_areas),
        },
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
    for path in sorted(script_root.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(repo_root).as_posix()
        if "__pycache__/" in rel:
            continue
        if rel.startswith("scripts/bin/"):
            name = path.name
            mapped = name
            if name.startswith("bijux-atlas-"):
                mapped = f"atlasctl {name.removeprefix('bijux-atlas-').replace('-', ' ')}"
            elif name == "run_drill.sh":
                mapped = "atlasctl ops drill"
            elif name in {"make_explain", "make_graph", "render_public_help"}:
                mapped = f"atlasctl make {name.replace('make_', '').replace('render_public_help', 'help')}"
            elif name in {"isolate", "require-isolate"}:
                mapped = "atlasctl gates run"
            bin_commands.append({"script": rel, "command": name, "mapped_cli": mapped})
        if rel.startswith("scripts/areas/") and rel.endswith("/INDEX.md"):
            area = rel.split("/")[2]
            mapped = f"atlasctl check {area}"
            if area in {"internal", "tools", "python"}:
                mapped = "atlasctl run <module>"
            if area in {"ops", "k8s", "stack", "e2e", "obs", "load", "datasets"}:
                mapped = f"atlasctl ops {area}"
            area_indexes.append({"index": rel, "area": area, "mapped_cli_group": mapped})
        lower = rel.lower()
        tags = [tag for tag, needles in categories.items() if any(needle in lower for needle in needles)]
        entries.append(
            {
                "path": rel,
                "extension": path.suffix,
                "is_executable": bool(path.stat().st_mode & 0o111),
                "tags": sorted(set(tags)),
            }
        )
    return {
        "kind": "scripts-migration",
        "entries": entries,
        "bin_commands": sorted(bin_commands, key=lambda item: item["script"]),
        "area_indexes": sorted(area_indexes, key=lambda item: item["index"]),
    }


def collect_legacy_scripts(repo_root: Path) -> dict[str, object]:
    root = repo_root / "scripts"
    files = []
    if root.exists():
        files = sorted(path.relative_to(repo_root).as_posix() for path in root.rglob("*") if path.is_file() and "__pycache__" not in path.parts)
    return {"kind": "legacy-scripts", "files": files, "count": len(files)}


def collect_commands(_repo_root: Path) -> dict[str, object]:
    from ..cli.registry import registry as command_registry

    commands = [{"name": c.name, "help": c.help_text, "stable": bool(c.stable)} for c in sorted(command_registry(), key=lambda c: c.name)]
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
    return {"kind": "touched-paths", "command": command, "paths": sorted(TOUCHED_PATHS.get(command, []))}
