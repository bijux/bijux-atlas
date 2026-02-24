# Scope: single source of truth for dev control-plane invocation from make.
# Public targets: none
SHELL := /bin/sh

BIJUX ?= bijux
DEV_ATLAS ?= cargo run -q -p bijux-dev-atlas --

# Compatibility alias during makefile cutover; wrappers should use DEV_ATLAS directly.
BIJUX_DEV_ATLAS ?= $(DEV_ATLAS)

export BIJUX DEV_ATLAS BIJUX_DEV_ATLAS
