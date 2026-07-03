## Decisions

### BerrySupply ECS component

Store `remaining: u8` on BerryBush entities only. Constant `BERRIES_PER_BUSH = 3` at spawn.

### Depletion = full removal

When `remaining` hits 0 after eating: clear `WorldGrid.buildings[i]`, despawn ECS entity. No empty-bush state.

### No reservation

If a colonist arrives at a depleted bush, the eat task fails (no food restore) and idle reassignment picks another bush next tick.

### Snapshot

`BuildingSnapshot` gains optional `berries: Option<u8>` — `Some(n)` for BerryBush, `None` for other buildings.

## Risks

- Three colonists can race for the last berry — acceptable for v1
- No starting bushes — player must build; unchanged from current behavior
