## 1. Core repath logic

- [x] 1.1 Add `set_path_from_route` / `waypoints_from_route` helpers in `systems.rs`
- [x] 1.2 Replace `release_blocked_idle_wander` with `repath_blocked_colonists` (all task kinds)
- [x] 1.3 Wire into `auto_assign_tasks` with `prefer_sleep_first` merge on unreachable Eat

## 2. Tests

- [x] 2.1 Add Eat/Build/Sleep intermediate-block repath tests
- [x] 2.2 Add unreachable repath clears task test
- [x] 2.3 Verify idle wander regression test still passes
- [x] 2.4 Add goal-stand occupied reassignment tests (Eat/Build/Sleep)

## 3. Goal-stand reassignment

- [x] 3.1 Clear Eat/Build/Sleep when blocked on task target waypoint
- [x] 3.2 Filter occupied stands in `nearest_build_assignment`

## 3. Verification

- [x] 3.1 Run `cargo test -p game-core`
