# Scope: verification orchestrator for running all targets in a module makefile.
# Usage: make verification <module>  (example: make verification atlasctl)
SHELL := /bin/sh

ifeq (verification,$(firstword $(MAKECMDGOALS)))
VERIFICATION_MODULE := $(word 2,$(MAKECMDGOALS))
ifneq ($(strip $(VERIFICATION_MODULE)),)
$(eval $(VERIFICATION_MODULE):;@:)
endif
endif

verification: ## Run every target declared in makefiles/<module>.mk
	@module="$(VERIFICATION_MODULE)"; \
	if [ -z "$$module" ]; then \
		printf '%s\n' "usage: make verification <module>"; \
		exit 2; \
	fi; \
	$(MAKE) -s _verification-run VERIFICATION_MODULE="$$module"

_verification-run:
	@module="$(VERIFICATION_MODULE)"; \
	mk_file="makefiles/$$module.mk"; \
	if [ ! -f "$$mk_file" ]; then \
		printf '%s\n' "verification: missing $$mk_file"; \
		exit 2; \
	fi; \
	targets="$(VERIFICATION_TARGETS_$(VERIFICATION_MODULE))"; \
	if [ -z "$$targets" ]; then \
		targets=$$(awk -F: '/^[A-Za-z0-9_.\/-]+:[^=]/{print $$1}' "$$mk_file" \
		| grep -v '^\.' \
		| grep -v '/internal$$' \
		| grep -v '/internal/' \
		| grep -v '^_' \
		| sort -u); \
	fi; \
	if [ -z "$$targets" ]; then \
		printf '%s\n' "verification: no runnable targets found in $$mk_file"; \
		exit 2; \
	fi; \
	total=0; failed=0; \
	for target in $$targets; do \
		total=$$((total + 1)); \
		printf '[%s] %s\n' "$$total" "$$target"; \
		if $(MAKE) --no-print-directory -s "$$target"; then \
			printf '  result: pass\n'; \
		else \
			printf '  result: fail\n'; \
			failed=$$((failed + 1)); \
		fi; \
	done; \
	printf 'verification summary: module=%s total=%s failed=%s\n' "$$module" "$$total" "$$failed"; \
	test "$$failed" -eq 0

.PHONY: verification _verification-run
