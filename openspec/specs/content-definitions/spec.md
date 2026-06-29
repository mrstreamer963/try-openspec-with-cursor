# Content Definitions

## Purpose

YAML-driven content packs for terrain, buildings, needs, and statuses; loaded once at startup and baked into a fast runtime registry for modding.

## Requirements

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

### Requirement: Base content pack in YAML
The repository SHALL ship a base content pack as YAML files defining all v1 simulation content: terrain types, buildings, needs, and statuses that reproduce current gameplay (water/sand/grass; wall/bed/berry_bush; food/sleep; hungry/wants_sleep). The repository SHALL NOT ship a duplicate merged JSON file (e.g. `pack.json`) as an authoritative content source.

#### Scenario: Base pack loads without errors
- **WHEN** the game starts with the bundled base content pack
- **THEN** all YAML files parse successfully and validation passes with zero errors

#### Scenario: Base pack reproduces v1 building set
- **WHEN** the base content pack is loaded
- **THEN** exactly three buildable buildings are available: `wall`, `bed`, and `berry_bush`

### Requirement: Content registry bake phase
The game core SHALL parse YAML at initialization, validate cross-references, intern string IDs to dense numeric indices, and store definitions in a `ContentRegistry` resource with O(1) indexed access for simulation hot paths.

#### Scenario: Registry available before first tick
- **WHEN** `Game` is constructed with a valid content payload
- **THEN** `ContentRegistry` is inserted into the ECS world before colonists spawn and before the first `tick` call

#### Scenario: Invalid content rejected at init
- **WHEN** content YAML references an unknown need, status, or building ID
- **THEN** initialization fails with a descriptive error and no simulation starts

#### Scenario: No string lookups in pathfinding hot loop
- **WHEN** A* pathfinding evaluates tile walkability or building blocking
- **THEN** it uses baked numeric indices and `Vec`-backed definition tables, not `HashMap<String, _>` lookups

### Requirement: YAML schema for needs
Each need definition in YAML SHALL specify at minimum: `id`, `label`, `max`, `decay_per_sec`, and `critical_threshold`.

#### Scenario: Need decay from definition
- **WHEN** a colonist has a need defined in the content pack
- **THEN** that need's value decreases each tick by `decay_per_sec * dt` from the YAML definition

### Requirement: YAML schema for statuses
Each status definition in YAML SHALL specify at minimum: `id`, `label`, an `apply_when` condition, and zero or more `effects` used by task assignment.

#### Scenario: Status applied from need threshold
- **WHEN** a colonist's `food` need value drops below the `critical_threshold` defined for need `food`
- **THEN** the `hungry` status (or equivalent base-pack status tied to food) becomes active on that colonist

#### Scenario: Status removed when condition clears
- **WHEN** a colonist's need rises above the critical threshold
- **THEN** the corresponding status is removed on the next status sync pass

### Requirement: YAML schema for buildings
Each building definition in YAML SHALL specify at minimum: `id`, `label`, `work_required`, `work_to_deconstruct`, `blocks_movement`, `blocks_settle`, display `color`, and `on_complete` / `interactions` primitives sufficient to express v1 wall, bed, and berry bush behavior.

#### Scenario: Wall blocks movement from definition
- **WHEN** a finished `wall` building occupies a tile
- **THEN** that tile is impassable per `blocks_movement: true` in the building definition

#### Scenario: Berry bush supply from definition
- **WHEN** a `berry_bush` construction order completes
- **THEN** the finished building spawns with berry supply equal to the amount specified in the building definition's `on_complete` supply primitive

### Requirement: YAML schema for terrain
Each terrain definition in YAML SHALL specify at minimum: `id`, `walkable`, and display `color`.

#### Scenario: Water impassable from definition
- **WHEN** a tile has terrain `water`
- **THEN** pathfinding treats that tile as impassable per `walkable: false` in the terrain definition

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

### Requirement: Content ID wire format
All content references in the event protocol and state snapshots SHALL use stable snake_case string IDs from YAML (e.g. `berry_bush`, `wants_sleep`), not PascalCase enum variant names.

#### Scenario: Build event uses content ID
- **WHEN** the host sends a build command for a berry bush
- **THEN** the payload uses building id `"berry_bush"`

#### Scenario: Snapshot uses content IDs
- **WHEN** a state snapshot includes a building entry
- **THEN** the `building` field is the string content id from YAML (e.g. `"bed"`)

### Requirement: work_to_deconstruct field
Each buildable building definition in YAML SHALL include a `work_to_deconstruct` field specifying the labor units required to remove that building. The game core SHALL expose this via `ContentRegistry::work_to_deconstruct(id: BuildingId) -> f32`.

#### Scenario: Base pack deconstruct values
- **WHEN** the base content pack is loaded
- **THEN** `wall` has `work_to_deconstruct: 15`, `bed` has `work_to_deconstruct: 25`, and `berry_bush` has `work_to_deconstruct: 20`

#### Scenario: Deconstruct work from registry
- **WHEN** a deconstruction site is created for a finished `bed`
- **THEN** its initial `work_remaining` equals the `work_to_deconstruct` value from the bed definition in YAML
