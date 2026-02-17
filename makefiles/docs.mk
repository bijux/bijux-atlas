SHELL := /bin/sh

DOCS_ARTIFACTS ?= $(if $(ISO_ROOT),$(ISO_ROOT)/docs,artifacts/docs)
DOCS_VENV ?= $(DOCS_ARTIFACTS)/.venv
DOCS_REQ ?= configs/docs/requirements.txt
DOCS_SITE ?= $(DOCS_ARTIFACTS)/site

_docs-venv:
	@mkdir -p "$(DOCS_ARTIFACTS)"
	@python3 -m venv "$(DOCS_VENV)"
	@"$(DOCS_VENV)/bin/pip" install --upgrade pip >/dev/null
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null

docs: ## Build docs + link-check + spell-check + lint
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@python3 scripts/docs/generate_concept_graph.py
	@python3 scripts/docs/check_concept_registry.py
	@"$(DOCS_VENV)/bin/mkdocs" build --strict --config-file mkdocs.yml --site-dir "$(DOCS_SITE)"
	@"$(DOCS_VENV)/bin/python" scripts/docs/check_mkdocs_site_links.py "$(DOCS_SITE)"
	@"$(DOCS_VENV)/bin/python" scripts/docs/spellcheck_docs.py docs
	@./scripts/docs/check_doc_naming.sh
	@./scripts/docs/check_index_pages.sh
	@./scripts/docs/check_title_case.sh
	@python3 scripts/docs/check_no_orphan_docs.py
	@python3 scripts/docs/lint_doc_contracts.py
	@if command -v vale >/dev/null 2>&1; then vale docs; else echo "vale not found; using contract style linter + codespell"; fi
	@python3 scripts/docs/check_runbooks_contract.py
	@python3 scripts/docs/check_k8s_docs_contract.py
	@python3 scripts/docs/check_load_docs_contract.py
	@python3 scripts/docs/check_broken_examples.py
	@python3 scripts/docs/check_terminology_units_ssot.py
	@./scripts/check-markdown-links.sh
	@./scripts/docs/check_duplicate_topics.sh

docs-serve: ## Serve docs locally
	@if [ ! -x "$(DOCS_VENV)/bin/mkdocs" ]; then $(MAKE) _docs-venv; fi
	@"$(DOCS_VENV)/bin/pip" install -r "$(DOCS_REQ)" >/dev/null
	@"$(DOCS_VENV)/bin/mkdocs" serve --config-file mkdocs.yml

docs-freeze: ## Generated docs must be up-to-date with SSOT contracts
	@./scripts/contracts/generate_contract_artifacts.py
	@if ! git diff --quiet -- docs/_generated/contracts; then \
		echo "docs freeze failed: docs/_generated/contracts drift detected" >&2; \
		git --no-pager diff -- docs/_generated/contracts >&2 || true; \
		exit 1; \
	fi
	@if ! git diff --quiet -- docs/contracts/errors.md docs/contracts/metrics.md docs/contracts/tracing.md docs/contracts/endpoints.md docs/contracts/config-keys.md docs/contracts/chart-values.md; then \
		echo "docs freeze failed: generated docs/contracts/*.md drift detected" >&2; \
		git --no-pager diff -- docs/contracts/errors.md docs/contracts/metrics.md docs/contracts/tracing.md docs/contracts/endpoints.md docs/contracts/config-keys.md docs/contracts/chart-values.md >&2 || true; \
		exit 1; \
	fi

.PHONY: docs docs-serve docs-freeze _docs-venv
