## Why

Building walls one tile at a time is slow, and deconstructing an area tile-by-tile is tedious. Left-click drag is already used for camera pan, which blocks using drag for line or rectangle placement tools. Separating pan to the right mouse button frees left-click drag for shape-based build and deconstruct actions.

## What Changes

- **Pan with right mouse button (RMB) drag** on the canvas; suppress browser context menu on the game canvas
- **Wall tool: LMB drag** places a horizontal or vertical line of walls (dominant axis snap, no diagonals); LMB tap places one wall
- **Deconstruct tool: LMB drag** selects an axis-aligned rectangle of tiles; LMB tap deconstructs one tile
- **Invalid tiles are skipped** when committing a line or rectangle (server still validates individual commands)
- **Preview overlay** while dragging (ghost walls / red deconstruct highlight)
- **Bed and berry bush** remain single-click placement only
- **Select mode**: LMB tap selects colonists or dismisses panel; pan via RMB drag, WASD, or arrows (LMB drag no longer pans)

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `view-layer`: camera pan input model (RMB drag), wall line drag placement, deconstruct rectangle drag, placement preview overlay, build mode suppresses colonist selection on LMB

## Impact

- **TypeScript (`packages/client`)**: `PixiRenderer.ts` (interaction state machine, preview layer, RMB pan), new tile geometry helpers, `GameSession.vue` (tool mode wiring, batch build/deconstruct on drag commit)
- **Unchanged**: game-core worker protocol (reuse existing per-tile `build` and `deconstruct` events), simulation logic, content definitions
