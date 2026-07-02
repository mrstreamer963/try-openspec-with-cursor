## 1. Content types

- [x] 1.1 Add `SpriteRef`, `AtlasDef`, `EntityDef` interfaces to `packages/client/src/content/types.ts`
- [x] 1.2 Extend `TerrainDef`, `BuildingDef` with optional `sprite?: SpriteRef`; add `entities: EntityDef[]` to `ContentPack`
- [x] 1.3 Fix `emptyContentPack` and merge call sites for new `entities` field; verify typecheck

## 2. Content loading

- [x] 2.1 Add `entities` to `CATEGORY_FILES` and `mergeContent.ts` merge logic
- [x] 2.2 Update `loadContent.ts` to load optional `entities.yaml` per mod
- [x] 2.3 Strip view-only `sprite` fields in `contentPackToJson` (`loadBaseContent.ts`)
- [x] 2.4 Add/update `loadContent.test.ts` for `entities.yaml` loading; update fixtures

## 3. Atlas frame math (TDD)

- [x] 3.1 Write failing tests in `atlasFrameMath.test.ts` (columns, rect, bounds)
- [x] 3.2 Implement `atlasFrameMath.ts` (`resolveColumns`, `rowCount`, `maxFrameCount`, `frameToRect`)
- [x] 3.3 Verify tests pass

## 4. CC0 assets and YAML config

- [x] 4.1 Download Kenney Roguelike, Kenney Tiny Town, and Ever Rogue packs; copy PNGs to `content/assets/`
- [x] 4.2 Measure PNG dimensions; create `content/base/atlases.yaml` with correct `columns`
- [x] 4.3 Create `content/base/entities.yaml` with colonist definition and sprite ref
- [x] 4.4 Add `sprite` refs to `content/base/terrain.yaml` and `content/base/buildings.yaml` (verify frame indices visually)

## 5. AtlasManager and loadAtlases

- [x] 5.1 Implement `AtlasManager` (parallel PNG load, lazy frame texture cache, warn on failure)
- [x] 5.2 Implement `loadAtlases.ts` (parse `atlases.yaml`, orchestrate load via `ResourceManager`)

## 6. SpriteResolver (TDD)

- [x] 6.1 Write failing tests in `spriteResolver.test.ts`
- [x] 6.2 Implement `SpriteResolver` (`resolveTerrain`, `resolveBuilding`, `resolveEntity`, warn-once)
- [x] 6.3 Verify tests pass

## 7. PixiRenderer sprite rendering

- [x] 7.1 Add `SpriteResolver` to `PixiRenderer` constructor; implement `addTileGraphic` helper
- [x] 7.2 Update `drawTerrain`, `drawBuildings`, and colonist rendering to use sprites with color fallback
- [x] 7.3 Refactor colonist graphics map for `Sprite | Graphics`; verify typecheck and tests

## 8. Session boot wiring

- [x] 8.1 Load atlases in `App.vue` `beginSession` after `loadContent`; construct `SpriteResolver`
- [x] 8.2 Pass `spriteResolver` prop to `GameSession.vue` and into `PixiRenderer`
- [x] 8.3 Verify `npm run build` succeeds in `packages/client`

## 9. Verification

- [x] 9.1 Manual check: terrain, buildings, and colonists render with tile art; pan/zoom/click still work
- [x] 9.2 Manual fallback check: rename/missing PNG → color rectangles + console warnings; game playable
- [x] 9.3 Run full `packages/client` test suite; tune frame indices in YAML if needed after visual review
