SHELL := /bin/sh

JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/policies.mk

.DEFAULT_GOAL := help

help:
	@printf '%s\n' \
	  'targets: fmt lint check test test-all coverage audit openapi-drift ci fetch-fixtures load-test load-test-1000qps cold-start-bench memory-profile-load run-medium-ingest run-medium-serve crate-structure cli-command-surface culprits-all culprits-max_loc culprits-max_depth culprits-file-max_rs_files_per_dir culprits-file-max_modules_per_dir e2e-local e2e-k8s-install-gate e2e-k8s-suite e2e-perf ssot-check observability-check' \
	  'perf targets: perf-nightly' \
	  'dev targets: dev-fmt dev-lint dev-check dev-test dev-test-all dev-coverage dev-audit dev-ci dev-clean'

e2e-local:
	@./e2e/scripts/up.sh
	@./e2e/scripts/cleanup_store.sh
	@./e2e/scripts/publish_dataset.sh \
	  --gff3 fixtures/minimal/minimal.gff3 \
	  --fasta fixtures/minimal/minimal.fa \
	  --fai fixtures/minimal/minimal.fa.fai \
	  --release 110 --species homo_sapiens --assembly GRCh38
	@./e2e/scripts/deploy_atlas.sh
	@./e2e/scripts/warmup.sh
	@./e2e/scripts/smoke_queries.sh
	@./e2e/scripts/verify_metrics.sh

e2e-k8s-install-gate:
	@./e2e/scripts/up.sh
	@./e2e/k8s/tests/test_install.sh

e2e-k8s-suite:
	@./e2e/scripts/up.sh
	@./e2e/k8s/tests/run_all.sh

e2e-perf:
	@./scripts/perf/run_e2e_perf.sh

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
