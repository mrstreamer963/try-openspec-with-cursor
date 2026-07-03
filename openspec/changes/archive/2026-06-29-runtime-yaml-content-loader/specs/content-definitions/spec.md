## ADDED Requirements

### Requirement: Runtime YAML loading at startup
The client SHALL load the base content pack by fetching `content/base/*.yaml` files over HTTP at application startup, parsing them with a YAML parser, and merging into a single `ContentPack` object before the WASM game is instantiated.

#### Scenario: Content fetched before game start
- **WHEN** the application boots
- **THEN** all base YAML files (`needs.yaml`, `statuses.yaml`, `buildings.yaml`, `terrain.yaml`) are fetched, parsed, and merged before `Game::new` is called

#### Scenario: Fetch failure blocks game start
- **WHEN** any required YAML file fails to fetch or parse
- **THEN** the application displays an error on the loading screen and does not start the simulation

#### Scenario: Page refresh reloads content
- **WHEN** the user refreshes the browser page after editing a YAML file
- **THEN** the updated content definitions are loaded without rebuilding WASM or running a build script

## MODIFIED Requirements

### Requirement: Base content pack in YAML
The repository SHALL ship a base content pack as YAML files defining all v1 simulation content: terrain types, buildings, needs, and statuses that reproduce current gameplay (water/sand/grass; wall/bed/berry_bush; food/sleep; hungry/wants_sleep). The repository SHALL NOT ship a duplicate merged JSON file (e.g. `pack.json`) as an authoritative content source.

#### Scenario: Base pack loads without errors
- **WHEN** the game starts with the bundled base content pack
- **THEN** all YAML files parse successfully and validation passes with zero errors

#### Scenario: Base pack reproduces v1 building set
- **WHEN** the base content pack is loaded
- **THEN** exactly three buildable buildings are available: `wall`, `bed`, and `berry_bush`

### Requirement: Single source of truth for client and core
The client and WASM game core SHALL load the same base content pack from the YAML files in `content/base/` so toolbar labels, colors, and simulation rules stay aligned. No separate embedded JSON artifact (e.g. `pack.json`) SHALL exist as a parallel content source.

#### Scenario: Toolbar matches registry
- **WHEN** the base content pack defines three buildable buildings
- **THEN** the client build toolbar shows exactly those three buildings with labels from YAML

#### Scenario: Colors match registry
- **WHEN** the renderer draws terrain and buildings
- **THEN** colors are taken from the loaded content definitions, not hardcoded constants in TypeScript

#### Scenario: Rust tests use same YAML files
- **WHEN** `cargo test` runs content registry tests
- **THEN** the test helper reads the same `content/base/*.yaml` files as the client, not an embedded `pack.json`
