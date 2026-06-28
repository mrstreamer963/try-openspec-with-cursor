## 1. Constants and helpers

- [x] 1.1 Add `WANDER_RADIUS` and `WANDER_PICK_ATTEMPTS` constants to `world.rs`
- [x] 1.2 Implement `pick_wander_target(grid, from, occupied)` in `systems.rs` — collect walkable cells within Manhattan radius, exclude current cell and occupied cells, random pick with retries until `find_path` succeeds

## 2. Wander assignment

- [x] 2.1 Extend `auto_assign_tasks`: after need/build branches, for idle colonists with completed or empty path, call `pick_wander_target` and populate `Path` waypoints
- [x] 2.2 Ensure Eat/Sleep/Build assignment overwrites wander path (verify existing path assignment path)
- [x] 2.3 Build per-tick occupancy set for wander target filtering (reuse occupancy helper if present, or inline `grid_cell()` map)

## 3. Tests

- [x] 3.1 Test idle colonist with no needs/build gets a non-empty path after `auto_assign_tasks`
- [x] 3.2 Test wander path is replaced when critical need triggers Eat assignment
- [x] 3.3 Test wander target excludes current cell and occupied cells
- [x] 3.4 Test task kind remains `Idle` while wander path is active

## 4. Verify

- [x] 4.1 Rebuild WASM and run dev server
- [x] 4.2 Manually verify idle colonists walk to nearby tiles when needs are satisfied and no build orders exist
- [x] 4.3 Manually verify colonists pick new destinations after arriving and still preempt wander for eat/sleep/build
