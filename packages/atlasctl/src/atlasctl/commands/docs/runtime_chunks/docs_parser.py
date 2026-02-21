def configure_docs_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("docs", help="docs checks and generation commands")
    p.add_argument("--list", action="store_true", help="list available docs commands")
    p.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    docs_sub = p.add_subparsers(dest="docs_cmd", required=False)

    check = docs_sub.add_parser("check", help="run canonical docs check suite")
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument("--fail-fast", action="store_true")
    check.add_argument("--emit-artifacts", action="store_true")
    check.add_argument("--fix", action="store_true")
    check.add_argument("--all", action="store_true", help="run full docs check pipeline")

    validate = docs_sub.add_parser("validate", help="validate docs links, nav, and required pages")
    validate.add_argument("--report", choices=["text", "json"], default="text")
    validate.add_argument("--fail-fast", action="store_true")
    validate.add_argument("--emit-artifacts", action="store_true")

    lint = docs_sub.add_parser("lint", help="run docs lint checks")
    lint.add_argument("--report", choices=["text", "json"], default="text")
    lint.add_argument("--fail-fast", action="store_true")
    lint.add_argument("--emit-artifacts", action="store_true")
    lint.add_argument("--fix", action="store_true")
    lint.add_argument("--all", action="store_true", help="run full docs lint pipeline")

    build = docs_sub.add_parser("build", help="build docs site + run canonical docs checks")
    build.add_argument("--report", choices=["text", "json"], default="text")
    build.add_argument("--all", action="store_true", help="run full docs build pipeline")
    build.add_argument("--fail-fast", action="store_true")

    serve = docs_sub.add_parser("serve", help="serve docs locally with mkdocs")
    serve.add_argument("--report", choices=["text", "json"], default="text")

    freeze = docs_sub.add_parser("freeze", help="validate generated docs are up-to-date")
    freeze.add_argument("--report", choices=["text", "json"], default="text")

    fmt = docs_sub.add_parser("fmt", help="run docs formatting/render steps")
    fmt.add_argument("--report", choices=["text", "json"], default="text")
    fmt.add_argument("--all", action="store_true", help="run full docs format pipeline")

    test = docs_sub.add_parser("test", help="run docs test gate (freeze + links + nav)")
    test.add_argument("--report", choices=["text", "json"], default="text")
    test.add_argument("--all", action="store_true", help="run full docs test pipeline")
    test.add_argument("--fail-fast", action="store_true")

    clean = docs_sub.add_parser("clean", help="clean docs build artifacts")
    clean.add_argument("--report", choices=["text", "json"], default="text")

    requirements = docs_sub.add_parser("requirements", help="docs requirements management")
    req_sub = requirements.add_subparsers(dest="docs_requirements_cmd", required=True)
    req_lock = req_sub.add_parser("lock-refresh", help="refresh docs requirements lock deterministically")
    req_lock.add_argument("--report", choices=["text", "json"], default="text")

    for name, help_text in (
        ("link-check", "run internal markdown link checks"),
        ("public-surface-check", "validate docs public-surface contract"),
        ("no-internal-target-refs", "forbid internal make target references in docs"),
        ("ops-entrypoints-check", "ensure docs mention only make targets and ops/run entrypoints"),
        ("nav-check", "validate mkdocs nav references existing docs files"),
        ("generated-check", "validate generated docs are up-to-date"),
        ("openapi-examples-check", "validate OpenAPI examples against declared schemas"),
        ("observability-surface-check", "validate observability surface generated docs are in sync"),
        ("runbooks-contract-check", "validate runbook content contract"),
        ("ops-readmes-make-only-check", "validate ops README files use make-only instructions"),
        ("ops-readme-canonical-links-check", "validate canonical links in ops README files"),
        ("ops-doc-duplication-check", "detect duplicate long sections in operations docs"),
        ("docs-make-only-ops-check", "forbid raw ops script references in docs"),
        ("generate-sli-doc", "generate docs/operations/slo/SLIS.md from SLI contract"),
        ("generate-slos-doc", "generate docs/operations/slo/SLOS.md from SLO contract"),
        ("generate-architecture-map", "generate docs/architecture/architecture-map.md"),
        ("generate-command-groups-docs", "generate docs/commands/groups/*.md from command registry"),
        ("generate-registry-indexes", "generate docs command/check/suite indexes from registry"),
        ("generate-crates-map", "generate docs/development/crates-map.md"),
        ("generate-upgrade-guide", "generate docs/_generated/upgrade-guide.md"),
        ("generate-make-targets-catalog", "generate makefiles/targets.json and docs/_generated/make-targets.md"),
        ("generate-env-vars-doc", "generate docs/_generated/env-vars.md"),
        ("generate-config-keys-doc", "generate docs/_generated/config-keys.md"),
        ("generate-layer-contract-doc", "generate docs/_generated/layer-contract.md"),
        ("generate-makefiles-surface", "generate docs/development/makefiles/surface.md"),
        ("generate-observability-surface", "generate docs/_generated/observability-surface.md"),
        ("generate-ops-contracts-doc", "generate docs/_generated/ops-contracts.md"),
        ("generate-ops-schema-docs", "generate docs/_generated/ops-schemas.md"),
        ("generate-ops-surface", "generate docs/_generated/ops-surface.md"),
        ("generate-repo-surface", "generate docs/_generated/repo-surface.md"),
        ("generate-ops-badge", "generate docs/_generated/ops-badge.md"),
        ("generate-k8s-values-doc", "generate docs/operations/k8s/values.md"),
        ("generate-openapi-docs", "generate docs/_generated/openapi/* artifacts"),
        ("generate-chart-contract-index", "generate docs/_generated/contracts/chart-contract-index.md"),
        ("generate-k8s-install-matrix", "generate docs/operations/k8s/release-install-matrix.md"),
        ("generate-make-targets-inventory", "generate docs/development/make-targets*.md"),
        ("generate-scripts-graph", "generate docs/development/scripts-graph.md"),
        ("crate-docs-contract-check", "validate per-crate docs contract"),
        ("durable-naming-check", "enforce durable naming rules across docs/scripts"),
        ("duplicate-topics-check", "enforce duplicate topics pointer and owner contract"),
        ("naming-inventory", "generate docs/_generated/naming-inventory.md"),
        ("legacy-terms-check", "forbid legacy planning/task wording in docs"),
        ("observability-docs-checklist", "validate observability docs checklist and page sections"),
        ("no-orphan-docs-check", "validate docs are discoverable by nav/index links"),
        ("script-locations-check", "validate script location policy"),
        ("runbook-map-registration-check", "validate runbook map has every runbook"),
        ("contract-doc-pairs-check", "validate JSON contracts have docs pairs"),
        ("index-pages-check", "validate docs/INDEX.md contract"),
        ("observability-acceptance-checklist", "validate observability acceptance checklist contract"),
        ("script-headers-check", "validate script header and docs script-group contract"),
        ("concept-registry-check", "validate docs concept registry and canonical ownership"),
        ("concept-graph-generate", "generate docs/_generated/concepts.md from concept registry"),
        ("adr-headers-check", "validate ADR naming and title/header contract"),
        ("broken-examples-check", "validate docs shell examples against make targets and tools"),
        ("doc-filename-style-check", "validate docs filename style policy"),
        ("no-placeholders-check", "forbid TODO/TBD placeholders outside drafts"),
        ("no-legacy-root-paths-check", "forbid legacy root ops paths in docs"),
        ("mkdocs-site-links-check", "validate rendered mkdocs site internal links"),
        ("nav-order-check", "validate top-level mkdocs nav ordering"),
        ("docs-deterministic-check", "validate docs determinism settings"),
        ("docs-make-targets-exist-check", "validate make targets referenced in docs exist"),
        ("critical-make-targets-referenced-check", "validate critical make targets are referenced in docs"),
        ("make-targets-documented-check", "validate public make targets have docs coverage"),
        ("make-targets-drift-check", "validate docs make-targets catalog is in sync"),
        ("docker-entrypoints-check", "validate docs use make docker entrypoints"),
        ("example-configs-check", "validate example config docs contract"),
        ("full-stack-page-check", "validate full-stack operations page contract"),
        ("k8s-docs-contract-check", "validate k8s docs values key contract"),
        ("load-docs-contract-check", "validate load docs suite/scenario contract"),
        ("make-help-drift-check", "validate make help output matches docs/development/make-targets.md"),
        ("no-removed-make-targets-check", "forbid references to removed public make targets"),
        ("ops-docs-make-targets-check", "validate operations docs reference make targets"),
        ("ops-observability-links-check", "validate observability docs local links resolve"),
        ("public-targets-docs-sections-check", "validate every public target appears in generated docs"),
        ("suite-id-docs-check", "forbid file-name references where suite IDs are required"),
        ("configmap-env-docs-check", "validate ATLAS_* configmap keys are documented"),
        ("generated-contract-docs-check", "validate generated contract markdown is drift-free"),
        ("lint-depth", "enforce docs depth rubric"),
        ("lint-doc-contracts", "enforce docs/contracts markdown contract"),
        ("glossary-check", "validate glossary and banned terms policy"),
        ("contracts-index", "validate or generate docs contracts index"),
        ("runbook-map", "validate or generate docs runbook map index"),
        ("evidence-policy-page", "generate docs evidence policy page"),
        ("inventory", "generate docs command inventory page"),
        ("extract-code", "extract code blocks from docs"),
        ("run-blessed-snippets", "run extracted blessed docs snippets with network guardrails"),
        ("render-diagrams", "render docs diagrams"),
        ("lint-spelling", "run docs spelling checks"),
        ("spellcheck", "run docs spelling checks"),
        ("style", "run docs style checks"),
        ("generate", "run docs generators"),
        ("rewrite-legacy-terms", "explicit legacy-term rewrite command"),
    ):
        cmd = docs_sub.add_parser(name, help=help_text)
        cmd.add_argument("--report", choices=["text", "json"], default="text")
        cmd.add_argument("--fix", action="store_true")
        if name == "inventory":
            cmd.add_argument("--out")
        if name == "evidence-policy-page":
            cmd.add_argument("--out")
        if name == "lint-spelling":
            cmd.add_argument("--path", default="docs")
        if name == "spellcheck":
            cmd.add_argument("--path", default="docs")
        if name == "mkdocs-site-links-check":
            cmd.add_argument("--site-dir", default="artifacts/docs/site")
        if name == "rewrite-legacy-terms":
            cmd.add_argument("--path", default="docs")
            cmd.add_argument("--apply", action="store_true")
        if name == "generate":
            cmd.add_argument("--fail-fast", action="store_true")
