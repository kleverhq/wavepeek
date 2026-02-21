# Backlog

## Issues

### `$schema` raw URL compatibility

- JSON `$schema` URL currently points to GitHub HTML (`https://github.com/.../blob/...`) instead of raw content.
- Update it to `https://raw.githubusercontent.com/...` so the schema is directly downloadable and auto-detected by tooling.
- Current pattern: `https://github.com/kleverhq/wavepeek/blob/vX.Y.Z/schema/wavepeek.json`
- Target pattern: `https://raw.githubusercontent.com/kleverhq/wavepeek/vX.Y.Z/schema/wavepeek.json`

## Tech Debt
