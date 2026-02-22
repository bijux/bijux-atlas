# Scope: deprecated scripts surface.
# Atlasctl is the only supported control plane; keep this file intentionally empty.
SHELL := /bin/sh

deps-check-venv: ## Validate dependency install/import in a clean temporary venv
	@./bin/atlasctl deps check-venv

deps-cold-start: ## Measure atlasctl import cold-start budget
	@./bin/atlasctl deps cold-start --runs 3 --max-ms 500

deps-lock: ## Refresh python lockfile deterministically via atlasctl
	@./bin/atlasctl deps lock

deps-sync: ## Install dependencies from lock into active interpreter
	@./bin/atlasctl deps sync
