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

ci-policy-relaxations:
	@$(MAKE) policy-audit

ci-policy-enforcement:
	@$(MAKE) policy-enforcement-status

ci-policy-allow-env:
	@$(MAKE) policy-allow-env-lint

ci-ops-policy-audit:
	@$(MAKE) ops-policy-audit

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
	@python3 ./scripts/public/contracts/check_breaking_contract_change.py

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
	@./scripts/public/qc-fixtures-gate.sh

ci-compatibility-matrix-validate:
	@$(MAKE) compat-matrix-validate

ci-runtime-security-scan-image:
	@$(MAKE) docker-build

ci-coverage:
	@if ! cargo llvm-cov --version >/dev/null 2>&1; then cargo install cargo-llvm-cov --locked; fi
	@cargo llvm-cov --workspace --all-features --lcov --output-path artifacts/isolates/coverage/lcov.info

ci-workflows-make-only:
	@python3 ./scripts/layout/check_workflows_make_only.py

ci-log-fields-contract:
	@python3 ./ops/obs/scripts/validate_logs_schema.py --file ops/obs/contract/logs.example.jsonl

ci-observability-pack-test:
	@$(MAKE) observability-pack-test

ci-observability-pack-drills:
	@$(MAKE) observability-pack-drills

ci-ops-index-surface:
	@python3 ./scripts/layout/check_ops_index_surface.py

ci-ops-run-entrypoints:
	@python3 ./scripts/layout/check_ops_run_entrypoints.py

ci-ops-readme-make-only:
	@python3 ./scripts/docs/check_ops_readmes_make_only.py

ci-ops-readme-canonical-links:
	@python3 ./scripts/docs/check_ops_readme_canonical_links.py

ci-ops-doc-duplication:
	@python3 ./scripts/docs/check_ops_doc_duplication.py

ci-docs-make-only-ops:
	@python3 ./scripts/docs/check_docs_make_only_ops.py

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
	@$(MAKE) policy-enforcement-status
	@$(MAKE) policy-allow-env-lint
	@$(MAKE) scripts-lint
	@$(MAKE) ops-policy-audit
	@$(MAKE) ci-workflows-make-only
	@$(MAKE) ci-docs-lint-names
	@$(MAKE) ci-forbid-raw-paths
	@$(MAKE) ci-make-safety
	@$(MAKE) ci-make-help-drift

.PHONY: \
	ci-root-layout ci-script-entrypoints ci-fmt ci-clippy ci-test-nextest ci-deny ci-audit ci-license-check \
	ci-policy-lint ci-policy-schema-drift ci-config-check ci-ssot-drift ci-crate-structure ci-crate-docs-contract ci-cli-command-surface \
	ci-release-binaries ci-docs-build ci-latency-regression ci-store-conformance ci-openapi-drift ci-chart-schema-validate ci-api-contract ci-query-plan-gate ci-critical-query-check \
	ci-sqlite-schema-drift ci-sqlite-index-drift ci-ingest-determinism ci-qc-fixtures ci-compatibility-matrix-validate ci-runtime-security-scan-image ci-coverage ci-workflows-make-only ci-policy-relaxations ci-policy-enforcement ci-policy-allow-env ci-ops-policy-audit ci-rename-lint ci-docs-lint-names governance-check \
	ci-make-help-drift ci-forbid-raw-paths ci-make-safety \
	ci-init-iso-dirs ci-init-tmp ci-dependency-lock-refresh ci-release-compat-matrix-verify ci-release-build-artifacts \
	ci-release-notes-render ci-release-publish-gh ci-cosign-sign ci-cosign-verify ci-chart-package-release ci-reproducible-verify \
	ci-security-advisory-render ci-ops-install-prereqs ci-ops-install-load-prereqs \
	ci-log-fields-contract ci-ops-index-surface ci-ops-readme-make-only ci-ops-readme-canonical-links ci-ops-doc-duplication ci-docs-make-only-ops
