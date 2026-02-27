make-contract-check: ## Enforce make contract constraints
	@set -euo pipefail; \
	test -f make/CONTRACT.mk; \
	test -f make/help.md; \
	test -f make/target-list.json; \
	grep -Eq '^include make/public.mk$$' Makefile; \
	grep -Eq '^include make/help.mk$$' Makefile; \
	test "$$(rg -n '^include ' Makefile | wc -l | tr -d ' ')" = "2"; \
	grep -Eq '^include makefiles/root.mk$$' make/internal.mk; \
	test "$$(rg -n '^include ' make/internal.mk | wc -l | tr -d ' ')" = "1"; \
	! rg -n '^\s*cd\s+' makefiles make || (echo 'contract violation: cd usage in make recipes' >&2; exit 1); \
	! rg -n 'scripts/' Makefile makefiles || (echo 'contract violation: scripts path usage in makefiles' >&2; exit 1); \
	! rg -n '\bcurl\b|\bwget\b' Makefile makefiles || (echo 'contract violation: network command in makefiles' >&2; exit 1); \
	for target in $$(sed -n '/^CURATED_TARGETS := \\/,/^\s*$$/p' makefiles/root.mk | tr '\\' ' ' | tr -s ' ' '\n' | grep -E '^[a-z0-9][a-z0-9-]*$$'); do \
		rg -n "^$${target}: .*## " makefiles >/dev/null || { echo "contract violation: missing one-line description for target '$${target}'" >&2; exit 1; }; \
	done

.PHONY: make-contract-check
