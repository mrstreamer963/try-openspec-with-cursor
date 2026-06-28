## 1. Need assignment refactor

- [ ] 1.1 Refactor `auto_assign_tasks` need branch: iterate critical needs in priority order `[Food, Sleep]`, attempt assignment per need, stop at first success
- [ ] 1.2 Reuse existing `nearest_eat_assignment`, `nearest_free_bed`, and `find_path` checks — no new assignment helpers unless loop clarity requires a small `try_need_assignment` wrapper
- [ ] 1.3 Verify successful Eat/Sleep assignment overwrites wander `Path` (existing path write path; no separate clear needed)

## 2. Tests

- [ ] 2.1 Test fallback to Sleep when Food critical, no bushes, Sleep critical, free bed exists
- [ ] 2.2 Test fallback to Eat when Sleep critical, no beds, Food critical, bush exists
- [ ] 2.3 Test Food priority when both needs critical and both targets available
- [ ] 2.4 Test colonist stays idle when both needs critical but neither target satisfiable
- [ ] 2.5 Test wander path replaced when fallback Sleep assignment succeeds

## 3. Verify

- [ ] 3.1 Run `cargo test` in `packages/game-core`
- [ ] 3.2 Rebuild WASM and manually verify colonists go to empty bed when food=0, sleep=0, and no food on map
