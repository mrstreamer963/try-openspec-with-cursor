## 1. Game core — position model

- [x] 1.1 Change `Position` in `components.rs` from `i32` to `f32` for `x` and `y`
- [x] 1.2 Add a small helper (e.g. `Position::grid_cell() -> (i32, i32)`) using `floor()` for grid lookups
- [x] 1.3 Update `ColonistSnapshot` in `events.rs` so `x` and `y` are `f32`
- [x] 1.4 Update building spawn in `game.rs` to use `x as f32, y as f32`; cast to `i32` in `BuildingSnapshot`

## 2. Game core — movement and systems

- [x] 2.1 Rewrite `colonist_movement`: remove `round() as i32`, use float delta; snap to waypoint on arrival
- [x] 2.2 Update `auto_assign_tasks` and `nearest_building_for_need` to use `grid_cell()` for path start and distance
- [x] 2.3 Update `task_execution` to check arrival via completed path + floored cell matching `task.target_x/y`
- [x] 2.4 Update `spawn_colonists` to initialize positions as `(x as f32, y as f32)`

## 3. Client — rendering and interaction

- [x] 3.1 Verify `PixiRenderer.drawColonists` uses float `c.x`/`c.y` directly (no `Math.floor`)
- [x] 3.2 Replace tile-equality colonist click test with distance-based hit test (radius `TILE_SIZE * 0.35`)
- [x] 3.3 Update `ColonistInfo` panel to display floored grid coordinates for position

## 4. Build and verify

- [x] 4.1 Rebuild WASM and run dev server
- [x] 4.2 Manually verify colonists glide smoothly between cells at 1× speed
- [x] 4.3 Manually verify eat/sleep tasks still assign, pathfind, and complete correctly
- [x] 4.4 Manually verify colonist click and info panel work while colonist is mid-cell
