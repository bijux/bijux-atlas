# Scope: CI-safe deterministic cargo targets and contracts.
# Public targets: none
NEXTEST_PROFILE ?= ci
NEXTEST_PROFILE_FAST ?= fast-unit
NEXTEST_PROFILE_SLOW ?= slow-integration
NEXTEST_PROFILE_CERT ?= certification
ARTIFACTS_DIR ?= $(if $(ISO_ROOT),$(ISO_ROOT),artifacts/isolate/$(or $(ISO_RUN_ID),local))
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

internal/cargo/fmt:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-fmt-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _fmt; \
	else \
		$(MAKE) _fmt; \
	fi

_fmt:
	@./bin/atlasctl env require-isolate >/dev/null
	@cargo fmt --all -- --check --config-path configs/rust/rustfmt.toml
	@./scripts/areas/layout/check_repo_hygiene.sh

internal/cargo/lint:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-lint-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _lint; \
	else \
		$(MAKE) _lint; \
	fi

_lint:
	@$(MAKE) _lint-rustfmt
	@$(MAKE) _lint-configs
	@$(MAKE) _lint-docs
	@$(MAKE) _lint-clippy

_lint-rustfmt:
	@./bin/atlasctl env require-isolate >/dev/null
	@cargo fmt --all -- --check --config-path configs/rust/rustfmt.toml

_lint-configs:
	@./bin/atlasctl env require-isolate >/dev/null
	@./scripts/areas/public/policy-lint.sh
	@./scripts/areas/layout/check_no_direct_script_runs.sh
	@./scripts/areas/layout/check_scripts_readme_drift.sh
	@./scripts/areas/layout/check_repo_hygiene.sh

_lint-docs:
	@./bin/atlasctl env require-isolate >/dev/null
	@./scripts/areas/public/check-markdown-links.sh

_lint-clippy:
	@./bin/atlasctl env require-isolate >/dev/null
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" CLIPPY_CONF_DIR="$(CURDIR)/configs/rust" cargo clippy --workspace --all-targets -- -D warnings

internal/cargo/check:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-check-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _check; \
	else \
		$(MAKE) _check; \
	fi

_check:
	@./bin/atlasctl env require-isolate >/dev/null
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" cargo check --workspace --all-targets
	@./scripts/areas/layout/check_repo_hygiene.sh

internal/cargo/test:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-test-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _test; \
	else \
		$(MAKE) _test; \
	fi

test-all:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-test-all-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _test-all; \
	else \
		$(MAKE) _test-all; \
	fi

test-contracts:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-test-contracts-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _test-contracts; \
	else \
		$(MAKE) _test-contracts; \
	fi

_test:
	@./bin/atlasctl env require-isolate >/dev/null
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
	@./scripts/areas/layout/check_repo_hygiene.sh

_test-all:
	@./bin/atlasctl env require-isolate >/dev/null
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
	@./scripts/areas/layout/check_repo_hygiene.sh

_test-contracts:
	@./bin/atlasctl env require-isolate >/dev/null
	@cargo test -p bijux-atlas-server --test observability_contract
	@./scripts/areas/layout/check_repo_hygiene.sh

coverage:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-coverage-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _coverage; \
	else \
		$(MAKE) _coverage; \
	fi

_coverage:
	@./bin/atlasctl env require-isolate >/dev/null
	@if ! cargo llvm-cov --version >/dev/null 2>&1; then \
		echo "cargo-llvm-cov is required. Install: cargo install cargo-llvm-cov --locked" >&2; \
		exit 1; \
	fi
	@mkdir -p "$(dir $(COVERAGE_OUT))"
	@cargo llvm-cov nextest --workspace --profile "$(NEXTEST_PROFILE)" $(NEXTEST_CONFIG) --lcov --output-path "$(COVERAGE_OUT)"
	@echo "coverage output: $(COVERAGE_OUT)"
	@echo "coverage thresholds config: $(COVERAGE_THRESHOLDS)"
	@./scripts/areas/layout/check_repo_hygiene.sh

internal/cargo/audit:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-audit-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _audit; \
	else \
		$(MAKE) _audit; \
	fi

_audit:
	@./bin/atlasctl env require-isolate >/dev/null
	@if ! cargo +stable deny --version >/dev/null 2>&1; then \
		echo "cargo-deny is required for stable toolchain. Installing..." >&2; \
		cargo +stable install cargo-deny --locked; \
	fi
	@cargo +stable deny check --config configs/security/deny.toml

ci-core: internal/cargo/fmt internal/cargo/lint internal/cargo/audit internal/cargo/test coverage

openapi-drift:
	@./scripts/areas/public/openapi-diff-check.sh

api-contract-check:
	@$(ATLAS_SCRIPTS) run ./scripts/areas/public/contracts/gen_openapi.py
	@./scripts/areas/public/openapi-diff-check.sh
	@$(ATLAS_SCRIPTS) run ./scripts/areas/public/contracts/check_endpoints_contract.py
	@$(ATLAS_SCRIPTS) run ./scripts/areas/public/contracts/check_error_codes_contract.py
	@$(ATLAS_SCRIPTS) run ./scripts/areas/public/contracts/check_v1_surface.py
	@$(ATLAS_SCRIPTS) run ./scripts/areas/public/contracts/check_breaking_contract_change.py

compat-matrix-validate:
	@$(ATLAS_SCRIPTS) compat validate-matrix

fetch-fixtures:
	@./scripts/areas/fixtures/fetch-medium.sh

load-test:
	@k6 run ops/load/k6/mixed-80-20.js

load-test-1000qps:
	@k6 run ops/load/k6/atlas_1000qps.js

perf-nightly:
	@./ops/load/scripts/run_nightly_perf.sh

query-plan-gate:
	@./scripts/areas/public/query-plan-gate.sh

critical-query-check:
	@$(ATLAS_SCRIPTS) run ./scripts/areas/public/contracts/check_sqlite_indexes_contract.py
	@$(ATLAS_SCRIPTS) run ./scripts/areas/public/perf/run_critical_queries.py

cold-start-bench:
	@./ops/load/scripts/cold_start_benchmark.sh

memory-profile-load:
	@echo "runbook: docs/runbooks/memory-profile-under-load.md"
	@echo "outputs: artifacts/benchmarks/memory/"

run-medium-ingest:
	@./scripts/areas/fixtures/run-medium-ingest.sh

ingest-sharded-medium:
	@./scripts/areas/fixtures/run-medium-ingest.sh --sharded

run-medium-serve:
	@./scripts/areas/fixtures/run-medium-serve.sh

bench-sqlite-query-latency:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-bench-sqlite-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _bench-sqlite-query-latency; \
	else \
		$(MAKE) _bench-sqlite-query-latency; \
	fi

bench-ingest-throughput-medium:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-bench-throughput-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _bench-ingest-throughput-medium; \
	else \
		$(MAKE) _bench-ingest-throughput-medium; \
	fi

bench-db-size-growth:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-bench-db-size-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _bench-db-size-growth; \
	else \
		$(MAKE) _bench-db-size-growth; \
	fi

bench-smoke:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/atlasctl env require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-bench-smoke-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/atlasctl env isolate --tag "$$tag" $(MAKE) _bench-smoke; \
	else \
		$(MAKE) _bench-smoke; \
	fi

_bench-sqlite-query-latency:
	@./bin/atlasctl env require-isolate >/dev/null
	@cargo bench -p bijux-atlas-ingest --features bench-ingest-throughput --bench sqlite_query_latency

_bench-ingest-throughput-medium:
	@./bin/atlasctl env require-isolate >/dev/null
	@cargo bench -p bijux-atlas-ingest --features bench-ingest-throughput --bench ingest_throughput

_bench-db-size-growth:
	@./bin/atlasctl env require-isolate >/dev/null
	@cargo bench -p bijux-atlas-ingest --features bench-ingest-throughput --bench db_size_growth

_bench-smoke:
	@$(MAKE) _bench-sqlite-query-latency

.PHONY: fmt _fmt lint _lint _lint-rustfmt _lint-configs _lint-docs _lint-clippy check _check test test-all test-contracts _test _test-all _test-contracts coverage _coverage audit _audit ci-core openapi-drift api-contract-check compat-matrix-validate fetch-fixtures load-test load-test-1000qps perf-nightly query-plan-gate critical-query-check cold-start-bench memory-profile-load run-medium-ingest ingest-sharded-medium run-medium-serve bench-sqlite-query-latency bench-ingest-throughput-medium bench-db-size-growth bench-smoke _bench-sqlite-query-latency _bench-ingest-throughput-medium _bench-db-size-growth _bench-smoke
