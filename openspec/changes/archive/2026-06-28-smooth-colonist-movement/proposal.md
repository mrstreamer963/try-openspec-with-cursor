## Why

Colonists currently move in discrete grid-cell jumps because positions are stored as `i32` and each movement step is rounded to whole tiles. This looks jerky and breaks the illusion of continuous motion. The simulation tick rate (20 Hz) is fine for an idle sim — the fix is to represent colonist world positions as floats while keeping grid logic (pathfinding, tasks, buildings) on integer cells.

## What Changes

- Change colonist `Position` from integer grid coordinates to **float world coordinates** (tile units, e.g. `5.3, 7.8`)
- Update `colonist_movement` to advance position continuously without `round()` — sub-tile steps per tick
- Keep pathfinding, task targets, and building placement on the **integer grid**; derive grid cell via `floor()` when needed
- Update `ColonistSnapshot` to expose float `x`/`y` to the client
- Update PixiJS rendering to draw colonists at float positions (already a multiplication — no client interpolation layer needed for v1)
- Update colonist click detection from tile-equality to distance-based hit test
- **BREAKING**: `ColonistSnapshot.x` and `.y` change from integer grid indices to float tile coordinates

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `colonist-simulation`: Colonist positions are float world coordinates; movement is continuous at sub-tile resolution; grid-based systems use floored cell coordinates
- `view-layer`: Colonist sprites render at float positions; click hit-testing uses distance to sprite center

## Impact

- **Rust (`packages/game-core`)**: `Position` component, `colonist_movement`, `auto_assign_tasks`, `task_execution`, `ColonistSnapshot`, spawn logic
- **TypeScript (`packages/client`)**: `ColonistSnapshot` types, `PixiRenderer` click detection; colonist info panel may show float coordinates
- **Protocol**: Snapshot JSON schema for colonist positions (float instead of int)
- **Unchanged**: 20 Hz worker tick (`TICK_MS = 50`), `MOVE_SPEED`, pathfinding algorithm, building grid placement
