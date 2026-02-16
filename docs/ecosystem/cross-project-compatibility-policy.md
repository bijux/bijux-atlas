# Cross-Project Compatibility Policy

- Cross-project compatibility is contract-driven, not branch-coupled.
- Contracts include:
  - plugin metadata handshake
  - artifact schema version
  - API error schema and cursor format
- Compatibility matrix docs must be maintained per producer/consumer pair.
- No project may import internal crates from another project without explicit contract adoption.
