# Scope: docs area internal targets and wrappers.
# Public targets: none
SHELL := /bin/sh

DOCS_ARTIFACTS ?= $(if $(ISO_ROOT),$(ISO_ROOT)/docs,artifacts/docs)
DOCS_VENV ?= $(DOCS_ARTIFACTS)/.venv
DOCS_REQ ?= configs/docs/requirements.lock.txt
DOCS_SITE ?= $(DOCS_ARTIFACTS)/site

docs-req-lock-refresh: ## Refresh docs requirements lock deterministically
	@./scripts/bin/bijux-atlas-scripts run -m venv "$(DOCS_VENV)"
	@"$(DOCS_VENV)/bin/pip" install --upgrade pip >/dev/null
	@"$(DOCS_VENV)/bin/pip" install -r configs/docs/requirements.txt >/dev/null
	@"$(DOCS_VENV)/bin/pip" freeze --exclude-editable | LC_ALL=C sort > "$(DOCS_REQ)"

_docs-venv:
	@mkdir -p "$(DOCS_ARTIFACTS)"
	@$(call py_venv,$(DOCS_VENV),"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null)

docs-build: ## Build docs + link-check + spell-check + lint
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_crates_map.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_architecture_map.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_make_targets_inventory.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_make_targets_drift.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_make_help_drift.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_k8s_values_doc.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_concept_graph.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_openapi_docs.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_observability_surface.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_ops_badge.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_ops_schema_docs.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_ops_surface.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_ops_contracts_doc.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_make_targets_catalog.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_config_keys_doc.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_env_vars_doc.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_contracts_index_doc.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_chart_contract_index.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/ops/generate_k8s_test_surface.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/generate_runbook_map_index.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_concept_registry.py
	@./scripts/areas/docs/render_diagrams.sh
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/lint_doc_status.py
	@SOURCE_DATE_EPOCH=946684800 "$(DOCS_VENV)/bin/mkdocs" build --strict --config-file mkdocs.yml --site-dir "$(DOCS_SITE)"
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_mkdocs_site_links.py "$(DOCS_SITE)"
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/spellcheck_docs.py docs
	@./scripts/areas/docs/check_doc_naming.sh
	@./scripts/areas/docs/ban_legacy_terms.sh
	@./scripts/areas/docs/check_index_pages.sh
	@./scripts/areas/docs/check_title_case.sh
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_no_orphan_docs.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_ops_observability_links.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/lint_doc_contracts.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_nav_order.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_adr_headers.py
	@if command -v vale >/dev/null 2>&1; then vale docs; else echo "vale not found; using contract style linter + codespell"; fi
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_runbooks_contract.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_k8s_docs_contract.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_load_docs_contract.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_ops_docs_make_targets.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_configmap_env_docs.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_docs_make_only.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_no_placeholders.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_broken_examples.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_example_configs.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_openapi_examples.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_generated_contract_docs.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_docker_entrypoints.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_terminology_units_ssot.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/lint_glossary_links.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/lint_depth.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/extract_code_blocks.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/run_blessed_snippets.py
	@./scripts/areas/public/check-markdown-links.sh
	@./scripts/areas/docs/check_duplicate_topics.sh
	@./scripts/areas/docs/check_crate_docs_contract.sh
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_script_headers.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_make_targets_documented.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_public_targets_docs_sections.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_docs_make_targets_exist.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_critical_make_targets_referenced.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_contracts_index_nav.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_doc_filename_style.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_docs_deterministic.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_observability_docs_checklist.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_no_legacy_root_paths.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/layout/check_no_legacy_targets_in_docs.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/layout/check_ops_external_entrypoints.py
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_full_stack_page.py

docs-serve: ## Serve docs locally
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@SOURCE_DATE_EPOCH=946684800 "$(DOCS_VENV)/bin/mkdocs" serve --config-file mkdocs.yml

docs-freeze: ## Generated docs must be up-to-date with SSOT contracts
	@./scripts/bin/bijux-atlas-scripts run scripts/areas/docs/check_docs_freeze_drift.py

docs-hardening: ## Run full docs hardening pipeline
	@$(MAKE) docs-build
	@$(MAKE) docs-freeze

docs-all: ## Canonical all-docs gate: must pass all docs sub-gates
	@$(MAKE) docs-hardening
	@$(MAKE) docs-lint-names

docs-check: ## Docs contract check alias (same as docs-build)
	@$(MAKE) docs-build

docs: ## Public docs alias (maps to docs-check only)
	@$(MAKE) docs-check

internal/docs/check: ## Fast docs verification
	@start="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; status=pass; fail=""; \
	if ! $(MAKE) docs-freeze; then status=fail; fail="docs-freeze failed"; fi; \
	end="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	python3 ./scripts/areas/layout/write_make_area_report.py --path "$${ISO_ROOT:-artifacts/isolate/docs/$${RUN_ID:-docs-check}}/report.docs.check.json" --lane "docs/check" --status "$$status" --start "$$start" --end "$$end" --artifact "$${ISO_ROOT:-artifacts/isolate/docs/$${RUN_ID:-docs-check}}" --failure "$$fail" >/dev/null; \
	[ "$$status" = "pass" ] || { $(call fail_banner,docs/check); exit 1; }

internal/docs/build: ## Build docs artifacts
	@$(MAKE) docs-build

internal/docs/fmt: ## Docs formatting helpers
	@./scripts/areas/docs/render_diagrams.sh

internal/docs/lint: ## Docs lint checks
	@$(MAKE) docs-lint-names

internal/docs/test: ## Docs tests/contract checks
	@$(MAKE) internal/docs/check

internal/docs/clean: ## Clean docs generated outputs only
	@rm -rf artifacts/docs

internal/docs/all: ## Uniform docs all target
	@$(MAKE) internal/docs/check
	@$(MAKE) internal/docs/lint
	@$(MAKE) internal/docs/build

.PHONY: docs docs-all docs-build docs-check docs-serve docs-freeze docs-hardening docs-req-lock-refresh internal/docs/check internal/docs/build internal/docs/fmt internal/docs/lint internal/docs/test internal/docs/clean internal/docs/all _docs-venv
