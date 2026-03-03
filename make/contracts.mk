# Scope: contracts wrapper targets delegated to bijux-dev-atlas suites and contract runners.
# Public targets: contracts, contracts-pr, contracts-merge, contracts-release, contracts-all, contracts-fast, contracts-changed, contracts-json, contracts-ci, contracts-root, contracts-repo, contracts-crates, contracts-runtime, contracts-configs, contracts-configs-required, contracts-docs, contracts-docs-required, contracts-docker, contracts-make, contracts-make-required, contracts-ops, contracts-help
CONTRACTS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/contracts/$(RUN_ID)
CONTRACTS_DEV_ATLAS_TARGET_DIR ?= $(WORKSPACE_ROOT)/artifacts/target
CONTRACTS_DEV_ATLAS_BIN ?= $(CONTRACTS_DEV_ATLAS_TARGET_DIR)/debug/bijux-dev-atlas
CONTRACTS_EFFECT_FLAGS := --mode effect --allow-subprocess --allow-network --allow-k8s --allow-fs-write --allow-docker-daemon

_contracts_guard:
	@if [ ! -x "$(CONTRACTS_DEV_ATLAS_BIN)" ]; then \
		printf '%s\n' "build: cargo build -p bijux-dev-atlas"; \
		CARGO_TARGET_DIR="$(CONTRACTS_DEV_ATLAS_TARGET_DIR)" cargo build -q -p bijux-dev-atlas; \
	fi
	@command -v "$(CONTRACTS_DEV_ATLAS_BIN)" >/dev/null 2>&1 || { \
		printf '%s\n' "missing $(CONTRACTS_DEV_ATLAS_BIN); run: cargo build -p bijux-dev-atlas"; \
		exit 2; \
	}

contracts-help: ## Show contracts gate targets
	@$(MAKE) -s help-contract

contracts: _contracts_guard ## Run the fast static contract lane
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-pr: _contracts_guard ## Run required and static contracts for pull requests
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --lane pr --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --lane pr --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-merge: _contracts_guard ## Run required and effect contracts for merge gating
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --lane merge --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --lane merge --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-release: _contracts_guard ## Run full release contracts matrix
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all --lane release --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all --lane release --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-all: _contracts_guard ## Run the full contract suite without static skips
	@mkdir -p $(ARTIFACT_ROOT)/contracts-all/$(RUN_ID)
	@printf '%s\n' "contracts-all runs: contracts-root contracts-repo-effect contracts-crates contracts-runtime contracts-configs contracts-docs contracts-docker-effect contracts-make contracts-ops-effect"
	@printf '%s\n' \
		"full-suite: contracts-all" \
		"contracts-root (static)" \
		"contracts-repo (effect)" \
		"contracts-crates (static)" \
		"contracts-runtime (static)" \
		"contracts-configs (static)" \
		"contracts-docs (static)" \
		"contracts-docker (effect)" \
		"contracts-make (static)" \
		"contracts-ops (effect)" \
		> $(ARTIFACT_ROOT)/contracts-all/$(RUN_ID)/manifest.txt
	@set -eu; \
	summary_file="$(ARTIFACT_ROOT)/contracts-all/$(RUN_ID)/summary.txt"; \
	: > "$$summary_file"; \
	printf '%-15s %-8s %10s %10s %10s %10s %10s %10s\n' "group" "mode" "contracts" "tests" "pass" "fail" "skip" "error" >> "$$summary_file"; \
	printf '%-15s %-8s %10s %10s %10s %10s %10s %10s\n' "---------------" "--------" "----------" "----------" "----------" "----------" "----------" "----------" >> "$$summary_file"; \
	total_contracts=0; \
	total_tests=0; \
	total_pass=0; \
	total_fail=0; \
	total_skip=0; \
	total_error=0; \
	run_and_capture() { \
		group="$$1"; \
		mode="$$2"; \
		shift 2; \
		log_file="$(ARTIFACT_ROOT)/contracts-all/$(RUN_ID)/$${group}.log"; \
		"$$@" > "$$log_file" 2>&1; \
		cat "$$log_file"; \
		stats="$$(awk '/^Summary:/ { c = $$2; t = $$4; p = $$6; f = $$8; s = $$10; e = $$12 } END { if (c != "") { print c, t, p, f, s, e } }' "$$log_file")"; \
		if [ -z "$$stats" ]; then \
			printf '%s\n' "missing summary line for $$group" >&2; \
			exit 1; \
		fi; \
		set -- $$stats; \
		printf '%-15s %-8s %10s %10s %10s %10s %10s %10s\n' "$$group" "$$mode" "$$1" "$$2" "$$3" "$$4" "$$5" "$$6" >> "$$summary_file"; \
		total_contracts=$$((total_contracts + $$1)); \
		total_tests=$$((total_tests + $$2)); \
		total_pass=$$((total_pass + $$3)); \
		total_fail=$$((total_fail + $$4)); \
		total_skip=$$((total_skip + $$5)); \
		total_error=$$((total_error + $$6)); \
	}; \
	run_and_capture "root" "static" $(MAKE) -s contracts-root; \
	run_and_capture "repo" "effect" $(DEV_ATLAS) contracts repo $(CONTRACTS_EFFECT_FLAGS) --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT); \
	run_and_capture "crates" "static" $(MAKE) -s contracts-crates; \
	run_and_capture "runtime" "static" $(MAKE) -s contracts-runtime; \
	run_and_capture "configs" "static" $(MAKE) -s contracts-configs; \
	run_and_capture "docs" "static" $(MAKE) -s contracts-docs; \
	run_and_capture "docker" "effect" $(DEV_ATLAS) contracts docker $(CONTRACTS_EFFECT_FLAGS) --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT); \
	run_and_capture "make" "static" $(MAKE) -s contracts-make; \
	run_and_capture "ops" "effect" $(DEV_ATLAS) contracts ops $(CONTRACTS_EFFECT_FLAGS) --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT); \
	printf '%-15s %-8s %10s %10s %10s %10s %10s %10s\n' "contracts-all" "mixed" "$$total_contracts" "$$total_tests" "$$total_pass" "$$total_fail" "$$total_skip" "$$total_error" >> "$$summary_file"; \
	printf '\n%s\n' "contracts-all summary"; \
	cat "$$summary_file"

contracts-fast: _contracts_guard ## Run static-only contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-changed: _contracts_guard ## Run changed-only contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --changed-only --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --changed-only --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-json: _contracts_guard ## Run all contracts and emit json
	@printf '%s\n' "run: $(DEV_ATLAS) contracts all --mode static --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts all --mode static --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-ci: _contracts_guard ## Run strict CI contracts lane
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) contracts all $(CONTRACTS_EFFECT_FLAGS) --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) contracts all $(CONTRACTS_EFFECT_FLAGS) --format json --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-root: _contracts_guard ## Run root contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts root --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts root --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-repo: _contracts_guard ## Run repository contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts repo --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts repo --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-crates: _contracts_guard ## Run crate contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts crates --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts crates --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-runtime: _contracts_guard ## Run runtime contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts runtime --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts runtime --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-configs: _contracts_guard ## Run configs contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts configs --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts configs --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-configs-required: _contracts_guard ## Run PR-required configs suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite configs_required --include-internal --include-slow --format json"
	@$(DEV_ATLAS) check run --suite configs_required --include-internal --include-slow --format json

contracts-docs: _contracts_guard ## Run docs contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts docs --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts docs --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-docs-required: _contracts_guard ## Run PR-required docs suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite docs_required --include-internal --include-slow --format json"
	@$(DEV_ATLAS) check run --suite docs_required --include-internal --include-slow --format json

contracts-docker: _contracts_guard ## Run docker contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts docker --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts docker --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-make: _contracts_guard ## Run make contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts make --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts make --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-make-required: _contracts_guard ## Run PR-required make suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) check run --suite make_required --include-internal --include-slow --format json"
	@$(DEV_ATLAS) check run --suite make_required --include-internal --include-slow --format json

contracts-ops: _contracts_guard ## Run ops contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contracts ops --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contracts ops --mode static --format human --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

.PHONY: _contracts_guard contracts-help contracts contracts-pr contracts-merge contracts-release contracts-all contracts-fast contracts-changed contracts-json contracts-ci contracts-root contracts-repo contracts-crates contracts-runtime contracts-configs contracts-configs-required contracts-docs contracts-docs-required contracts-docker contracts-make contracts-make-required contracts-ops
