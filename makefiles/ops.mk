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

ops-check-all: ## Full ops check gate (includes slow/k8s/docker validations)
	@./bin/atlasctl ops check --report text --all

ops-gen: ## Regenerate committed ops outputs
	@./bin/atlasctl ops gen --report text run

ops-gen-check: ## Regenerate ops outputs and fail on drift
	@./bin/atlasctl ops gen --report text check

ops-fmt: ## Ops formatting lane
	@./bin/atlasctl ops surface --report text --fix

ops-lint: ## Ops lint lane
	@./bin/atlasctl ops lint --report text

ops-lint-all: ## Ops lint lane (full validation set)
	@./bin/atlasctl ops lint --report text --all

ops-test: ## Ops test lane (smoke)
	@./bin/atlasctl ops e2e --report text run --suite smoke

ops-test-all: ## Ops test lane (full e2e suite)
	@./bin/atlasctl ops e2e --report text run --suite realdata

ops-up: ## Bring up full local ops environment
	@./bin/atlasctl ops up --report text

ops-down: ## Tear down full local ops environment
	@./bin/atlasctl ops down --report text

ops-clean: ## Clean generated ops outputs
	@./bin/atlasctl ops clean --report text

.PHONY: ops-help ops-surface ops-env-validate ops-env-print pins/check pins/update ops-check ops-check-all ops-gen ops-gen-check ops-fmt ops-lint ops-lint-all ops-test ops-test-all ops-up ops-down ops-clean
