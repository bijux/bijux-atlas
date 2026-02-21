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
    return 0, f"{out_json.relative_to(ctx.repo_root)}\n{out_md.relative_to(ctx.repo_root)}"
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
def _generate_makefiles_surface(ctx: RunContext) -> tuple[int, str]:
    surface = json.loads((ctx.repo_root / "configs" / "ops" / "public-surface.json").read_text(encoding="utf-8"))
    out = ctx.repo_root / "docs" / "development" / "makefiles" / "surface.md"
    lines = [
        "# Makefiles Public Surface",
        "",
        "Generated from `configs/ops/public-surface.json`. Do not edit manually.",
        "",
        "## Core Gates",
    ]
    lines.extend(f"- `make {target}`" for target in surface.get("core_targets", []))
    lines.extend(["", "## Public Targets"])
    lines.extend(f"- `make {target}`" for target in surface.get("make_targets", []))
    lines.extend(["", "## Public Ops Run Commands"])
    lines.extend(f"- `./{cmd}`" for cmd in surface.get("ops_run_commands", []))
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))
def _generate_observability_surface(ctx: RunContext) -> tuple[int, str]:
    out = ctx.repo_root / "docs" / "_generated" / "observability-surface.md"
    rendered = _render_observability_surface(ctx)
    out.write_text(rendered + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"
def _generate_ops_contracts_doc(ctx: RunContext) -> tuple[int, str]:
    out = ctx.repo_root / "docs" / "_generated" / "ops-contracts.md"
    contracts = json.loads((ctx.repo_root / "ops" / "_meta" / "contracts.json").read_text(encoding="utf-8"))
    schemas = sorted((ctx.repo_root / "ops" / "_schemas").rglob("*.json"))
    lines = ["# Ops Contracts", "", "Generated from ops contracts and schemas.", "", "## Contract Files", ""]
    for item in contracts.get("contracts", []):
        lines.append(f"- `{item['path']}` (version `{item['version']}`)")
    lines.extend(["", "## Schemas", ""])
    lines.extend(f"- `{schema.relative_to(ctx.repo_root).as_posix()}`" for schema in schemas)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))
def _generate_ops_schema_docs(ctx: RunContext) -> tuple[int, str]:
    schemas = sorted((ctx.repo_root / "ops" / "_schemas").rglob("*.json"))
    out = ctx.repo_root / "docs" / "_generated" / "ops-schemas.md"
    lines = ["# Ops Schemas", "", "Generated from `ops/_schemas`. Do not edit manually.", ""]
    for path in schemas:
        rel = path.relative_to(ctx.repo_root).as_posix()
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            payload = {}
        required = payload.get("required", []) if isinstance(payload, dict) else []
        lines.append(f"## `{rel}`")
        lines.append("")
        if required:
            lines.append("Required keys:")
            lines.extend(f"- `{key}`" for key in required)
        else:
            lines.append("Required keys: none")
        lines.append("")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))
def _generate_ops_surface(ctx: RunContext) -> tuple[int, str]:
    surface = json.loads((ctx.repo_root / "ops" / "_meta" / "surface.json").read_text(encoding="utf-8"))
    suites = json.loads((ctx.repo_root / "ops" / "e2e" / "suites" / "suites.json").read_text(encoding="utf-8"))
    out = ctx.repo_root / "docs" / "_generated" / "ops-surface.md"
    lines = ["# Ops Surface", "", "Generated from ops manifests.", "", "## Stable Entrypoints", ""]
    lines.extend(f"- `make {target}`" for target in surface.get("entrypoints", []))
    lines.extend(["", "## E2E Suites", ""])
    for suite in suites.get("suites", []):
        capabilities = ",".join(suite.get("required_capabilities", []))
        lines.append(f"- `{suite['id']}`: capabilities=`{capabilities}`")
        for scenario in suite.get("scenarios", []):
            budget = scenario.get("budget", {})
            lines.append(
                f"- scenario `{scenario['id']}`: runner=`{scenario['runner']}`, budget(time_s={budget.get('expected_time_seconds')}, qps={budget.get('expected_qps')})"
            )
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))
def _generate_repo_surface(ctx: RunContext) -> tuple[int, str]:
    exclude_dirs = {".git", ".github", ".cargo", "target", ".idea", "node_modules"}
    out = ctx.repo_root / "docs" / "_generated" / "repo-surface.md"
    surface = json.loads((ctx.repo_root / "configs" / "ops" / "public-surface.json").read_text(encoding="utf-8"))
    top_dirs = sorted(
        path.name
        for path in ctx.repo_root.iterdir()
        if path.is_dir() and path.name not in exclude_dirs and not path.name.startswith(".")
    )
    lines = ["# Repository Surface", "", "## Top-level Areas"]
    lines.extend(f"- `{name}/`" for name in top_dirs)
    lines.extend(["", "## Public Make Targets"])
    lines.extend(f"- `make {target}`" for target in surface.get("make_targets", []))
    lines.extend(["", "## Public Ops Run Commands"])
    lines.extend(f"- `./{cmd}`" for cmd in surface.get("ops_run_commands", []))
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"
def _generate_ops_badge(ctx: RunContext) -> tuple[int, str]:
    scorecard = ctx.repo_root / "ops" / "_generated_committed" / "scorecard.json"
    out = ctx.repo_root / "docs" / "_generated" / "ops-badge.md"
    status = "unknown"
    score = 0
    if scorecard.exists():
        payload = json.loads(scorecard.read_text(encoding="utf-8"))
        status = str(payload.get("status", "unknown"))
        score = int(payload.get("score", 0))
    color = "red"
    if status == "pass":
        color = "brightgreen"
    elif status == "unknown":
        color = "lightgrey"
    lines = [
        "# Ops Badge",
        "",
        f"![ops confidence](https://img.shields.io/badge/ops%20confidence-{status}%20({score}%25)-{color})",
        "",
        f"Source: `{scorecard.relative_to(ctx.repo_root)}`",
        "",
    ]
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"
def _generate_k8s_values_doc(ctx: RunContext) -> tuple[int, str]:
    src = ctx.repo_root / "docs" / "contracts" / "CHART_VALUES.json"
    out = ctx.repo_root / "docs" / "operations" / "k8s" / "values.md"
    data = json.loads(src.read_text(encoding="utf-8"))
    keys = data.get("top_level_keys", [])
    lines = [
        "# Kubernetes Values",
        "",
        "- Owner: `bijux-atlas-operations`",
        "",
        "## What",
        "",
        "Generated summary of Helm top-level values from the chart-values contract.",
        "",
        "## Why",
        "",
        "Keeps operations docs aligned to chart values SSOT.",
        "",
        "## Scope",
        "",
        "Top-level chart values keys only.",
        "",
        "## Non-goals",
        "",
        "Does not redefine schema semantics beyond contract references.",
        "",
        "## Contracts",
    ]
    lines.extend(f"- `values.{key}`" for key in keys)
    lines.extend(
        [
            "",
            "## Failure modes",
            "",
            "Missing or stale keys can break deployments and profile docs.",
            "",
            "## How to verify",
            "",
            "```bash",
            "$ make ops-values-validate",
            "```",
            "",
            "Expected output: generated values doc and chart contract check pass.",
            "",
            "## See also",
            "",
            "- [Chart Values Contract](../../contracts/chart-values.md)",
            "- [Helm Chart Contract](chart.md)",
            "- [K8s Index](INDEX.md)",
            "- `ops-values-validate`",
            "",
        ]
    )
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out.relative_to(ctx.repo_root)}"
def _generate_openapi_docs(ctx: RunContext) -> tuple[int, str]:
    src_dir = ctx.repo_root / "configs" / "openapi" / "v1"
    out_dir = ctx.repo_root / "docs" / "_generated" / "openapi"
    out_dir.mkdir(parents=True, exist_ok=True)
    generated = src_dir / "openapi.generated.json"
    snapshot = src_dir / "openapi.snapshot.json"
    if not generated.exists():
        return 1, f"missing {generated.relative_to(ctx.repo_root)}"
    if not snapshot.exists():
        return 1, f"missing {snapshot.relative_to(ctx.repo_root)}"
    spec = json.loads(generated.read_text(encoding="utf-8"))
    paths = sorted(spec.get("paths", {}).keys())
    (out_dir / "openapi.generated.json").write_text(generated.read_text(encoding="utf-8"))
    (out_dir / "openapi.snapshot.json").write_text(snapshot.read_text(encoding="utf-8"))
    index = [
        "# OpenAPI Artifacts",
        "",
        "Generated from `configs/openapi/v1/`.",
        "",
        "- Canonical source: `configs/openapi/v1/openapi.generated.json`",
        "- Snapshot: `configs/openapi/v1/openapi.snapshot.json`",
        "",
        "## Paths",
        "",
    ]
    index.extend(f"- `{path}`" for path in paths)
    (out_dir / "INDEX.md").write_text("\n".join(index) + "\n", encoding="utf-8")
    return 0, "generated docs/_generated/openapi"
def _generate_chart_contract_index(ctx: RunContext) -> tuple[int, str]:
    manifest = ctx.repo_root / "ops" / "k8s" / "tests" / "manifest.json"
    out = ctx.repo_root / "docs" / "_generated" / "contracts" / "chart-contract-index.md"
    doc = json.loads(manifest.read_text(encoding="utf-8"))
    tests: list[dict[str, object]] = []
    for test in doc.get("tests", []):
        groups = set(test.get("groups", []))
        if "chart-contract" not in groups:
            continue
        script = test["script"]
        if not script.startswith("checks/"):
            continue
        tests.append(
            {
                "script": script,
                "owner": test.get("owner", "unknown"),
                "failure": ", ".join(test.get("expected_failure_modes", [])) or "n/a",
                "timeout": test.get("timeout_seconds", "n/a"),
            }
        )
    tests.sort(key=lambda item: item["script"])
    lines = [
        "# Chart Contract Index",
        "",
        "Generated from `ops/k8s/tests/manifest.json` entries tagged with `chart-contract`.",
        "",
        "| Contract Test | Owner | Timeout (s) | Failure Modes |",
        "| --- | --- | ---: | --- |",
    ]
    for test in tests:
        lines.append(f"| `{test['script']}` | `{test['owner']}` | {test['timeout']} | `{test['failure']}` |")
    lines.extend(["", "## Regenerate", "", "```bash", "atlasctl docs generate-chart-contract-index", "```", ""])
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out.relative_to(ctx.repo_root)} ({len(tests)} contracts)"
def _generate_k8s_install_matrix(ctx: RunContext) -> tuple[int, str]:
    src = ctx.repo_root / "artifacts" / "ops" / "k8s-install-matrix.json"
    out = ctx.repo_root / "docs" / "operations" / "k8s" / "release-install-matrix.md"
    if not src.exists():
        data = {"generated_at": "unknown", "profiles": [], "tests": []}
    else:
        data = json.loads(src.read_text(encoding="utf-8"))
    lines = [
        "# Release Install Matrix",
        "",
        "- Owner: `bijux-atlas-operations`",
        "",
        "## What",
        "",
        "Generated matrix of k8s install/test profiles from CI summaries.",
        "",
        "## Why",
        "",
        "Provides a stable compatibility view across supported chart profiles.",
        "",
        "## Contracts",
        "",
        f"Generated at: `{data.get('generated_at', 'unknown')}`",
        "",
        "Profiles:",
    ]
    lines.extend(f"- `{profile}`" for profile in data.get("profiles", []))
    lines.extend(["", "Verified test groups:"])
    lines.extend(f"- `{test}`" for test in data.get("tests", []))
    lines.extend(
        [
            "",
            "## Failure modes",
            "",
            "Missing profile/test entries indicate CI generation drift or skipped suites.",
            "",
            "## How to verify",
            "",
            "```bash",
            "$ ops/k8s/ci/install-matrix.sh",
            "$ make docs",
            "```",
            "",
            "Expected output: matrix doc updated and docs checks pass.",
            "",
            "## See also",
            "",
            "- [K8s Test Contract](k8s-test-contract.md)",
            "- [Helm Chart Contract](chart.md)",
            "- `ops-k8s-tests`",
        ]
    )
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"
