## 1. Game core — Task model and helpers

- [x] 1.1 Add `building_x`, `building_y` fields to `Task` (stand tile remains `target_x`, `target_y`)
- [x] 1.2 Add `best_adjacent_stand` helper: orthogonally adjacent walkable cells, pick shortest path from colonist
- [x] 1.3 Add `BedOccupancy { reserved_by: Option<Entity> }` component; spawn on Bed placement

## 2. Game core — Eat interaction (adjacent)

- [x] 2.1 Update `auto_assign_tasks` for Eat: resolve bush + adjacent stand tile, path to stand
- [x] 2.2 Skip bushes with no adjacent walkable stand tile
- [x] 2.3 Update `task_execution` for Eat: verify colonist on stand tile adjacent to bush at `building_x/y`
- [x] 2.4 Update depleted-bush fail path to use building coords + adjacency check

## 3. Game core — Sleep interaction (single occupancy)

- [x] 3.1 Update `auto_assign_tasks` for Sleep: filter beds where `BedOccupancy.reserved_by` is None
- [x] 3.2 Reserve bed (`reserved_by = colonist entity`) on Sleep task assignment
- [x] 3.3 Update `task_execution` for Sleep: verify reservation matches colonist, release on complete
- [x] 3.4 Release bed reservation when Sleep task is cancelled or path fails

## 4. Verification

- [x] 4.1 Manual test: colonist eats from adjacent tile, never stands on bush
- [x] 4.2 Manual test: two sleepy colonists, one bed — only one sleeps; other finds another bed or waits
