# Scope: repository layout/path contract internal targets.
# Public targets: none
SHELL := /bin/sh

layout-check: ## Validate repository layout contract and root shape
	@./scripts/layout/check_root_shape.sh
	@./scripts/check_no_root_dumping.sh
	@./scripts/layout/check_forbidden_root_names.sh
	@./scripts/layout/check_forbidden_root_files.sh
	@./scripts/layout/check_no_forbidden_paths.sh
	@python3 ./scripts/layout/check_generated_dirs_policy.py
	@python3 ./scripts/layout/check_generated_committed_no_timestamp_dirs.py
	@python3 ./scripts/layout/check_evidence_not_tracked.py
	@./scripts/layout/check_ops_workspace.sh
	@python3 ./scripts/layout/check_ops_layout_contract.py
	@python3 ./scripts/layout/check_ops_index_surface.py
	@python3 ./scripts/layout/check_ops_artifacts_writes.py
	@python3 ./scripts/layout/check_ops_concept_ownership.py
	@python3 ./scripts/layout/check_ops_single_validators.py
	@python3 ./scripts/layout/check_ops_single_owner_contracts.py
	@./scripts/layout/check_ops_canonical_shims.sh
	@./scripts/layout/check_ops_lib_canonical.sh
	@./scripts/layout/check_repo_hygiene.sh
	@./scripts/layout/check_artifacts_allowlist.sh
	@./scripts/layout/check_artifacts_policy.sh
	@./scripts/layout/check_symlink_index.sh
	@python3 ./scripts/layout/check_symlink_policy.py
	@./scripts/layout/check_chart_canonical_path.sh
	@python3 ./scripts/layout/check_workflows_make_only.py

layout-migrate: ## Apply deterministic layout/path migration helpers
	@./scripts/layout/replace_paths.sh --apply
	@./scripts/layout/migrate.sh

layout-fix: ## Repair known layout/symlink drift deterministically
	@$(MAKE) layout-migrate

.PHONY: layout-check layout-migrate layout-fix
