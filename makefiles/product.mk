# Scope: product-facing docker/chart/contracts bootstrap targets split from root.
# Public targets: none
SHELL := /bin/sh

bootstrap:
	@python3 --version
	@command -v pip >/dev/null 2>&1 || { echo "missing pip" >&2; exit 1; }
	@python3 -m pip install -r configs/docs/requirements.txt >/dev/null
	@command -v k6 >/dev/null 2>&1 || echo "k6 not found (optional for non-perf workflows)"
	@command -v kind >/dev/null 2>&1 || echo "kind not found (required for k8s e2e)"
	@command -v kubectl >/dev/null 2>&1 || echo "kubectl not found (required for k8s e2e)"

fmt: ## Run formatter checks
	@$(MAKE) -s internal/cargo/fmt

lint: ## Run lint checks
	@$(MAKE) -s internal/cargo/lint

test: ## Run test suite
	@$(MAKE) -s internal/cargo/test

audit: ## Run dependency and policy audits
	@$(MAKE) -s internal/cargo/audit

docs: ## Run docs verification lane
	@$(MAKE) -s docs/check

k8s: ## Run canonical k8s verification lane
	@$(MAKE) -s ops/smoke

load: ## Run canonical load verification lane
	@$(MAKE) -s ops-load-smoke

obs: ## Run canonical observability verification lane
	@$(MAKE) -s ops-obs-verify SUITE=cheap

docker-build:
	@IMAGE_TAG="$${DOCKER_IMAGE:-bijux-atlas:local}"; \
	IMAGE_VERSION="$${IMAGE_VERSION:-$$(git rev-parse --short=12 HEAD)}"; \
	VCS_REF="$${VCS_REF:-$$(git rev-parse HEAD)}"; \
	BUILD_DATE="$${BUILD_DATE:-$$(date -u +%Y-%m-%dT%H:%M:%SZ)}"; \
	RUST_VERSION="$${RUST_VERSION:-1.84.1}"; \
	docker build --pull=false -t "$$IMAGE_TAG" -f docker/images/runtime/Dockerfile \
	  --build-arg RUST_VERSION="$$RUST_VERSION" \
	  --build-arg IMAGE_VERSION="$$IMAGE_VERSION" \
	  --build-arg VCS_REF="$$VCS_REF" \
	  --build-arg BUILD_DATE="$$BUILD_DATE" \
	  --build-arg IMAGE_PROVENANCE="$${IMAGE_PROVENANCE:-$${IMAGE_TAG}}" \
	  .

docker-check: ## Docker fast checks: contracts + build + runtime smoke
	@$(MAKE) -s docker-contracts
	@$(MAKE) -s docker-build
	@$(MAKE) -s docker-smoke

docker-smoke:
	@./bin/bijux-atlas docker smoke --image "$${DOCKER_IMAGE:-bijux-atlas:local}"

docker-scan:
	@./bin/bijux-atlas docker scan --image "$${DOCKER_IMAGE:-bijux-atlas:local}"

docker-push:
	@if [ "$${CI:-0}" != "1" ]; then echo "docker-push is CI-only"; exit 2; fi
	@IMAGE_TAG="$${DOCKER_IMAGE:?DOCKER_IMAGE is required for docker-push}"; \
	docker push "$$IMAGE_TAG"

docker-release: ## CI-only docker release lane (build + contracts + push)
	@if [ "$${CI:-0}" != "1" ]; then echo "docker-release is CI-only"; exit 2; fi
	@$(MAKE) -s docker-check
	@$(MAKE) -s docker-push

chart-package:
	@mkdir -p artifacts/chart
	@helm package ops/k8s/charts/bijux-atlas --destination artifacts/chart

chart-verify:
	@helm lint ops/k8s/charts/bijux-atlas
	@helm template atlas ops/k8s/charts/bijux-atlas >/dev/null

chart-validate: ## Validate chart via lint/template and values contract schema checks
	@$(MAKE) chart-verify
	@./scripts/areas/contracts/generate_chart_values_schema.py
	@./scripts/areas/contracts/check_chart_values_contract.py

docker-contracts: ## Validate Docker layout/policy/no-latest contracts
	@./bin/bijux-atlas run ./scripts/areas/check/check-docker-layout.py
	@./bin/bijux-atlas run ./scripts/areas/check/check-docker-policy.py
	@./bin/bijux-atlas run ./scripts/areas/check/check-no-latest-tags.py
	@./bin/bijux-atlas run ./scripts/areas/check/check-docker-image-size.py

rename-lint: ## Enforce durable naming rules for docs/scripts and concept ownership
	@./bin/bijux-atlas run ./scripts/areas/docs/check-durable-naming.py
	@./scripts/areas/docs/check_duplicate_topics.sh

docs-lint-names: ## Enforce durable naming contracts, registries, and inventory
	@./bin/bijux-atlas run ./scripts/areas/docs/naming_inventory.py
	@./scripts/areas/docs/ban_legacy_terms.sh
	@./bin/bijux-atlas run ./scripts/areas/docs/check_observability_docs_checklist.py
	@./bin/bijux-atlas run ./scripts/areas/docs/check_no_orphan_docs.py
	@./bin/bijux-atlas run ./scripts/areas/docs/check_script_locations.py
	@./bin/bijux-atlas run ./scripts/areas/docs/check_runbook_map_registration.py
	@./bin/bijux-atlas run ./scripts/areas/docs/check_contract_doc_pairs.py
	@./bin/bijux-atlas run ./packages/bijux-atlas-scripts/src/bijux_atlas_scripts/load/validate_suite_manifest.py
	@./scripts/areas/docs/check_index_pages.sh

doctor: ## Print tool/env/path diagnostics and store doctor report
	@RUN_ID="$${RUN_ID:-doctor-$(MAKE_RUN_TS)}" ./bin/bijux-atlas run ./scripts/areas/layout/make_doctor.py

prereqs: ## Check required binaries and versions and store prereqs report
	@RUN_ID="$${RUN_ID:-prereqs-$(MAKE_RUN_TS)}" ./bin/bijux-atlas run ./scripts/areas/layout/make_prereqs.py --run-id "$${RUN_ID:-prereqs-$(MAKE_RUN_TS)}"

dataset-id-lint: ## Validate DatasetId/DatasetKey contract usage across ops fixtures
	@./bin/bijux-atlas run ./scripts/areas/layout/dataset_id_lint.py

internal/tooling-versions:
	@echo "Rust toolchain (rust-toolchain.toml):"
	@grep '^channel' rust-toolchain.toml | sed -E 's/channel *= *\"([^\"]+)\"/  channel=\1/'
	@echo "Python tooling pins (configs/ops/pins/tools.json):"
	@python3 -c 'import json; from pathlib import Path; pins=json.loads(Path("configs/ops/pins/tools.json").read_text(encoding="utf-8"))["tools"]; [print("  {}={}".format(k, pins[k]["version"])) for k in ("python3","pip-tools","uv","ruff","mypy") if k in pins]'
	@echo "Local binaries:"
	@python3 --version | sed 's/^/  /'
	@{ command -v uv >/dev/null 2>&1 && uv --version | sed 's/^/  /'; } || echo "  uv=missing"
	@{ command -v ruff >/dev/null 2>&1 && ruff --version | sed 's/^/  /'; } || echo "  ruff=missing"
	@{ command -v mypy >/dev/null 2>&1 && mypy --version | sed 's/^/  /'; } || echo "  mypy=missing"

internal/packages/check:
	@python3 -m venv artifacts/isolate/py/packages-check/.venv
	@artifacts/isolate/py/packages-check/.venv/bin/pip --disable-pip-version-check install --upgrade pip >/dev/null
	@artifacts/isolate/py/packages-check/.venv/bin/pip --disable-pip-version-check install -e packages/bijux-atlas-scripts -e packages/bijux-atlas-py >/dev/null
	@artifacts/isolate/py/packages-check/.venv/bin/python -c "import bijux_atlas_scripts, bijux_atlas_py"
	@python3 ops/_lint/check-surfaces.py
	@python3 ops/_lint/no-root-ad-hoc-python.py
	@python3 ops/_lint/no-direct-bash-entrypoints.py
	@python3 ops/_lint/no-duplicate-cli.py
	@python3 ops/_lint/no-scripts-dir.py
