## Context

A* pathfinding in `packages/game-core/src/pathfinding.rs` expands only four cardinal neighbors (`DIRS: [(1,0), (-1,0), (0,1), (0,-1)]`) with uniform cost 1 and a Manhattan heuristic. Colonist movement (`colonist_movement` in `systems.rs`) already advances continuously along straight lines between waypoints using Euclidean distance, so diagonal waypoints require no movement-system changes.

Building interaction stands (`best_adjacent_stand`) intentionally remain orthogonal-only — colonists eat, build, and deconstruct from cardinal-adjacent tiles. Only route computation changes.

## Goals / Non-Goals

**Goals:**
- Enable 8-directional A* pathfinding with correct diagonal step costs
- Prevent corner-cutting through impassable tiles
- Use an admissible octile heuristic for efficient search
- Add unit tests for diagonal paths and corner-cutting

**Non-Goals:**
- Diagonal building stand tiles (eat/build/deconstruct adjacency stays orthogonal)
- Changing settled-cell occupancy rules (still one colonist per integer grid cell)
- Changing wander radius selection (Manhattan distance filter is fine for target picking)
- Client/renderer changes

## Decisions

### 1. Eight neighbors with mixed step costs

**Choice:** Expand 8 directions; orthogonal cost = `1.0`, diagonal cost = `SQRT_2` (≈ 1.414).

**Rationale:** Uniform cost for diagonals would make diagonal routes artificially cheap (√2 tiles for cost 1). Mixed costs keep orthogonal and diagonal distances consistent with continuous movement speed.

**Alternative considered:** Integer-scaled costs (10 / 14) — rejected; floats already used in `g_score` and work fine.

### 2. Corner-cutting prevention

**Choice:** For a diagonal step from `(x,y)` to `(x+dx, y+dy)` where `dx,dy ∈ {-1,1}`, require both `(x+dx, y)` and `(x, y+dy)` to be walkable (and not blocked).

**Rationale:** Standard grid pathfinding rule; prevents colonists from slipping through 1-tile-wide diagonal gaps between walls.

### 3. Octile heuristic

**Choice:** Replace Manhattan heuristic with octile distance: `D * (dx + dy) + (SQRT_2 - 2*D) * min(dx, dy)` where `D = 1.0`.

**Rationale:** Admissible for mixed orthogonal/diagonal costs; yields tighter f-scores than Manhattan, reducing explored nodes.

### 4. No movement-system changes

**Choice:** Leave `colonist_movement` unchanged.

**Rationale:** Movement already interpolates along the vector to the next waypoint. Diagonal waypoints produce natural diagonal visual motion at the same `MOVE_SPEED`.

### 5. Keep `best_adjacent_stand` orthogonal

**Choice:** Do not change `ORTHOGONAL_DIRS` in `best_adjacent_stand`.

**Rationale:** Gameplay semantics (standing next to a building face) are orthogonal by design; pathfinding reaches those stands via whatever route is shortest.

## Risks / Trade-offs

- **[Diagonal paths may look faster visually]** → Diagonal step cost √2 compensates in path length; `MOVE_SPEED` is tiles/second along the interpolation vector, so travel time per diagonal tile edge is correct.
- **[More neighbors per A* expansion]** → Octile heuristic mitigates extra branching; grid is small (idle colony sim), so performance impact is negligible.
- **[Corner-cutting edge cases in tests]** → Explicit unit tests for blocked-corner scenarios.

## Migration Plan

Single deploy: update `pathfinding.rs`, add tests, run `cargo test -p game-core`. No save-format or snapshot migration. Existing saved paths in running games will complete on old waypoints; new paths computed after deploy use 8-directional search.

## Open Questions

None — behavior is fully specified.
