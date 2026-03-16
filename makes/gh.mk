# Scope: GitHub Actions helper targets for thin public workflows.
# Public targets: none
SHELL := /bin/bash

GH_RELEASE_TAG_PATTERN ?= ^v[0-9]+\.[0-9]+\.[0-9]+$$
GH_CRATES_RELEASE_PACKAGES ?= bijux-atlas
GH_RELEASE_CI_WORKFLOW_FILE ?= ci.yml
GH_RELEASE_CI_WAIT_TIMEOUT_SECONDS ?= 1800
GH_RELEASE_CI_POLL_INTERVAL_SECONDS ?= 15
GH_RELEASE_CI_LOOKBACK_SECONDS ?= 120
GH_RELEASE_CI_APPEARANCE_GRACE_SECONDS ?= 20
GH_TEST_CARGO_NEXTEST_VERSION ?= 0.9.100

.PHONY: gh-fmt gh-lint gh-security gh-test gh-test-install-rust-tools gh-docs-install \
	gh-release-plan-github gh-release-plan-crates gh-release-require-cargo-token \
	gh-release-wait-for-ci

gh-fmt: fmt ## Run GitHub formatting checks without modifying files

gh-lint: lint ## Run GitHub lint checks

gh-security: ## Run GitHub security checks through the Rust control plane
	@mkdir -p artifacts/governance "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
	@cargo run --locked -q -p bijux-dev-atlas -- governance exceptions validate --repo-root "$(CURDIR)" --format json || true
	@cargo run --locked -q -p bijux-dev-atlas -- governance deprecations validate --repo-root "$(CURDIR)" --format json || true
	@cargo run --locked -q -p bijux-dev-atlas -- governance breaking validate --repo-root "$(CURDIR)" --format json || true
	@cargo run --locked -q -p bijux-dev-atlas -- governance doctor --repo-root "$(CURDIR)" --format json || true
	@for file in \
		artifacts/governance/exceptions-summary.json \
		artifacts/governance/exceptions-expiry-warning.json \
		artifacts/governance/exceptions-churn.json \
		artifacts/governance/deprecations-summary.json \
		artifacts/governance/compat-warnings.json \
		artifacts/governance/breaking-changes.json \
		artifacts/governance/governance-doctor.json \
		artifacts/governance/institutional-delta.md \
		artifacts/governance/institutional-delta-inputs.json; do \
		test -f "$${file}" || { echo "missing governance evidence file: $${file}" >&2; exit 1; }; \
	done
	@cargo run --locked -q -p bijux-dev-atlas -- security validate --format json
	@cargo run --locked -q -p bijux-dev-atlas -- security dependency-audit --format json

gh-test: test ## Run GitHub test suites

gh-test-install-rust-tools: ## Install cargo-nextest that matches the pinned CI toolchain
	@cargo install --locked cargo-nextest --version "$(GH_TEST_CARGO_NEXTEST_VERSION)"

gh-docs-install: ## Install the documentation toolchain for GitHub Actions
	@python3 -m pip install -r configs/sources/repository/docs/requirements.lock.txt
	@npm ci --prefix configs/sources/repository/docs

gh-release-plan-github: ## Determine whether the tagged commit should publish a GitHub Release
	@$(call require_var,GITHUB_OUTPUT)
	@$(call require_var,TARGET_SHA)
	@set -euo pipefail; \
	git fetch --tags --force --prune >/dev/null 2>&1; \
	tags="$$(git tag --points-at "$(TARGET_SHA)" | grep -E "$(GH_RELEASE_TAG_PATTERN)" || true)"; \
	if [ -z "$${tags}" ]; then \
		echo "publish=false" >> "$${GITHUB_OUTPUT}"; \
		exit 0; \
	fi; \
	tag="$$(printf '%s\n' "$${tags}" | head -n 1)"; \
	version="$${tag#v}"; \
	{ \
		echo "publish=true"; \
		echo "tag=$${tag}"; \
		echo "version=$${version}"; \
	} >> "$${GITHUB_OUTPUT}"

gh-release-plan-crates: ## Determine whether the tagged commit should publish workspace crates
	@$(call require_var,GITHUB_OUTPUT)
	@$(call require_var,TARGET_SHA)
	@set -euo pipefail; \
	git fetch --tags --force --prune >/dev/null 2>&1; \
	tags="$$(git tag --points-at "$(TARGET_SHA)" | grep -E "$(GH_RELEASE_TAG_PATTERN)" || true)"; \
	if [ -z "$${tags}" ]; then \
		echo "publish=false" >> "$${GITHUB_OUTPUT}"; \
		exit 0; \
	fi; \
	tag="$$(printf '%s\n' "$${tags}" | head -n 1)"; \
	version="$${tag#v}"; \
	unpublished=""; \
	for package in $(GH_CRATES_RELEASE_PACKAGES); do \
		status=""; \
		for attempt in 1 2 3 4 5; do \
			status="$$(curl -fsS -o /dev/null -w '%{http_code}' "https://crates.io/api/v1/crates/$${package}/$${version}" || true)"; \
			if [ -n "$${status}" ] && [ "$${status}" != "000" ]; then \
				break; \
			fi; \
			sleep "$${attempt}"; \
		done; \
		if [ "$${status}" != "200" ]; then \
			if [ -n "$${unpublished}" ]; then unpublished="$${unpublished} "; fi; \
			unpublished="$${unpublished}$${package}"; \
		fi; \
	done; \
	if [ -z "$${unpublished}" ]; then \
		{ \
			echo "publish=false"; \
			echo "packages="; \
			echo "tag=$${tag}"; \
			echo "version=$${version}"; \
		} >> "$${GITHUB_OUTPUT}"; \
		exit 0; \
	fi; \
	{ \
		echo "publish=true"; \
		echo "packages=$${unpublished}"; \
		echo "tag=$${tag}"; \
		echo "version=$${version}"; \
	} >> "$${GITHUB_OUTPUT}"

gh-release-require-cargo-token: ## Verify that crates.io credentials are available
	@$(call require_var,CARGO_REGISTRY_TOKEN)

gh-release-wait-for-ci: ## Wait for the latest CI run on the release SHA to succeed
	@$(call require_var,GITHUB_TOKEN)
	@$(call require_var,GITHUB_REPOSITORY)
	@$(call require_var,TARGET_SHA)
	@$(call require_var,CI_WAIT_STARTED_AT)
	@GH_RELEASE_CI_WORKFLOW_FILE="$(GH_RELEASE_CI_WORKFLOW_FILE)" \
	GH_RELEASE_CI_WAIT_TIMEOUT_SECONDS="$(GH_RELEASE_CI_WAIT_TIMEOUT_SECONDS)" \
	GH_RELEASE_CI_POLL_INTERVAL_SECONDS="$(GH_RELEASE_CI_POLL_INTERVAL_SECONDS)" \
	GH_RELEASE_CI_LOOKBACK_SECONDS="$(GH_RELEASE_CI_LOOKBACK_SECONDS)" \
	GH_RELEASE_CI_APPEARANCE_GRACE_SECONDS="$(GH_RELEASE_CI_APPEARANCE_GRACE_SECONDS)" \
	python3 .github/scripts/wait_for_ci.py
