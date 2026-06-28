## 1. Game core — berry supply

- [x] 1.1 Add `BerrySupply` component and `BERRIES_PER_BUSH` constant
- [x] 1.2 Add `WorldGrid::remove_building` and spawn bushes with `BerrySupply`
- [x] 1.3 Update `auto_assign_tasks` to filter bushes with berries > 0
- [x] 1.4 Update `task_execution` to consume berries and remove depleted bushes
- [x] 1.5 Expose `berries` in `BuildingSnapshot`

## 2. Client

- [x] 2.1 Add `berries` to TypeScript `BuildingSnapshot` and render depletion hint
