# Scope: canonical Rust cargo gates delegated to cargo-native tooling.
# Public targets: audit, check, coverage, fmt, lint, test
SHELL := /bin/sh

audit: ## Run cargo dependency audit
	@command -v cargo-audit >/dev/null 2>&1 || { \
		echo "cargo-audit is required. Install with: cargo install cargo-audit"; \
		exit 1; \
	}
	@cargo audit

check: ## Run cargo check for the workspace
	@cargo check --workspace --all-targets

coverage: ## Run workspace coverage with cargo llvm-cov + nextest
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	@mkdir -p artifacts/coverage
	@cargo llvm-cov nextest --workspace --all-features --lcov --output-path artifacts/coverage/lcov.info --config-file configs/nextest/nextest.toml --user-config-file none --run-ignored all
	@cargo llvm-cov report

fmt: ## Run cargo fmt --check
	@cargo fmt --all -- --check --config-path configs/rust/rustfmt.toml

lint: ## Run cargo clippy with warnings denied
	@CLIPPY_CONF_DIR=configs/rust cargo clippy --workspace --all-targets --all-features -- -D warnings

test: ## Run workspace tests with cargo nextest
	@command -v cargo-nextest >/dev/null 2>&1 || { \
		echo "cargo-nextest is required. Install with: cargo install cargo-nextest"; \
		exit 1; \
	}
	@cargo nextest run --workspace --config-file configs/nextest/nextest.toml --user-config-file none --target-dir "$(CARGO_TARGET_DIR)" --profile "$${NEXTEST_PROFILE:-default}" -E "$${NEXTEST_FILTER_EXPR:-not test(/(^|::)slow_/)}"

test-slow: ## Run only slow_ tests with cargo nextest
	@command -v cargo-nextest >/dev/null 2>&1 || { \
		echo "cargo-nextest is required. Install with: cargo install cargo-nextest"; \
		exit 1; \
	}
	@cargo nextest run --workspace --config-file configs/nextest/nextest.toml --user-config-file none --target-dir "$(CARGO_TARGET_DIR)" --profile "$${NEXTEST_PROFILE:-default}" -E "test(/(^|::)slow_/)"

.PHONY: audit check coverage fmt lint test test-slow
