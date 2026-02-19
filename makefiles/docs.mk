SHELL := /bin/sh

DOCS_ARTIFACTS ?= $(if $(ISO_ROOT),$(ISO_ROOT)/docs,artifacts/docs)
DOCS_VENV ?= $(DOCS_ARTIFACTS)/.venv
DOCS_REQ ?= configs/docs/requirements.lock.txt
DOCS_SITE ?= $(DOCS_ARTIFACTS)/site

docs-req-lock-refresh: ## Refresh docs requirements lock deterministically
	@python3 -m venv "$(DOCS_VENV)"
	@"$(DOCS_VENV)/bin/pip" install --upgrade pip >/dev/null
	@"$(DOCS_VENV)/bin/pip" install -r configs/docs/requirements.txt >/dev/null
	@"$(DOCS_VENV)/bin/pip" freeze --exclude-editable | LC_ALL=C sort > "$(DOCS_REQ)"

_docs-venv:
	@mkdir -p "$(DOCS_ARTIFACTS)"
	@$(call py_venv,$(DOCS_VENV),"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null)

docs-build: ## Build docs + link-check + spell-check + lint
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@python3 scripts/docs/generate_crates_map.py
	@python3 scripts/docs/generate_architecture_map.py
	@python3 scripts/docs/generate_make_targets_inventory.py
	@python3 scripts/docs/check_make_targets_drift.py
	@python3 scripts/docs/check_make_help_drift.py
	@python3 scripts/docs/generate_k8s_values_doc.py
	@python3 scripts/docs/generate_concept_graph.py
	@python3 scripts/docs/generate_openapi_docs.py
	@python3 scripts/docs/generate_observability_surface.py
	@python3 scripts/docs/generate_ops_schema_docs.py
	@python3 scripts/docs/generate_ops_surface.py
	@python3 scripts/docs/generate_ops_contracts_doc.py
	@python3 scripts/docs/check_concept_registry.py
	@./scripts/docs/render_diagrams.sh
	@python3 scripts/docs/lint_doc_status.py
	@SOURCE_DATE_EPOCH=946684800 "$(DOCS_VENV)/bin/mkdocs" build --strict --config-file mkdocs.yml --site-dir "$(DOCS_SITE)"
	@"$(DOCS_VENV)/bin/python" scripts/docs/check_mkdocs_site_links.py "$(DOCS_SITE)"
	@"$(DOCS_VENV)/bin/python" scripts/docs/spellcheck_docs.py docs
	@./scripts/docs/check_doc_naming.sh
	@./scripts/docs/ban_legacy_terms.sh
	@./scripts/docs/check_index_pages.sh
	@./scripts/docs/check_title_case.sh
	@python3 scripts/docs/check_no_orphan_docs.py
	@python3 scripts/docs/check_ops_observability_links.py
	@python3 scripts/docs/lint_doc_contracts.py
	@python3 scripts/docs/check_nav_order.py
	@python3 scripts/docs/check_adr_headers.py
	@if command -v vale >/dev/null 2>&1; then vale docs; else echo "vale not found; using contract style linter + codespell"; fi
	@python3 scripts/docs/check_runbooks_contract.py
	@python3 scripts/docs/check_k8s_docs_contract.py
	@python3 scripts/docs/check_load_docs_contract.py
	@python3 scripts/docs/check_ops_docs_make_targets.py
	@python3 scripts/docs/check_docs_make_only.py
	@python3 scripts/docs/check_no_placeholders.py
	@python3 scripts/docs/check_broken_examples.py
	@python3 scripts/docs/check_example_configs.py
	@python3 scripts/docs/check_openapi_examples.py
	@python3 scripts/docs/check_generated_contract_docs.py
	@python3 scripts/docs/check_docker_entrypoints.py
	@python3 scripts/docs/check_terminology_units_ssot.py
	@python3 scripts/docs/lint_glossary_links.py
	@python3 scripts/docs/lint_depth.py
	@python3 scripts/docs/extract_code_blocks.py
	@python3 scripts/docs/run_blessed_snippets.py
	@./scripts/public/check-markdown-links.sh
	@./scripts/docs/check_duplicate_topics.sh
	@./scripts/docs/check_crate_docs_contract.sh
	@python3 scripts/docs/check_script_headers.py
	@python3 scripts/docs/check_make_targets_documented.py
	@python3 scripts/docs/check_docs_make_targets_exist.py
	@python3 scripts/docs/check_critical_make_targets_referenced.py
	@python3 scripts/docs/check_doc_filename_style.py
	@python3 scripts/docs/check_docs_deterministic.py
	@python3 scripts/docs/check_observability_docs_checklist.py
	@python3 scripts/docs/check_no_legacy_root_paths.py
	@python3 scripts/docs/check_full_stack_page.py

docs-serve: ## Serve docs locally
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@SOURCE_DATE_EPOCH=946684800 "$(DOCS_VENV)/bin/mkdocs" serve --config-file mkdocs.yml

docs-freeze: ## Generated docs must be up-to-date with SSOT contracts
	@python3 scripts/docs/check_docs_freeze_drift.py

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

.PHONY: docs docs-all docs-build docs-check docs-serve docs-freeze docs-hardening docs-req-lock-refresh _docs-venv
