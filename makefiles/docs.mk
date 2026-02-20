# Scope: docs area internal targets and wrappers.
# Public targets: none
SHELL := /bin/sh

DOCS_ARTIFACTS ?= $(if $(ISO_ROOT),$(ISO_ROOT)/docs,artifacts/docs)
DOCS_VENV ?= $(DOCS_ARTIFACTS)/.venv
DOCS_REQ ?= configs/docs/requirements.lock.txt
DOCS_SITE ?= $(DOCS_ARTIFACTS)/site

docs-req-lock-refresh: ## Refresh docs requirements lock deterministically
	@./bin/bijux-atlas run -m venv "$(DOCS_VENV)"
	@"$(DOCS_VENV)/bin/pip" install --upgrade pip >/dev/null
	@"$(DOCS_VENV)/bin/pip" install -r configs/docs/requirements.txt >/dev/null
	@"$(DOCS_VENV)/bin/pip" freeze --exclude-editable | LC_ALL=C sort > "$(DOCS_REQ)"

_docs-venv:
	@mkdir -p "$(DOCS_ARTIFACTS)"
	@$(call py_venv,$(DOCS_VENV),"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null)

docs-build: ## Build docs + link-check + spell-check + lint
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@./bin/bijux-atlas run scripts/areas/docs/generate_crates_map.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_architecture_map.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_make_targets_inventory.py
	@./bin/bijux-atlas run scripts/areas/docs/check_make_targets_drift.py
	@./bin/bijux-atlas run scripts/areas/docs/check_make_help_drift.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_k8s_values_doc.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_concept_graph.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_openapi_docs.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_observability_surface.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_ops_badge.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_ops_schema_docs.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_ops_surface.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_ops_contracts_doc.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_make_targets_catalog.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_config_keys_doc.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_env_vars_doc.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_contracts_index_doc.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_chart_contract_index.py
	@./bin/bijux-atlas run scripts/areas/ops/generate_k8s_test_surface.py
	@./bin/bijux-atlas run scripts/areas/docs/generate_runbook_map_index.py
	@./bin/bijux-atlas run scripts/areas/docs/check_concept_registry.py
	@./scripts/areas/docs/render_diagrams.sh
	@./bin/bijux-atlas run scripts/areas/docs/lint_doc_status.py
	@SOURCE_DATE_EPOCH=946684800 "$(DOCS_VENV)/bin/mkdocs" build --strict --config-file mkdocs.yml --site-dir "$(DOCS_SITE)"
	@./bin/bijux-atlas run scripts/areas/docs/check_mkdocs_site_links.py "$(DOCS_SITE)"
	@./bin/bijux-atlas run scripts/areas/docs/spellcheck_docs.py docs
	@./scripts/areas/docs/check_doc_naming.sh
	@./scripts/areas/docs/ban_legacy_terms.sh
	@./scripts/areas/docs/check_index_pages.sh
	@./scripts/areas/docs/check_title_case.sh
	@./bin/bijux-atlas run scripts/areas/docs/check_no_orphan_docs.py
	@./bin/bijux-atlas run scripts/areas/docs/check_ops_observability_links.py
	@./bin/bijux-atlas run scripts/areas/docs/lint_doc_contracts.py
	@./bin/bijux-atlas run scripts/areas/docs/check_nav_order.py
	@./bin/bijux-atlas run scripts/areas/docs/check_adr_headers.py
	@if command -v vale >/dev/null 2>&1; then vale --config=configs/docs/.vale.ini docs; else echo "vale not found; using contract style linter + codespell"; fi
	@./bin/bijux-atlas run scripts/areas/docs/check_runbooks_contract.py
	@./bin/bijux-atlas run scripts/areas/docs/check_k8s_docs_contract.py
	@./bin/bijux-atlas run scripts/areas/docs/check_load_docs_contract.py
	@./bin/bijux-atlas run scripts/areas/docs/check_ops_docs_make_targets.py
	@./bin/bijux-atlas run scripts/areas/docs/check_configmap_env_docs.py
	@./bin/bijux-atlas run scripts/areas/docs/check_docs_make_only.py
	@./bin/bijux-atlas run scripts/areas/docs/check_no_placeholders.py
	@./bin/bijux-atlas run scripts/areas/docs/check_broken_examples.py
	@./bin/bijux-atlas run scripts/areas/docs/check_example_configs.py
	@./bin/bijux-atlas run scripts/areas/docs/check_openapi_examples.py
	@./bin/bijux-atlas run scripts/areas/docs/check_generated_contract_docs.py
	@./bin/bijux-atlas run scripts/areas/docs/check_docker_entrypoints.py
	@./bin/bijux-atlas run scripts/areas/docs/check_terminology_units_ssot.py
	@./bin/bijux-atlas run scripts/areas/docs/lint_glossary_links.py
	@./bin/bijux-atlas run scripts/areas/docs/lint_depth.py
	@./bin/bijux-atlas run scripts/areas/docs/extract_code_blocks.py
	@./bin/bijux-atlas run scripts/areas/docs/run_blessed_snippets.py
	@./scripts/areas/public/check-markdown-links.sh
	@./scripts/areas/docs/check_duplicate_topics.sh
	@./scripts/areas/docs/check_crate_docs_contract.sh
	@./bin/bijux-atlas run scripts/areas/docs/check_script_headers.py
	@./bin/bijux-atlas run scripts/areas/docs/check_make_targets_documented.py
	@./bin/bijux-atlas run scripts/areas/docs/check_public_targets_docs_sections.py
	@./bin/bijux-atlas run scripts/areas/docs/check_docs_make_targets_exist.py
	@./bin/bijux-atlas run scripts/areas/docs/check_critical_make_targets_referenced.py
	@./bin/bijux-atlas run scripts/areas/docs/check_contracts_index_nav.py
	@./bin/bijux-atlas run scripts/areas/docs/check_doc_filename_style.py
	@./bin/bijux-atlas run scripts/areas/docs/check_docs_deterministic.py
	@./bin/bijux-atlas run scripts/areas/docs/check_observability_docs_checklist.py
	@./bin/bijux-atlas run scripts/areas/docs/check_no_legacy_root_paths.py
	@./bin/bijux-atlas run scripts/areas/docs/check_no_removed_make_targets.py
	@./bin/bijux-atlas run scripts/areas/layout/check_no_legacy_targets_in_docs.py
	@./bin/bijux-atlas run scripts/areas/layout/check_ops_external_entrypoints.py
	@./bin/bijux-atlas run scripts/areas/docs/check_full_stack_page.py

docs-serve: ## Serve docs locally
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@SOURCE_DATE_EPOCH=946684800 "$(DOCS_VENV)/bin/mkdocs" serve --config-file mkdocs.yml

docs-freeze: ## Generated docs must be up-to-date with SSOT contracts
	@./bin/bijux-atlas run scripts/areas/docs/check_docs_freeze_drift.py

docs-hardening: ## Run full docs hardening pipeline
	@$(MAKE) docs-build
	@$(MAKE) docs-freeze

docs-all: ## Canonical all-docs gate: must pass all docs sub-gates
	@$(MAKE) docs-hardening
	@$(MAKE) docs-lint-names

docs-check: ## Docs contract check alias (same as docs-build)
	@./bin/bijux-atlas docs check --report text --emit-artifacts

internal/docs/public: ## Public docs alias implementation (root wrapper only)
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

.PHONY: docs-all docs-build docs-check docs-serve docs-freeze docs-hardening docs-req-lock-refresh internal/docs/public internal/docs/check internal/docs/build internal/docs/fmt internal/docs/lint internal/docs/test internal/docs/clean internal/docs/all _docs-venv
