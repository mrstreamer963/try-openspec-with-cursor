## Why

Berry bushes currently provide infinite food. Adding a finite berry supply creates scarcity and encourages the player to plant more bushes.

## What Changes

- Each BerryBush spawns with a fixed berry count (3 portions)
- Eating at a bush consumes one portion and restores Food need
- When berries reach zero, the bush is removed from the grid and despawned
- Task assignment only targets bushes with berries remaining
- Snapshot exposes remaining berries for optional UI feedback

## Capabilities

### Modified Capabilities

- `world-simulation`: Berry bushes are depletable and removed when empty
- `colonist-simulation`: Eat tasks target only bushes with berries; depleted mid-path fails gracefully

## Impact

- `packages/game-core`: new `BerrySupply` component, `remove_building`, updated systems
- `packages/client`: `BuildingSnapshot.berries` field, optional visual depletion hint
