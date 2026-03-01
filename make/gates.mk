# Scope: progressive aggregate gates that orchestrate existing make targets only.
# Public targets: gate-10, gate-20
SHELL := /bin/sh

gate-10: ## Run the first 10 high-signal gates in parallel via make
	@$(MAKE) -j10 check fmt lint audit test coverage configs docs ops policies

gate-20: ## Run the second 10 release-readiness gates in parallel via make
	@$(MAKE) -j10 docker verify lint-root lint-makefiles lint-configs lint-docs lint-ops lint-docker lint-policies lanes

.PHONY: gate-10 gate-20
