## 1. Game core — occupancy helpers

- [ ] 1.1 Add `colonist_occupancy_map(world) -> HashMap<(i32,i32), Entity>` helper from settled `grid_cell()` positions
- [ ] 1.2 Add `is_cell_free(map, cell, self_entity) -> bool` helper

## 2. Game core — movement snap with occupancy

- [ ] 2.1 Refactor `colonist_movement` to intent/apply two-phase: collect snap intents when waypoint reached
- [ ] 2.2 Apply snaps in stable entity order; skip snap if target cell occupied by another colonist
- [ ] 2.3 Keep partial-step interpolation unchanged (pass-through during movement)

## 3. Game core — eat assignment filtering

- [ ] 3.1 Pass occupancy map and batch-assigned stands into `nearest_eat_assignment`
- [ ] 3.2 Update `best_adjacent_stand` (or wrapper) to skip stands occupied by settled colonists
- [ ] 3.3 Track `reserved_stands: HashSet<(i32,i32)>` in `auto_assign_tasks` batch (mirror `reserved_beds`)

## 4. Verification

- [ ] 4.1 Manual test: two colonists path to same cell — second waits, no overlap after snap
- [ ] 4.2 Manual test: colonists cross mid-path — pass through visually, separate cells after snap
- [ ] 4.3 Manual test: single stand at bush — second hungry colonist waits or uses another bush/stand
