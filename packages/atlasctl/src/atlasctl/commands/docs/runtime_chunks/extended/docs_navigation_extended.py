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
    lines = ["# Ops Schemas", "", "Generated from `ops/schema`. Do not edit manually.", ""]
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
            "$ python3 packages/atlasctl/src/atlasctl/commands/ops/k8s/ci/install_matrix.py",
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
