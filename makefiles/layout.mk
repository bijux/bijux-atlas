# Scope: repository layout/path contract internal targets.
# Public targets: none
SHELL := /bin/sh

layout-check: ## Validate repository layout contract and root shape
	@./scripts/areas/layout/check_root_shape.sh
	@./scripts/areas/layout/check_no_root_dumping.sh
	@./scripts/areas/layout/check_forbidden_root_names.sh
	@./scripts/areas/layout/check_forbidden_root_files.sh
	@./scripts/areas/layout/check_no_forbidden_paths.sh
	@python3 ./scripts/areas/layout/check_generated_dirs_policy.py
	@python3 ./scripts/areas/layout/check_generated_committed_no_timestamp_dirs.py
	@python3 ./scripts/areas/layout/check_evidence_not_tracked.py
	@./scripts/areas/layout/check_ops_workspace.sh
	@python3 ./scripts/areas/layout/check_ops_layout_contract.py
	@python3 ./scripts/areas/layout/check_ops_index_surface.py
	@python3 ./scripts/areas/layout/check_ops_artifacts_writes.py
	@python3 ./scripts/areas/layout/check_ops_concept_ownership.py
	@python3 ./scripts/areas/layout/check_ops_single_validators.py
	@python3 ./scripts/areas/layout/check_ops_single_owner_contracts.py
	@./scripts/areas/layout/check_ops_canonical_shims.sh
	@./scripts/areas/layout/check_ops_lib_canonical.sh
	@./scripts/areas/layout/check_repo_hygiene.sh
	@./scripts/areas/layout/check_artifacts_allowlist.sh
	@./scripts/areas/layout/check_artifacts_policy.sh
	@./scripts/areas/layout/check_symlink_index.sh
	@python3 ./scripts/areas/layout/check_symlink_policy.py
	@./scripts/areas/layout/check_chart_canonical_path.sh
	@python3 ./scripts/areas/layout/check_workflows_make_only.py
	@python3 ./scripts/areas/layout/check_legacy_deprecation.py
	@python3 ./scripts/areas/layout/legacy_inventory.py --check-policy --json-out artifacts/evidence/legacy/inventory.json --format text
	@python3 ./scripts/areas/layout/check_ops_external_entrypoints.py
	@python3 ./scripts/areas/layout/check_dir_budgets.py

layout-migrate: ## Apply deterministic layout/path migration helpers
	@./scripts/areas/layout/replace_paths.sh --apply
	@./scripts/areas/layout/migrate.sh

layout-fix: ## Repair known layout/symlink drift deterministically
	@$(MAKE) layout-migrate

.PHONY: layout-check layout-migrate layout-fix
