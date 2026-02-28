# Make Surface

Make is a thin wrapper layer for curated developer entrypoints.

Public targets are exposed through `make help` and route through the control plane or approved local cargo lanes.
Make does not own policy or operational logic.

Contracts are defined in `make/CONTRACT.md` and enforced by `bijux dev atlas contracts make`.
Contracts gate targets are defined only in `make/contracts.mk` and delegated through `make/public.mk`.
