# Command Groups

Atlasctl command implementation lives in `src/atlasctl/commands/` and follows `configure(parser)` + `run(ctx, ns)`.

## Groups

- `check`: check registry execution and check-related commands.
- `docs`: docs generation, validation, and contracts.
- `ops`: operations command surface.
- `make`: make surface and contracts commands.
- `configs`: config validation and generation commands.
- `contracts`: schema and contract-oriented commands.
- `docker`: docker-related checks and commands.
- `ci`: CI-oriented commands.
- `inventory`: inventory generation and validation.
- `report`: report aggregation and artifacts.
- `legacy`: legacy inventory and historical migration metadata.
