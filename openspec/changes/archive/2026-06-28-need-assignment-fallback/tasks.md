## 1. Need assignment refactor

- [x] 1.1 Refactor `auto_assign_tasks` need branch: iterate critical needs in priority order `[Food, Sleep]`, attempt assignment per need, stop at first success
- [x] 1.2 Reuse existing `nearest_eat_assignment`, `nearest_free_bed`, and `find_path` checks — no new assignment helpers unless loop clarity requires a small `try_need_assignment` wrapper
- [x] 1.3 Verify successful Eat/Sleep assignment overwrites wander `Path` (existing path write path; no separate clear needed)

## 2. Tests

- [x] 2.1 Test fallback to Sleep when Food critical, no bushes, Sleep critical, free bed exists
- [x] 2.2 Test fallback to Eat when Sleep critical, no beds, Food critical, bush exists
- [x] 2.3 Test Food priority when both needs critical and both targets available
- [x] 2.4 Test colonist stays idle when both needs critical but neither target satisfiable
- [x] 2.5 Test wander path replaced when fallback Sleep assignment succeeds

## 3. Verify

- [x] 3.1 Run `cargo test` in `packages/game-core`
- [x] 3.2 Rebuild WASM and manually verify colonists go to empty bed when food=0, sleep=0, and no food on map
