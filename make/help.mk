help-contract: ## Show make contract and public target documentation pointers
	@printf '%s\n' "contract: $(MAKE_CONTRACT_PATH)" "public-targets: $(MAKE_HELP_PATH)" "target-list: make/target-list.json"

make-target-list: ## Regenerate make public target list artifact
	@python3 -c "import json,re,pathlib; text=pathlib.Path('makefiles/root.mk').read_text().splitlines(); start=next((i for i,l in enumerate(text) if l.startswith('CURATED_TARGETS :=')),None); targets=[];\
[targets.extend([t for t in ((line.split(':=',1)[1] if line.strip().startswith('CURATED_TARGETS :=') else line).replace('\\\\',' ').strip().split()) if re.match(r'^[a-z0-9][a-z0-9-]*$$',t)]) for line in ([] if start is None else text[start:]) if line.strip()];\
payload={'schema_version':1,'source':'makefiles/root.mk:CURATED_TARGETS','public_targets':sorted(set(targets))}; pathlib.Path('make/target-list.json').write_text(json.dumps(payload,indent=2)+'\\n')"

.PHONY: help-contract make-target-list
