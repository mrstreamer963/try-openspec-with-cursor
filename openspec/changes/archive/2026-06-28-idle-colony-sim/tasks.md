## 1. Project Scaffolding

- [x] 1.1 Initialize monorepo: root `Cargo.toml` workspace, root `package.json` with npm workspaces, `packages/game-core/` and `packages/client/` directories
- [x] 1.2 Scaffold Rust crate in `packages/game-core/` with bevy_ecs, serde, serde_json, wasm-bindgen dependencies
- [x] 1.3 Scaffold Vite + Vue 3 + TypeScript project in `packages/client/` with PixiJS 8 and vite-plugin-wasm
- [x] 1.4 Configure WASM build script: `cargo build --target wasm32-unknown-unknown` + wasm-bindgen glue generation
- [x] 1.5 Verify dev workflow: `vite dev` loads WASM module in WebWorker without errors

## 2. Game Core — ECS Foundation

- [x] 2.1 Define core components: `Position`, `TerrainType`, `BuildingType`, `Needs`, `Task`, `ColonistId`
- [x] 2.2 Define `IncomingEvent` and `OutgoingEvent` enums with serde derives and wasm-bindgen exports
- [x] 2.3 Implement `Game` struct wrapping bevy_ecs `World` with `new()`, `handle_event()`, and `tick(dt)` methods
- [x] 2.4 Implement event dispatch: route `IncomingEvent` variants (SetPaused, SetSpeed, Build) to handler functions
- [x] 2.5 Implement state snapshot serialization: collect world state into `OutgoingEvent::StateSnapshot` after each tick

## 3. Game Core — World Simulation

- [x] 3.1 Implement 50×50 world generation with Water, Sand, Grass terrain assignment (simple noise or pattern)
- [x] 3.2 Store terrain as ECS entities or a flat grid resource with walkability lookup
- [x] 3.3 Implement building placement: validate walkable tile, create building entity with `BuildingType` component
- [x] 3.4 Implement building interaction logic: BerryBush restores Food, Bed restores Sleep

## 4. Game Core — Colonist Simulation

- [x] 4.1 Spawn 3 colonists at valid walkable positions on game init
- [x] 4.2 Implement needs decay system: decrease Food and Sleep values each tick at configurable rate
- [x] 4.3 Implement auto task assignment: when need drops below threshold, assign Eat/Sleep task to nearest building
- [x] 4.4 Implement A* pathfinding on 50×50 grid respecting terrain walkability
- [x] 4.5 Implement colonist movement: advance along path each tick, arrive at destination
- [x] 4.6 Implement task execution: on arrival, restore need, clear task, return colonist to idle

## 5. Worker Bridge

- [x] 5.1 Create WebWorker entry point that instantiates WASM `Game` module
- [x] 5.2 Implement 50 ms game loop with `setInterval`, calling `tick(0.05)` and posting state snapshots
- [x] 5.3 Implement pause/resume: skip tick calls when paused flag is set
- [x] 5.4 Implement speed multiplier: scale dt by 1×/2×/3× based on `IncomingEvent::SetSpeed`
- [x] 5.5 Create main-thread worker manager: spawn worker, forward events, receive snapshots via postMessage

## 6. View Layer — PixiJS Rendering

- [x] 6.1 Initialize PixiJS Application with three Container layers (terrain, buildings, entities)
- [x] 6.2 Render 50×50 terrain tiles from state snapshot with distinct colors per terrain type
- [x] 6.3 Render building sprites at their grid positions on the buildings layer
- [x] 6.4 Render colonist sprites (colored circles) on the entities layer
- [x] 6.5 Implement camera pan (mouse drag) and zoom (scroll wheel)
- [x] 6.6 Wire requestAnimationFrame render loop: update PixiJS scene when new snapshot arrives

## 7. View Layer — UI Controls

- [x] 7.1 Build HUD overlay with pause button and speed selector (1×, 2×, 3×)
- [x] 7.2 Build toolbar with Wall, Bed, BerryBush placement mode buttons
- [x] 7.3 Implement tile click handler: in build mode, send `IncomingEvent::Build` to worker
- [x] 7.4 Implement colonist click handler: show info panel with Food, Sleep, task, and position
- [x] 7.5 Implement info panel dismiss on click elsewhere

## 8. Integration & Polish

- [x] 8.1 End-to-end test: start game, watch colonists auto-satisfy needs, place buildings, verify rendering
- [x] 8.2 Add loading screen in Vue while WASM module initializes in worker
- [x] 8.3 Run `wasm-opt` on release build to minimize WASM binary size
- [x] 8.4 Verify production build (`vite build`) bundles and runs correctly
