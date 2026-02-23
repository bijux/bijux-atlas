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
    schemas = repo_files(repo_root, "configs/schema/**/*.json") + repo_files(repo_root, "ops/schema/**/*.json")
    return {"kind": "schemas", "files": sorted(set(schemas))}


def collect_owners(repo_root: Path) -> dict[str, object]:
    owners: dict[str, dict[str, object]] = {}
    for rel in ("configs/meta/ownership.json", "configs/inventory/owners.json", "configs/make/ownership.json"):
        path = repo_root / rel
        if not path.exists():
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        if isinstance(payload, dict):
            owners[rel] = payload
    return {"kind": "owners", "sources": owners}


def collect_contracts(repo_root: Path) -> dict[str, object]:
    contracts = sorted(str(path.relative_to(repo_root)) for path in repo_root.rglob("CONTRACT.md") if path.is_file())
    schemas = sorted(set(repo_files(repo_root, "configs/contracts/*.schema.json") + repo_files(repo_root, "ops/schema/**/*.schema.json")))
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


def collect_commands(_repo_root: Path) -> dict[str, object]:
    from ....cli.surface_registry import registry as command_registry

    commands = [{"name": c.name, "help": c.help_text, "stable": bool(c.stable)} for c in sorted(command_registry(), key=lambda c: c.name)]
    return {"kind": "commands", "commands": commands, "count": len(commands)}


TOUCHED_PATHS: dict[str, list[str]] = {
    "check": ["makefiles/", "configs/", ".github/workflows/"],
    "docs": ["docs/", "mkdocs.yml", "docs/_generated/"],
    "configs": ["configs/", "docs/_generated/config*"],
    "ops": ["ops/", "artifacts/evidence/"],
    "make": ["makefiles/", "docs/development/make-targets.md"],
    "report": ["artifacts/evidence/", "ops/_generated.example/"],
    "gates": ["configs/gates/lanes.json", "artifacts/evidence/"],
}


def collect_touched_paths(command: str) -> dict[str, object]:
    return {"kind": "touched-paths", "command": command, "paths": sorted(TOUCHED_PATHS.get(command, []))}
