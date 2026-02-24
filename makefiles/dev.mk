# Scope: canonical developer wrappers delegated to cargo-native entrypoints.
# Keep one target per gate with deterministic cargo commands.
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
	@cargo llvm-cov --workspace --all-features --lcov --output-path artifacts/coverage/lcov.info

fmt: ## Rust formatter check
	@cargo fmt --all -- --check

lint: ## Rust lint lane
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

test: ## Rust tests lane
	@command -v cargo-nextest >/dev/null 2>&1 || { \
		echo "cargo-nextest is required. Install with: cargo install cargo-nextest"; \
		exit 1; \
	}
	@cargo nextest run --workspace --target-dir "$(CARGO_TARGET_DIR)" --profile "$${NEXTEST_PROFILE:-default}"

test-all: ## Rust tests full variant (includes ignored)
	@command -v cargo-nextest >/dev/null 2>&1 || { \
		echo "cargo-nextest is required. Install with: cargo install cargo-nextest"; \
		exit 1; \
	}
	@cargo nextest run --workspace --target-dir "$(CARGO_TARGET_DIR)" --profile "$${NEXTEST_PROFILE:-default}" --run-ignored all

dev-doctor: ## Run dev control-plane doctor suite
	@$(DEV_ATLAS) check doctor --format text

dev-check-ci: ## Run dev control-plane ci suite
	@$(DEV_ATLAS) check run --suite ci --format text

dev-check: ## Alias for dev-check-ci
	@$(MAKE) -s dev-check-ci

dev-ci: ## Alias for dev-check-ci
	@$(MAKE) -s dev-check-ci

install-local: ## Build and install bijux-atlas + bijux-dev-atlas into artifacts/bin
	@mkdir -p artifacts/bin
	@cargo build -p bijux-atlas-cli -p bijux-dev-atlas
	@cp artifacts/target/debug/bijux-atlas artifacts/bin/bijux-atlas
	@cp artifacts/target/debug/bijux-dev-atlas artifacts/bin/bijux-dev-atlas
	@chmod +x artifacts/bin/bijux-atlas artifacts/bin/bijux-dev-atlas
	@echo "installed artifacts/bin/bijux-atlas"
	@echo "installed artifacts/bin/bijux-dev-atlas"

.PHONY: audit cargo-check coverage fmt lint test test-all dev-doctor dev-check-ci dev-check dev-ci install-local
