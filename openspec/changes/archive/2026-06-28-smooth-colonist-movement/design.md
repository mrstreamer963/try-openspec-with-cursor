## Context

Colonist movement is implemented in `colonist_movement` (`packages/game-core/src/systems.rs`). Positions are stored as `i32` grid coordinates. Each tick advances movement by `MOVE_SPEED * dt` tiles, but the delta is rounded to integers (`round() as i32`). At 20 Hz with `MOVE_SPEED = 4.0`, each step is 0.2 tiles — which rounds to zero for several ticks, then snaps to the next cell. The client renders colonists at `c.x * TILE_SIZE`, so visual position jumps in sync with integer snapshots.

Pathfinding, buildings, and task targets remain grid-based (A* on 50×50 integer cells). Only colonist **world position** needs to become continuous.

## Goals / Non-Goals

**Goals:**
- Colonists move smoothly in sub-tile increments between path waypoints
- Simulation tick stays at 20 Hz (`TICK_MS = 50`, `BASE_DT = 0.05`)
- Grid logic (pathfinding, walkability, task arrival, building interaction) unchanged in semantics
- Snapshot protocol carries float colonist positions; renderer uses them directly

**Non-Goals:**
- Client-side interpolation/extrapolation between snapshots (not needed once sim emits floats)
- Changing `MOVE_SPEED`, tick rate, or pathfinding algorithm
- Diagonal movement or physics-based collision
- Splitting `Position` into separate components for buildings vs colonists (buildings stay at integer coords stored as `f32`)

## Decisions

### 1. Float world coordinates in tile units

**Choice:** Change colonist `Position.x` / `Position.y` from `i32` to `f32`, measured in tile units (center of tile `(5, 7)` → world pos `(5.0, 7.0)`).

**Rationale:** One coordinate system for sim and render. `MOVE_SPEED` already means tiles/sec; movement math becomes `pos += direction * step` without rounding.

**Alternatives considered:**
- *Separate `WorldPos` component* — cleaner types but more refactor for little gain at this scale
- *Client-only lerp between integer snapshots* — two sources of truth; click/UI desync

### 2. Grid cell derived via `floor()`

**Choice:** All grid operations use `pos.x.floor() as i32` and `pos.y.floor() as i32`.

**Applies to:**
- `find_path` start cell in `auto_assign_tasks`
- `nearest_building_for_need` distance from current cell
- `task_execution` arrival check and building lookup
- Path waypoint targets remain integer `(i32, i32)` in `Path.waypoints`

**Rationale:** Pathfinding and buildings live on the grid; float position is only for in-transit rendering and movement.

### 3. Movement algorithm (no rounding)

**Choice:** In `colonist_movement`:

```
step = MOVE_SPEED * dt
if dist <= step:
    pos = waypoint (as f32)
    path.index += 1
else:
    pos.x += (dx / dist) * step
    pos.y += (dy / dist) * step
```

**Rationale:** Direct fix for the root cause. At 0.2 tiles/tick, colonists visibly glide across cells.

### 4. Shared `Position` struct with `f32` for all entities

**Choice:** Change the single `Position` struct to `f32`. Buildings spawn at `(x as f32, y as f32)`. Building snapshots cast back to `i32` for the protocol.

**Rationale:** Minimal diff; buildings are always on cell centers anyway.

### 5. Snapshot schema: colonist `x`/`y` as float

**Choice:** `ColonistSnapshot.x` and `.y` become `f32` in Rust serde output (JSON numbers with decimals). TypeScript types already use `number`.

**Rationale:** Renderer multiplies by `TILE_SIZE` — works unchanged. Breaking change is acceptable (no external consumers).

### 6. Click hit test by distance

**Choice:** In `PixiRenderer.handleClick`, detect colonist clicks by comparing click position to sprite center within radius `TILE_SIZE * 0.35` (matching draw radius).

**Rationale:** Integer tile equality (`c.x === tileX`) breaks with float positions.

## Risks / Trade-offs

- **[Task arrival edge case]** Colonist at `(5.99, 7.0)` floors to cell 5, not 6 → Mitigation: snap to waypoint center on arrival; task execution checks path complete + floored cell matches target
- **[Path reassignment mid-transit]** `auto_assign_tasks` uses floored position; colonist mid-cell may get path from adjacent cell → Acceptable; same as before with integer pos
- **[20 Hz visual stepping]** Sub-tile motion at 20 Hz is smooth enough for idle sim; extreme zoom may show micro-stutter → Mitigation: defer client interpolation to future change if needed
- **[Breaking snapshot JSON]** Any external tooling expecting integer colonist coords breaks → None exists in v1

## Migration Plan

1. Change `Position` to `f32` in `components.rs`
2. Update movement, task, and pathfinding call sites with `floor()` helpers
3. Rebuild WASM (`npm run build:wasm` or dev script)
4. Update client click detection
5. Manual verify: colonists glide smoothly; eat/sleep tasks still complete

No data migration — fresh browser session.

## Open Questions

- **Colonist info panel coordinates:** Show float world position (e.g. `5.3, 7.8`) or floored grid cell (`5, 7`)? Default: show grid cell for player readability, optional one-decimal float.
