help-contract: ## Show make contract and public target documentation pointers
	@printf '%s\n' "contract: $(MAKE_CONTRACT_PATH)" "public-targets: $(MAKE_HELP_PATH)" "target-list: make/target-list.json"

make-target-list: ## Regenerate make public target list artifact
	@targets="$$(sed -n '/^CURATED_TARGETS := \\/,/^$$/p' makefiles/root.mk | tr '\t\\' '  ' | tr -s ' ' '\n' | grep -E '^[a-z0-9][a-z0-9-]*$$')"; \
	TARGETS="$$targets" python3 -c "import json,os,pathlib; targets=sorted({t for t in os.environ.get('TARGETS','').splitlines() if t}); payload={'schema_version':1,'source':'makefiles/root.mk:CURATED_TARGETS','public_targets':targets}; pathlib.Path('make/target-list.json').write_text(json.dumps(payload, indent=2)+'\\n')"

.PHONY: help-contract make-target-list
