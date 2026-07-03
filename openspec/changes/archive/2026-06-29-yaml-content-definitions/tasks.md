## 1. Base content pack

- [x] 1.1 Create `content/base/` with `mod.yaml`, `needs.yaml`, `statuses.yaml`, `buildings.yaml`, `terrain.yaml` reproducing v1 values (food/sleep decay, threshold 30, wall/bed/berry_bush work costs, 3 berries, sleep duration, colors, walkability)
- [x] 1.2 Add Vite alias or import path so `packages/client` can load base YAML files at build time
- [x] 1.3 Add `packages/client/src/content/loadBaseContent.ts` to parse/merge YAML into a single `ContentPack` object

## 2. Content registry (game-core)

- [x] 2.1 Add `packages/game-core/src/content/` module with `ContentPack`, `TerrainDef`, `BuildingDef`, `NeedDef`, `StatusDef`, and primitive enums (`SpawnPrimitive`, `InteractionDef`, `StatusEffect`)
- [x] 2.2 Implement `ContentRegistry::from_json` with validation (unknown refs, duplicate ids, required fields) and intern maps (`String` → `TerrainId` / `BuildingId` / `NeedId` / `StatusId`)
- [x] 2.3 Add unit tests for registry load, validation errors, and O(1) `defs[id]` access
- [x] 2.4 Add `serde_json` (and optionally `serde_yaml` for native tests) to `game-core/Cargo.toml`
- [x] 2.5 Insert `ContentRegistry` as bevy_ecs `Resource` in `Game::new(content_json)`

## 3. World grid and terrain

- [x] 3.1 Replace `TerrainType` enum with `TerrainId` in `WorldGrid` and world generation
- [x] 3.2 Update `is_walkable` and pathfinding to read `walkable` / `blocks_movement` from registry via baked indices
- [x] 3.3 Replace `BuildingType` with `BuildingId` in `WorldGrid.buildings` and placement APIs
- [x] 3.4 Update terrain generation to emit `TerrainId` values mapped from current noise thresholds to `water` / `sand` / `grass`

## 4. Needs and statuses

- [x] 4.1 Replace `Needs { food, sleep }` with `Needs(HashMap<NeedId, f32>)` and registry-driven `needs_decay`
- [x] 4.2 Add `ActiveStatuses(HashSet<StatusId>)` and `sync_statuses` system replacing `update_need_buffs` / `Hungry` / `WantsSleep`
- [x] 4.3 Map snapshot fields `food`, `sleep`, `hungry`, `wants_sleep` to/from dynamic needs and statuses via registry ids

## 5. Buildings and interactions

- [x] 5.1 Implement `complete_construction` spawn dispatch from `BuildingDef.on_complete` primitives (`Supply`, `Reservation`)
- [x] 5.2 Refactor `task_execution` eat/sleep paths to use `InteractionDef` primitives (`restore_need`, `consume_supply`, `duration_sec`, `mode`)
- [x] 5.3 Replace `work_required_for` and `BERRIES_PER_BUSH` with registry lookups
- [x] 5.4 Update `is_colonist_settle_cell` / wander exclusion to use `blocks_settle` from building defs
- [x] 5.5 Update `auto_assign_tasks` to discover berry bushes and beds via primitives + status effects instead of `BuildingType` matches

## 6. Protocol and snapshots

- [x] 6.1 Change `events.rs` snapshot and build event types to use `String` content ids for terrain and buildings
- [x] 6.2 Update `game.rs` snapshot build/restore to intern/serialize content ids
- [x] 6.3 Update `IncomingEvent::Build` handling to resolve building id through registry
- [x] 6.4 Update client `types.ts`, `protocol.ts`, and `saveFile.ts` for snake_case content ids

## 7. Client integration

- [x] 7.1 Pass merged content JSON from `gameWorker.ts` into `Game::new` on worker init
- [x] 7.2 Update `Toolbar.vue` to render buildable buildings from `ContentPack` (remove hardcoded tool list)
- [x] 7.3 Update `PixiRenderer.ts` to use terrain/building colors from content definitions
- [x] 7.4 Update `ColonistInfo.vue` to show need labels and status labels from content definitions
- [x] 7.5 Handle content init failure in worker with error postMessage to main thread

## 8. Cleanup and verification

- [x] 8.1 Remove obsolete enums (`TerrainType`, `BuildingType`), marker components (`Hungry`, `WantsSleep`), and duplicated constants superseded by YAML
- [x] 8.2 Run `cargo test` in `packages/game-core` — all existing `systems.rs` integration tests pass with base content pack
- [x] 8.3 Manual smoke test: build wall/bed/bush, eat, sleep, save/load, toolbar and colors match pre-change behavior
- [x] 8.4 Document save-file ID migration note (PascalCase → snake_case) in save file helper or README if needed
