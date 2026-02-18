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

ci-config-check:
	@$(MAKE) config-validate

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

ci-forbid-raw-paths:
	@./scripts/layout/check_no_forbidden_paths.sh

ci-make-safety:
	@python3 ./scripts/layout/check_make_safety.py

ci-make-help-drift:
	@python3 ./scripts/docs/check_make_help_drift.py

ci-init-iso-dirs:
	@mkdir -p "$${CARGO_TARGET_DIR:-artifacts/isolates/tmp/target}" "$${CARGO_HOME:-artifacts/isolates/tmp/cargo-home}" "$${TMPDIR:-artifacts/isolates/tmp/tmp}" "$${ISO_ROOT:-artifacts/isolates/tmp}"

ci-init-tmp:
	@mkdir -p "$${TMPDIR:-artifacts/isolates/tmp/tmp}" "$${ISO_ROOT:-artifacts/isolates/tmp}"

ci-dependency-lock-refresh:
	@cargo update --workspace
	@cargo generate-lockfile
	@cargo check --workspace --locked

ci-release-compat-matrix-verify:
	@$(MAKE) ci-init-tmp
	@$(MAKE) release-update-compat-matrix TAG=""
	@git diff --exit-code docs/reference/compatibility/umbrella-atlas-matrix.md

ci-release-build-artifacts:
	@$(MAKE) ci-init-iso-dirs
	@cargo build --locked --release --workspace --bins
	@mkdir -p artifacts/release
	@cp "$${CARGO_TARGET_DIR}/release/atlas-server" artifacts/release/
	@cp "$${CARGO_TARGET_DIR}/release/bijux-atlas" artifacts/release/

ci-release-notes-render:
	@$(MAKE) ci-init-tmp
	@mkdir -p artifacts/isolates/release-notes
	@sed -e "s/{{tag}}/$${GITHUB_REF_NAME}/g" \
	  -e "s/{{date}}/$$(date -u +%Y-%m-%d)/g" \
	  -e "s/{{commit}}/$${GITHUB_SHA}/g" \
	  .github/release-notes-template.md > artifacts/isolates/release-notes/RELEASE_NOTES.md

ci-release-publish-gh:
	@gh release create "$${GITHUB_REF_NAME}" \
	  --title "bijux-atlas $${GITHUB_REF_NAME}" \
	  --notes-file artifacts/isolates/release-notes/RELEASE_NOTES.md \
	  --verify-tag || \
	gh release edit "$${GITHUB_REF_NAME}" \
	  --title "bijux-atlas $${GITHUB_REF_NAME}" \
	  --notes-file artifacts/isolates/release-notes/RELEASE_NOTES.md

ci-cosign-sign:
	@[ -n "$${COSIGN_IMAGE_REF:-}" ] || { echo "COSIGN_IMAGE_REF is required"; exit 2; }
	@cosign sign --yes "$${COSIGN_IMAGE_REF}"

ci-cosign-verify:
	@[ -n "$${COSIGN_IMAGE_REF:-}" ] || { echo "COSIGN_IMAGE_REF is required"; exit 2; }
	@[ -n "$${COSIGN_CERT_IDENTITY:-}" ] || { echo "COSIGN_CERT_IDENTITY is required"; exit 2; }
	@cosign verify \
	  --certificate-identity-regexp "$${COSIGN_CERT_IDENTITY}" \
	  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
	  "$${COSIGN_IMAGE_REF}"

ci-chart-package-release:
	@helm package ops/k8s/charts/bijux-atlas --destination .cr-release-packages

ci-reproducible-verify:
	@$(MAKE) ci-init-iso-dirs
	@mkdir -p artifacts/isolates/reproducible-build
	@cargo build --release --locked --bin bijux-atlas --bin atlas-server
	@sha256sum "$${CARGO_TARGET_DIR}/release/bijux-atlas" "$${CARGO_TARGET_DIR}/release/atlas-server" > artifacts/isolates/reproducible-build/build1.sha256
	@cargo clean
	@cargo build --release --locked --bin bijux-atlas --bin atlas-server
	@sha256sum "$${CARGO_TARGET_DIR}/release/bijux-atlas" "$${CARGO_TARGET_DIR}/release/atlas-server" > artifacts/isolates/reproducible-build/build2.sha256
	@diff -u artifacts/isolates/reproducible-build/build1.sha256 artifacts/isolates/reproducible-build/build2.sha256

ci-security-advisory-render:
	@$(MAKE) ci-init-tmp
	@mkdir -p docs/security/advisories
	@DATE_UTC="$$(date -u +%Y-%m-%d)"; \
	FILE="docs/security/advisories/$${ADVISORY_ID}.md"; \
	printf '%s\n' \
	"# Security Advisory $${ADVISORY_ID}" \
	"" \
	"- Published: $${DATE_UTC}" \
	"- Severity: $${ADVISORY_SEVERITY}" \
	"- Affected versions: $${ADVISORY_AFFECTED_VERSIONS}" \
	"- Fixed version: $${ADVISORY_FIXED_VERSION}" \
	"" \
	"## Summary" \
	"$${ADVISORY_SUMMARY}" \
	"" \
	"## Mitigation" \
	"Upgrade to \`$${ADVISORY_FIXED_VERSION}\` or newer." > "$$FILE"

ci-ops-install-prereqs:
	@sudo apt-get update && sudo apt-get install -y curl netcat-openbsd shellcheck

ci-ops-install-load-prereqs:
	@sudo apt-get update && sudo apt-get install -y curl

governance-check: ## Run governance gates: layout + docs + contracts + scripts + workflow policy
	@$(MAKE) layout-check
	@$(MAKE) docs-freeze
	@$(MAKE) ssot-check
	@$(MAKE) scripts-lint
	@$(MAKE) ci-workflows-make-only
	@$(MAKE) ci-forbid-raw-paths
	@$(MAKE) ci-make-safety
	@$(MAKE) ci-make-help-drift

.PHONY: \
	ci-root-layout ci-script-entrypoints ci-fmt ci-clippy ci-test-nextest ci-deny ci-audit ci-license-check \
	ci-policy-lint ci-policy-schema-drift ci-config-check ci-ssot-drift ci-crate-structure ci-crate-docs-contract ci-cli-command-surface \
	ci-release-binaries ci-docs-build ci-latency-regression ci-store-conformance ci-openapi-drift ci-query-plan-gate \
	ci-compatibility-matrix-validate ci-runtime-security-scan-image ci-coverage ci-workflows-make-only governance-check \
	ci-make-help-drift ci-forbid-raw-paths ci-make-safety \
	ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts \
	ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify \
	ci-security-advisory-render ci-ops-install-prereqs ci-ops-install-load-prereqs
