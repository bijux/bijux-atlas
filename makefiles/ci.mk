SHELL := /bin/sh

ci-root-layout:
	@$(MAKE) layout-check

ci-script-entrypoints:
	@$(MAKE) no-direct-scripts

ci-fmt:
	@$(MAKE) fmt

ci-clippy:
	@$(MAKE) _lint-clippy

ci-test-nextest:
	@$(MAKE) test

ci-deny:
	@if ! cargo +stable deny --version >/dev/null 2>&1; then cargo +stable install cargo-deny --locked; fi
	@cargo +stable deny check

ci-audit:
	@if ! cargo audit --version >/dev/null 2>&1; then cargo install cargo-audit --locked; fi
	@cargo audit

ci-license-check:
	@if ! cargo +stable deny --version >/dev/null 2>&1; then cargo +stable install cargo-deny --locked; fi
	@cargo +stable deny check licenses

ci-policy-lint:
	@$(MAKE) policy-lint

ci-policy-schema-drift:
	@$(MAKE) policy-schema-drift

ci-ssot-drift:
	@$(MAKE) ssot-check

ci-crate-structure:
	@$(MAKE) crate-structure

ci-crate-docs-contract:
	@$(MAKE) crate-docs-contract

ci-cli-command-surface:
	@$(MAKE) cli-command-surface

ci-release-binaries:
	@cargo build --workspace --release --bins --locked
	@"$${CARGO_TARGET_DIR:-target}/release/bijux-atlas" --help
	@"$${CARGO_TARGET_DIR:-target}/release/atlas-server" --help

ci-docs-build:
	@$(MAKE) docs
	@$(MAKE) docs-freeze

ci-latency-regression:
	@cargo test -p bijux-atlas-server --test latency_guard --locked

ci-store-conformance:
	@cargo test -p bijux-atlas-store --locked
	@cargo test -p bijux-atlas-server --test s3_backend --locked

ci-openapi-drift:
	@$(MAKE) openapi-drift

ci-query-plan-gate:
	@$(MAKE) query-plan-gate

ci-compatibility-matrix-validate:
	@$(MAKE) compat-matrix-validate

ci-runtime-security-scan-image:
	@$(MAKE) docker-build

ci-coverage:
	@if ! cargo llvm-cov --version >/dev/null 2>&1; then cargo install cargo-llvm-cov --locked; fi
	@cargo llvm-cov --workspace --all-features --lcov --output-path artifacts/isolates/coverage/lcov.info

ci-workflows-make-only:
	@python3 ./scripts/layout/check_workflows_make_only.py

governance-check: ## Run governance gates: layout + docs + contracts + scripts + workflow policy
	@$(MAKE) layout-check
	@$(MAKE) docs-freeze
	@$(MAKE) ssot-check
	@$(MAKE) scripts-lint
	@$(MAKE) ci-workflows-make-only

.PHONY: \
	ci-root-layout ci-script-entrypoints ci-fmt ci-clippy ci-test-nextest ci-deny ci-audit ci-license-check \
	ci-policy-lint ci-policy-schema-drift ci-ssot-drift ci-crate-structure ci-crate-docs-contract ci-cli-command-surface \
	ci-release-binaries ci-docs-build ci-latency-regression ci-store-conformance ci-openapi-drift ci-query-plan-gate \
	ci-compatibility-matrix-validate ci-runtime-security-scan-image ci-coverage ci-workflows-make-only governance-check
