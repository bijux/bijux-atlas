SHELL := /bin/sh

layout-check: ## Validate repository layout contract and root shape
	@./scripts/layout/check_root_shape.sh
	@./scripts/layout/check_forbidden_root_names.sh
	@./scripts/layout/check_forbidden_root_files.sh
	@./scripts/layout/check_no_forbidden_paths.sh
	@./scripts/layout/check_ops_workspace.sh
	@./scripts/layout/check_ops_canonical_shims.sh
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

.PHONY: layout-check layout-migrate
