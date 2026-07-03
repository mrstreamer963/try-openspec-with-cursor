## ADDED Requirements

### Requirement: WebWorker game loop
The worker bridge SHALL run the WASM game core in a dedicated WebWorker with a fixed-interval game loop at 50 ms (20 ticks per second).

#### Scenario: Game loop timing
- **WHEN** the worker is initialized and not paused
- **THEN** `tick(0.05)` is called every 50 ms via `setInterval`

#### Scenario: Paused loop
- **WHEN** a pause event is received
- **THEN** the setInterval callback skips tick calls until unpaused

### Requirement: postMessage communication
The worker bridge SHALL communicate between the main thread and WebWorker exclusively via `postMessage` with typed event payloads.

#### Scenario: Main to worker command
- **WHEN** the main thread sends a build command via `worker.postMessage`
- **THEN** the worker deserializes it as an `IncomingEvent` and dispatches it to the game core

#### Scenario: Worker to main state update
- **WHEN** the game core emits an `OutgoingEvent::StateSnapshot` after a tick
- **THEN** the worker posts the serialized snapshot to the main thread via `postMessage`

### Requirement: requestAnimationFrame rendering
The main thread SHALL render the latest state snapshot on each animation frame using `requestAnimationFrame`, independent of the game tick rate.

#### Scenario: Render on new snapshot
- **WHEN** a new state snapshot arrives via postMessage
- **THEN** the view layer updates its PixiJS scene on the next animation frame

#### Scenario: No snapshot change
- **WHEN** no new snapshot arrives between frames
- **THEN** the previous frame is re-rendered without visual changes

### Requirement: Vite WASM integration
The build pipeline SHALL use `vite-plugin-wasm` to load the compiled WASM module in the WebWorker during development and production builds.

#### Scenario: Dev server loads WASM
- **WHEN** `vite dev` is started
- **THEN** the WebWorker successfully instantiates the WASM game core module without manual fetch configuration

#### Scenario: Production build bundles WASM
- **WHEN** `vite build` is executed
- **THEN** the output includes the WASM binary and the worker entry point loads it correctly

### Requirement: Speed multiplier
The worker bridge SHALL apply a speed multiplier to the delta time passed to `tick()`, supporting 1×, 2×, and 3× speeds.

#### Scenario: Double speed
- **WHEN** speed is set to 2×
- **THEN** each tick call passes `dt = 0.10` instead of `0.05`
