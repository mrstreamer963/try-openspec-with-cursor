## Why

Camera panning currently requires click-and-drag on the canvas, which is awkward on trackpads and when the user wants to keep the mouse free for building or selecting colonists. WASD (and arrow keys) are a familiar RTS-style control for map navigation and should work alongside the existing drag-to-pan behavior.

## What Changes

- Add keyboard camera panning with **W/A/S/D** and arrow keys while the game session is active
- Pan speed scales with zoom level so movement feels consistent at different zoom levels
- Support holding multiple keys simultaneously for diagonal panning
- Do not interfere with existing keyboard shortcuts (Space for pause, 1/2/3 for speed)
- Do not pan when focus is in a text input (if any are added later)

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `view-layer`: Extend the camera pan requirement to include WASD and arrow-key panning alongside mouse drag

## Impact

- **TypeScript (`packages/client`)**: `PixiRenderer.ts` (camera offset updates on a per-frame ticker or key state), possibly `GameSession.vue` if keyboard routing is centralized there
- **Unchanged**: game simulation, worker protocol, Rust game-core, HUD shortcuts
