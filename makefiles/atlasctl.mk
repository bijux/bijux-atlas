# Scope: atlasctl policy/check wrappers.
# These targets run atlasctl's own check domains directly.
SHELL := /bin/sh
PYTHONPATH ?= packages/atlasctl/src
ATLASCTL_ARTIFACT_ROOT ?= artifacts/atlasctl
atlasctl-check: ## Run all atlasctl checks across all groups
	@./bin/atlasctl check run --profile fast --durations 10 --quiet --show-skips

atlasctl-check-all: ## Run all atlasctl checks including slow checks
	@./bin/atlasctl check run --profile all --all --timeout-ms 30000 --ignore-speed-regressions --durations 20 --quiet --show-skips

atlasctl-check-contracts: ## Run atlasctl contracts checks
	@./bin/atlasctl check run --group contracts --durations 10

atlasctl-check-docs: ## Run atlasctl docs checks
	@./bin/atlasctl check run --group docs --durations 10

atlasctl-check-layout: ## Validate repository layout/root-shape checks
	@./bin/atlasctl check layout

atlasctl-check-make: ## Run atlasctl makefile checks
	@./bin/atlasctl check run --group make --durations 10

atlasctl-check-ops: ## Run atlasctl ops checks
	@./bin/atlasctl check run --group ops --durations 10

atlasctl-check-python: ## Run atlasctl python checks
	@./bin/atlasctl check run --group python --durations 10

atlasctl-check-repo: ## Run atlasctl repo checks
	@./bin/atlasctl check run --group repo --durations 10

atlasctl-registry-list: ## Print atlasctl registry inventory
	@./bin/atlasctl registry checks

atlasctl/internal: ## Run all atlasctl internal helper gates (quiet; prints logs on failure)
	@targets="atlasctl/internal/cli-check atlasctl/internal/deps/check-venv atlasctl/internal/deps/cold-start atlasctl/internal/deps/lock atlasctl/internal/deps/sync"; \
	total=0; failed=0; \
	for target in $$targets; do \
		total=$$((total + 1)); \
		log="artifacts/reports/atlasctl/internal-$${target##*/}.log"; \
		mkdir -p "$$(dirname "$$log")"; \
		if $(MAKE) --no-print-directory -s "$$target" >"$$log" 2>&1; then \
			printf 'PASS %s\n' "$$target"; \
		else \
			failed=$$((failed + 1)); \
			printf 'FAIL %s (log: %s)\n' "$$target" "$$log"; \
			cat "$$log"; \
		fi; \
	done; \
	printf 'atlasctl/internal summary: total=%s failed=%s\n' "$$total" "$$failed"; \
	test "$$failed" -eq 0

atlasctl/internal/cli-check:
	@./bin/atlasctl --version >/dev/null 2>&1 || { \
		echo "atlasctl CLI is not runnable via ./bin/atlasctl"; \
		echo "run: make atlasctl/internal/deps/sync or make dev-bootstrap"; \
		exit 2; \
	}

atlasctl/internal/deps/check-venv: ## Validate dependency install/import in a clean temporary venv
	@./bin/atlasctl --quiet deps check-venv

atlasctl/internal/deps/cold-start: ## Measure atlasctl import cold-start budget
	@./bin/atlasctl --quiet deps cold-start --runs 3 --max-ms 500

atlasctl/internal/deps/lock: ## Refresh python lockfile deterministically via atlasctl
	@./bin/atlasctl --quiet deps lock

atlasctl/internal/deps/sync: ## Install dependencies from lock into active interpreter
	@./bin/atlasctl --quiet deps sync

.PHONY: atlasctl-check atlasctl-check-all atlasctl-check-contracts atlasctl-check-docs atlasctl-check-layout atlasctl-check-make atlasctl-check-ops atlasctl-check-python atlasctl-check-repo atlasctl-registry-list atlasctl/internal atlasctl/internal/cli-check atlasctl/internal/deps/check-venv atlasctl/internal/deps/cold-start atlasctl/internal/deps/lock atlasctl/internal/deps/sync
