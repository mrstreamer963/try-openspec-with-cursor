## Context

The client renders a 50×50 tile world in PixiJS 8 using `Graphics` rectangles colored from YAML `color` fields. Content loads via the existing mod merge pipeline (`loadContent` → `ContentPack`). Session boot already gates the renderer until content is ready. The approved design spec (`docs/superpowers/specs/2026-07-02-tile-graphics-design.md`) and implementation plan define v1 scope: static sprites for terrain, buildings, and colonists from three CC0 atlas packs.

## Goals / Non-Goals

**Goals:**

- Load entire spritesheet PNGs (one GPU texture per atlas), slice frames by row-major index
- Configure atlases in `atlases.yaml`; bind content ids to frames via optional `sprite` refs in YAML
- `SpriteResolver` maps content id → `Texture`; `PixiRenderer` uses `Sprite` with existing `color` fallback
- Load atlases in parallel during session boot before `PixiRenderer` construction
- Warn once per missing atlas/frame/id via `console.warn`; game remains playable
- Unit tests for frame math and resolver logic

**Non-Goals:**

- Colonist walk animation
- Mod-provided atlas PNGs (mods may reference base atlas frames only)
- Sprite-based HUD or progress bars
- Build-time mega-atlas merge
- Changing WASM simulation API or save format

## Decisions

### 1. Multi-atlas + lazy frame Texture cache

**Choice:** `AtlasManager` loads each atlas PNG via `Assets.load`, computes columns (explicit or `auto`), and lazily creates lightweight `Texture` sub-rectangles cached in a `Map<number, Texture>` per atlas.

**Alternatives:** Per-sprite PNG files — rejected; too many HTTP requests and no shared GPU batching. Build-time texture packer — rejected for v1 complexity.

### 2. Separate `atlases.yaml` from content defs

**Choice:** Atlas metadata (path, tile_size, spacing, columns) lives in `content/base/atlases.yaml`. Content defs (`terrain.yaml`, `buildings.yaml`, `entities.yaml`) hold only `sprite: { atlas, frame }` refs.

**Alternatives:** Inline atlas config per sprite — rejected; duplicates tile_size/spacing across every entry.

### 3. `entities.yaml` as new content category

**Choice:** Colonist visual definition (`id: colonist`, `color`, optional `sprite`) lives in `entities.yaml`, merged like other categories. `SpriteResolver.resolveEntity('colonist')` supplies the colonist texture.

**Alternatives:** Hardcoded colonist sprite in renderer — rejected; inconsistent with YAML-driven content model.

### 4. View-only fields stripped from WASM JSON

**Choice:** `contentPackToJson` omits `sprite` (and other view-only fields) when baking content for the WASM worker. Simulation continues to use `color` only.

**Alternatives:** Pass sprite refs to WASM — rejected; rendering is client-only.

### 5. Session boot wiring

**Choice:** `App.vue` `beginSession` calls `loadAtlases(resources)` after `loadContent`, constructs `SpriteResolver`, passes to `GameSession` → `PixiRenderer`.

**Alternatives:** Load atlases inside `PixiRenderer` — rejected; couples renderer to resource loading and complicates testing.

### 6. Frame index convention

Row-major: `col = frame % columns`, `row = floor(frame / columns)`, pixel offset `x = col * (tileSize + spacing)`, `y = row * (tileSize + spacing)`.

## Risks / Trade-offs

- **[Frame indices wrong]** → Visual QA during implementation; YAML-only fix after download. Mitigation: document suggested starting frames; tune in final commit.
- **[Large PNG memory]** → Three atlases loaded at once. Mitigation: acceptable for v1; atlases are CC0 packs already used in indie games.
- **[Kenney download URLs change]** → Manual download fallback documented in tasks. Mitigation: commit PNGs to repo.
- **[Missing atlas at runtime]** → Color fallback + warn; game playable. No hard failure except `atlases.yaml` parse error (same as content load failure).

## Migration Plan

1. Ship types and content loading changes (backward compatible: `entities` defaults to `[]`, `sprite` optional)
2. Add assets and YAML sprite refs
3. Deploy renderer changes; existing saves unaffected (no format change)
4. Rollback: revert renderer to color-only; YAML `sprite` fields are ignored if code rolled back

## Open Questions

- Final frame indices for water/sand/grass/wall/bed/berry_bush/colonist require visual inspection after asset download (starting suggestions in implementation plan).
