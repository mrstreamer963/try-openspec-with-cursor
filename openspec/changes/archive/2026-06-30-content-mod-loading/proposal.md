## Why

The game loads a single `base` content pack at startup. Modders cannot override or extend terrain, buildings, needs, or statuses without editing core files. A multi-mod merge pipeline enables data-driven balance and content packs without WASM recompilation.

## What Changes

- Add `content/mods.yaml` manifest listing mods in load order (fallback: `base` only when missing)
- Add `content/mods/<id>/` folders with optional partial YAML files per mod
- Replace `loadBaseContent()` with `loadContent()` that fetches each mod, merges by `id`, and returns `{ pack, modIds }`
- Keep WASM `Game::new(content_json)` unchanged — merged JSON is validated by existing `ContentRegistry`
- Add optional `content_mods` field to save files; warn on mod mismatch when loading
- Ship demo mod `content/mods/hardmode/` (not enabled by default)
- Add Vitest unit tests for merge logic

## Capabilities

### New Capabilities

_None — extends existing content-loading architecture._

### Modified Capabilities

- `content-definitions`: multi-mod manifest, merge-by-id overlay, mod path resolution
- `game-state-persistence`: optional `content_mods` in save file; mod mismatch warning on load
- `view-layer`: load flow uses `loadContent()`; mod mismatch confirm before load

## Impact

- `content/mods.yaml`, `content/mods/hardmode/` — new files
- `packages/client/src/content/` — `loadContent.ts`, `mergeContent.ts`; refactor `loadBaseContent.ts`
- `packages/client/src/App.vue`, `saveFile.ts` — mod ids in save/load
- `packages/game-core` — test helper for merged packs (optional)
- No WASM API changes
