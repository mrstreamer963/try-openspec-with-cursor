## Why

We need a playable v1 of a RimWorld-like idle colony simulation that runs entirely in the browser. The project validates a three-module architecture—Rust ECS in a WebWorker, Vue/PixiJS view layer, and a message bridge—so we can iterate on colony mechanics without blocking the UI thread.

## What Changes

- Add a **Game Core** module: Rust + bevy_ecs compiled to WASM, running in a WebWorker with manual `tick(dt)`, A* pathfinding, and typed events via `IncomingEvent`/`OutgoingEvent`
- Add a **View Layer** module: Vite + Vue 3 + TypeScript + PixiJS 8 for tile/sprite rendering, camera controls, HUD (pause/speed), build toolbar, and colonist info panel
- Add a **Worker Bridge** module: `postMessage`-based communication with a 50 ms game loop in the worker and `requestAnimationFrame` rendering on the main thread
- Implement v1 gameplay: 50×50 world (water, sand, grass), 3 colonists with Food/Sleep needs, auto-assigned tasks, buildable structures (bed, berry bush), resource-free construction, and pausable real-time simulation

## Capabilities

### New Capabilities

- `game-core`: WASM ECS engine with manual tick, event protocol, and serde/wasm-bindgen data exchange
- `world-simulation`: 50×50 tile world generation, terrain types, and placeable buildings (bed, berry bush)
- `colonist-simulation`: Colonist entities with Food/Sleep needs, automatic task assignment, and A* pathfinding
- `view-layer`: PixiJS rendering (tiles, sprites, entities), camera pan/zoom, HUD, build toolbar, and colonist info panel
- `worker-bridge`: WebWorker game loop, postMessage protocol, and Vite WASM plugin integration

### Modified Capabilities

_(none — greenfield project)_

## Impact

- **New codebase**: Rust crate (`game-core`), Vite/Vue frontend (`view-layer`), WebWorker entry point
- **Dependencies**: bevy_ecs, wasm-bindgen, serde, Vue 3, PixiJS 8, vite-plugin-wasm
- **Build pipeline**: Rust → WASM compilation step integrated into Vite dev/build
- **No existing APIs affected** — this is the initial application scaffold
