make-contract-check: ## Enforce make contract constraints
	@set -euo pipefail; \
	public_targets="$$(sed -n '/^CURATED_TARGETS := \\/,/^$$/p' makefiles/root.mk | tr '\t\\' '  ' | tr -s ' ' '\n' | grep -E '^[a-z0-9][a-z0-9-]*$$')"; \
	target_count="$$(printf '%s\n' "$$public_targets" | sed '/^$$/d' | wc -l | tr -d ' ')"; \
	test -f make/CONTRACT.mk; \
	test -f make/help.md; \
	test -f make/target-list.json; \
	test "$$target_count" -le "25"; \
	grep -Eq '^include make/public.mk$$' Makefile; \
	grep -Eq '^include make/help.mk$$' Makefile; \
	test "$$(rg -n '^include ' Makefile | wc -l | tr -d ' ')" = "2"; \
	grep -Eq '^include makefiles/root.mk$$' make/internal.mk; \
	test "$$(rg -n '^include ' make/internal.mk | wc -l | tr -d ' ')" = "1"; \
	! rg -n '^\s*cd\s+' makefiles make || (echo 'contract violation: cd usage in make recipes' >&2; exit 1); \
	! rg -n 'scripts/' Makefile makefiles || (echo 'contract violation: scripts path usage in makefiles' >&2; exit 1); \
	! rg -n '\bcurl\b|\bwget\b' Makefile makefiles || (echo 'contract violation: network command in makefiles' >&2; exit 1); \
	for target in $$public_targets; do \
		rg -n "^$${target}: .*## " makefiles make >/dev/null || { echo "contract violation: missing one-line description for target '$${target}'" >&2; exit 1; }; \
		rg -n "^\- $${target}:" make/help.md >/dev/null || { echo "contract violation: missing help.md entry for target '$${target}'" >&2; exit 1; }; \
	done

make-target-governance-check: ## Enforce target naming and duplicate target rules
	@set -euo pipefail; \
	public_targets="$$(sed -n '/^CURATED_TARGETS := \\/,/^$$/p' makefiles/root.mk | tr '\t\\' '  ' | tr -s ' ' '\n' | grep -E '^[a-z0-9][a-z0-9-]*$$')"; \
	all_targets="$$(rg -n '^[a-zA-Z0-9_.-]+:.*## ' makefiles/root.mk | sed -E 's|.*:([a-zA-Z0-9_.-]+):.*|\1|' | sort)"; \
	dupes="$$(printf '%s\n' "$$all_targets" | uniq -d)"; \
	test -z "$$dupes" || { echo "governance violation: duplicate make targets: $$dupes" >&2; exit 1; }; \
	for target in $$all_targets; do \
		printf '%s\n' "$$public_targets" | grep -qx "$$target" || { \
			case "$$target" in _internal-*|internal-*) ;; *) echo "governance violation: non-public target '$$target' must use _internal- or internal- prefix" >&2; exit 1 ;; esac; \
		}; \
	done

make-ci-surface-check: ## Ensure workflow make calls use bounded public targets
	@set -euo pipefail; \
	public_targets="$$(sed -n '/^CURATED_TARGETS := \\/,/^$$/p' makefiles/root.mk | tr '\t\\' '  ' | tr -s ' ' '\n' | grep -E '^[a-z0-9][a-z0-9-]*$$')"; \
	for target in $$(rg -n "make [a-z0-9-]+" .github/workflows -g'*.yml' -g'*.yaml' | sed -E 's|.*make ([a-z0-9-]+).*|\1|' | sort -u); do \
		printf '%s\n' "$$public_targets" | grep -qx "$$target" || { \
			echo "governance violation: workflow uses non-public make target '$$target'" >&2; \
			exit 1; \
		}; \
	done

make-public-surface-sync-check: ## Ensure make target list matches configs/make/public-targets.json
	@set -euo pipefail; \
	tmp_make="$$(mktemp)"; \
	tmp_cfg="$$(mktemp)"; \
	jq -r '.public_targets[]' make/target-list.json | sort > "$$tmp_make"; \
	jq -r '.public_targets[].name' configs/make/public-targets.json | sort > "$$tmp_cfg"; \
	diff -u "$$tmp_cfg" "$$tmp_make" >/dev/null || { echo "governance violation: make public target list drift vs configs/make/public-targets.json" >&2; rm -f "$$tmp_make" "$$tmp_cfg"; exit 1; }; \
	rm -f "$$tmp_make" "$$tmp_cfg"

make-size-budget-check: ## Enforce make directory size budget
	@set -euo pipefail; \
	max_loc=200; \
	actual_loc="$$(find make -type f \( -name '*.mk' -o -name '*.md' \) | xargs wc -l | tail -n 1 | awk '{print $$1}')"; \
	test "$$actual_loc" -le "$$max_loc" || { echo "governance violation: make/ size budget exceeded ($$actual_loc > $$max_loc)" >&2; exit 1; }

make-include-cycle-check: ## Fail on cyclic include graph under make/
	@set -euo pipefail; \
	edges="$$(for f in make/*.mk; do \
	  src="$$(basename "$$f")"; \
	  awk '/^include / {print $$2}' "$$f" | sed -E 's|^make/||' | while read -r dep; do \
	    [ -n "$$dep" ] || continue; \
	    printf '%s %s\n' "$$src" "$$(basename "$$dep")"; \
	  done; \
	done)"; \
	if [ -n "$$edges" ]; then \
	  printf '%s\n' "$$edges" | tsort >/dev/null 2>&1 || { echo "governance violation: include cycle detected under make/*.mk" >&2; exit 1; }; \
	fi

.PHONY: make-contract-check make-target-governance-check make-ci-surface-check make-public-surface-sync-check make-size-budget-check make-include-cycle-check
