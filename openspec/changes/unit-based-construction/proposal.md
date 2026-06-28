## Why

Buildings currently appear instantly when the player clicks a tile, which breaks the colony-sim fantasy: colonists should do the work while the player only designates where structures go. Requiring unit-based construction makes placement feel like giving orders rather than spawning objects.

## What Changes

- **Build command becomes a construction order**: clicking with a build tool creates a pending construction site at the tile instead of spawning a finished building
- **New Build task**: idle colonists (when not satisfying critical needs) pick up construction orders, path to the site, and work until completion
- **Construction progress**: each site tracks work remaining; when progress reaches zero the building becomes functional (Bed, BerryBush, or Wall)
- **Snapshot exposes construction sites**: the client renders in-progress blueprints separately from finished buildings
- **Validation unchanged**: orders are rejected on water, occupied tiles, or duplicate orders at the same cell
- **Resource-free construction preserved**: no material costs in v1; only colonist labor and time

## Capabilities

### New Capabilities

_(none — construction orders extend existing simulation capabilities)_

### Modified Capabilities

- `world-simulation`: Build commands create construction orders; finished buildings appear only after colonist work completes
- `colonist-simulation`: New Build task type, construction assignment priority below critical needs, pathfinding and execution for building sites
- `view-layer`: Render construction-site blueprints (ghost/preview) distinct from completed buildings

## Impact

- `packages/game-core`: `ConstructionSite` component or resource, `TaskKind::Build`, updated `IncomingEvent::Build` handler, construction progress system, snapshot fields for pending sites
- `packages/client`: `ConstructionSiteSnapshot` type, PixiJS layer or styling for blueprint tiles, toolbar behavior unchanged (still sends build at click)
