## ADDED Requirements

### Requirement: Content pack delivery to WASM
The worker bridge SHALL load the base content pack YAML (or a pre-parsed JSON equivalent) and pass it to the WASM `Game` constructor when the worker initializes.

#### Scenario: Worker init with content
- **WHEN** the WebWorker starts and instantiates the game core
- **THEN** the bundled base content payload is provided before the first `tick` call

#### Scenario: Content load failure blocks game start
- **WHEN** content validation fails during WASM `Game` construction
- **THEN** the worker posts an error to the main thread and does not start the game loop

### Requirement: Build commands use content ids
The worker bridge SHALL forward build commands using string building content ids from the client's loaded content definitions, not hardcoded TypeScript union literals.

#### Scenario: Build event forwarded with id
- **WHEN** the main thread sends a build command for building id `wall`
- **THEN** the worker serializes `IncomingEvent::Build` with `building: "wall"`
