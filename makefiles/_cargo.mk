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
	@CLIPPY_CONF_DIR=configs/rust cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

lint-policy-report: ## Emit effective lint policy report artifact
	@mkdir -p artifacts/lint
	@{ \
		echo "schema_version=1"; \
		echo "workspace_lints_file=Cargo.toml"; \
		echo "clippy_conf_dir=configs/rust"; \
		echo "clippy_conf_file=configs/rust/clippy.toml"; \
		echo "cargo_clippy_version=$$(cargo clippy --version 2>/dev/null || true)"; \
		echo "workspace_lints:"; \
		awk '/^\[workspace.lints.rust\]/{p=1} p{print} /^\[workspace.dependencies\]/{if(p){exit}}' Cargo.toml; \
		echo "clippy_toml:"; \
		cat configs/rust/clippy.toml; \
	} > artifacts/lint/effective-clippy-policy.txt
	@printf '%s\n' "artifacts/lint/effective-clippy-policy.txt"

lint-policy-enforce: ## Enforce repository lint drift guards
	@! rg -n '\btodo!\(' crates
	@! rg -n '\bdbg!\(' crates
	@! rg -n '\b(?:println|eprintln)!\(' crates/bijux-atlas-server crates/bijux-atlas-core crates/bijux-atlas-model crates/bijux-atlas-query crates/bijux-atlas-store crates/bijux-atlas-policies --glob '!**/tests/**' --glob '!**/benches/**'
	@! rg -n 'reqwest\s*=.*blocking' crates/bijux-atlas-server/Cargo.toml

lint-clippy-json: ## Emit clippy diagnostics as a machine-readable artifact
	@mkdir -p artifacts/lint
	@CLIPPY_CONF_DIR=configs/rust cargo clippy --workspace --all-targets --all-features --locked --message-format=json -- -D warnings > artifacts/lint/clippy.json
	@printf '%s\n' "artifacts/lint/clippy.json"

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

.PHONY: audit check coverage fmt lint lint-policy-report lint-policy-enforce lint-clippy-json test test-slow
