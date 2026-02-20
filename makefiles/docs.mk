# Scope: docs area internal targets and wrappers.
# Public targets: none
SHELL := /bin/sh

DOCS_ARTIFACTS ?= $(if $(ISO_ROOT),$(ISO_ROOT)/docs,artifacts/docs)
DOCS_VENV ?= $(DOCS_ARTIFACTS)/.venv
DOCS_REQ ?= configs/docs/requirements.lock.txt
DOCS_SITE ?= $(DOCS_ARTIFACTS)/site

docs-req-lock-refresh: ## Refresh docs requirements lock deterministically
	@$(ATLAS_SCRIPTS) run -m venv "$(DOCS_VENV)"
	@"$(DOCS_VENV)/bin/pip" install --upgrade pip >/dev/null
	@"$(DOCS_VENV)/bin/pip" install -r configs/docs/requirements.txt >/dev/null
	@"$(DOCS_VENV)/bin/pip" freeze --exclude-editable | LC_ALL=C sort > "$(DOCS_REQ)"

_docs-venv:
	@mkdir -p "$(DOCS_ARTIFACTS)"
	@$(call py_venv,$(DOCS_VENV),"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null)

docs-build: ## Build docs + link-check + spell-check + lint
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@$(ATLAS_SCRIPTS) docs generate --report text
	@$(ATLAS_SCRIPTS) gen make-targets
	@$(ATLAS_SCRIPTS) gen surface
	@$(ATLAS_SCRIPTS) gen scripting-surface
	@$(ATLAS_SCRIPTS) docs render-diagrams --report text
	@$(ATLAS_SCRIPTS) docs style --report text
	@SOURCE_DATE_EPOCH=946684800 "$(DOCS_VENV)/bin/mkdocs" build --strict --config-file mkdocs.yml --site-dir "$(DOCS_SITE)"
	@$(ATLAS_SCRIPTS) docs nav-check --report text
	@$(ATLAS_SCRIPTS) docs spellcheck --path docs --report text
	@$(ATLAS_SCRIPTS) docs lint --report text
	@if command -v vale >/dev/null 2>&1; then vale --config=configs/docs/.vale.ini docs; else echo "vale not found; using contract style linter + codespell"; fi
	@$(ATLAS_SCRIPTS) docs extract-code --report text
	@$(ATLAS_SCRIPTS) docs link-check --report text
	@$(ATLAS_SCRIPTS) docs check --report text

docs-serve: ## Serve docs locally
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@SOURCE_DATE_EPOCH=946684800 "$(DOCS_VENV)/bin/mkdocs" serve --config-file mkdocs.yml

docs-freeze: ## Generated docs must be up-to-date with SSOT contracts
	@$(ATLAS_SCRIPTS) docs generated-check --report text

docs-hardening: ## Run full docs hardening pipeline
	@$(MAKE) docs-build
	@$(MAKE) docs-freeze

docs-all: ## Canonical all-docs gate: must pass all docs sub-gates
	@$(MAKE) docs-hardening
	@$(MAKE) docs-lint-names

docs-check: ## Docs contract check alias (same as docs-build)
	@$(ATLAS_SCRIPTS) docs check --report text --emit-artifacts

internal/docs/public: ## Public docs alias implementation (root wrapper only)
	@$(MAKE) docs-check

internal/docs/check: ## Fast docs verification
	@start="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; status=pass; fail=""; \
	if ! $(MAKE) docs-freeze; then status=fail; fail="docs-freeze failed"; fi; \
	end="$$(date -u +%Y-%m-%dT%H:%M:%SZ)"; \
	PYTHONPATH=packages/bijux-atlas-scripts/src python3 -m bijux_atlas_scripts.reporting.make_area_report --path "$${ISO_ROOT:-artifacts/isolate/docs/$${RUN_ID:-docs-check}}/report.docs.check.json" --lane "docs/check" --status "$$status" --start "$$start" --end "$$end" --artifact "$${ISO_ROOT:-artifacts/isolate/docs/$${RUN_ID:-docs-check}}" --failure "$$fail" >/dev/null; \
	[ "$$status" = "pass" ] || { $(call fail_banner,docs/check); exit 1; }

internal/docs/build: ## Build docs artifacts
	@$(MAKE) docs-build

internal/docs/fmt: ## Docs formatting helpers
	@$(ATLAS_SCRIPTS) docs render-diagrams --report text

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
