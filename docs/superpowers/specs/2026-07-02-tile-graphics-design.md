# Tile Graphics Design

**Date:** 2026-07-02  
**Status:** Approved (brainstorming)

## Summary

Replace colored-rectangle rendering in PixiJS with sprites from CC0 tile atlases. Load entire spritesheet PNGs (no per-sprite PNG files). Support multiple atlases and pick frames from any pack. Fallback to existing YAML `color` values with `console.warn` when sprites are missing.

## Requirements (from brainstorming)

| Decision | Choice |
|----------|--------|
| Scope v1 | Terrain + buildings + colonists |
| Colonist animation | Static single sprite (no walk animation) |
| Asset packs | Kenney Roguelike/RPG, Kenney Tiny Town, Ever Rogue — all loaded |
| Sprite config | `atlases.yaml` for atlas metadata; `sprite` refs in content defs |
| Colonist sprite | `entities.yaml` as separate entity type |
| Missing sprite / failed load | Fallback to `color` + `console.warn` |
| Loading approach | Multi-atlas + SpriteResolver (one PNG per atlas, frame sub-rectangles) |

## Architecture

```
loadContent() ──► ContentPack (terrain, buildings, entities, …)
loadAtlases() ──► AtlasManager (one TextureSource per atlas PNG)
                      │
                      ▼
               SpriteResolver(content, atlasManager)
                      │
                      ▼
               PixiRenderer (Sprite per tile/building/colonist, color fallback)
```

### Key modules (new)

| File | Responsibility |
|------|----------------|
| `packages/client/src/game/atlasManager.ts` | Load PNG atlases, compute frame rectangles, lazy frame Texture cache |
| `packages/client/src/game/spriteResolver.ts` | Map content id → Texture via `sprite: { atlas, frame }` |
| `packages/client/src/game/loadAtlases.ts` | Parse `atlases.yaml`, orchestrate parallel atlas load |

### Modified modules

| File | Changes |
|------|---------|
| `packages/client/src/content/types.ts` | `SpriteRef`, `AtlasDef`, `EntityDef`; extend `TerrainDef`, `BuildingDef`, `ContentPack` |
| `packages/client/src/content/loadContent.ts` | Load `entities.yaml`; load `atlases.yaml` separately |
| `packages/client/src/content/mergeContent.ts` | Merge `entities` category |
| `packages/client/src/game/PixiRenderer.ts` | `Sprite` rendering with color fallback |
| `packages/client/src/game/loadFlow.ts` | Wire atlas load before renderer init |

## Data model

### `content/base/atlases.yaml`

Atlas metadata only (not content-id bindings):

```yaml
atlases:
  - id: kenney-roguelike
    path: assets/kenney-roguelike/spritesheet.png
    tile_size: 16
    spacing: 1
    columns: 49          # from tilemap.txt or computed
  - id: kenney-tiny-town
    path: assets/kenney-tiny-town/tilemap.png
    tile_size: 16
    spacing: 1
    columns: 12
  - id: ever-rogue
    path: assets/ever-rogue/everroguetileset_full_packed.png
    tile_size: 16
    spacing: 0
    columns: auto        # floor(imageWidth / tile_size)
```

### Content defs — optional `sprite` field

```yaml
# terrain.yaml
terrain:
  - id: grass
    walkable: true
    color: 0x38a169
    sprite: { atlas: kenney-roguelike, frame: 28 }

# buildings.yaml
buildings:
  - id: wall
    # …existing fields…
    color: 0x718096
    sprite: { atlas: kenney-roguelike, frame: 256 }
```

### `content/base/entities.yaml`

```yaml
entities:
  - id: colonist
    color: 0xf6e05e
    sprite: { atlas: kenney-roguelike, frame: 736 }
```

### TypeScript types

```ts
export interface SpriteRef {
  atlas: string;
  frame: number;
}

export interface EntityDef {
  id: string;
  color: number;
  sprite?: SpriteRef;
}

// TerrainDef, BuildingDef gain optional sprite?: SpriteRef
// ContentPack gains entities: EntityDef[]
```

## Atlas loading and frame resolution

### Load once per atlas

1. Parse `atlases.yaml`
2. `Assets.load(path)` for each atlas PNG (parallel via `Promise.all`)
3. Compute `columns`: explicit value or `Math.floor(texture.width / (tile_size + spacing))` when `auto`
4. Store per atlas: `{ source, tileSize, spacing, columns }`

### Frame index → sub-rectangle

Row-major indexing: `col = frame % columns`, `row = floor(frame / columns)`.

```
x = col * (tileSize + spacing)
y = row * (tileSize + spacing)
```

A frame is a lightweight `Texture` view (`{ source, frame: Rectangle(x, y, tileSize, tileSize) }`) — not a separate PNG or pixel copy. Frames are cached lazily in a `Map<number, Texture>` per atlas.

### SpriteResolver API

- `resolveTerrain(id: string): Texture | null`
- `resolveBuilding(id: string): Texture | null`
- `resolveEntity(id: string): Texture | null`

Returns `null` when: no `sprite` field, atlas not loaded, frame out of bounds. Renderer uses `color` fallback and logs a warning.

## PixiRenderer changes

### Terrain layer

On first snapshot (`terrainDrawn`): for each of 2500 tiles, create `Sprite(texture)` or `Graphics` color fallback. Position at `(x * TILE_SIZE, y * TILE_SIZE)`.

### Buildings layer

Each snapshot refresh: completed buildings as `Sprite`; construction/deconstruction sites as semi-transparent `Sprite` (same alpha rules as today) or color `Graphics` fallback. Progress bars remain `Graphics`.

### Entities layer

One static `Sprite` per colonist from `resolveEntity('colonist')`, centered on tile. Name `Text` label unchanged. Fallback: yellow circle (`COLONIST_COLOR`).

### Constructor

```ts
constructor(mount: HTMLElement, content: ContentPack, spriteResolver: SpriteResolver)
```

Keep `terrainColors` / `buildingColors` maps for fallback.

## Asset files

### Directory layout

```
content/
  assets/
    kenney-roguelike/spritesheet.png
    kenney-tiny-town/tilemap.png
    ever-rogue/everroguetileset_full_packed.png
  base/
    atlases.yaml
    entities.yaml
    terrain.yaml
    buildings.yaml
```

Commit PNGs only (not source ZIPs). CC0 license — Kenney.nl and OpenGameArt sources noted in `atlases.yaml` comments.

### Frame selection at implementation time

1. Download the three packs
2. Inspect spritesheets visually
3. Assign frame indices for water, sand, grass, wall, bed, berry_bush, colonist
4. Indices are YAML-only changes thereafter

## Error handling

| Situation | Behavior |
|-----------|----------|
| PNG failed to load | `console.warn`, skip atlas, color fallback for affected sprites |
| No `sprite` in YAML | color fallback, warn once per id |
| Frame out of bounds | color fallback, warn with atlas + frame |
| `atlases.yaml` parse error | loading screen error (same as content load failure) |

## Mod support (v1 scope)

Atlases are base-only. Mods may override `sprite` refs in terrain/buildings YAML pointing at base atlases. Custom mod atlases are out of scope for v1.

## Testing

### Unit tests

- `atlasManager.test.ts`: column computation, frame rectangle math (spacing 0 and 1), out-of-bounds frame
- `spriteResolver.test.ts`: valid ref, missing sprite, missing atlas

### Manual verification

- Game renders terrain, buildings, colonists with sprites
- Remove/rename a PNG → color fallback + console warnings
- Camera pan/zoom and colonist click hit-testing still work

## Out of scope (v1)

- Walk animation for colonists
- Mod-provided atlas PNGs
- Replacing progress bars / HUD with sprites
- Build-time mega-atlas merge
