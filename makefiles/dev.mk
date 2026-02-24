# Scope: canonical developer wrappers delegated to cargo-native entrypoints.
# Keep one target per gate with deterministic cargo commands.
SHELL := /bin/sh

audit: ## Rust dependency audit lane
	@command -v cargo-audit >/dev/null 2>&1 || { \
		echo "cargo-audit is required. Install with: cargo install cargo-audit"; \
		exit 1; \
	}
	@cargo audit

check: ## Rust cargo check lane
	@cargo check --workspace --all-targets

coverage: ## Rust coverage lane
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	@cargo llvm-cov --workspace --all-features --lcov --output-path artifacts/coverage/lcov.info

fmt: ## Rust formatter check
	@cargo fmt --all -- --check

lint: ## Rust lint lane
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

test: ## Rust tests lane
	@cargo test --workspace

test-all: ## Rust tests full variant (includes ignored)
	@cargo test --workspace -- --include-ignored

.PHONY: audit check coverage fmt lint test test-all
