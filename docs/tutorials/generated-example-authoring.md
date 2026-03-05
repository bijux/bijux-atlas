# Generated Example Authoring

Tutorial command output examples must be generated through `bijux-dev-atlas` commands and committed as generated artifacts.

## Authoring rules

1. Source examples from generator commands only.
2. Store generated output under governed generated-doc locations.
3. Reference generated snippets in tutorial pages instead of pasting ad hoc output.
4. Re-run generation and commit updates whenever command surfaces change.

## Validation path

1. Run the docs generation command set.
2. Run docs verification checks.
3. Ensure no manual edits are made to generated blocks.
