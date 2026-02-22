# Scope: policy guardrail wrappers for crates and atlasctl source trees.
# Public targets: none
SHELL := /bin/sh

culprits-crates-max_loc:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-crates-max_loc

culprits-crates-max_depth:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-crates-max_depth

culprits-crates-file-max_rs_files_per_dir:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-crates-file-max_rs_files_per_dir

culprits-crates-file-max_modules_per_dir:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-crates-file-max_modules_per_dir

culprits-atlasctl-max_loc:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-atlasctl-max_loc

culprits-atlasctl-max_depth:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-atlasctl-max_depth

culprits-atlasctl-file-max_py_files_per_dir:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-atlasctl-file-max_py_files_per_dir

culprits-atlasctl-file-max_modules_per_dir:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-atlasctl-file-max_modules_per_dir

culprits-all-crates:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-all-crates

culprits-all-atlasctl:
	@./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/policies/culprits_make.py --gate culprits-all-atlasctl

bypass-report:
	@./bin/atlasctl policies bypass report --out artifacts/reports/atlasctl/policies-bypass-report.json

policies-report:
	@./bin/atlasctl policies bypass report --out artifacts/reports/atlasctl/policies-bypass-report.json

.PHONY: culprits-crates-max_loc culprits-crates-max_depth culprits-crates-file-max_rs_files_per_dir culprits-crates-file-max_modules_per_dir culprits-atlasctl-max_loc culprits-atlasctl-max_depth culprits-atlasctl-file-max_py_files_per_dir culprits-atlasctl-file-max_modules_per_dir culprits-all-crates culprits-all-atlasctl bypass-report policies-report
