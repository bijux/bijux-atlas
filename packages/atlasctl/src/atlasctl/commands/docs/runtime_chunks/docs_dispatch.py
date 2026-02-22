def _emit_status(report: str, code: int, output: str, pass_text: str | None = None) -> int:
    if report == "json":
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
    elif output:
        print(output)
    elif pass_text:
        print(pass_text)
    return code


def _run_check_fn(ctx: RunContext, report: str, fn: Callable[[RunContext], tuple[int, str]], pass_text: str | None = None) -> int:
    code, output = fn(ctx)
    return _emit_status(report, code, output, pass_text)


def _run_simple_path(ctx: RunContext, report: str, path: str) -> int:
    return _run_simple(ctx, ["python3", path], report)


def _docs_artifacts_root(ctx: RunContext) -> Path:
    iso_root = ctx.repo_root / "artifacts" / "isolate" / "docs" / ctx.run_id
    if iso_root.exists():
        return iso_root / "docs"
    return ctx.repo_root / "artifacts" / "docs" / ctx.run_id


def _docs_venv_path(ctx: RunContext) -> Path:
    return _docs_artifacts_root(ctx) / ".venv"


def _ensure_docs_venv(ctx: RunContext) -> tuple[int, str]:
    venv = _docs_venv_path(ctx)
    req_lock = ctx.repo_root / "configs" / "docs" / "requirements.lock.txt"
    for cmd in (
        ["python3", "-m", "venv", str(venv)],
        [str(venv / "bin" / "pip"), "install", "--upgrade", "pip"],
        [str(venv / "bin" / "pip"), "install", "-r", str(req_lock)],
    ):
        code, output = _run_check(cmd, ctx.repo_root)
        if code != 0:
            return code, output
    return 0, str(venv)


def _run_docs_pipeline(ctx: RunContext, steps: list[tuple[str, list[str]]], report: str, fail_fast: bool = True) -> int:
    rows: list[dict[str, object]] = []
    for step_id, cmd in steps:
        code, output = _run_check(cmd, ctx.repo_root)
        rows.append(
            {
                "id": step_id,
                "status": "pass" if code == 0 else "fail",
                "command": " ".join(cmd),
                "output": output,
            }
        )
        if code != 0 and fail_fast:
            break
    failed = [row for row in rows if row["status"] == "fail"]
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "run_id": ctx.run_id,
        "status": "pass" if not failed else "fail",
        "steps": rows,
    }
    if report == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        for row in rows:
            print(f"{row['status'].upper()} {row['id']}")
            if row["status"] == "fail" and row["output"]:
                first = str(row["output"]).splitlines()
                if first:
                    print(first[0])
        if not failed:
            print("docs pipeline passed")
    return 0 if not failed else 1


def run_docs_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if not getattr(ns, "docs_cmd", None) and bool(getattr(ns, "list", False)):
        items = sorted(
            {
                "check",
                "validate",
                "lint",
                "generate",
                "generate-registry-indexes",
                "inventory",
                "extract-code",
                "run-blessed-snippets",
                "rewrite-legacy-terms",
                "contracts-index",
                "runbook-map",
                "style",
                "spellcheck",
                "lint-spelling",
                "nav-check",
                "link-check",
                "mkdocs-site-links-check",
                "public-surface-check",
                "generated-check",
                "ops-entrypoints-check",
                "build",
                "serve",
                "freeze",
                "fmt",
                "test",
                "clean",
                "requirements",
            }
        )
        if bool(getattr(ns, "json", False)):
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "ok", "group": "docs", "items": items}, sort_keys=True))
        else:
            for item in items:
                print(item)
        return 0
    if ns.docs_cmd == "requirements":
        if ns.docs_requirements_cmd != "lock-refresh":
            return 2
        venv = _docs_venv_path(ctx)
        req = ctx.repo_root / "configs" / "docs" / "requirements.txt"
        out = ctx.repo_root / "configs" / "docs" / "requirements.lock.txt"
        steps = [
            ("venv-create", ["python3", "-m", "venv", str(venv)]),
            ("pip-upgrade", [str(venv / "bin" / "pip"), "install", "--upgrade", "pip"]),
            ("req-install", [str(venv / "bin" / "pip"), "install", "-r", str(req)]),
            ("req-freeze", ["sh", "-c", f"\"{venv}/bin/pip\" freeze --exclude-editable | LC_ALL=C sort > \"{out}\""]),
        ]
        return _run_docs_pipeline(ctx, steps, ns.report, fail_fast=True)

    if ns.docs_cmd == "clean":
        target = _docs_artifacts_root(ctx)
        if target.exists():
            shutil.rmtree(target)
            return _emit_status(ns.report, 0, f"removed {target.relative_to(ctx.repo_root)}")
        return _emit_status(ns.report, 0, "nothing to clean")

    if ns.docs_cmd == "freeze":
        return _run_check_fn(ctx, ns.report, _check_docs_freeze_drift, "docs freeze check passed")

    if ns.docs_cmd == "fmt":
        steps = [
            ("render-diagrams", ["python3", "-m", "atlasctl.cli", "docs", "render-diagrams", "--report", "text"]),
            ("docs-style", ["python3", "-m", "atlasctl.cli", "docs", "style", "--report", "text"]),
        ]
        if bool(getattr(ns, "all", False)):
            steps.append(("extract-code", ["python3", "-m", "atlasctl.cli", "docs", "extract-code", "--report", "text"]))
        return _run_docs_pipeline(ctx, steps, ns.report, fail_fast=True)

    if ns.docs_cmd == "test":
        steps = [
            ("docs-freeze", ["python3", "-m", "atlasctl.cli", "docs", "freeze", "--report", "text"]),
            ("docs-nav-check", ["python3", "-m", "atlasctl.cli", "docs", "nav-check", "--report", "text"]),
            ("docs-link-check", ["python3", "-m", "atlasctl.cli", "docs", "link-check", "--report", "text"]),
        ]
        if bool(getattr(ns, "all", False)):
            steps.append(("docs-check", ["python3", "-m", "atlasctl.cli", "docs", "check", "--report", "text"]))
        return _run_docs_pipeline(ctx, steps, ns.report, fail_fast=bool(getattr(ns, "fail_fast", False)))

    if ns.docs_cmd == "build":
        code, output = _ensure_docs_venv(ctx)
        if code != 0:
            return _emit_status(ns.report, code, output)
        venv = Path(output)
        docs_artifacts = _docs_artifacts_root(ctx)
        site_dir = docs_artifacts / "site"
        steps = [
            ("docs-generate", ["python3", "-m", "atlasctl.cli", "docs", "generate", "--report", "text"]),
            ("docs-fmt", ["python3", "-m", "atlasctl.cli", "docs", "fmt", "--report", "text"]),
            (
                "mkdocs-build",
                [
                    "env",
                    "SOURCE_DATE_EPOCH=946684800",
                    str(venv / "bin" / "mkdocs"),
                    "build",
                    "--strict",
                    "--config-file",
                    "mkdocs.yml",
                    "--site-dir",
                    str(site_dir),
                ],
            ),
            ("docs-check", ["python3", "-m", "atlasctl.cli", "docs", "check", "--report", "text"]),
        ]
        if bool(getattr(ns, "all", False)):
            steps.append(("docs-test", ["python3", "-m", "atlasctl.cli", "docs", "test", "--report", "text", "--all"]))
        return _run_docs_pipeline(ctx, steps, ns.report, fail_fast=bool(getattr(ns, "fail_fast", False)))

    if ns.docs_cmd == "serve":
        code, output = _ensure_docs_venv(ctx)
        if code != 0:
            return _emit_status(ns.report, code, output)
        venv = Path(output)
        cmd = ["env", "SOURCE_DATE_EPOCH=946684800", str(venv / "bin" / "mkdocs"), "serve", "--config-file", "mkdocs.yml"]
        return _run_simple(ctx, cmd, ns.report)

    if ns.docs_cmd in {"check", "validate"}:
        def _check_nav_contract(inner_ctx: RunContext) -> tuple[int, str]:
            missing = _mkdocs_missing_files(inner_ctx.repo_root)
            if missing:
                return 1, "\n".join(missing)
            return 0, ""

        checks = [
            _check("docs-link-check", "Validate markdown links", None, "Fix broken internal links and anchors.", fn=_check_markdown_links),
            _check("docs-nav-check", "Validate mkdocs nav references existing docs files", None, "Fix mkdocs.yml nav entries to point to real docs pages.", fn=_check_nav_contract),
            _check("docs-public-surface", "Validate docs public surface", None, "Regenerate/align docs public surface JSON and docs references.", fn=_check_public_surface_docs),
            _check("docs-no-internal-target-refs", "Validate no internal make target refs", None, "Replace internal make targets with public targets in docs.", fn=_check_docs_make_only),
            _check("docs-ops-entrypoints", "Validate ops docs entrypoint policy", ["python3", "packages/atlasctl/src/atlasctl/checks/domains/ops/ops_checks/check_ops_external_entrypoints.py"], "Reference only make targets and ops/run entrypoints in docs."),
            _check("docs-generated", "Validate generated docs are up-to-date", None, "Regenerate docs outputs and commit deterministic updates.", fn=_check_docs_freeze_drift),
        ]
        if ns.docs_cmd == "check":
            checks = DOCS_LINT_CHECKS + checks
            if bool(getattr(ns, "all", False)):
                checks.append(
                    _check(
                        "docs-mkdocs-site-links",
                        "Validate rendered mkdocs site links",
                        None,
                        "Rebuild docs site and fix broken internal links.",
                        fn=lambda inner_ctx: _check_mkdocs_site_links(inner_ctx, str(_docs_artifacts_root(inner_ctx) / "site")),
                    )
                )
        return _run_docs_checks(ctx, checks, ns.report, ns.fail_fast, ns.emit_artifacts)

    if ns.docs_cmd == "lint":
        if ns.fix:
            code, output = _rewrite_legacy_terms(ctx, "docs", apply=True)
            if code != 0:
                if output:
                    print(output)
                return code
        checks = list(DOCS_LINT_CHECKS)
        if bool(getattr(ns, "all", False)):
            checks.extend(
                [
                    _check("docs-spelling", "Validate docs spelling", None, "Fix docs spelling and glossary violations.", fn=lambda inner_ctx: _spellcheck(inner_ctx, "docs")),
                    _check("docs-style", "Validate docs style checks", None, "Fix docs style lint issues.", fn=lambda inner_ctx: _lint_doc_status(inner_ctx)),
                ]
            )
        return _run_docs_checks(ctx, checks, ns.report, ns.fail_fast, ns.emit_artifacts)

    if ns.docs_cmd == "ops-entrypoints-check":
        return _run_simple_path(ctx, ns.report, "packages/atlasctl/src/atlasctl/checks/domains/ops/ops_checks/check_ops_external_entrypoints.py")

    if ns.docs_cmd == "nav-check":
        missing = _mkdocs_missing_files(ctx.repo_root)
        payload = {"schema_version": 1, "tool": "bijux-atlas", "run_id": ctx.run_id, "status": "pass" if not missing else "fail", "missing_files": missing}
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        elif missing:
            print("mkdocs nav check failed:")
            for item in missing:
                print(f"- missing docs/{item}")
        else:
            print("mkdocs nav check passed")
        return 0 if not missing else 1

    check_cmds: dict[str, tuple[Callable[[RunContext], tuple[int, str]], str | None]] = {
        "link-check": (_check_markdown_links, "docs link-check passed"),
        "public-surface-check": (_check_public_surface_docs, "docs public surface check passed"),
        "no-internal-target-refs": (_check_docs_make_only, "docs make-only check passed"),
        "generated-check": (_check_docs_freeze_drift, "docs freeze check passed"),
        "glossary-check": (_glossary_check, None),
        "openapi-examples-check": (_check_openapi_examples, "openapi examples check passed"),
        "observability-surface-check": (_check_observability_surface_drift, "observability surface drift check passed"),
        "runbooks-contract-check": (_check_runbooks_contract, "runbook contract check passed"),
        "ops-readmes-make-only-check": (_check_ops_readmes_make_only, "ops README make-only contract passed"),
        "ops-readme-canonical-links-check": (_check_ops_readme_canonical_links, "ops README canonical-link check passed"),
        "ops-doc-duplication-check": (_check_ops_doc_duplication, "ops docs duplication check passed"),
        "docs-make-only-ops-check": (_check_docs_make_only_ops, "docs make-only ops entrypoint check passed"),
        "crate-docs-contract-check": (_check_crate_docs_contract, None),
        "durable-naming-check": (_check_durable_naming, None),
        "duplicate-topics-check": (_check_duplicate_topics, None),
        "naming-inventory": (_generate_naming_inventory, None),
        "legacy-terms-check": (_check_legacy_terms, None),
        "observability-docs-checklist": (_check_observability_docs_checklist, None),
        "no-orphan-docs-check": (_check_no_orphan_docs, None),
        "script-locations-check": (_check_script_locations, None),
        "runbook-map-registration-check": (_check_runbook_map_registration, None),
        "contract-doc-pairs-check": (_check_contract_doc_pairs, None),
        "index-pages-check": (_check_index_pages, None),
        "observability-acceptance-checklist": (_check_observability_acceptance_checklist, None),
        "script-headers-check": (_check_script_headers, None),
        "concept-registry-check": (_check_concept_registry, None),
        "concept-graph-generate": (_generate_concept_graph, None),
        "adr-headers-check": (_check_adr_headers, None),
        "broken-examples-check": (_check_broken_examples, None),
        "doc-filename-style-check": (_check_doc_filename_style, None),
        "no-placeholders-check": (_check_no_placeholders, None),
        "no-legacy-root-paths-check": (_check_no_legacy_root_paths, None),
        "nav-order-check": (_check_nav_order, None),
        "docs-deterministic-check": (_check_docs_deterministic, None),
        "docs-make-targets-exist-check": (_check_docs_make_targets_exist, None),
        "critical-make-targets-referenced-check": (_check_critical_make_targets_referenced, None),
        "make-targets-documented-check": (_check_make_targets_documented, None),
        "make-targets-drift-check": (_check_make_targets_drift, None),
        "docker-entrypoints-check": (_check_docker_entrypoints, None),
        "example-configs-check": (_check_example_configs, None),
        "full-stack-page-check": (_check_full_stack_page, None),
        "k8s-docs-contract-check": (_check_k8s_docs_contract, None),
        "load-docs-contract-check": (_check_load_docs_contract, None),
        "make-help-drift-check": (_check_make_help_drift, None),
        "no-removed-make-targets-check": (_check_no_removed_make_targets, None),
        "ops-docs-make-targets-check": (_check_ops_docs_make_targets, None),
        "ops-observability-links-check": (_check_ops_observability_links, None),
        "public-targets-docs-sections-check": (_check_public_targets_docs_sections, None),
        "suite-id-docs-check": (_check_suite_id_docs, None),
        "configmap-env-docs-check": (_check_configmap_env_docs, None),
        "generated-contract-docs-check": (_check_generated_contract_docs, None),
        "lint-depth": (_lint_depth, None),
        "lint-doc-contracts": (_lint_doc_contracts, None),
        "extract-code": (_extract_code, None),
        "run-blessed-snippets": (_run_blessed_snippets, None),
        "render-diagrams": (_render_diagrams, None),
        "lint-spelling": (lambda c: _spellcheck(c, ns.path), None),
        "spellcheck": (lambda c: _spellcheck(c, ns.path), None),
        "style": (_lint_doc_status, None),
        "mkdocs-site-links-check": (lambda c: _check_mkdocs_site_links(c, ns.site_dir), None),
    }
    if ns.docs_cmd in check_cmds:
        fn, pass_text = check_cmds[ns.docs_cmd]
        return _run_check_fn(ctx, ns.report, fn, pass_text)

    if ns.docs_cmd == "contracts-index":
        fn = _generate_contracts_index_doc if ns.fix else _check_contracts_index_nav
        return _run_check_fn(ctx, ns.report, fn)

    if ns.docs_cmd == "runbook-map":
        fn = _generate_runbook_map_index if ns.fix else _check_runbook_map_registration
        return _run_check_fn(ctx, ns.report, fn)

    if ns.docs_cmd == "evidence-policy-page":
        out_rel = ns.out or "docs/_generated/evidence-policy.md"
        written = _generate_docs_evidence_policy(ctx.repo_root, out_rel)
        payload = {"schema_version": 1, "tool": "bijux-atlas", "run_id": ctx.run_id, "status": "pass", "output": written}
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else payload["output"])
        return 0

    if ns.docs_cmd == "inventory":
        out = Path(ns.out or "docs/_generated/docs-inventory.md")
        _generate_docs_inventory(ctx.repo_root, ctx.repo_root / out)
        payload = {"schema_version": 1, "tool": "bijux-atlas", "run_id": ctx.run_id, "status": "pass", "output": out.as_posix()}
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else payload["output"])
        return 0

    if ns.docs_cmd == "rewrite-legacy-terms":
        code, output = _rewrite_legacy_terms(ctx, ns.path, ns.apply)
        return _emit_status(ns.report, code, output)

    generate_cmds: dict[str, Callable[[RunContext], tuple[int, str]]] = {
        "generate-sli-doc": _generate_sli_doc,
        "generate-slos-doc": _generate_slos_doc,
        "generate-architecture-map": _generate_architecture_map,
        "generate-command-groups-docs": _generate_command_group_docs,
        "generate-registry-indexes": _generate_registry_indexes,
        "generate-crates-map": _generate_crates_map,
        "generate-upgrade-guide": _generate_upgrade_guide,
        "generate-make-targets-catalog": _generate_make_targets_catalog,
        "generate-env-vars-doc": _generate_env_vars_doc,
        "generate-config-keys-doc": _generate_config_keys_doc,
        "generate-layer-contract-doc": _generate_layer_contract_doc,
        "generate-makefiles-surface": _generate_makefiles_surface,
        "generate-observability-surface": _generate_observability_surface,
        "generate-ops-contracts-doc": _generate_ops_contracts_doc,
        "generate-ops-schema-docs": _generate_ops_schema_docs,
        "generate-ops-surface": _generate_ops_surface,
        "generate-repo-surface": _generate_repo_surface,
        "generate-ops-badge": _generate_ops_badge,
        "generate-k8s-values-doc": _generate_k8s_values_doc,
        "generate-openapi-docs": _generate_openapi_docs,
        "generate-chart-contract-index": _generate_chart_contract_index,
        "generate-k8s-install-matrix": _generate_k8s_install_matrix,
        "generate-make-targets-inventory": _generate_make_targets_inventory,
        "generate-scripts-graph": _generate_scripts_graph,
    }
    if ns.docs_cmd in generate_cmds:
        return _run_check_fn(ctx, ns.report, generate_cmds[ns.docs_cmd])

    if ns.docs_cmd == "generate":
        errors: list[str] = []
        for cmd in DOCS_GENERATE_COMMANDS:
            code, output = _run_check(cmd, ctx.repo_root)
            if code != 0:
                errors.append(f"{' '.join(cmd)} -> {output}")
                if ns.fail_fast:
                    break
        if not errors:
            _generate_docs_inventory(ctx.repo_root, ctx.repo_root / "packages/atlasctl/docs/_generated/commands-inventory.md")
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "pass" if not errors else "fail", "generated_count": len(DOCS_GENERATE_COMMANDS) - len(errors), "errors": errors}
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else json.dumps(payload, indent=2, sort_keys=True))
        return 0 if not errors else 1

    return 2
