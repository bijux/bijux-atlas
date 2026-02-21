# Scope: ops area wrappers only.
# Public targets: none
SHELL := /bin/sh

OPS_ENV_SCHEMA ?= configs/ops/env.schema.json

ops-help: ## Show canonical ops runbook index
	@./bin/atlasctl ops help --report text

ops-surface: ## Print stable ops entrypoints from SSOT metadata
	@./bin/atlasctl ops surface --report text

ops-env-validate: ## Validate canonical ops environment contract
	@./bin/atlasctl ops env --report text validate --schema "$(OPS_ENV_SCHEMA)"

ops-env-print: ## Print canonical ops environment settings
	@./bin/atlasctl ops env --report text print --schema "$(OPS_ENV_SCHEMA)" --format json

pins/check: ## Validate unified reproducibility pins
	@./bin/atlasctl ops pins --report text check

pins/update: ## Refresh unified reproducibility pins
	@./bin/atlasctl ops pins --report text update

ops-check: ## Canonical ops check gate
	@./bin/atlasctl ops check --report text

ops-gen: ## Regenerate committed ops outputs
	@./bin/atlasctl ops gen --report text run

ops-gen-check: ## Regenerate ops outputs and fail on drift
	@./bin/atlasctl ops gen --report text check

ops-fmt: ## Ops formatting lane
	@./bin/atlasctl ops surface --report text --fix

ops-lint: ## Ops lint lane
	@./bin/atlasctl ops lint --report text

ops-test: ## Ops test lane (smoke)
	@./bin/atlasctl ops e2e --report text run --suite smoke

ops-up: ## Bring up full local ops environment
	@./bin/atlasctl ops up --report text

ops-down: ## Tear down full local ops environment
	@./bin/atlasctl ops down --report text

ops-clean: ## Clean generated ops outputs
	@./bin/atlasctl ops clean --report text

.PHONY: ops-help ops-surface ops-env-validate ops-env-print pins/check pins/update ops-check ops-gen ops-gen-check ops-fmt ops-lint ops-test ops-up ops-down ops-clean
