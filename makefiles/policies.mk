# Scope: policy guardrail targets for crates and atlasctl source trees.
# Public targets: none
SHELL := /bin/sh

# -----------------------------
# Crates Gates
# -----------------------------

culprits-crates-max_loc:
	@err=$$(find crates -name "*.rs" -print0 \
	| xargs -0 wc -l \
	| sort -n \
	| awk '$$2 ~ /^crates\// && $$1 > 1000'); \
	warn=$$(find crates -name "*.rs" -print0 \
	| xargs -0 wc -l \
	| sort -n \
	| awk '$$2 ~ /^crates\// && $$1 > 800 && $$1 <= 1000'); \
	if [ -n "$$err" ]; then \
		printf '%s\n' "ERROR: crates max_loc policy violations (LOC > 1000):"; \
		printf '%s\n' "$$err"; \
		exit 1; \
	fi; \
	if [ -n "$$warn" ]; then \
		printf '%s\n' "WARN: crates max_loc advisory violations (800 < LOC <= 1000):"; \
		printf '%s\n' "$$warn"; \
	else \
		printf '%s\n' "INFO: crates max_loc policy compliant."; \
	fi

culprits-crates-max_depth:
	@out=$$(find crates -name "*.rs" -print0 \
	| xargs -0 -I{} sh -c 'p="{}"; d=$$(printf "%s\n" "$$p" | awk -F/ "{print NF}"); echo "$$d $$p"' \
	| sort -n \
	| awk '$$1 > 7'); \
	if [ -n "$$out" ]; then \
		printf '%s\n' "ERROR: crates max_depth policy violations (depth > 7):"; \
		printf '%s\n' "$$out"; \
		exit 1; \
	else \
		printf '%s\n' "INFO: crates max_depth policy compliant."; \
	fi

culprits-crates-file-max_rs_files_per_dir:
	@out=$$(find crates -name "*.rs" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 10' \
	| sort -nr); \
	if [ -n "$$out" ]; then \
		printf '%s\n' "ERROR: crates max_rs_files_per_dir policy violations (files > 10):"; \
		printf '%s\n' "$$out"; \
		exit 1; \
	else \
		printf '%s\n' "INFO: crates max_rs_files_per_dir policy compliant."; \
	fi

culprits-crates-file-max_modules_per_dir:
	@out=$$(find crates -name "*.rs" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 16' \
	| sort -nr); \
	if [ -n "$$out" ]; then \
		printf '%s\n' "ERROR: crates max_modules_per_dir policy violations (modules > 16):"; \
		printf '%s\n' "$$out"; \
		exit 1; \
	else \
		printf '%s\n' "INFO: crates max_modules_per_dir policy compliant."; \
	fi

culprits-all-crates: culprits-crates-max_loc culprits-crates-max_depth culprits-crates-file-max_rs_files_per_dir culprits-crates-file-max_modules_per_dir
	@printf '%s\n' "INFO: culprits-all-crates completed."

# -----------------------------
# Atlasctl Gates
# -----------------------------

culprits-atlasctl-max_loc:
	@err=$$(find packages/atlasctl/src/atlasctl -name "*.py" -print0 \
	| xargs -0 wc -l \
	| sort -n \
	| awk '$$2 ~ /^packages\/atlasctl\/src\/atlasctl\// && $$1 > 600'); \
	warn=$$(find packages/atlasctl/src/atlasctl -name "*.py" -print0 \
	| xargs -0 wc -l \
	| sort -n \
	| awk '$$2 ~ /^packages\/atlasctl\/src\/atlasctl\// && $$1 > 400 && $$1 <= 600'); \
	if [ -n "$$err" ]; then \
		printf '%s\n' "ERROR: atlasctl max_loc policy violations (LOC > 600):"; \
		printf '%s\n' "$$err"; \
		exit 1; \
	fi; \
	if [ -n "$$warn" ]; then \
		printf '%s\n' "WARN: atlasctl max_loc advisory violations (400 < LOC <= 600):"; \
		printf '%s\n' "$$warn"; \
	else \
		printf '%s\n' "INFO: atlasctl max_loc policy compliant."; \
	fi

culprits-atlasctl-max_depth:
	@root="packages/atlasctl/src/atlasctl"; \
	out=$$(find "$$root" -type f \( -name "*.py" -o -name "*.json" -o -name "*.md" \) ! -path "*/__pycache__/*" -print0 \
	| xargs -0 -I{} sh -c 'p="{}"; rel=$${p#'"$$root"'/}; d=$$(printf "%s\n" "$$rel" | awk -F/ "{print NF}"); echo "$$d $$p"' \
	| sort -n \
	| awk '$$1 > 8'); \
	if [ -n "$$out" ]; then \
		printf '%s\n' "ERROR: atlasctl max_depth violations (depth > 8):"; \
		printf '%s\n' "$$out"; \
		exit 1; \
	else \
		printf '%s\n' "INFO: atlasctl max_depth policy compliant."; \
	fi

culprits-atlasctl-file-max_py_files_per_dir:
	@out=$$(find packages/atlasctl/src/atlasctl -name "*.py" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 10' \
	| sort -nr); \
	if [ -n "$$out" ]; then \
		printf '%s\n' "ERROR: atlasctl max_py_files_per_dir policy violations (files > 10):"; \
		printf '%s\n' "$$out"; \
		exit 1; \
	else \
		printf '%s\n' "INFO: atlasctl max_py_files_per_dir policy compliant."; \
	fi

culprits-atlasctl-file-max_modules_per_dir:
	@out=$$(find packages/atlasctl/src/atlasctl -name "*.py" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 10' \
	| sort -nr); \
	if [ -n "$$out" ]; then \
		printf '%s\n' "ERROR: atlasctl max_modules_per_dir policy violations (modules > 10):"; \
		printf '%s\n' "$$out"; \
		exit 1; \
	else \
		printf '%s\n' "INFO: atlasctl max_modules_per_dir policy compliant."; \
	fi

culprits-all-atlasctl: culprits-atlasctl-max_loc culprits-atlasctl-max_depth culprits-atlasctl-file-max_py_files_per_dir culprits-atlasctl-file-max_modules_per_dir
	@printf '%s\n' "INFO: culprits-all-atlasctl completed."

.PHONY: \
	culprits-crates-max_loc \
	culprits-crates-max_depth \
	culprits-crates-file-max_rs_files_per_dir \
	culprits-crates-file-max_modules_per_dir \
	culprits-atlasctl-max_loc \
	culprits-atlasctl-max_depth \
	culprits-atlasctl-file-max_py_files_per_dir \
	culprits-atlasctl-file-max_modules_per_dir \
	culprits-all-crates \
	culprits-all-atlasctl
