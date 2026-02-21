def _generate_make_targets_inventory(ctx: RunContext) -> tuple[int, str]:
    out_main = ctx.repo_root / "docs" / "development" / "make-targets.md"
    out_compat = ctx.repo_root / "docs" / "development" / "make-targets-inventory.md"
    help_text = subprocess.check_output(["python3", "-m", "atlasctl.cli", "make", "help"], cwd=ctx.repo_root, text=True)
    sections = _parse_make_help_sections(help_text)
    lines: list[str] = [
        "# Make Targets Inventory",
        "",
        "- Owner: `docs-governance`",
        "",
        "Generated from `make help`. Do not edit manually.",
        "",
    ]
    for section, targets in sections.items():
        lines.append(f"## {section}")
        lines.append("")
        lines.extend(f"- `{target}`" for target in targets)
        lines.append("")
    rendered = "\n".join(lines)
    out_main.write_text(rendered, encoding="utf-8")
    out_compat.write_text(rendered, encoding="utf-8")
    return 0, str(out_main.relative_to(ctx.repo_root))
def _generate_scripts_graph(ctx: RunContext) -> tuple[int, str]:
    mk_files = [ctx.repo_root / "Makefile", *sorted((ctx.repo_root / "makefiles").glob("*.mk"))]
    out = ctx.repo_root / "docs" / "development" / "scripts-graph.md"
    target_re = re.compile(r"^([a-zA-Z0-9_.-]+):(?:\s|$)")
    script_re = re.compile(r"(?:\./|python3\s+|python\s+)(scripts/(?:public|internal)/[^\s\"']+)")
    rows: list[tuple[str, str]] = []
    for mk in mk_files:
        current = ""
        for line in mk.read_text(encoding="utf-8").splitlines():
            if line.startswith("\t"):
                for match in script_re.finditer(line):
                    rows.append((current, match.group(1).rstrip(";")))
                continue
            m = target_re.match(line)
            if m and not line.startswith("."):
                current = m.group(1)
    rows = sorted(set((target, script) for target, script in rows if target and script))
    lines = [
        "# Scripts Graph",
        "",
        "Generated file. Do not edit by hand.",
        "",
        "| Make Target | Script |",
        "|---|---|",
    ]
    lines.extend(f"| `{target}` | `{script}` |" for target, script in rows)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"
def _generate_upgrade_guide(ctx: RunContext) -> tuple[int, str]:
    payload = _read_json(ctx.repo_root / "configs/ops/target-renames.json")
    rows = payload.get("renames", [])
    lines = [
        "# Make Target Upgrade Guide",
        "",
        "Use this table to migrate renamed or aliased make targets.",
        "",
        "| Old Target | New Target | Status |",
        "|---|---|---|",
    ]
    if isinstance(rows, list):
        for row in rows:
            if not isinstance(row, dict):
                continue
            lines.append(f"| `{row.get('from','')}` | `{row.get('to','')}` | `{row.get('status','')}` |")
    lines.append("")
    out = ctx.repo_root / "docs/_generated/upgrade-guide.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))
def _check_crate_docs_contract(ctx: RunContext) -> tuple[int, str]:
    crates_root = ctx.repo_root / "crates"
    if not crates_root.exists():
        return 1, "crates directory missing"
    crates = sorted([p for p in crates_root.iterdir() if p.is_dir()])
    required_docs = {"INDEX.md", "architecture.md", "effects.md", "public-api.md", "testing.md"}
    contracts_required = {"bijux-atlas-api", "bijux-atlas-server", "bijux-atlas-policies", "bijux-atlas-store"}
    failure_modes_required = {"bijux-atlas-server", "bijux-atlas-store", "bijux-atlas-ingest"}
    required_sections = ["## Purpose", "## Invariants", "## Boundaries", "## Failure modes", "## How to test"]
    placeholder_pat = re.compile(r"\b(TODO|TBD|coming soon)\b", re.IGNORECASE)
    pub_pat = re.compile(r"^\s*pub\s+(?:struct|enum|trait|type)\s+([A-Z][A-Za-z0-9_]*)\b", re.MULTILINE)
    errors: list[str] = []

    for crate in crates:
        name = crate.name
        docs = crate / "docs"
        readme = crate / "README.md"
        if not docs.is_dir():
            errors.append(f"{crate}: missing docs directory")
            continue
        files = {p.name for p in docs.glob("*.md")}
        for req in required_docs:
            if req not in files:
                errors.append(f"{crate}/docs: missing {req}")
        if name in contracts_required and "contracts.md" not in files:
            errors.append(f"{crate}/docs: missing contracts.md (required)")
        if name in failure_modes_required and "failure-modes.md" not in files:
            errors.append(f"{crate}/docs: missing failure-modes.md (required)")

        for forbidden in [
            "HUMAN_MACHINE.md",
            "PUBLIC_SURFACE_CHECKLIST.md",
            "EFFECT_BOUNDARY_MAP.md",
            "PUBLIC_API.md",
            "ARCHITECTURE.md",
            "EFFECTS.md",
        ]:
            if forbidden in files:
                errors.append(f"{crate}/docs: legacy filename forbidden: {forbidden}")

        if "patterns.md" in files:
            text = (docs / "patterns.md").read_text(encoding="utf-8")
            if len(text.strip()) < 120:
                errors.append(f"{crate}/docs/patterns.md: too small; remove or document real patterns")

        major = [docs / "testing.md"]
        if name in contracts_required:
            major.append(docs / "contracts.md")
        if name in failure_modes_required:
            major.append(docs / "failure-modes.md")
        for md in major:
            if not md.exists():
                continue
            txt = md.read_text(encoding="utf-8")
            if not re.search(r"^- Owner:\s*`[^`]+`\s*$", txt, re.MULTILINE):
                errors.append(f"{md}: missing owner header \"- Owner: `...`\"")
            for sec in required_sections:
                if sec not in txt:
                    errors.append(f"{md}: missing section {sec}")
            if md.name == "contracts.md" and "## Versioning" not in txt:
                errors.append(f"{md}: missing section ## Versioning")
            if (txt.count("```") // 2) < 2:
                errors.append(f"{md}: requires at least 2 examples")
            if placeholder_pat.search(txt):
                errors.append(f"{md}: contains placeholder marker TODO/TBD/coming soon")
            if re.search(r"\]\((?:https?://|file://|/)", txt):
                errors.append(f"{md}: contains non-relative internal link")

        if not readme.exists():
            errors.append(f"{crate}: missing README.md")
        else:
            rtxt = readme.read_text(encoding="utf-8")
            for sec in [
                "## Purpose",
                "## Public API",
                "## Boundaries",
                "## Effects",
                "## Telemetry",
                "## Tests",
                "## Benches",
                "## Docs index",
            ]:
                if sec not in rtxt:
                    errors.append(f"{readme}: missing section {sec}")
            for req_link in ["docs/INDEX.md", "docs/public-api.md"]:
                if req_link not in rtxt:
                    errors.append(f"{readme}: missing link {req_link}")
            docs_index_block = re.search(r"## Docs index\n([\s\S]*?)(?:\n## |\Z)", rtxt)
            if not docs_index_block:
                errors.append(f"{readme}: missing docs index block")
            else:
                links = re.findall(r"\[[^\]]+\]\([^\)]+\)", docs_index_block.group(1))
                if len(links) < 5:
                    errors.append(f"{readme}: docs index must list at least 5 important docs")

        idx = docs / "INDEX.md"
        if idx.exists():
            itxt = idx.read_text(encoding="utf-8")
            for req in ["public-api.md", "effects.md", "testing.md"]:
                if req not in itxt:
                    errors.append(f"{idx}: must link {req}")
            if "#how-to-extend" not in itxt and "How to extend" not in itxt:
                errors.append(f"{idx}: must provide How to extend linkage")

        lib = crate / "src" / "lib.rs"
        public_api = docs / "public-api.md"
        if lib.exists() and public_api.exists():
            names = sorted(set(pub_pat.findall(lib.read_text(encoding="utf-8"))))
            ptxt = public_api.read_text(encoding="utf-8")
            if (
                "../../../../docs/_style/stability-levels.md" not in ptxt
                and "../../../docs/_style/stability-levels.md" not in ptxt
            ):
                errors.append(f"{public_api}: missing stability reference link")
            for n in names:
                if n not in ptxt:
                    errors.append(f"{public_api}: missing mention of public type {n}")
    return (0, "crate docs contract OK") if not errors else (1, "\n".join(errors[:300]))
def _mkdocs_nav_file_refs(mkdocs_text: str) -> list[str]:
    refs: list[str] = []
    for line in mkdocs_text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("- ") and ": " not in stripped:
            continue
        m = re.search(r":\s*([A-Za-z0-9_./-]+\.md)\s*$", stripped)
        if m:
            refs.append(m.group(1))
    return refs
def _mkdocs_missing_files(repo_root: Path) -> list[str]:
    mkdocs = repo_root / "mkdocs.yml"
    text = mkdocs.read_text(encoding="utf-8")
    refs = _mkdocs_nav_file_refs(text)
    missing = []
    for ref in refs:
        p = repo_root / "docs" / ref
        if not p.exists():
            missing.append(ref)
    return sorted(set(missing))
def _run_docs_checks(
    ctx: RunContext,
    checks: list[DocsCheck],
    report_format: str,
    fail_fast: bool,
    emit_artifacts: bool,
    runner: Callable[[list[str], Path], tuple[int, str]] = _run_check,
) -> int:
    started_at = datetime.now(timezone.utc).isoformat()
    rows: list[dict[str, object]] = []
    for check in checks:
        if check.fn is not None:
            code, output = check.fn(ctx)
        elif check.cmd is not None:
            code, output = runner(check.cmd, ctx.repo_root)
        else:
            code, output = 2, "invalid docs check configuration"
        row: dict[str, object] = {
            "id": check.check_id,
            "description": check.description,
            "status": "pass" if code == 0 else "fail",
            "command": " ".join(check.cmd) if check.cmd else "native",
            "actionable": check.actionable,
        }
        if code != 0:
            row["error"] = output
        rows.append(row)
        if fail_fast and code != 0:
            break
    ended_at = datetime.now(timezone.utc).isoformat()
    failed_count = len([r for r in rows if r["status"] == "fail"])
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "status": "fail" if failed_count else "pass",
        "started_at": started_at,
        "ended_at": ended_at,
        "failed_count": failed_count,
        "total_count": len(rows),
        "checks": rows,
    }

    if emit_artifacts:
        out = ensure_evidence_path(ctx, ctx.evidence_root / "docs" / "check" / ctx.run_id / "report.json")
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(
            "docs checks: "
            f"status={payload['status']} "
            f"checks={payload['total_count']} "
            f"failed={payload['failed_count']}"
        )
        for row in rows:
            if row["status"] == "fail":
                first = str(row.get("error", "")).splitlines()[:1]
                print(f"- FAIL {row['id']}: {first[0] if first else 'check failed'}")
                print(f"  fix: {row['actionable']}")

    return 0 if failed_count == 0 else 1
def _run_simple(ctx: RunContext, cmd: list[str], report: str) -> int:
    code, output = _run_check(cmd, ctx.repo_root)
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "status": "pass" if code == 0 else "fail",
        "command": " ".join(cmd),
        "output": output,
    }
    if report == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(output)
    return code
def _generate_docs_inventory(repo_root: Path, out: Path) -> None:
    from ...cli.surface_registry import command_registry

    out.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Docs Inventory",
        "",
        "Generated by `atlasctl docs inventory`.",
        "",
        "## Command Surface",
        "",
    ]
    for spec in sorted(command_registry(), key=lambda item: item.name):
        if spec.internal:
            continue
        lines.append(f"- `atlasctl {spec.name}`")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")


def _generate_command_group_docs(ctx: RunContext) -> tuple[int, str]:
    from ...cli.surface_registry import command_registry
    from ...core.effects import command_group

    repo_root = ctx.repo_root
    groups_dir = repo_root / "packages/atlasctl/docs/commands/groups"
    groups_dir.mkdir(parents=True, exist_ok=True)
    grouped: dict[str, list[object]] = {}
    for spec in sorted(command_registry(), key=lambda item: item.name):
        grouped.setdefault(command_group(spec.name), []).append(spec)

    written: list[str] = []
    for group, specs in sorted(grouped.items()):
        path = groups_dir / f"{group}.md"
        lines = [
            f"# {group.capitalize()} Command Group",
            "",
            f"Commands mapped to `{group}` effects policy.",
            "",
            "## Commands",
            "",
        ]
        for spec in specs:
            lines.append(f"- `{spec.name}`")
        lines.extend(["", "## Examples", ""])
        for spec in specs:
            examples = list(spec.examples) if getattr(spec, "examples", ()) else [f"atlasctl {spec.name} --help"]
            lines.append(f"- `{examples[0]}`")
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        written.append(path.relative_to(repo_root).as_posix())
    return 0, ",".join(written)
def _generate_docs_evidence_policy(repo_root: Path, out_rel: str = "docs/_generated/evidence-policy.md") -> str:
    out = repo_root / out_rel
    out.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Evidence Policy",
        "",
        "Generated by `bijux-atlas docs evidence-policy-page`.",
        "",
        "- Runtime evidence location: `artifacts/evidence/`",
        "- Committed generated docs location: `docs/_generated/`",
        "- Ops committed generated location: `ops/_generated_committed/`",
        "- Runtime evidence must not be committed to git.",
    ]
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out_rel
def _check_markdown_links(ctx: RunContext) -> tuple[int, str]:
    import os
    import re

    root = ctx.repo_root
    exclude_parts = {".git", "artifacts", "target", ".cargo"}
    link_re = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
    md_files: list[Path] = []

    for dirpath, dirnames, filenames in os.walk(root, topdown=True, followlinks=False):
        dirnames[:] = [d for d in dirnames if d not in exclude_parts and not Path(dirpath, d).is_symlink()]
        for filename in filenames:
            if not filename.endswith(".md"):
                continue
            path = Path(dirpath) / filename
            if exclude_parts.intersection(path.parts):
                continue
            md_files.append(path)

    errors: list[str] = []
    for md in md_files:
        text = md.read_text(encoding="utf-8")
        for target in link_re.findall(text):
            if target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            rel = target.split("#", 1)[0]
            if not rel:
                continue
            resolved = (md.parent / rel).resolve()
            if not resolved.exists():
                errors.append(f"{md.relative_to(root)}: missing link target {target}")

    if errors:
        return 1, "docs markdown link-check failed:\n" + "\n".join(f"- {e}" for e in errors[:200])
    return 0, f"markdown links OK ({len(md_files)} files)"
