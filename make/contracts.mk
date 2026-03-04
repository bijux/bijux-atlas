# Scope: contracts wrapper targets delegated to bijux-dev-atlas contract and suite runners.
# Public targets: contract, contract-effect, contract-all, contract-list, contract-report, contracts, contracts-pr, contracts-merge, contracts-release, contracts-all, contracts-fast, contracts-changed, contracts-json, contracts-ci, contracts-root, contracts-repo, contracts-crates, contracts-runtime, contracts-configs, contracts-configs-required, contracts-docs, contracts-docs-required, contracts-docker, contracts-make, contracts-make-required, contracts-ops, contracts-help, contracts-group, contracts-tag, contracts-pure, contracts-effect
# Wrapper rule: canonical contract targets may only delegate to $(DEV_ATLAS); no grep/jq/sed/awk parsing is allowed.
CONTRACTS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/contracts/$(RUN_ID)
CONTRACTS_DEV_ATLAS_TARGET_DIR ?= $(WORKSPACE_ROOT)/artifacts/target
CONTRACTS_DEV_ATLAS_BIN ?= $(CONTRACTS_DEV_ATLAS_TARGET_DIR)/debug/bijux-dev-atlas
CONTRACTS_EFFECT_FLAGS := --mode effect --allow-subprocess --allow-network --allow-k8s --allow-fs-write --allow-docker-daemon
NO_ANSI ?= 0
CONTRACT_FAIL_FAST_FLAG := $(if $(filter 1 true yes,$(FAIL_FAST)),--fail-fast,--no-fail-fast)
CONTRACT_NO_ANSI_FLAG := $(if $(filter 1 true yes,$(NO_ANSI)),--no-ansi,)

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

contract: _contracts_guard ## Run static contract execution through the canonical contract runner
	@$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode static --jobs $(JOBS) $(CONTRACT_FAIL_FAST_FLAG) $(CONTRACT_NO_ANSI_FLAG) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contract-effect: _contracts_guard ## Run effect contract execution through the canonical contract runner
	@$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode effect --effects-policy allow --jobs $(JOBS) $(CONTRACT_FAIL_FAST_FLAG) $(CONTRACT_NO_ANSI_FLAG) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contract-all: _contracts_guard ## Run the complete contract execution set through the canonical contract runner
	@$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode all --effects-policy allow --jobs $(JOBS) $(CONTRACT_FAIL_FAST_FLAG) $(CONTRACT_NO_ANSI_FLAG) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contract-list: _contracts_guard ## List canonical contracts exposed by the contract runner
	@$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract list

contract-report: _contracts_guard ## Read the last canonical contract run summary from the artifacts root
	@$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract report --last --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts: ## Deprecated alias for make contract
	@printf '%s\n' "deprecated: use \`make contract\`"
	@$(MAKE) -s contract JOBS="$(JOBS)" FAIL_FAST="$(FAIL_FAST)" NO_ANSI="$(NO_ANSI)" FORMAT="$(FORMAT)"

contracts-pr: _contracts_guard ## Run required and static contracts for pull requests
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode static --jobs $(JOBS) --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode static --jobs $(JOBS) --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-merge: _contracts_guard ## Run required and effect contracts for merge gating
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode all --effects-policy allow --jobs $(JOBS) --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode all --effects-policy allow --jobs $(JOBS) --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-release: _contracts_guard ## Run full release contracts matrix
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode all --effects-policy allow --jobs $(JOBS) --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode all --effects-policy allow --jobs $(JOBS) --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-all: ## Deprecated alias for make contract-all
	@printf '%s\n' "deprecated: use \`make contract-all\`"
	@$(MAKE) -s contract-all JOBS="$(JOBS)" FAIL_FAST="$(FAIL_FAST)" NO_ANSI="$(NO_ANSI)" FORMAT="$(FORMAT)"

contracts-group: _contracts_guard ## Run one contracts suite group (GROUP=<name>)
	@[ -n "$${GROUP:-}" ] || { echo "usage: make contracts-group GROUP=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) suites run --suite contracts --group "$${GROUP}" --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format $(FORMAT)

contracts-tag: _contracts_guard ## Run contracts suite entries with a shared tag (TAG=<name>)
	@[ -n "$${TAG:-}" ] || { echo "usage: make contracts-tag TAG=<name>" >&2; exit 2; }
	@$(DEV_ATLAS) suites run --suite contracts --tag "$${TAG}" --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format $(FORMAT)

contracts-pure: _contracts_guard ## Run only pure contracts suite entries
	@$(DEV_ATLAS) suites run --suite contracts --mode pure --jobs $(JOBS) $(SUITE_FAIL_FAST_FLAG) --format $(FORMAT)

contracts-effect: ## Deprecated alias for make contract-effect
	@printf '%s\n' "deprecated: use \`make contract-effect\`"
	@$(MAKE) -s contract-effect JOBS="$(JOBS)" FAIL_FAST="$(FAIL_FAST)" NO_ANSI="$(NO_ANSI)" FORMAT="$(FORMAT)"

contracts-fast: ## Deprecated alias for make contract
	@printf '%s\n' "deprecated: use \`make contract\`"
	@$(MAKE) -s contract JOBS="$(JOBS)" FAIL_FAST="$(FAIL_FAST)" NO_ANSI="$(NO_ANSI)" FORMAT="$(FORMAT)"

contracts-changed: _contracts_guard ## Run changed-only contracts
	@printf '%s\n' "run: $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode static --jobs $(JOBS) --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode static --jobs $(JOBS) --color always --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-json: _contracts_guard ## Run all contracts and emit json
	@printf '%s\n' "run: $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode static --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode static --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-ci: _contracts_guard ## Run strict CI contracts lane
	@printf '%s\n' "run: CI=1 $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode all --effects-policy allow --jobs $(JOBS) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@CI=1 $(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode all --effects-policy allow --jobs $(JOBS) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-root: _contracts_guard ## Run root contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain root --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contract run --mode static --domain root --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-repo: _contracts_guard ## Run repository contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain repo --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contract run --mode static --domain repo --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-crates: _contracts_guard ## Run crate contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain crates --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contract run --mode static --domain crates --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-runtime: _contracts_guard ## Run runtime contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain runtime --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contract run --mode static --domain runtime --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-configs: _contracts_guard ## Run configs contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain configs --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contract run --mode static --domain configs --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-configs-required: _contracts_guard ## Run PR-required configs suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) checks run --suite configs_required --include-internal --include-slow --format $(FORMAT)"
	@$(DEV_ATLAS) checks run --suite configs_required --include-internal --include-slow --format $(FORMAT)

contracts-docs: _contracts_guard ## Run docs contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain docs --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contract run --mode static --domain docs --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-docs-required: _contracts_guard ## Run PR-required docs suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) checks run --suite docs_required --include-internal --include-slow --format $(FORMAT)"
	@$(DEV_ATLAS) checks run --suite docs_required --include-internal --include-slow --format $(FORMAT)

contracts-docker: _contracts_guard ## Run docker contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain docker --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contract run --mode static --domain docker --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-make: _contracts_guard ## Run make contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain make --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contract run --mode static --domain make --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

contracts-make-required: _contracts_guard ## Run PR-required make suite in static mode
	@printf '%s\n' "run: $(DEV_ATLAS) checks run --suite make_required --include-internal --include-slow --format $(FORMAT)"
	@$(DEV_ATLAS) checks run --suite make_required --include-internal --include-slow --format $(FORMAT)

contracts-ops: _contracts_guard ## Run ops contracts
	@printf '%s\n' "run: $(DEV_ATLAS) contract run --mode static --domain ops --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
	@$(DEV_ATLAS) contract run --mode static --domain ops --color always --format $(FORMAT) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)

.PHONY: _contracts_guard contract contract-effect contract-all contract-list contract-report contracts-help contracts contracts-pr contracts-merge contracts-release contracts-all contracts-changed contracts-ci contracts-configs contracts-crates contracts-docker contracts-docs contracts-effect contracts-fast contracts-group contracts-json contracts-make contracts-make-required contracts-merge contracts-ops contracts-pr contracts-pure contracts-release contracts-repo contracts-root contracts-runtime contracts-tag
