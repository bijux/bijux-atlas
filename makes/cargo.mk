# Scope: canonical Rust cargo gates delegated to cargo-native tooling.
# Public targets: none (internal cargo execution surface)
SHELL := /bin/bash
CARGO_TERM_PROGRESS_WHEN ?= always
CARGO_TERM_PROGRESS_WIDTH ?= 120
CARGO_TERM_VERBOSE ?= false
CARGO_TERM_COLOR ?= always
NEXTEST_PROFILE ?= full
NEXTEST_PROFILE_FAST ?= fast-unit
NEXTEST_PROFILE_SLOW ?= slow-integration
NEXTEST_PROFILE_CERT ?= certification
NEXTEST_PROFILE_ALL ?= full
CARGO_BUILD_JOBS ?= $(JOBS)
NEXTEST_THREADS_ALL ?= $(if $(filter auto,$(CARGO_BUILD_JOBS)),num-cpus,$(if $(CARGO_BUILD_JOBS),$(CARGO_BUILD_JOBS),8))
NEXTEST_TOML := configs/rust/nextest.toml
NEXTEST_EXPR_BIN ?= makes/bin/nextest_expr.sh
NEXTEST_FAST_EXPR ?= $(shell "$(NEXTEST_EXPR_BIN)" fast)
NEXTEST_SLOW_EXPR ?= $(shell "$(NEXTEST_EXPR_BIN)" slow)
RUST_GATE_BIN ?= makes/bin/rust_gate.sh
PINNED_REF_GATE_BIN ?= makes/bin/run_pinned_ref_gate.sh
CRATES_IO_API_USER_AGENT ?= bijux-atlas-release-check/1
RS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/rust
RS_RUN_ID ?= $(RUN_ID)
RS_TARGET_DIR ?= $(CARGO_TARGET_DIR)
RS_NEXTEST_CACHE_DIR ?= $(NEXTEST_CACHE_DIR)
RS_NEXTEST_CONFIG_HOME ?= $(abspath $(RS_ARTIFACT_ROOT)/nextest/config)
RS_PROFRAW_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/coverage/profraw)
RS_LLVM_PROFILE_FILE ?= $(abspath $(RS_PROFRAW_DIR)/default_%m_%p.profraw)
RS_COVERAGE_TARGET_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/coverage/target)
RS_FMT_REPORT ?= $(RS_ARTIFACT_ROOT)/fmt/$(RS_RUN_ID)/report.txt
RS_LINT_REPORT ?= $(RS_ARTIFACT_ROOT)/lint/$(RS_RUN_ID)/report.txt
RS_TEST_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest.log
RS_TEST_SLOW_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest-slow.log
RS_TEST_ALL_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest-all.log
RS_AUDIT_REPORT ?= $(RS_ARTIFACT_ROOT)/audit/$(RS_RUN_ID)/report.txt

audit:
	@$(MAKE) audit-rs

audit-rs: ## Run cargo dependency audit
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_AUDIT_REPORT="$(RS_AUDIT_REPORT)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" audit

check: ## Run cargo check for the workspace
	@CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo check --workspace --all-targets

coverage: ## Run workspace coverage with cargo llvm-cov + nextest
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" RS_NEXTEST_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" RS_PROFRAW_DIR="$(RS_PROFRAW_DIR)" RS_LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" RS_COVERAGE_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" NEXTEST_CONFIG_FILE="$(NEXTEST_TOML)" NEXTEST_PROFILE_ALL="$(NEXTEST_PROFILE_ALL)" NEXTEST_THREADS_ALL="$(NEXTEST_THREADS_ALL)" NEXTEST_STATUS_LEVEL="$(NEXTEST_STATUS_LEVEL)" NEXTEST_FINAL_STATUS_LEVEL="$(NEXTEST_FINAL_STATUS_LEVEL)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" coverage

fmt:
	@$(MAKE) fmt-rs

fmt-rs: ## Run cargo fmt --check
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_FMT_REPORT="$(RS_FMT_REPORT)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" fmt

lint:
	@$(MAKE) lint-rs

lint-rs: ## Run cargo clippy with warnings denied
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_LINT_REPORT="$(RS_LINT_REPORT)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" lint

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

test:
	@$(MAKE) test-rs

test-rs: ## Run workspace tests with cargo nextest
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" RS_NEXTEST_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" RS_PROFRAW_DIR="$(RS_PROFRAW_DIR)" RS_LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" RS_TEST_REPORT="$(RS_TEST_REPORT)" NEXTEST_CONFIG_FILE="$(NEXTEST_TOML)" NEXTEST_EXPR_BIN="$(NEXTEST_EXPR_BIN)" NEXTEST_PROFILE_FAST="$(NEXTEST_PROFILE_FAST)" NEXTEST_FAST_EXPR="$(NEXTEST_FAST_EXPR)" NEXTEST_STATUS_LEVEL="$(NEXTEST_STATUS_LEVEL)" NEXTEST_FINAL_STATUS_LEVEL="$(NEXTEST_FINAL_STATUS_LEVEL)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" test

test-slow: ## Run only slow_ tests with cargo nextest
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" RS_NEXTEST_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" RS_PROFRAW_DIR="$(RS_PROFRAW_DIR)" RS_LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" RS_TEST_SLOW_REPORT="$(RS_TEST_SLOW_REPORT)" NEXTEST_CONFIG_FILE="$(NEXTEST_TOML)" NEXTEST_EXPR_BIN="$(NEXTEST_EXPR_BIN)" NEXTEST_PROFILE_SLOW="$(NEXTEST_PROFILE_SLOW)" NEXTEST_SLOW_EXPR="$(NEXTEST_SLOW_EXPR)" NEXTEST_STATUS_LEVEL="$(NEXTEST_STATUS_LEVEL)" NEXTEST_FINAL_STATUS_LEVEL="$(NEXTEST_FINAL_STATUS_LEVEL)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" test-slow

test-all:
	@$(MAKE) test-all-rs

test-all-rs: ## Run all workspace tests including slow_ and ignored tests
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" RS_NEXTEST_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" RS_PROFRAW_DIR="$(RS_PROFRAW_DIR)" RS_LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" RS_TEST_ALL_REPORT="$(RS_TEST_ALL_REPORT)" NEXTEST_CONFIG_FILE="$(NEXTEST_TOML)" NEXTEST_PROFILE_ALL="$(NEXTEST_PROFILE_ALL)" NEXTEST_THREADS_ALL="$(NEXTEST_THREADS_ALL)" NEXTEST_STATUS_LEVEL="$(NEXTEST_STATUS_LEVEL)" NEXTEST_FINAL_STATUS_LEVEL="$(NEXTEST_FINAL_STATUS_LEVEL)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" test-all

test-all-frozen: ## Start a detached background full-suite run for a frozen commit and write artifacts plus frozen source under artifacts/<sha>/.
	@PINNED_REF_GATE_TARGET="test-all" "$(PINNED_REF_GATE_BIN)"

lint-frozen: ## Start a detached background lint run for a frozen commit and write artifacts plus frozen source under artifacts/<sha>/.
	@PINNED_REF_GATE_TARGET="lint" "$(PINNED_REF_GATE_BIN)"

audit-frozen: ## Start a detached background audit run for a frozen commit and write artifacts plus frozen source under artifacts/<sha>/.
	@PINNED_REF_GATE_TARGET="audit" "$(PINNED_REF_GATE_BIN)"

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

.PHONY: audit audit-rs audit-frozen check coverage fmt fmt-rs lint lint-rs lint-frozen lint-policy-report lint-policy-enforce lint-clippy-json test test-rs test-slow test-all test-all-rs test-all-frozen publish-rs
