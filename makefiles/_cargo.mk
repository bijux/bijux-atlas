# Scope: canonical Rust cargo gates delegated to cargo-native tooling.
# Public targets: audit, cargo-check, coverage, fmt, lint, test
SHELL := /bin/sh

audit: ## Rust dependency audit lane
	@command -v cargo-audit >/dev/null 2>&1 || { \
		echo "cargo-audit is required. Install with: cargo install cargo-audit"; \
		exit 1; \
	}
	@cargo audit

cargo-check: ## Rust cargo check lane
	@cargo check --workspace --all-targets

coverage: ## Rust coverage lane
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	@mkdir -p artifacts/coverage
	@cargo llvm-cov nextest --workspace --all-features --lcov --output-path artifacts/coverage/lcov.info --config-file configs/nextest/nextest.toml --user-config-file none --run-ignored all
	@cargo llvm-cov report

fmt: ## Rust formatter check
	@cargo fmt --all -- --check

lint: ## Rust lint lane
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

test: ## Rust tests lane
	@command -v cargo-nextest >/dev/null 2>&1 || { \
		echo "cargo-nextest is required. Install with: cargo install cargo-nextest"; \
		exit 1; \
	}
	@cargo nextest run --workspace --config-file configs/nextest/nextest.toml --user-config-file none --target-dir "$(CARGO_TARGET_DIR)" --profile "$${NEXTEST_PROFILE:-default}"

.PHONY: audit cargo-check coverage fmt lint test
