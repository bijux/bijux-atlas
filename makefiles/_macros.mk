SHELL := /bin/sh

# Shared Make macros for run IDs, isolation, and command wrappers.
RUN_ID ?= $(MAKE_PRIMARY_GOAL)-$(MAKE_RUN_TS)

# Usage: $(call iso_dir,<lane>) => artifacts/isolate/<lane>/<RUN_ID>
iso_dir = artifacts/isolate/$(1)/$(RUN_ID)

# Usage: $(call with_iso,<lane>,<command>)
with_iso = run_id="$(RUN_ID)"; iso="$(call iso_dir,$(1))"; \
	mkdir -p "$$iso/target" "$$iso/cargo-home" "$$iso/tmp"; \
	ISO_ROOT="$$iso" CARGO_TARGET_DIR="$$iso/target" CARGO_HOME="$$iso/cargo-home" TMPDIR="$$iso/tmp" TMP="$$iso/tmp" TEMP="$$iso/tmp" RUN_ID="$$run_id" ARTIFACT_DIR="$$iso" $(2)

# Usage: $(call gate_json,<gate-name>,<command...>)
gate_json = run_id="$${RUN_ID:-gates-$(MAKE_RUN_TS)}"; \
	RUN_ID="$$run_id" python3 ./scripts/layout/run_gate.py $(1) $(2)

# Usage: $(call py_venv,<venv_path>,<command>)
py_venv = if [ ! -x "$(1)/bin/python" ]; then python3 -m venv "$(1)"; fi; \
	"$(1)/bin/python" -m pip install --upgrade pip >/dev/null; \
	$(2)
