## ADDED Requirements

### Requirement: YAML schema for entities
Each entity definition in YAML SHALL specify at minimum: `id` and display `color`. An optional `sprite` field MAY reference an atlas frame as `{ atlas, frame }`. Entity definitions are view-only and SHALL NOT be passed to the WASM simulation.

#### Scenario: Colonist entity in base pack
- **WHEN** the base content pack is loaded
- **THEN** `entities.yaml` defines a `colonist` entity with `color` and optional `sprite`

#### Scenario: Entity sprite ref format
- **WHEN** an entity definition includes `sprite: { atlas: kenney-roguelike, frame: 736 }`
- **THEN** the client parses it as a valid `SpriteRef` without affecting simulation initialization

### Requirement: Atlas configuration file
The repository SHALL ship `content/base/atlases.yaml` listing CC0 atlas metadata for client-side sprite loading. Atlas PNG files SHALL live under `content/assets/`. Atlases are base-only in v1; mods SHALL NOT ship custom atlas PNGs but MAY reference base atlas frames in content defs.

#### Scenario: Base atlases present
- **WHEN** the repository is checked out
- **THEN** `content/base/atlases.yaml` and at least one atlas PNG exist under `content/assets/`

#### Scenario: Mod references base atlas frame
- **WHEN** a mod's `terrain.yaml` overrides `grass` with `sprite: { atlas: kenney-roguelike, frame: 42 }`
- **THEN** the merged pack uses that sprite ref without requiring a mod atlas PNG

### Requirement: Optional sprite field on terrain and buildings
Terrain and building definitions MAY include an optional `sprite: { atlas, frame }` field for client rendering. The `color` field SHALL remain required as the fallback display value.

#### Scenario: Terrain with sprite ref
- **WHEN** terrain `grass` includes both `color` and `sprite` fields
- **THEN** the merged pack retains both; the renderer prefers sprite with color fallback

#### Scenario: Building with sprite ref
- **WHEN** building `wall` includes a `sprite` field alongside existing simulation fields
- **THEN** simulation behavior is unchanged and the client uses the sprite for rendering

## MODIFIED Requirements

### Requirement: Mod overlay merge by id
The client SHALL load each mod in manifest order, parse optional category YAML files (`needs`, `statuses`, `buildings`, `terrain`, `entities`), and merge into a single `ContentPack` by replacing entries with the same `id` and appending entries with new ids.

#### Scenario: Override existing building
- **WHEN** a later mod's `buildings.yaml` defines `id: wall` with different `work_required`
- **THEN** the merged pack uses the later mod's full wall definition

#### Scenario: Add new building
- **WHEN** a later mod's `buildings.yaml` defines a new `id` not in base
- **THEN** the merged pack includes that building in addition to base buildings

#### Scenario: Partial mod files
- **WHEN** a mod folder contains only `buildings.yaml` and `mod.yaml`
- **THEN** other categories remain from previously merged mods

#### Scenario: Override entity definition
- **WHEN** a later mod's `entities.yaml` defines `id: colonist` with a different `sprite` ref
- **THEN** the merged pack uses the later mod's colonist entity definition

### Requirement: Single source of truth for client and core
The client and WASM game core SHALL use the same merged content pack produced from manifest-ordered YAML files so toolbar labels, colors, and simulation rules stay aligned. View-only fields (`sprite` on terrain, buildings, and entities) SHALL be stripped from the JSON payload sent to WASM.

#### Scenario: Toolbar matches registry
- **WHEN** the merged content pack defines buildable buildings from base and mods
- **THEN** the client build toolbar shows exactly those buildings with labels from YAML

#### Scenario: Colors match registry
- **WHEN** the renderer draws terrain and buildings without resolvable sprites
- **THEN** colors are taken from the merged content definitions, not hardcoded constants in TypeScript

#### Scenario: WASM JSON excludes sprite fields
- **WHEN** the client bakes content for `Game::new`
- **THEN** the JSON payload omits `sprite` fields and `entities` category entries

#### Scenario: Rust tests use same YAML files
- **WHEN** `cargo test` runs content registry tests
- **THEN** the test helper can load merged YAML equivalent to the client merge result

### Requirement: YAML schema for buildings
Each building definition in YAML SHALL specify at minimum: `id`, `label`, `work_required`, `work_to_deconstruct`, `blocks_movement`, `blocks_settle`, display `color`, and `on_complete` / `interactions` primitives sufficient to express v1 wall, bed, and berry bush behavior. An optional `sprite` field MAY reference an atlas frame.

#### Scenario: Wall blocks movement from definition
- **WHEN** a finished `wall` building occupies a tile
- **THEN** that tile is impassable per `blocks_movement: true` in the building definition

#### Scenario: Berry bush supply from definition
- **WHEN** a `berry_bush` construction order completes
- **THEN** the finished building spawns with berry supply equal to the amount specified in the building definition's `on_complete` supply primitive

### Requirement: YAML schema for terrain
Each terrain definition in YAML SHALL specify at minimum: `id`, `walkable`, and display `color`. An optional `sprite` field MAY reference an atlas frame.

#### Scenario: Water impassable from definition
- **WHEN** a tile has terrain `water`
- **THEN** pathfinding treats that tile as impassable per `walkable: false` in the terrain definition
