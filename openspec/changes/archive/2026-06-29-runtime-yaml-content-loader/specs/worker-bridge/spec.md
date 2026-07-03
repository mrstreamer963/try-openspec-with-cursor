## MODIFIED Requirements

### Requirement: Content pack delivery to WASM
The worker bridge SHALL receive the merged content pack JSON from the main thread (produced by fetching and parsing `content/base/*.yaml` at application startup) and pass it to the WASM `Game` constructor when the worker starts. The worker SHALL NOT load content from compile-time bundled imports or an embedded `pack.json` file.

#### Scenario: Worker init with content
- **WHEN** the main thread sends a `start` message with `contentJson` after YAML has been fetched and merged
- **THEN** the worker instantiates `Game` with that payload before the first `tick` call

#### Scenario: Content load failure blocks game start
- **WHEN** content validation fails during WASM `Game` construction
- **THEN** the worker posts an error to the main thread and does not start the game loop
