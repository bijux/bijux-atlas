# Scope: CI mapping and workflow-specific internal targets.
# Public targets: none
SHELL := /bin/sh

ci-root-layout:
	@$(MAKE) layout-check

ci-script-entrypoints:
	@$(MAKE) no-direct-scripts

ci-rename-lint:
	@$(MAKE) rename-lint

ci-docs-lint-names:
	@$(MAKE) docs-lint-names

ci-fmt:
	@$(MAKE) fmt

ci-clippy:
	@$(MAKE) _lint-clippy

ci-test-nextest:
	@$(MAKE) test

ci-deny:
	@if ! cargo +stable deny --version >/dev/null 2>&1; then cargo +stable install cargo-deny --locked; fi
	@cargo +stable deny check --config configs/security/deny.toml

ci-audit:
	@if ! cargo audit --version >/dev/null 2>&1; then cargo install cargo-audit --locked; fi
	@cargo audit

ci-license-check:
	@if ! cargo +stable deny --version >/dev/null 2>&1; then cargo +stable install cargo-deny --locked; fi
	@cargo +stable deny check licenses --config configs/security/deny.toml

ci-policy-lint:
	@$(MAKE) policy-lint

ci-policy-schema-drift:
	@$(MAKE) policy-schema-drift

ci-policy-relaxations:
	@$(MAKE) policy-audit

ci-policy-enforcement:
	@$(MAKE) policy-enforcement-status

ci-policy-allow-env:
	@$(MAKE) policy-allow-env-lint

ci-policy-boundaries:
	@$(MAKE) policies/boundaries-check

ci-ops-policy-audit:
	@$(MAKE) ops-policy-audit

ci-config-check:
	@$(MAKE) configs-check

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
	@cargo test -p bijux-atlas-server --test p99-regression --locked

ci-store-conformance-localfs:
	@cargo test -p bijux-atlas-store --test store_contract store_errors_have_stable_codes -- --exact
	@cargo test -p bijux-atlas-store --test store_contract verified_sqlite_read_rejects_checksum_mismatch -- --exact

ci-store-conformance-http:
	@cargo test -p bijux-atlas-store --test store_contract cached_only_mode_never_touches_network -- --exact
	@cargo test -p bijux-atlas-store --test store_contract http_store_blocks_private_ssrf_targets -- --exact

ci-store-conformance-s3:
	@cargo test -p bijux-atlas-store --test store_contract s3_store_uses_etag_cache_and_handles_304_for_catalog -- --exact
	@cargo test -p bijux-atlas-store --test store_contract s3_cached_only_mode_is_conformance_compatible -- --exact
	@cargo test -p bijux-atlas-server --test s3_backend --locked

ci-store-conformance:
	@$(MAKE) ci-store-conformance-localfs
	@$(MAKE) ci-store-conformance-http
	@$(MAKE) ci-store-conformance-s3

ci-openapi-drift:
	@$(MAKE) openapi-drift
	@$(ATLAS_SCRIPTS) contracts check --checks breakage

ci-chart-schema-validate:
	@$(MAKE) ops-values-validate

ci-api-contract:
	@$(MAKE) api-contract-check

ci-query-plan-gate:
	@$(MAKE) query-plan-gate

ci-critical-query-check:
	@$(MAKE) critical-query-check

ci-sqlite-schema-drift:
	@cargo test -p bijux-atlas-ingest sqlite::tests::schema_drift_gate_sqlite_master_digest_is_stable --locked

ci-sqlite-index-drift:
	@cargo test -p bijux-atlas-ingest sqlite::tests::index_drift_gate_required_indexes_exist --locked

ci-ingest-determinism:
	@cargo test -p bijux-atlas-ingest tests::deterministic_across_parallelism_settings --locked
	@cargo test -p bijux-atlas-ingest tests::tiny_fixture_matches_cross_machine_golden_hashes --locked

ci-qc-fixtures:
	@./scripts/areas/public/qc-fixtures-gate.sh

ci-compatibility-matrix-validate:
	@$(MAKE) compat-matrix-validate

ci-runtime-security-scan-image:
	@$(MAKE) docker-contracts
	@$(MAKE) docker-build
	@$(MAKE) docker-scan

ci-coverage:
	@if ! cargo llvm-cov --version >/dev/null 2>&1; then cargo install cargo-llvm-cov --locked; fi
	@cargo llvm-cov --workspace --all-features --lcov --output-path artifacts/isolate/coverage/lcov.info

ci-workflows-make-only:
	@$(ATLAS_SCRIPTS) check forbidden-paths

ci-log-fields-contract:
	@$(ATLAS_SCRIPTS) run ./packages/bijux-atlas-scripts/src/bijux_atlas_scripts/obs/validate_logs_schema.py --file ops/obs/contract/logs.example.jsonl

ci-observability-pack-test:
	@$(MAKE) observability-pack-test

ci-observability-pack-drills:
	@$(MAKE) observability-pack-drills

ci-ops-index-surface:
	@$(ATLAS_SCRIPTS) ops surface

ci-ops-gen-check:
	@$(MAKE) -s ops-gen-check

ci-ops-run-entrypoints:
	@$(ATLAS_SCRIPTS) ops lint --fail-fast

ci-ops-readme-make-only:
	@$(ATLAS_SCRIPTS) docs ops-readmes-make-only-check --report text

ci-ops-readme-canonical-links:
	@$(ATLAS_SCRIPTS) docs ops-readme-canonical-links-check --report text

ci-ops-doc-duplication:
	@$(ATLAS_SCRIPTS) docs ops-doc-duplication-check --report text

ci-docs-make-only-ops:
	@$(ATLAS_SCRIPTS) docs docs-make-only-ops-check --report text

internal/ci/scripts-group:
	@$(ATLAS_SCRIPTS) ci scripts

internal/ci/no-xtask:
	@$(ATLAS_SCRIPTS) check no-xtask

ci-forbid-raw-paths:
	@$(ATLAS_SCRIPTS) check forbidden-paths

ci-make-safety:
	@$(ATLAS_SCRIPTS) run ./scripts/areas/layout/check_make_safety.py

ci-make-help-drift:
	@$(ATLAS_SCRIPTS) check make-help

internal/ci/scripts-path-usage:
	@$(ATLAS_SCRIPTS) check make-scripts-refs

internal/ci/docs-old-script-paths:
	@$(ATLAS_SCRIPTS) compat check --include-docs

internal/ci/bin-shims:
	@$(ATLAS_SCRIPTS) check cli-help
	@$(ATLAS_SCRIPTS) check ownership
	@$(ATLAS_SCRIPTS) check duplicate-script-names
	@$(ATLAS_SCRIPTS) --help >/dev/null

internal/ci/scripts-ssot-final:
	@$(ATLAS_SCRIPTS) migration gate

internal/ci/repo-hygiene:
	@$(ATLAS_SCRIPTS) check repo

ci-slo-config-validate:
	@$(ATLAS_SCRIPTS) run ./scripts/areas/layout/check_slo_contracts.py --mode schema

ci-slo-no-loosen:
	@$(ATLAS_SCRIPTS) run ./scripts/areas/layout/check_slo_no_loosen_without_approval.py

ci-slo-metrics-contract:
	@$(ATLAS_SCRIPTS) run ./scripts/areas/layout/check_slo_contracts.py --mode metrics

ci-sli-contract:
	@$(ATLAS_SCRIPTS) run ./scripts/areas/layout/check_slo_contracts.py --mode slis

ci-sli-docs-drift:
	@$(ATLAS_SCRIPTS) docs generate-sli-doc --report text
	@git diff --exit-code docs/operations/slo/SLIS.md

ci-slo-docs-drift:
	@$(ATLAS_SCRIPTS) docs generate-slos-doc --report text
	@git diff --exit-code docs/operations/slo/SLOS.md

ci-init-iso-dirs:
	@mkdir -p "$${CARGO_TARGET_DIR:-artifacts/isolate/tmp/target}" "$${CARGO_HOME:-artifacts/isolate/tmp/cargo-home}" "$${TMPDIR:-artifacts/isolate/tmp/tmp}" "$${ISO_ROOT:-artifacts/isolate/tmp}"

ci-init-tmp:
	@mkdir -p "$${TMPDIR:-artifacts/isolate/tmp/tmp}" "$${ISO_ROOT:-artifacts/isolate/tmp}"

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
	@mkdir -p artifacts/isolate/release-notes
	@sed -e "s/{{tag}}/$${GITHUB_REF_NAME}/g" \
	  -e "s/{{date}}/$$(date -u +%Y-%m-%d)/g" \
	  -e "s/{{commit}}/$${GITHUB_SHA}/g" \
	  .github/release-notes-template.md > artifacts/isolate/release-notes/RELEASE_NOTES.md

ci-release-publish-gh:
	@gh release create "$${GITHUB_REF_NAME}" \
	  --title "bijux-atlas $${GITHUB_REF_NAME}" \
	  --notes-file artifacts/isolate/release-notes/RELEASE_NOTES.md \
	  --verify-tag || \
	gh release edit "$${GITHUB_REF_NAME}" \
	  --title "bijux-atlas $${GITHUB_REF_NAME}" \
	  --notes-file artifacts/isolate/release-notes/RELEASE_NOTES.md

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
	@mkdir -p artifacts/isolate/reproducible-build
	@cargo build --release --locked --bin bijux-atlas --bin atlas-server
	@sha256sum "$${CARGO_TARGET_DIR}/release/bijux-atlas" "$${CARGO_TARGET_DIR}/release/atlas-server" > artifacts/isolate/reproducible-build/build1.sha256
	@cargo clean
	@cargo build --release --locked --bin bijux-atlas --bin atlas-server
	@sha256sum "$${CARGO_TARGET_DIR}/release/bijux-atlas" "$${CARGO_TARGET_DIR}/release/atlas-server" > artifacts/isolate/reproducible-build/build2.sha256
	@diff -u artifacts/isolate/reproducible-build/build1.sha256 artifacts/isolate/reproducible-build/build2.sha256

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
	@$(MAKE) policy-enforcement-status
	@$(MAKE) policy-allow-env-lint
	@$(MAKE) scripts-lint
	@$(MAKE) ops-policy-audit
	@$(MAKE) ci-workflows-make-only
	@$(MAKE) ci-docs-lint-names
	@$(MAKE) ci-forbid-raw-paths
	@$(MAKE) ci-make-safety
	@$(MAKE) ci-make-help-drift
	@$(MAKE) internal/ci/scripts-path-usage
	@$(MAKE) internal/ci/docs-old-script-paths
	@$(MAKE) internal/ci/bin-shims
	@$(MAKE) internal/ci/repo-hygiene
	@$(MAKE) internal/ci/scripts-ssot-final

.PHONY: \
	ci-root-layout ci-script-entrypoints ci-fmt ci-clippy ci-test-nextest ci-deny ci-audit ci-license-check \
	ci-policy-lint ci-policy-schema-drift ci-config-check ci-ssot-drift ci-crate-structure ci-crate-docs-contract ci-cli-command-surface \
	ci-release-binaries ci-docs-build ci-latency-regression ci-store-conformance ci-openapi-drift ci-chart-schema-validate ci-api-contract ci-query-plan-gate ci-critical-query-check \
	ci-sqlite-schema-drift ci-sqlite-index-drift ci-ingest-determinism ci-qc-fixtures ci-compatibility-matrix-validate ci-runtime-security-scan-image ci-coverage ci-workflows-make-only ci-policy-relaxations ci-policy-enforcement ci-policy-allow-env ci-policy-boundaries ci-policy-boundaries ci-ops-policy-audit ci-rename-lint ci-docs-lint-names governance-check \
	ci-make-help-drift ci-forbid-raw-paths ci-make-safety internal/ci/scripts-path-usage internal/ci/docs-old-script-paths internal/ci/bin-shims internal/ci/repo-hygiene internal/ci/scripts-ssot-final \
	ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts \
	ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify \
	ci-security-advisory-render ci-ops-install-prereqs ci-ops-install-load-prereqs \
	ci-log-fields-contract ci-ops-index-surface ci-ops-readme-make-only ci-ops-readme-canonical-links ci-ops-doc-duplication ci-docs-make-only-ops
