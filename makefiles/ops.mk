# Scope: ops area wrappers only.
# Public targets: ops, ops-up, ops-down, ops-clean, ops-pins-check, ops-pins-update
SHELL := /bin/sh

ops: ## Canonical ops gate
	@./bin/atlasctl check run --group ops --quiet --show-skips --durations 10

ops-up: ## Bring up full local ops environment
	@ATLAS_E2E_NAMESPACE=atlas-e2e ATLAS_NS=atlas-e2e ./bin/atlasctl ops up --report text

ops-down: ## Tear down full local ops environment
	@ATLAS_E2E_NAMESPACE=atlas-e2e ATLAS_NS=atlas-e2e ./bin/atlasctl ops down --report text

ops-clean: ## Clean generated ops outputs
	@./bin/atlasctl ops clean --report text

ops-pins-check: ## Validate unified reproducibility pins
	@./bin/atlasctl check run --group ops -k pins --durations 10

ops-pins-update: ## Refresh unified reproducibility pins
	@./bin/atlasctl ops pins update --i-know-what-im-doing

.PHONY: ops ops-up ops-down ops-clean ops-pins-check ops-pins-update
