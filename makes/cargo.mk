# Scope: canonical Rust cargo gates delegated to cargo-native tooling.
# Public targets: none (internal cargo execution surface)
SHELL := /bin/bash
CARGO_TERM_PROGRESS_WHEN ?= always
CARGO_TERM_PROGRESS_WIDTH ?= 120
CARGO_TERM_VERBOSE ?= false
CARGO_TERM_COLOR ?= always
PINNED_REF_GATE_BIN ?= makes/bin/run_pinned_ref_gate.sh
RS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/rust
RS_RUN_ID ?= $(RUN_ID)
RS_FMT_REPORT ?= $(RS_ARTIFACT_ROOT)/fmt/$(RS_RUN_ID)/report.txt
RS_LINT_REPORT ?= $(RS_ARTIFACT_ROOT)/lint/$(RS_RUN_ID)/report.txt
RS_TEST_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest.log
RS_TEST_SLOW_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest-slow.log
RS_TEST_ALL_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest-all.log
RS_AUDIT_REPORT ?= $(RS_ARTIFACT_ROOT)/audit/$(RS_RUN_ID)/report.txt

cleanup_root_nextest = \
	if [ -d "$(CURDIR)/target/nextest" ]; then rm -rf "$(CURDIR)/target/nextest"; fi; \
	if [ -d "$(CURDIR)/target" ]; then rm -rf "$(CURDIR)/target"; fi

nextest_summary = \
	summary_line=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$$report_file" | grep 'Summary \[' | tail -n 1); \
	set -- $$(printf '%s\n' "$$summary_line" | awk ' \
		{ \
			for (i = 1; i <= NF; i++) { \
				prev = (i > 1) ? $$(i - 1) : $$1; \
				gsub(/[^0-9]/, "", prev); \
				if ($$i ~ /^test/) total = prev; \
				else if ($$i ~ /^passed/) passed = prev; \
				else if ($$i ~ /^failed/) failed = prev; \
				else if ($$i ~ /^skipped/) skipped = prev; \
			} \
		} \
		END { \
			printf "%s %s %s %s\n", total + 0, passed + 0, failed + 0, skipped + 0; \
		}'); \
	total=$$1; \
	passed=$$2; \
	failed=$$3; \
	skipped=$$4; \
	leaky=$$(grep -c ' LEAK ' "$$report_file" || true); \
	max_list_items=50; \
	failed_tests=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$$report_file" | awk '/ FAIL / { test_name = $$0; sub(/^.* FAIL \[[^]]*\] \([^)]*\) /, "", test_name); seen[test_name] = 1 } END { for (test_name in seen) print test_name }' | LC_ALL=C sort); \
	skipped_tests=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$$report_file" | awk '/ SKIP / { test_name = $$0; sub(/^.* SKIP \[[^]]*\] \([^)]*\) /, "", test_name); seen[test_name] = 1 } END { for (test_name in seen) print test_name }' | LC_ALL=C sort); \
	leaky_tests=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$$report_file" | awk '/ LEAK / { test_name = $$0; sub(/^.* LEAK \[[^]]*\] \([^)]*\) /, "", test_name); seen[test_name] = 1 } END { for (test_name in seen) print test_name }' | LC_ALL=C sort); \
	print_test_group() { \
		label="$$1"; color="$$2"; tests="$$3"; \
		[ -n "$$tests" ] || return 0; \
		total_items=$$(printf '%s\n' "$$tests" | sed '/^$$/d' | wc -l | tr -d ' '); \
		printf '\033[%sm%s\033[0m\n' "$$color" "$$label"; \
		printf '%s\n' "$$tests" | sed '/^$$/d' | head -n "$$max_list_items" | sed 's/^/  /'; \
		if [ "$$total_items" -gt "$$max_list_items" ]; then \
			printf '  ... %s more\n' "$$((total_items - max_list_items))"; \
		fi; \
	}; \
	printf '\033[1;36m%s\033[0m total=%s \033[1;32mpassed=%s\033[0m \033[1;31mfailed=%s\033[0m \033[1;33mskipped=%s\033[0m \033[1;35mleaky=%s\033[0m\n' "nextest-summary:" "$$total" "$$passed" "$$failed" "$$skipped" "$$leaky"; \
	print_test_group "failed-tests:" "1;31" "$$failed_tests"; \
	print_test_group "leaky-tests:" "1;35" "$$leaky_tests"; \
	print_test_group "skipped-tests:" "1;33" "$$skipped_tests"

audit:
	@$(MAKE) audit-rs

audit-rs: ## Run cargo dependency audit
	@mkdir -p "$(dir $(RS_AUDIT_REPORT))"
	@{ \
		$(DEV_ATLAS) security dependency-audit --repo-root "$(CURDIR)" --format json >/dev/null && \
		command -v cargo-audit >/dev/null 2>&1 || (echo "cargo-audit is required. Install with: cargo install cargo-audit"; exit 1); \
		CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo audit; \
	} 2>&1 | tee "$(RS_AUDIT_REPORT)"

check: ## Run cargo check for the workspace
	@CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo check --workspace --all-targets

coverage: ## Run workspace coverage with cargo llvm-cov + nextest
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	@mkdir -p artifacts/coverage
	@mkdir -p artifacts/coverage/profraw
	@status=0; \
	LLVM_PROFILE_FILE="$(CURDIR)/artifacts/coverage/profraw/default_%m_%p.profraw" CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" cargo llvm-cov nextest --color always --workspace --all-features --lcov --output-path artifacts/coverage/lcov.info --config-file configs/rust/nextest.toml --run-ignored all --cargo-quiet || status=$$?; \
	$(cleanup_root_nextest); \
	test $$status -eq 0
	@CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo llvm-cov report

fmt:
	@$(MAKE) fmt-rs

fmt-rs: ## Run cargo fmt --check
	@printf '%s\n' "run: cargo fmt --all -- --check --config-path configs/sources/repository/rust-tooling/rustfmt.toml"
	@mkdir -p "$(dir $(RS_FMT_REPORT))"
	@output="$$(CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo fmt --all -- --check --config-path configs/sources/repository/rust-tooling/rustfmt.toml 2>&1)"; \
	status=$$?; \
	printf '%s\n' "$$output" | tee "$(RS_FMT_REPORT)"; \
	if [ $$status -eq 0 ]; then \
		printf '%s\n' "fmt check complete"; \
	fi; \
	exit $$status

lint:
	@$(MAKE) lint-rs

lint-rs: ## Run cargo clippy with warnings denied
	@printf '%s\n' "run: cargo clippy -p bijux-dev-atlas --all-targets --all-features --locked --no-deps -- -D warnings"
	@printf '%s\n' "run: cargo check -p bijux-atlas --all-targets --all-features --locked"
	@mkdir -p "$(dir $(RS_LINT_REPORT))"
	@{ \
		CLIPPY_CONF_DIR=configs/rust CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo clippy -p bijux-dev-atlas --all-targets --all-features --locked --no-deps -- -D warnings && \
		CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo check -p bijux-atlas --all-targets --all-features --locked; \
	} 2>&1 | tee "$(RS_LINT_REPORT)"

lint-policy-report: ## Emit effective lint policy report artifact
	@$(DEV_ATLAS) makes lint-policy-report --allow-write --format $(FORMAT)

lint-policy-enforce: ## Enforce repository lint drift guards
	@! rg -n '\btodo!\(' crates
	@! rg -n '\bdbg!\(' crates
	@! rg -n '\b(?:println|eprintln)!\(' crates/bijux-atlas/src crates/bijux-dev-atlas/src --glob '!**/tests/**' --glob '!**/benches/**' --glob '!**/main.rs' --glob '!**/bin/**'
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
	@command -v cargo-nextest >/dev/null 2>&1 || { \
		echo "cargo-nextest is required. Install with: cargo install cargo-nextest"; \
		exit 1; \
	}
	@printf '%s\n' "run: cargo nextest run --workspace --profile $${NEXTEST_PROFILE:-default} --status-level $${NEXTEST_STATUS_LEVEL:-all} --final-status-level $${NEXTEST_FINAL_STATUS_LEVEL:-all}"
	@mkdir -p "$(dir $(RS_TEST_REPORT))" "$(CARGO_TARGET_DIR)" "$(NEXTEST_CACHE_DIR)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@status=0; report_file="$(RS_TEST_REPORT)"; \
	cleanup() { $(cleanup_root_nextest); }; trap cleanup EXIT INT TERM; \
	CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" cargo nextest run --color always --workspace --config-file configs/rust/nextest.toml --target-dir "$(CARGO_TARGET_DIR)" --profile "$${NEXTEST_PROFILE:-default}" --status-level "$${NEXTEST_STATUS_LEVEL:-all}" --final-status-level "$${NEXTEST_FINAL_STATUS_LEVEL:-all}" -E "$${NEXTEST_FILTER_EXPR:-not test(/(^|::)slow_/)}" 2>&1 | tee "$$report_file"; \
	status=$${PIPESTATUS[0]}; \
	$(nextest_summary); \
	trap - EXIT INT TERM; cleanup; \
	test $$status -eq 0

test-slow: ## Run only slow_ tests with cargo nextest
	@command -v cargo-nextest >/dev/null 2>&1 || { \
		echo "cargo-nextest is required. Install with: cargo install cargo-nextest"; \
		exit 1; \
	}
	@printf '%s\n' "run: cargo nextest run --workspace --profile $${NEXTEST_PROFILE:-default} --status-level $${NEXTEST_STATUS_LEVEL:-all} --final-status-level $${NEXTEST_FINAL_STATUS_LEVEL:-all} -E test(/(^|::)slow_/)"
	@mkdir -p "$(dir $(RS_TEST_SLOW_REPORT))" "$(CARGO_TARGET_DIR)" "$(NEXTEST_CACHE_DIR)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@status=0; report_file="$(RS_TEST_SLOW_REPORT)"; \
	cleanup() { $(cleanup_root_nextest); }; trap cleanup EXIT INT TERM; \
	CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" cargo nextest run --color always --cargo-quiet --workspace --config-file configs/rust/nextest.toml --target-dir "$(CARGO_TARGET_DIR)" --profile "$${NEXTEST_PROFILE:-default}" --status-level "$${NEXTEST_STATUS_LEVEL:-all}" --final-status-level "$${NEXTEST_FINAL_STATUS_LEVEL:-all}" -E "test(/(^|::)slow_/)" 2>&1 | tee "$$report_file"; \
	status=$${PIPESTATUS[0]}; \
	$(nextest_summary); \
	trap - EXIT INT TERM; cleanup; \
	test $$status -eq 0

test-all:
	@$(MAKE) test-all-rs

test-all-rs: ## Run all workspace tests including slow_ and ignored tests
	@command -v cargo-nextest >/dev/null 2>&1 || { \
		echo "cargo-nextest is required. Install with: cargo install cargo-nextest"; \
		exit 1; \
	}
	@printf '%s\n' "run: cargo nextest run --workspace --all-features --run-ignored all --retries 0 --profile $${NEXTEST_PROFILE:-default} --status-level $${NEXTEST_STATUS_LEVEL:-all} --final-status-level $${NEXTEST_FINAL_STATUS_LEVEL:-all}"
	@mkdir -p "$(dir $(RS_TEST_ALL_REPORT))" "$(CARGO_TARGET_DIR)" "$(NEXTEST_CACHE_DIR)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@status=0; report_file="$(RS_TEST_ALL_REPORT)"; \
	cleanup() { $(cleanup_root_nextest); }; trap cleanup EXIT INT TERM; \
	CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" cargo nextest run --color always --cargo-quiet --workspace --all-features --config-file configs/rust/nextest.toml --target-dir "$(CARGO_TARGET_DIR)" --run-ignored all --retries 0 --profile "$${NEXTEST_PROFILE:-default}" --status-level "$${NEXTEST_STATUS_LEVEL:-all}" --final-status-level "$${NEXTEST_FINAL_STATUS_LEVEL:-all}" 2>&1 | tee "$$report_file"; \
	status=$${PIPESTATUS[0]}; \
	$(nextest_summary); \
	trap - EXIT INT TERM; cleanup; \
	test $$status -eq 0

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
				status="$$(curl -fsS -o /dev/null -w '%{http_code}' "https://crates.io/api/v1/crates/$$pkg/$(RELEASE_VERSION)" || true)"; \
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
					status="$$(curl -fsS -o /dev/null -w '%{http_code}' "https://crates.io/api/v1/crates/$$pkg/$(RELEASE_VERSION)" || true)"; \
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
