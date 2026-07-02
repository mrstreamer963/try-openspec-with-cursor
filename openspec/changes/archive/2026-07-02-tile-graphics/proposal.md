## Why

The game currently renders terrain, buildings, and colonists as flat colored rectangles in PixiJS. This makes the colony hard to read at a glance and limits visual polish. Replacing rectangles with CC0 tile sprites—configured via YAML and loaded from multi-atlas PNG sheets—gives the world recognizable art while keeping the existing color values as a resilient fallback when assets are missing.

## What Changes

- Add CC0 tile atlas PNGs (Kenney Roguelike/RPG, Kenney Tiny Town, Ever Rogue) under `content/assets/`
- Add `content/base/atlases.yaml` for atlas metadata (tile size, spacing, columns)
- Add `content/base/entities.yaml` with colonist entity definition and optional `sprite` refs
- Extend terrain and building YAML with optional `sprite: { atlas, frame }` fields
- Add client modules: `AtlasManager`, `SpriteResolver`, `loadAtlases`, `atlasFrameMath`
- Update `PixiRenderer` to draw `Sprite` nodes with `Graphics` color fallback
- Load atlases during session boot in `App.vue` before `GameSession` mounts
- Strip view-only fields (`sprite`) from WASM JSON payload; no simulation API changes
- Add unit tests for frame math and sprite resolution

## Capabilities

### New Capabilities

- `tile-graphics`: Multi-atlas PNG loading, frame rectangle math, sprite resolution from content defs, and PixiJS sprite rendering with color fallback

### Modified Capabilities

- `view-layer`: Terrain, building, and colonist rendering switch from colored rectangles to sprites with YAML-driven fallback colors; session boot loads atlases before renderer init
- `content-definitions`: New `entities` category and `atlases.yaml`; optional `sprite` refs on terrain/building/entity defs; view-only fields excluded from WASM payload

## Impact

- `content/assets/` — new CC0 atlas PNGs (committed, not source ZIPs)
- `content/base/atlases.yaml`, `entities.yaml` — new files
- `content/base/terrain.yaml`, `buildings.yaml` — add `sprite` refs
- `packages/client/src/content/types.ts` — `SpriteRef`, `AtlasDef`, `EntityDef`; extend `ContentPack`
- `packages/client/src/content/loadContent.ts`, `mergeContent.ts`, `loadBaseContent.ts` — entities category, WASM JSON stripping
- `packages/client/src/game/` — `atlasManager.ts`, `loadAtlases.ts`, `spriteResolver.ts`, `atlasFrameMath.ts`; modify `PixiRenderer.ts`
- `packages/client/src/App.vue`, `GameSession.vue` — atlas load and `SpriteResolver` wiring
- No WASM/Rust changes; no new npm dependencies
