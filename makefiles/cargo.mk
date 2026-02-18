NEXTEST_PROFILE ?= ci
NEXTEST_PROFILE_FAST ?= fast-unit
NEXTEST_PROFILE_SLOW ?= slow-integration
NEXTEST_PROFILE_CERT ?= certification
ARTIFACTS_DIR ?= $(if $(ISO_ROOT),$(ISO_ROOT),artifacts/isolates/$(or $(ISO_RUN_ID),local))
NEXTEST_TOML := configs/nextest/nextest.toml
NEXTEST_CONFIG ?= --config-file $(NEXTEST_TOML)
NEXTEST_FAST_EXPR ?= not test(/::slow__/)
NEXTEST_NO_TESTS ?= pass
RUN_IGNORED = --run-ignored all
TEST_FEATURES = --all-features
CARGO_BUILD_JOBS ?= $(JOBS)
NEXTEST_TEST_THREADS ?= $(CARGO_BUILD_JOBS)
COVERAGE_BASELINE = $(ARTIFACTS_DIR)/coverage/baseline.json
COVERAGE_THRESHOLDS := configs/coverage/thresholds.toml
COVERAGE_OUT = $(ARTIFACTS_DIR)/coverage/lcov.info
AUTO_ISO_TAG_PREFIX ?= make

fmt:
	@if [ -n "$$ISO_ROOT" ]; then ./scripts/bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-fmt-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./scripts/bin/isolate --tag "$$tag" $(MAKE) _fmt; \
	else \
		$(MAKE) _fmt; \
	fi

_fmt:
	@./scripts/bin/require-isolate >/dev/null
	@cargo fmt --all --check
	@./scripts/layout/check_repo_hygiene.sh

lint:
	@if [ -n "$$ISO_ROOT" ]; then ./scripts/bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-lint-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./scripts/bin/isolate --tag "$$tag" $(MAKE) _lint; \
	else \
		$(MAKE) _lint; \
	fi

_lint:
	@$(MAKE) _lint-rustfmt
	@$(MAKE) _lint-configs
	@$(MAKE) _lint-docs
	@$(MAKE) _lint-clippy

_lint-rustfmt:
	@./scripts/bin/require-isolate >/dev/null
	@cargo fmt --all --check

_lint-configs:
	@./scripts/bin/require-isolate >/dev/null
	@./scripts/public/policy-lint.sh
	@./scripts/layout/check_no_direct_script_runs.sh
	@./scripts/layout/check_scripts_readme_drift.sh
	@./scripts/layout/check_repo_hygiene.sh

_lint-docs:
	@./scripts/bin/require-isolate >/dev/null
	@./scripts/public/check-markdown-links.sh

_lint-clippy:
	@./scripts/bin/require-isolate >/dev/null
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" cargo clippy --workspace --all-targets -- -D warnings

check:
	@if [ -n "$$ISO_ROOT" ]; then ./scripts/bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-check-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./scripts/bin/isolate --tag "$$tag" $(MAKE) _check; \
	else \
		$(MAKE) _check; \
	fi

_check:
	@./scripts/bin/require-isolate >/dev/null
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" cargo check --workspace --all-targets
	@./scripts/layout/check_repo_hygiene.sh

test:
	@if [ -n "$$ISO_ROOT" ]; then ./scripts/bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-test-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./scripts/bin/isolate --tag "$$tag" $(MAKE) _test; \
	else \
		$(MAKE) _test; \
	fi

test-all:
	@if [ -n "$$ISO_ROOT" ]; then ./scripts/bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-test-all-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./scripts/bin/isolate --tag "$$tag" $(MAKE) _test-all; \
	else \
		$(MAKE) _test-all; \
	fi

_test:
	@./scripts/bin/require-isolate >/dev/null
	@if ! cargo nextest --version >/dev/null 2>&1; then \
		echo "cargo-nextest is required. Install: cargo install cargo-nextest --locked" >&2; \
		exit 1; \
	fi
	@NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" \
	NEXTEST_TARGET_DIR="$${CARGO_TARGET_DIR}/nextest" \
	cargo nextest run --workspace --all-targets --profile "$(NEXTEST_PROFILE)" $(NEXTEST_CONFIG)
	@if [ -d target/nextest ]; then find target/nextest -type f -delete 2>/dev/null || true; fi
	@if [ -d target/nextest ]; then find target/nextest -type d -empty -delete 2>/dev/null || true; fi
	@if [ -d target ]; then find target -type d -empty -delete 2>/dev/null || true; fi
	@./scripts/layout/check_repo_hygiene.sh

_test-all:
	@./scripts/bin/require-isolate >/dev/null
	@if ! cargo nextest --version >/dev/null 2>&1; then \
		echo "cargo-nextest is required. Install: cargo install cargo-nextest --locked" >&2; \
		exit 1; \
	fi
	@NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" \
	NEXTEST_TARGET_DIR="$${CARGO_TARGET_DIR}/nextest" \
	cargo nextest run --workspace --all-targets --profile "$(NEXTEST_PROFILE)" $(NEXTEST_CONFIG) $(RUN_IGNORED)
	@if [ -d target/nextest ]; then find target/nextest -type f -delete 2>/dev/null || true; fi
	@if [ -d target/nextest ]; then find target/nextest -type d -empty -delete 2>/dev/null || true; fi
	@if [ -d target ]; then find target -type d -empty -delete 2>/dev/null || true; fi
	@./scripts/layout/check_repo_hygiene.sh

coverage:
	@if [ -n "$$ISO_ROOT" ]; then ./scripts/bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-coverage-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./scripts/bin/isolate --tag "$$tag" $(MAKE) _coverage; \
	else \
		$(MAKE) _coverage; \
	fi

_coverage:
	@./scripts/bin/require-isolate >/dev/null
	@if ! cargo llvm-cov --version >/dev/null 2>&1; then \
		echo "cargo-llvm-cov is required. Install: cargo install cargo-llvm-cov --locked" >&2; \
		exit 1; \
	fi
	@mkdir -p "$(dir $(COVERAGE_OUT))"
	@cargo llvm-cov nextest --workspace --profile "$(NEXTEST_PROFILE)" $(NEXTEST_CONFIG) --lcov --output-path "$(COVERAGE_OUT)"
	@echo "coverage output: $(COVERAGE_OUT)"
	@echo "coverage thresholds config: $(COVERAGE_THRESHOLDS)"
	@./scripts/layout/check_repo_hygiene.sh

audit:
	@if [ -n "$$ISO_ROOT" ]; then ./scripts/bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-audit-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./scripts/bin/isolate --tag "$$tag" $(MAKE) _audit; \
	else \
		$(MAKE) _audit; \
	fi

_audit:
	@./scripts/bin/require-isolate >/dev/null
	@if ! cargo +stable deny --version >/dev/null 2>&1; then \
		echo "cargo-deny is required for stable toolchain. Installing..." >&2; \
		cargo +stable install cargo-deny --locked; \
	fi
	@cargo +stable deny check

ci-core: fmt lint audit test coverage

openapi-drift:
	@./scripts/public/openapi-diff-check.sh

compat-matrix-validate:
	@./scripts/release/validate-compat-matrix.sh

fetch-fixtures:
	@./scripts/fixtures/fetch-medium.sh

load-test:
	@k6 run ops/load/k6/atlas_phase11.js

load-test-1000qps:
	@k6 run ops/load/k6/atlas_1000qps.js

perf-nightly:
	@./ops/load/scripts/run_nightly_perf.sh

query-plan-gate:
	@./scripts/public/query-plan-gate.sh

cold-start-bench:
	@./ops/load/scripts/cold_start_benchmark.sh

memory-profile-load:
	@echo "runbook: docs/runbooks/memory-profile-under-load.md"
	@echo "outputs: artifacts/benchmarks/memory/"

run-medium-ingest:
	@./scripts/fixtures/run-medium-ingest.sh

run-medium-serve:
	@./scripts/fixtures/run-medium-serve.sh

bench-sqlite-query-latency:
	@cargo bench -p bijux-atlas-ingest --features bench-ingest-throughput --bench sqlite_query_latency

bench-ingest-throughput-medium:
	@cargo bench -p bijux-atlas-ingest --features bench-ingest-throughput --bench ingest_throughput

bench-db-size-growth:
	@cargo bench -p bijux-atlas-ingest --features bench-ingest-throughput --bench db_size_growth

.PHONY: fmt _fmt lint _lint _lint-rustfmt _lint-configs _lint-docs _lint-clippy check _check test test-all _test _test-all coverage _coverage audit _audit ci-core openapi-drift compat-matrix-validate fetch-fixtures load-test load-test-1000qps perf-nightly query-plan-gate cold-start-bench memory-profile-load run-medium-ingest run-medium-serve bench-sqlite-query-latency bench-ingest-throughput-medium bench-db-size-growth
