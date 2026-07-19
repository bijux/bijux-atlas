# Scope: Atlas Rust policy and repository-owned Cargo targets.
# Public targets: Rust aliases, check, policy reports, and publish-rs
SHELL := /bin/bash
CARGO_TERM_PROGRESS_WHEN ?= always
CARGO_TERM_PROGRESS_WIDTH ?= 120
CARGO_TERM_VERBOSE ?= false
CARGO_TERM_COLOR ?= always
NEXTEST_PROFILE_FAST ?= fast-unit
NEXTEST_PROFILE_SLOW ?= slow-integration
NEXTEST_PROFILE_CERT ?= certification
NEXTEST_PROFILE_ALL ?= full
CARGO_BUILD_JOBS ?= $(JOBS)
NEXTEST_THREADS_ALL ?= $(if $(filter auto,$(CARGO_BUILD_JOBS)),num-cpus,$(if $(CARGO_BUILD_JOBS),$(CARGO_BUILD_JOBS),8))
NEXTEST_CONFIG_FILE ?= $(CURDIR)/configs/rust/nextest.toml
NEXTEST_SLOW_NAME_EXPR ?= test(/(^|::)slow__/)
ATLAS_RUST_GATE_BIN ?= makes/bin/run_atlas_rust_gate.sh
RUST_GATE_BIN ?= $(ATLAS_RUST_GATE_BIN)
RUSTFMT_CONFIG ?= configs/sources/repository/rust-tooling/rustfmt.toml
RUST_AUDIT_PREREQUISITES += audit-policy-rs
CRATES_IO_API_USER_AGENT ?= bijux-atlas-release-check/1

fmt: fmt-rs
lint: lint-rs
test: test-rs
test-slow: test-slow-rs
test-all: test-all-rs
audit: audit-rs
coverage: coverage-rs

audit-policy-rs: ## Verify Atlas dependency-audit policy before Cargo advisory checks
	@mkdir -p "$(RS_TARGET_DIR)"
	@CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
		cargo run --locked -q -p bijux-atlas-dev -- \
		security dependency-audit --repo-root "$(CURDIR)" --format json

check: ## Run cargo check for the workspace
	@CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo check --workspace --all-targets

lint-policy-report: ## Emit effective lint policy report artifact
	@$(DEV_ATLAS) makes lint-policy-report --allow-write --format $(FORMAT)

lint-policy-enforce: ## Enforce repository lint drift guards
	@! rg -n '\btodo!\(' crates
	@! rg -n '\bdbg!\(' crates
	@! rg -n '\b(?:println|eprintln)!\(' crates/bijux-atlas/src crates/bijux-atlas-dev/src --glob '!**/tests/**' --glob '!**/benches/**' --glob '!**/main.rs' --glob '!**/bin/**'
	@! rg -n '\bpanic!\(' crates/bijux-atlas --glob '!**/tests/**' --glob '!**/benches/**'
	@! rg -n 'reqwest\s*=.*blocking' crates/bijux-atlas/Cargo.toml
	@! rg -n 'reqwest::blocking' crates/bijux-atlas/src

lint-clippy-json: ## Emit clippy diagnostics as a machine-readable artifact
	@mkdir -p artifacts/lint
	@CLIPPY_CONF_DIR=configs/rust CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo clippy --workspace --all-targets --all-features --locked --message-format=json -- -D warnings > artifacts/lint/clippy.json
	@printf '%s\n' "artifacts/lint/clippy.json"

publish-rs: ## Publish Rust crates and dry-run by default
	@set -euo pipefail; \
	if [ -z "$(RUST_PUBLISH_PACKAGES)" ]; then \
		echo "RUST_PUBLISH_PACKAGES is empty; nothing to publish" >&2; \
		exit 1; \
	fi; \
	mkdir -p "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"; \
	dry_run_flag=""; \
	if [ "$(RUST_PUBLISH_DRY_RUN)" = "1" ]; then \
		dry_run_flag="--dry-run"; \
	fi; \
	for pkg in $(RUST_PUBLISH_PACKAGES); do \
		if [ "$(RUST_PUBLISH_SKIP_EXISTING)" = "1" ] && [ "$(RUST_PUBLISH_DRY_RUN)" != "1" ]; then \
			status=""; \
			for attempt in 1 2 3 4 5; do \
				status="$$(curl -A "$(CRATES_IO_API_USER_AGENT)" -fsS -o /dev/null -w '%{http_code}' "https://crates.io/api/v1/crates/$$pkg/$(RELEASE_VERSION)" 2>/dev/null || true)"; \
				if [ -n "$${status}" ] && [ "$${status}" != "000" ]; then \
					break; \
				fi; \
				sleep "$${attempt}"; \
			done; \
			if [ "$${status}" = "200" ]; then \
				echo "skipping $$pkg $(RELEASE_VERSION); already present on crates.io"; \
				continue; \
			fi; \
		fi; \
		publish_log="$$(mktemp)"; \
		if ! cargo publish --locked $$dry_run_flag -p "$$pkg" >"$$publish_log" 2>&1; then \
			cat "$$publish_log" >&2; \
			if [ "$(RUST_PUBLISH_SKIP_EXISTING)" = "1" ] && [ "$(RUST_PUBLISH_DRY_RUN)" != "1" ]; then \
				if grep -Eiq 'already (uploaded|exists)|previously uploaded|same version' "$$publish_log"; then \
					echo "skipping $$pkg $(RELEASE_VERSION); cargo publish reported the version is already present"; \
					rm -f "$$publish_log"; \
					continue; \
				fi; \
				status=""; \
				for attempt in 1 2 3 4 5; do \
					status="$$(curl -A "$(CRATES_IO_API_USER_AGENT)" -fsS -o /dev/null -w '%{http_code}' "https://crates.io/api/v1/crates/$$pkg/$(RELEASE_VERSION)" 2>/dev/null || true)"; \
					if [ -n "$${status}" ] && [ "$${status}" != "000" ]; then \
						break; \
					fi; \
					sleep "$${attempt}"; \
				done; \
				if [ "$${status}" = "200" ]; then \
					echo "skipping $$pkg $(RELEASE_VERSION); crates.io now reports the version as published"; \
					rm -f "$$publish_log"; \
					continue; \
				fi; \
			fi; \
			rm -f "$$publish_log"; \
			exit 1; \
		fi; \
		cat "$$publish_log"; \
		rm -f "$$publish_log"; \
	done

.PHONY: audit audit-policy-rs check coverage fmt lint lint-policy-report
.PHONY: lint-policy-enforce lint-clippy-json test test-slow test-all publish-rs
