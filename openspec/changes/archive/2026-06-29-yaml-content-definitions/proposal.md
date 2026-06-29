## Why

Game balance, building behavior, needs, and debuffs are hardcoded in Rust enums and `match` arms across `game-core` and duplicated in the client (`types.ts`, `Toolbar.vue`). Adding or tuning content requires recompiling WASM and editing TypeScript in multiple places, which blocks modding and slows iteration. Moving definitions to YAML with a baked runtime registry keeps simulation fast while making content editable and extensible without code changes.

## What Changes

- Introduce a **base content pack** in YAML (`needs`, `statuses`, `buildings`, `terrain`) that reproduces current v1 behavior (Wall, Bed, BerryBush; Food/Sleep; Hungry/WantsSleep)
- Add a **ContentRegistry** in `game-core`: parse YAML once at startup, validate references, intern string IDs to dense `u32` indices, expose O(1) lookups in the simulation hot path
- Replace hardcoded `BuildingType` / `TerrainType` enums and per-type `match` logic with **content IDs** and data-driven primitives (`blocks_movement`, `supply`, `reservation`, `interaction` modes, `restore_need`, etc.)
- Replace fixed `Needs { food, sleep }` and marker components `Hungry` / `WantsSleep` with a **dynamic needs map** and **status system** driven by YAML conditions and effects
- Load content at **game initialization**: client reads/bundles YAML and passes a merged JSON/YAML payload to WASM via `Game::new` or a dedicated init event
- Update the **client** to build toolbar, terrain/building colors, and colonist info labels from the same content pack (single source of truth)
- **BREAKING**: wire protocol and save snapshots use string content IDs (e.g. `"berry_bush"`) instead of PascalCase enum variant names (`"BerryBush"`); provide a one-time migration note for existing save files
- Phase out Rust constants that duplicate YAML (`NEED_THRESHOLD`, `work_required_for`, `BERRIES_PER_BUSH`, `TERRAIN_COLORS`, etc.) in favor of registry lookups

## Capabilities

### New Capabilities

- `content-definitions`: YAML schema, base content pack layout, registry loading/validation/baking, mod-merge rules (base pack only in v1), and performance constraints (no string/HashMap lookups in pathfinding hot loops)

### Modified Capabilities

- `game-core`: Accept content pack at init; store `ContentRegistry` as ECS resource; serialize/deserialize content IDs in events and snapshots
- `world-simulation`: Terrain and building behavior defined by YAML; construction and interactions use registry primitives instead of enum matches
- `colonist-simulation`: Needs decay, critical thresholds, status flags, and task assignment driven by YAML definitions; preserve existing v1 gameplay via base pack
- `view-layer`: Toolbar, colors, and colonist need/status labels rendered from loaded content definitions
- `worker-bridge`: Bundle or fetch base YAML on worker init and pass content payload when instantiating WASM `Game`

## Impact

- `packages/game-core/` — new `content/` module; refactor `components.rs`, `systems.rs`, `world.rs`, `events.rs`, `game.rs`; add `serde_yaml` dependency
- `packages/content/` or `content/base/` — YAML files for base pack (needs, statuses, buildings, terrain)
- `packages/client/` — content loader, updated `types.ts`, `Toolbar.vue`, `PixiRenderer.ts`, `ColonistInfo.vue`, worker init
- `packages/game-core/Cargo.toml` — `serde_yaml`
- `openspec/specs/` — new `content-definitions` spec; deltas for `game-core`, `world-simulation`, `colonist-simulation`, `view-layer`, `worker-bridge`
- Existing integration tests in `systems.rs` must pass unchanged behavior with base YAML definitions
