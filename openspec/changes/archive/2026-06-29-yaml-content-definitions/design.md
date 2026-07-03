## Context

The colony sim hardcodes content in Rust enums (`TerrainType`, `BuildingType`, `NeedKind`), marker components (`Hungry`, `WantsSleep`), and scattered `match` arms in `systems.rs` / `game.rs`. The client duplicates types and colors in `types.ts`, `Toolbar.vue`, and `PixiRenderer.ts`. Serde already serializes enums for save/load and the worker protocol, but adding content requires WASM recompilation and multi-file edits.

The game runs as WASM in a WebWorker with a 50×50 grid, ~3 colonists, and A* pathfinding as the dominant CPU cost. Modding requires external YAML definitions loaded once at startup and baked into fast runtime structures.

## Goals / Non-Goals

**Goals:**

- Ship a **base content pack** in YAML that reproduces all v1 behavior (terrain, 3 buildings, food/sleep needs, hungry/wants_sleep statuses).
- Introduce **`ContentRegistry`** with validate → intern → bake pipeline; simulation hot paths use `Copy` numeric IDs and `Vec` lookups.
- Replace enum-driven behavior with a small set of **interaction primitives** (`supply`, `reservation`, `restore_need`, `consume_supply`, `blocks_movement`, `blocks_settle`, interaction `mode`).
- Pass content from **client to WASM at init**; client UI reads the same pack for toolbar, colors, and labels.
- Use **snake_case content IDs** in protocol and snapshots (`berry_bush`, not `BerryBush`).
- Keep existing integration tests passing with base YAML definitions.

**Non-Goals (v1 of this change):**

- User-facing mod folder UI or zip mod installation.
- Multi-mod merge / `depends_on` dependency resolution beyond a single bundled base pack.
- Scripting (Lua/Rhai) or WASM plugin mods.
- Dynamic addition of new interaction primitive types (only data-configurable primitives implemented in Rust).
- Changing world size, tick rate, or pathfinding algorithm.

## Decisions

### 1. YAML on disk, JSON across the WASM boundary

**Decision:** Store authoritative definitions as YAML in `content/base/`. Client loads YAML via Vite (`?raw` or `js-yaml`), optionally normalizes to JSON, and passes a string payload to `Game::new(content_json)`. Rust parses with `serde_yaml` (or `serde_json` if client pre-serializes).

**Rationale:** YAML is human-friendly for modders; JSON string crossing wasm-bindgen is simpler than file I/O inside WASM.

**Alternative considered:** `include_str!` YAML only in Rust — rejected because client also needs the same data without duplicating files.

### 2. Intern IDs to dense indices at load time

**Decision:**

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerrainId(u16);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuildingId(u16);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeedId(u8);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusId(u8);

pub struct ContentRegistry {
    pub terrain: Vec<TerrainDef>,
    pub buildings: Vec<BuildingDef>,
    pub needs: Vec<NeedDef>,
    pub statuses: Vec<StatusDef>,
    terrain_by_str: HashMap<String, TerrainId>,  // load-time only
    // ...
}
```

`WorldGrid` stores `Vec<TerrainId>` and `Vec<Option<BuildingId>>`. Wire format and snapshots still use strings; conversion happens at serde boundary or snapshot build/restore.

**Rationale:** Near-enum performance in pathfinding; strings only at I/O edges.

**Alternative considered:** `String` IDs everywhere — rejected for hot-loop cost at scale.

### 3. Dynamic needs map with fixed base-pack snapshot compatibility

**Decision:** ECS component `Needs(HashMap<NeedId, f32>)` internally. Snapshots keep **flat fields** `food`, `sleep`, `hungry`, `wants_sleep` for v1 save compatibility, mapped via registry (`food` ↔ `NeedId` for need `food`, `hungry` ↔ `StatusId` for status `hungry`).

**Rationale:** Enables modding architecture without forcing save format v2 immediately.

**Alternative considered:** Snapshot `needs: Record<string, number>` — deferred to a follow-up save-format bump.

### 4. Status system replaces marker components

**Decision:** Single component `ActiveStatuses(HashSet<StatusId>)`. `sync_statuses` system evaluates YAML `apply_when` rules (v1: only `need_below`). `auto_assign_tasks` reads status `effects` (v1: `task_priority` for eat/sleep).

Remove `Hungry` / `WantsSleep` marker components after migration; snapshot flags derived from active status set.

**Rationale:** One extensible path for future debuffs (`freezing`, etc.) without new Rust component types.

### 5. Building behavior via primitives, not per-type code

**Decision:** `BuildingDef` contains `on_complete: Vec<SpawnPrimitive>` and `interactions: Vec<InteractionDef>`. v1 primitives:

| Primitive | Replaces |
|-----------|----------|
| `Supply { resource, amount }` | `BerrySupply` |
| `Reservation { kind: SingleOccupant }` | `BedOccupancy` |
| `blocks_movement`, `blocks_settle` | hardcoded matches |
| `Interaction { mode, duration_sec, effects }` | Eat/Sleep task execution branches |

`complete_construction` and `task_execution` dispatch on primitives, not `if building == BerryBush`.

**Rationale:** New buildings in YAML combine primitives; no Rust changes for simple mods.

### 6. Content pack file layout

```
content/base/
  mod.yaml          # id: base, version: 1
  needs.yaml
  statuses.yaml
  buildings.yaml
  terrain.yaml
```

Client merges into one object `{ needs, statuses, buildings, terrain }` before sending to WASM.

### 7. Game constructor API change

**Decision:** `Game::new(content_json: &str) -> Result<Game, JsValue>` (wasm-bindgen). Worker loads base pack and passes it. Existing `Game::new()` removed.

**Rationale:** Registry must exist before world generation and colonist spawn.

### 8. Client content module

**Decision:** `packages/client/src/content/` exports `loadBaseContent(): ContentPack` used by `gameWorker.ts`, `Toolbar.vue`, `PixiRenderer.ts`, `ColonistInfo.vue`.

Shared types in `packages/client/src/content/types.ts`. No separate npm package yet — `content/` lives at repo root, imported by client via relative path or Vite alias `@content`.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Large refactor breaks many tests | Migrate in phases; base YAML must pass all existing `systems.rs` tests unchanged |
| Save files break on ID rename | Document migration; keep snapshot flat fields; optional save v2 later |
| Primitive set too limited for mods | Document supported primitives; extensibility via follow-up changes |
| YAML parse errors at runtime | Validate at init; worker posts error to main thread before game loop |
| Client/core drift | Single `content/base/` directory; both sides load same files |
| Performance regression | Bake indices; ban string HashMap in `WorldGrid` and pathfinding |

## Migration Plan

1. Add `content/base/` YAML reproducing current constants (threshold 30, decay rates, work_required, berries=3, sleep duration, colors).
2. Implement `ContentRegistry` + tests for load/validate.
3. Switch `WorldGrid` to `TerrainId` / `BuildingId`; update pathfinding walkability to use registry.
4. Replace `Needs` struct fields with map; add `sync_statuses`; update `auto_assign_tasks` / `task_execution` to use primitives.
5. Update events/snapshots to string IDs at serde layer with intern on load/export.
6. Wire client content loader + update UI.
7. Remove dead enums, constants, and marker components.
8. Bump save file docs if needed (IDs in embedded state change PascalCase → snake_case).

**Rollback:** Revert to enum-based code; YAML files remain in repo unused.

## Open Questions

- Vite alias path: `content/` at repo root vs `packages/content/` — prefer repo root `content/base/` for modder visibility.
- Parse YAML in Rust vs only in client: start with client → JSON string → Rust `serde_json` to avoid `serde_yaml` in WASM if binary size matters; otherwise `serde_yaml` in `game-core` for native tests.
- Whether to add save format `version: 2` in the same change or accept breaking building id strings only in raw `StateSnapshot` fields.
