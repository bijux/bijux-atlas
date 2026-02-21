def _spellcheck(ctx: RunContext, path_arg: str) -> tuple[int, str]:
    exe = shutil.which("codespell")
    if not exe:
        return 2, "codespell not found in PATH"
    root = ctx.repo_root / path_arg
    targets = [root / "index.md", root / "_style"]
    cmd = [exe, "--quiet-level", "2", "--skip", "*.json,*.png,*.jpg,*.svg"]
    ignore_words = ctx.repo_root / "configs/docs/codespell-ignore-words.txt"
    if ignore_words.exists():
        cmd.extend(["--ignore-words", str(ignore_words)])
    for target in targets:
        if target.exists():
            cmd.append(str(target))
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        return proc.returncode, (proc.stdout + proc.stderr).strip() or "spellcheck failed"
    return 0, "spellcheck passed"
def _generate_architecture_map(ctx: RunContext) -> tuple[int, str]:
    category_hints = {
        "bijux-atlas-api": "api-surface",
        "bijux-atlas-server": "runtime-server",
        "bijux-atlas-query": "query-engine",
        "bijux-atlas-store": "artifact-store",
        "bijux-atlas-ingest": "ingest-pipeline",
        "bijux-atlas-cli": "cli-ops",
        "bijux-atlas-model": "shared-model",
        "bijux-atlas-core": "shared-core",
        "bijux-atlas-policies": "policy-contracts",
    }
    code, out = _run_check(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
        ctx.repo_root,
    )
    if code != 0:
        return 1, out
    meta = json.loads(out)
    packages = {
        p.get("name"): p
        for p in meta.get("packages", [])
        if isinstance(p, dict) and isinstance(p.get("name"), str) and p["name"].startswith("bijux-atlas-")
    }
    names = sorted(packages.keys())
    lines = [
        "# Architecture Map",
        "",
        "- Owner: `atlas-platform`",
        "- Stability: `stable`",
        "",
        "Generated crate-level architecture map from workspace metadata.",
        "",
        "## Crate Nodes",
        "",
        "| Crate | Role | Internal Dependencies |",
        "| --- | --- | --- |",
    ]
    for name in names:
        pkg = packages[name]
        deps = sorted(
            d.get("name")
            for d in pkg.get("dependencies", [])
            if isinstance(d, dict) and isinstance(d.get("name"), str) and d["name"].startswith("bijux-atlas-")
        )
        dep_str = ", ".join(f"`{d}`" for d in deps) if deps else "`(none)`"
        role = category_hints.get(name, "unspecified")
        lines.append(f"| `{name}` | `{role}` | {dep_str} |")
    lines += [
        "",
        "## Runtime Direction",
        "",
        "`bijux-atlas-server -> bijux-atlas-query -> bijux-atlas-store -> immutable artifacts`",
        "",
        "## Notes",
        "",
        "- This file is generated; do not hand-edit.",
        "- Regenerate via `atlasctl docs generate-architecture-map`.",
        "",
    ]
    out_path = ctx.repo_root / "docs/architecture/architecture-map.md"
    out_path.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out_path.relative_to(ctx.repo_root)}"
def _generate_crates_map(ctx: RunContext) -> tuple[int, str]:
    cargo = ctx.repo_root / "Cargo.toml"
    out = ctx.repo_root / "docs" / "development" / "crates-map.md"
    purpose_hints = {
        "bijux-atlas-core": "deterministic primitives, canonicalization, error types",
        "bijux-atlas-model": "domain and artifact data types",
        "bijux-atlas-policies": "runtime policy schema and validation",
        "bijux-atlas-store": "artifact backends and integrity boundaries",
        "bijux-atlas-ingest": "deterministic ingest pipeline to artifacts",
        "bijux-atlas-query": "query planning, limits, and pagination",
        "bijux-atlas-api": "wire contracts and request/response schemas",
        "bijux-atlas-server": "runtime orchestration and effectful serving",
        "bijux-atlas-cli": "plugin CLI and operational commands",
    }
    text = cargo.read_text(encoding="utf-8")
    members_match = re.search(r"members\s*=\s*\[(.*?)\]", text, re.S)
    if not members_match:
        return 1, "workspace members not found in Cargo.toml"
    crates = sorted({Path(item).name for item in re.findall(r'"([^"]+)"', members_match.group(1)) if item.startswith("crates/")})
    lines = [
        "# Crates Map",
        "",
        "- Owner: `docs-governance`",
        "",
        "## What",
        "",
        "Generated map of workspace crates and primary purpose.",
        "",
        "## Why",
        "",
        "Provides a stable navigation index for crate responsibilities.",
        "",
        "## Scope",
        "",
        "Covers workspace crates from `Cargo.toml` members under `crates/`.",
        "",
        "## Non-goals",
        "",
        "Does not replace crate-level architecture and API docs.",
        "",
        "## Contracts",
    ]
    for crate in crates:
        lines.append(f"- `{crate}`: {purpose_hints.get(crate, 'crate responsibility documented in crate docs')}.")
    lines.extend(
        [
            "",
            "## Failure modes",
            "",
            "Stale maps can hide ownership drift and boundary violations.",
            "",
            "## How to verify",
            "",
            "```bash",
            "$ atlasctl docs generate-crates-map",
            " docs crate-docs-contract-check",
            "```",
            "",
            "Expected output: crates map is regenerated and crate docs contract passes.",
            "",
            "## See also",
            "",
            "- [Crate Layout Contract](../architecture/crate-layout-contract.md)",
            "- [Crate Boundary Graph](../architecture/crate-boundary-dependency-graph.md)",
            "- [Terms Glossary](../_style/terms-glossary.md)",
            "",
        ]
    )
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out.relative_to(ctx.repo_root)}"
def _generate_make_targets_catalog(ctx: RunContext) -> tuple[int, str]:
    ssot = json.loads((ctx.repo_root / "configs" / "make" / "public-targets.json").read_text(encoding="utf-8"))
    owners = json.loads((ctx.repo_root / "makefiles" / "ownership.json").read_text(encoding="utf-8"))
    out_json = ctx.repo_root / "makefiles" / "targets.json"
    out_md = ctx.repo_root / "docs" / "_generated" / "make-targets.md"
    out_catalog = ctx.repo_root / "artifacts" / "generated" / "make" / "targets.catalog.json"
    entries: list[dict[str, object]] = []
    for item in sorted(ssot.get("public_targets", []), key=lambda entry: entry["name"]):
        name = item["name"]
        meta = owners.get(name, {})
        lanes = item.get("lanes", [])
        entries.append(
            {
                "name": name,
                "description": item.get("description", ""),
                "owner": meta.get("owner", ""),
                "area": item.get("area", meta.get("area", "")),
                "lane": lanes[0] if lanes else "",
                "lanes": lanes,
            }
        )
    payload = {
        "schema_version": 1,
        "source": {"ssot": "configs/make/public-targets.json", "ownership": "makefiles/ownership.json"},
        "targets": entries,
    }
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    out_catalog.parent.mkdir(parents=True, exist_ok=True)
    out_catalog.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    lines = [
        "# Make Targets",
        "",
        "Generated from `makefiles/targets.json`. Do not edit manually.",
        "",
        "| target | description | owner | area | lane |",
        "|---|---|---|---|---|",
    ]
    for target in entries:
        desc = str(target["description"]).replace("|", "/")
        lines.append(
            f"| `{target['name']}` | {desc} | `{target['owner']}` | `{target['area']}` | `{target['lane']}` |"
        )
    out_md.parent.mkdir(parents=True, exist_ok=True)
    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, (
        f"{out_json.relative_to(ctx.repo_root)}\n"
        f"{out_md.relative_to(ctx.repo_root)}\n"
        f"{out_catalog.relative_to(ctx.repo_root)}"
    )
def _generate_env_vars_doc(ctx: RunContext) -> tuple[int, str]:
    contract = ctx.repo_root / "configs" / "contracts" / "env.schema.json"
    out = ctx.repo_root / "docs" / "_generated" / "env-vars.md"
    payload = json.loads(contract.read_text(encoding="utf-8"))
    keys = sorted(payload.get("allowed_env", []))
    lines = [
        "# Env Vars (Generated)",
        "",
        "Generated from `configs/contracts/env.schema.json`. Do not edit manually.",
        "",
        f"- Count: `{len(keys)}`",
        f"- Enforced prefixes: `{', '.join(payload.get('enforced_prefixes', []))}`",
        f"- Dev escape hatch: `{payload.get('dev_mode_allow_unknown_env', '')}`",
        "",
        "## Allowed Env Vars",
        "",
    ]
    lines.extend(f"- `{key}`" for key in keys)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))
def _generate_config_keys_doc(ctx: RunContext) -> tuple[int, str]:
    registry = ctx.repo_root / "configs" / "config-key-registry.md"
    out = ctx.repo_root / "docs" / "_generated" / "config-keys.md"
    keys: list[str] = []
    for line in registry.read_text(encoding="utf-8").splitlines():
        item = line.strip()
        if item.startswith("- `") and item.endswith("`"):
            keys.append(item[3:-1])
    lines = [
        "# Config Keys (Generated)",
        "",
        "Generated from `configs/config-key-registry.md`. Do not edit manually.",
        "",
        f"- Count: `{len(keys)}`",
        "",
        "## Keys",
        "",
    ]
    lines.extend(f"- `{key}`" for key in keys)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))
def _generate_layer_contract_doc(ctx: RunContext) -> tuple[int, str]:
    inp = ctx.repo_root / "ops" / "_meta" / "layer-contract.json"
    out = ctx.repo_root / "docs" / "_generated" / "layer-contract.md"
    contract = json.loads(inp.read_text(encoding="utf-8"))
    lines = [
        "# Layer Contract",
        "",
        f"- Contract version: `{contract['contract_version']}`",
        f"- Compatibility policy: {contract['compatibility']['policy']}",
        "",
        "## Namespaces",
    ]
    for key, val in contract["namespaces"].items():
        lines.append(f"- `{key}`: `{val}`")
    lines.extend(["", "## Services"])
    for key, val in contract["services"].items():
        lines.append(f"- `{key}`: service `{val['service_name']}`, selector `{json.dumps(val['selector'], sort_keys=True)}`")
    lines.extend(["", "## Ports"])
    for key, val in contract["ports"].items():
        lines.append(f"- `{key}`: `{json.dumps(val, sort_keys=True)}`")
    lines.extend(["", "## Labels", "- Required labels:"])
    for item in contract["labels"]["required"]:
        lines.append(f"- `{item}`")
    lines.extend(["", "## Release Metadata"])
    lines.append(f"- Required fields: `{', '.join(contract['release_metadata']['required_fields'])}`")
    lines.append(f"- Defaults: `{json.dumps(contract['release_metadata']['defaults'], sort_keys=True)}`")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"
