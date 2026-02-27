# Scope: verification orchestrator for running all targets in a module makefile.
# Usage: make verification <module>  (example: make verification ops)
SHELL := /bin/sh

ifeq (verification,$(firstword $(MAKECMDGOALS)))
VERIFICATION_MODULE := $(word 2,$(MAKECMDGOALS))
ifneq ($(strip $(VERIFICATION_MODULE)),)
$(eval $(VERIFICATION_MODULE):;@:)
endif
endif

VERIFICATION_ACCEPT_CODES ?= 0
VERIFICATION_ACCEPT_CODES_configs ?= 0 2
VERIFICATION_ACCEPT_CODES__configs ?= 0 2
VERIFICATION_ACCEPT_CODES_docs ?= 0 2
VERIFICATION_ACCEPT_CODES__docs ?= 0 2

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
		underscore_file="makefiles/_$$module.mk"; \
		if [ -f "$$underscore_file" ]; then \
			mk_file="$$underscore_file"; \
		fi; \
	fi; \
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
	accept_codes="$(VERIFICATION_ACCEPT_CODES_$(VERIFICATION_MODULE))"; \
	if [ -z "$$accept_codes" ]; then accept_codes="$(VERIFICATION_ACCEPT_CODES)"; fi; \
	for target in $$targets; do \
		total=$$((total + 1)); \
		printf '[%s] %s\n' "$$total" "$$target"; \
		case "$$target" in \
			*-serve) \
				log_file="/tmp/bijux-verification-$$module-$$target-$$$$.log"; \
				$(MAKE) --no-print-directory -s "$$target" >"$$log_file" 2>&1 & \
				pid=$$!; \
				sleep 2; \
				if kill -0 "$$pid" >/dev/null 2>&1; then \
					kill "$$pid" >/dev/null 2>&1 || true; \
					wait "$$pid" >/dev/null 2>&1 || true; \
					printf '  result: pass (startup verified, process terminated)\n'; \
				else \
					wait "$$pid"; \
					code=$$?; \
					cat "$$log_file"; \
					case " $$accept_codes " in \
						*" $$code "*) printf '  result: pass (accepted exit=%s)\n' "$$code" ;; \
						*) printf '  result: fail (exit=%s)\n' "$$code"; failed=$$((failed + 1));; \
					esac; \
				fi; \
				rm -f "$$log_file";; \
			*) \
		if $(MAKE) --no-print-directory -s "$$target"; then \
			printf '  result: pass\n'; \
		else \
			code=$$?; \
			case " $$accept_codes " in \
				*" $$code "*) printf '  result: pass (accepted exit=%s)\n' "$$code" ;; \
				*) printf '  result: fail (exit=%s)\n' "$$code"; failed=$$((failed + 1));; \
			esac; \
		fi;; \
		esac; \
	done; \
	printf 'verification summary: module=%s total=%s failed=%s\n' "$$module" "$$total" "$$failed"; \
	test "$$failed" -eq 0

.PHONY: verification _verification-run
