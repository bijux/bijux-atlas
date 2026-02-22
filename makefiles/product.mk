# Scope: product-facing docker/chart/contracts bootstrap targets split from root.
# Public targets: none
SHELL := /bin/sh

bootstrap:
	@./bin/atlasctl run ./ops/run/product/product_bootstrap.sh

k8s: ## Run canonical k8s verification lane
	@$(MAKE) -s ops/smoke

load: ## Run canonical load verification lane
	@$(MAKE) -s ops-load-smoke

obs: ## Run canonical observability verification lane
	@$(MAKE) -s ops-obs-verify SUITE=cheap

docker-build:
	@./bin/atlasctl run ./ops/run/product/product_docker_build.sh

docker-check: ## Docker fast checks: contracts + build + runtime smoke
	@./bin/atlasctl run ./ops/run/product/product_docker_check.sh

docker-smoke:
	@./bin/atlasctl docker smoke --image "$${DOCKER_IMAGE:-bijux-atlas:local}"

docker-scan:
	@./bin/atlasctl docker scan --image "$${DOCKER_IMAGE:-bijux-atlas:local}"

docker-push:
	@./bin/atlasctl run ./ops/run/product/product_docker_push.sh

docker-release: ## CI-only docker release lane (build + contracts + push)
	@./bin/atlasctl run ./ops/run/product/product_docker_release.sh

chart-package:
	@./bin/atlasctl run ./ops/run/product/product_chart_package.sh

chart-verify:
	@./bin/atlasctl run ./ops/run/product/product_chart_verify.sh

chart-validate: ## Validate chart via lint/template and values contract schema checks
	@./bin/atlasctl run ./ops/run/product/product_chart_validate.sh

docker-contracts: ## Validate Docker layout/policy/no-latest contracts
	@./bin/atlasctl check domain docker

rename-lint: ## Enforce durable naming rules for docs/scripts and concept ownership
	@./bin/atlasctl run ./ops/run/product/product_rename_lint.sh

docs-lint-names: ## Enforce durable naming contracts, registries, and inventory
	@./bin/atlasctl run ./ops/run/product/product_docs_lint_names.sh

internal/product/doctor: ## Print tool/env/path diagnostics and store doctor report
	@RUN_ID="$${RUN_ID:-doctor-$(MAKE_RUN_TS)}" ./bin/atlasctl make doctor

prereqs: ## Check required binaries and versions and store prereqs report
	@RUN_ID="$${RUN_ID:-prereqs-$(MAKE_RUN_TS)}" ./bin/atlasctl make prereqs --run-id "$${RUN_ID:-prereqs-$(MAKE_RUN_TS)}"

dataset-id-lint: ## Validate DatasetId/DatasetKey contract usage across ops fixtures
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/checks/layout/scripts/dataset_id_lint.py

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
	@artifacts/isolate/py/packages-check/.venv/bin/pip --disable-pip-version-check install -e packages/atlasctl >/dev/null
	@artifacts/isolate/py/packages-check/.venv/bin/python -c "import atlasctl"
	@./bin/atlasctl check run repo
