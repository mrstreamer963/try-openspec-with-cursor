## ADDED Requirements

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
