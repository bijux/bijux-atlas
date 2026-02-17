SHELL := /bin/sh

JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/docs.mk
include makefiles/policies.mk

.DEFAULT_GOAL := help

help:
	@printf '%s\n' \
	  'dev:' \
	  '  dev-fmt dev-lint dev-check dev-test dev-test-all dev-coverage dev-audit dev-ci dev-clean' \
	  'docs:' \
	  '  docs docs-serve docs-freeze docs-hardening' \
	  'ops:' \
	  '  e2e-local e2e-k8s-install-gate e2e-k8s-suite e2e-perf e2e-realdata observability-check layout-check layout-migrate' \
	  'release/surface:' \
	  '  fmt lint check test test-all coverage audit openapi-drift ci ssot-check crate-structure crate-docs-contract cli-command-surface' \
	  'tooling:' \
	  '  bootstrap doctor help'

layout-check:
	@./scripts/layout/check_root_shape.sh

layout-migrate:
	@./scripts/layout/migrate.sh

bootstrap:
	@python3 --version
	@command -v pip >/dev/null 2>&1 || { echo "missing pip" >&2; exit 1; }
	@python3 -m pip install -r ops/docs/requirements.txt >/dev/null
	@command -v k6 >/dev/null 2>&1 || echo "k6 not found (optional for non-perf workflows)"
	@command -v kind >/dev/null 2>&1 || echo "kind not found (required for k8s e2e)"
	@command -v kubectl >/dev/null 2>&1 || echo "kubectl not found (required for k8s e2e)"

doctor:
	@printf 'rustc: '; rustc --version
	@printf 'cargo: '; cargo --version
	@printf 'python3: '; python3 --version
	@printf 'k6: '; (k6 version 2>/dev/null | head -n1 || echo 'missing')
	@printf 'kind: '; (kind version 2>/dev/null | head -n1 || echo 'missing')
	@printf 'kubectl: '; (kubectl version --client --short 2>/dev/null || echo 'missing')
	@printf 'helm: '; (helm version --short 2>/dev/null || echo 'missing')

e2e-local:
	@./ops/e2e/scripts/up.sh
	@./ops/e2e/scripts/cleanup_store.sh
	@./ops/e2e/scripts/publish_dataset.sh \
	  --gff3 fixtures/minimal/minimal.gff3 \
	  --fasta fixtures/minimal/minimal.fa \
	  --fai fixtures/minimal/minimal.fa.fai \
	  --release 110 --species homo_sapiens --assembly GRCh38
	@./ops/e2e/scripts/deploy_atlas.sh
	@./ops/e2e/scripts/warmup.sh
	@./ops/e2e/scripts/smoke_queries.sh
	@./ops/e2e/scripts/verify_metrics.sh

e2e-k8s-install-gate:
	@./ops/e2e/scripts/up.sh
	@./ops/e2e/k8s/tests/test_install.sh

e2e-k8s-suite:
	@./ops/e2e/scripts/up.sh
	@./ops/e2e/k8s/tests/run_all.sh

e2e-perf:
	@./scripts/perf/run_e2e_perf.sh

fetch-real-datasets:
	@./scripts/fixtures/fetch-real-datasets.sh

e2e-realdata:
	@./ops/e2e/realdata/run_all.sh

ssot-check:
	@./scripts/contracts/check_all.sh

observability-check:
	@./scripts/observability/check_metrics_contract.py
	@./scripts/observability/check_dashboard_contract.py
	@./scripts/observability/check_alerts_contract.py
	@./scripts/observability/check_tracing_contract.py
	@./scripts/observability/lint_runbooks.py
	@./scripts/observability/check_runtime_metrics.py
	@cargo test -p bijux-atlas-server --test observability_contract --test logging_format
