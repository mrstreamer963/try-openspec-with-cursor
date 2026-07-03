## Why

Colonists can settle on BerryBush tiles because bushes do not block movement. Eating requires an adjacent stand, so standing on the bush looks wrong and prevents eating from completing.

## What Changes

- Colonists may traverse bush cells during movement but SHALL NOT snap or settle on them
- Bush cells are skipped as path waypoints; movement continues toward the next waypoint
- Wander, vacate, eat stand, and build stand selection exclude bush tiles as destinations
- Colonists already on a bush cell are ejected to a nearby settleable cell

## Capabilities

### Modified Capabilities

- `colonist-simulation`: Berry bush pass-through without settling
- `world-simulation`: Bush tiles are traversable but not valid colonist settle cells

## Impact

- `packages/game-core`: `systems.rs` movement, wander, vacate, assignment filters
