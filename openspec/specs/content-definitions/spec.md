# Content Definitions

## Purpose

YAML-driven content packs for terrain, buildings, needs, and statuses; loaded once at startup and baked into a fast runtime registry for modding.

## Requirements

### Requirement: Runtime YAML loading at startup
The client SHALL load content by resolving the enabled mod list from `settings.json` (desktop) or web settings storage, fetching bundled mod YAML via HTTP and user mod YAML via filesystem when on desktop, merging into a single `ContentPack`, and completing before `Game::new` is called.

#### Scenario: Content fetched before game start
- **WHEN** the user starts a game session from the main menu
- **THEN** enabled mod YAML files are fetched or read, parsed, and merged before `Game::new` is called

#### Scenario: Fetch failure blocks game start
- **WHEN** any required mod YAML file fails to fetch or parse
- **THEN** the application displays an error on the loading screen and does not start the simulation

#### Scenario: Page refresh reloads content
- **WHEN** the user refreshes the browser page after editing a bundled mod YAML file in dev
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

### Requirement: Content mod manifest
The repository SHALL ship a `content/mods.yaml` file listing bundled optional content mods (excluding `base`). Runtime mod activation SHALL be controlled by `settings.enabled_mods`. Each mod id SHALL match a folder under `content/base/` (for id `base`) or `content/mods/<id>/` (for bundled mods) or `app_data/mods/<id>/` (for user mods on desktop).

#### Scenario: Default settings loads base only
- **WHEN** `settings.enabled_mods` is `["base"]`
- **THEN** the merged content pack matches the base YAML definitions

#### Scenario: Missing settings falls back to base
- **WHEN** settings cannot be read at startup
- **THEN** the client loads only the `base` mod as if `enabled_mods: ["base"]` were specified

#### Scenario: Bundled catalog lists shipped mods
- **WHEN** `content/mods.yaml` lists `hardmode`
- **THEN** the mod picker shows `hardmode` as an available bundled mod even if not enabled

### Requirement: Settings-driven enabled mod list
The application SHALL persist `enabled_mods: string[]` in `settings.json` (desktop) or web settings storage. The list SHALL always include `base` as the first entry. The mod picker SHALL allow toggling optional bundled and user mods.

#### Scenario: Enable hardmode in settings
- **WHEN** the user enables `hardmode` in the mod picker and starts a new game
- **THEN** merged content includes hardmode overrides

#### Scenario: Base mod not toggleable
- **WHEN** the user views the mod picker
- **THEN** `base` is shown as required and cannot be disabled

### Requirement: User mod directory discovery
On desktop, the client SHALL scan `app_data/mods/*/mod.yaml` at startup, validate each mod's `id` matches its folder name, and include discovered mods in the mod picker catalog. User mods with the same `id` as a bundled mod SHALL override the bundled mod's YAML at merge time.

#### Scenario: User mod appears in picker
- **WHEN** a valid `app_data/mods/my-mod/mod.yaml` exists
- **THEN** `my-mod` appears in the mod picker as a user mod

#### Scenario: User mod overrides bundled id
- **WHEN** a user mod and bundled mod share id `hardmode`
- **THEN** the user mod's YAML is loaded from app data and replaces the bundled hardmode entries in the merged pack

#### Scenario: Invalid user mod rejected
- **WHEN** a user mod folder is missing `mod.yaml` or has an id mismatch
- **THEN** that mod is excluded from the catalog and an error is logged; other mods continue loading

### Requirement: Pluggable content sources
Content loading SHALL use a `ContentSource` interface. Bundled mods SHALL use HTTP fetch with `import.meta.env.BASE_URL`. User mods on desktop SHALL use filesystem reads via the platform adapter.

#### Scenario: Bundled mod via fetch
- **WHEN** loading bundled mod `hardmode`
- **THEN** category YAML files are read via HTTP from the bundled content paths

#### Scenario: User mod via filesystem
- **WHEN** loading user mod `my-mod` on desktop
- **THEN** category YAML files are read from `app_data/mods/my-mod/` via the platform adapter

#### Scenario: Web has no user mod source
- **WHEN** the client runs in a browser
- **THEN** only bundled content sources are used and user mod folders are ignored

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

### Requirement: Mod metadata validation
Each mod SHALL include `mod.yaml` with `id` and `version`. The client SHALL reject startup when a listed mod folder is missing, `mod.yaml` is missing, or `mod.yaml` `id` does not match the manifest entry.

#### Scenario: Unknown mod in manifest
- **WHEN** `mods.yaml` lists a mod id with no served folder
- **THEN** content loading fails with an error on the loading screen

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
