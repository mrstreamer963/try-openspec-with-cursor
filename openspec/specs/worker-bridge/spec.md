# Worker Bridge

## Purpose

WebWorker game loop, postMessage communication between main thread and WASM core, and Vite WASM integration.

## Requirements

### Requirement: Content pack delivery to WASM
The worker bridge SHALL receive the merged content pack JSON from the main thread (produced by fetching and parsing `content/base/*.yaml` at application startup) and pass it to the WASM `Game` constructor when the worker starts. The worker SHALL NOT load content from compile-time bundled imports or an embedded `pack.json` file.

#### Scenario: Worker init with content
- **WHEN** the main thread sends a `start` message with `contentJson` after YAML has been fetched and merged
- **THEN** the worker instantiates `Game` with that payload before the first `tick` call

#### Scenario: Content load failure blocks game start
- **WHEN** content validation fails during WASM `Game` construction
- **THEN** the worker posts an error to the main thread and does not start the game loop

### Requirement: Build commands use content ids
The worker bridge SHALL forward build commands using string building content ids from the client's loaded content definitions, not hardcoded TypeScript union literals.

#### Scenario: Build event forwarded with id
- **WHEN** the main thread sends a build command for building id `wall`
- **THEN** the worker serializes `IncomingEvent::Build` with `building: "wall"`

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
The worker bridge SHALL apply a speed multiplier to the delta time passed to `tick()`, supporting 1×, 5×, and 10× speeds.

#### Scenario: Five times speed
- **WHEN** speed is set to 5×
- **THEN** each tick call passes `dt = 0.25` instead of `0.05`

#### Scenario: Ten times speed
- **WHEN** speed is set to 10×
- **THEN** each tick call passes `dt = 0.50` instead of `0.05`

### Requirement: Load state event routing
The worker bridge SHALL accept `IncomingEvent::LoadState` from the main thread, dispatch it to the WASM game core, and post the resulting snapshot to the main thread.

#### Scenario: Load event from main thread
- **WHEN** the main thread sends a `load_state` event with a validated snapshot
- **THEN** the worker deserializes it as `IncomingEvent::LoadState` and calls `handle_event` on the game core

#### Scenario: Snapshot after load
- **WHEN** load completes successfully in the game core
- **THEN** the worker posts the restored snapshot via `postMessage` with kind `snapshot`

#### Scenario: Load error propagation
- **WHEN** the game core returns an error response for an invalid load
- **THEN** the worker posts an error message to the main thread and does not replace the last successful snapshot on the main thread listener contract (error handlers surface the message)

### Requirement: Worker pause and speed sync on load
The worker bridge SHALL update its local `paused` and `speed` variables to match the loaded snapshot so the game loop applies the correct tick behavior immediately after load.

#### Scenario: Loaded paused state
- **WHEN** a loaded snapshot has `paused: true`
- **THEN** the worker skips `tick` calls until an explicit unpause event, matching pre-load pause behavior

#### Scenario: Loaded speed multiplier
- **WHEN** a loaded snapshot has `speed: 10`
- **THEN** subsequent tick calls pass `dt = BASE_DT * 10` when unpaused

### Requirement: Deconstruct command forwarding
The worker bridge SHALL forward deconstruct commands from the main thread to the WASM game core as `IncomingEvent::Deconstruct { x, y }`.

#### Scenario: Deconstruct event forwarded
- **WHEN** the main thread sends a deconstruct command with tile coordinates `(x, y)`
- **THEN** the worker deserializes and dispatches `IncomingEvent::Deconstruct { x, y }` to the game core

### Requirement: Deconstruction sites in snapshot
The worker bridge SHALL include `deconstruction_sites` from the game core snapshot when posting state updates to the main thread.

#### Scenario: Snapshot includes deconstruction sites
- **WHEN** the game core emits a state snapshot with pending deconstruction sites
- **THEN** the worker posts the snapshot including a `deconstruction_sites` array with x, y, building id, and progress for each site
