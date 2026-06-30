## ADDED Requirements

### Requirement: Content mod manifest
The repository SHALL ship a `content/mods.yaml` file listing content mods in load order. Each entry SHALL be a mod id matching a folder under `content/base/` (for id `base`) or `content/mods/<id>/` (for other mods).

#### Scenario: Default manifest loads base only
- **WHEN** `mods.yaml` contains `mods: [base]`
- **THEN** the merged content pack matches the base YAML definitions

#### Scenario: Missing manifest falls back to base
- **WHEN** `mods.yaml` cannot be fetched at startup
- **THEN** the client loads only the `base` mod as if `mods: [base]` were specified

### Requirement: Mod overlay merge by id
The client SHALL load each mod in manifest order, parse optional category YAML files (`needs`, `statuses`, `buildings`, `terrain`), and merge into a single `ContentPack` by replacing entries with the same `id` and appending entries with new ids.

#### Scenario: Override existing building
- **WHEN** a later mod's `buildings.yaml` defines `id: wall` with different `work_required`
- **THEN** the merged pack uses the later mod's full wall definition

#### Scenario: Add new building
- **WHEN** a later mod's `buildings.yaml` defines a new `id` not in base
- **THEN** the merged pack includes that building in addition to base buildings

#### Scenario: Partial mod files
- **WHEN** a mod folder contains only `buildings.yaml` and `mod.yaml`
- **THEN** other categories remain from previously merged mods

### Requirement: Mod metadata validation
Each mod SHALL include `mod.yaml` with `id` and `version`. The client SHALL reject startup when a listed mod folder is missing, `mod.yaml` is missing, or `mod.yaml` `id` does not match the manifest entry.

#### Scenario: Unknown mod in manifest
- **WHEN** `mods.yaml` lists a mod id with no served folder
- **THEN** content loading fails with an error on the loading screen

## MODIFIED Requirements

### Requirement: Runtime YAML loading at startup
The client SHALL load content by reading `content/mods.yaml` (or defaulting to base only), fetching each mod's YAML files over HTTP, merging into a single `ContentPack`, and completing before `Game::new` is called.

#### Scenario: Content fetched before game start
- **WHEN** the application boots
- **THEN** the manifest and all required mod YAML files are fetched, parsed, and merged before `Game::new` is called

#### Scenario: Fetch failure blocks game start
- **WHEN** any required mod YAML file fails to fetch or parse
- **THEN** the application displays an error on the loading screen and does not start the simulation

#### Scenario: Page refresh reloads content
- **WHEN** the user refreshes the browser page after editing a mod YAML file
- **THEN** the updated content definitions are loaded without rebuilding WASM or running a build script

### Requirement: Single source of truth for client and core
The client and WASM game core SHALL use the same merged content pack produced from manifest-ordered YAML files so toolbar labels, colors, and simulation rules stay aligned.

#### Scenario: Toolbar matches registry
- **WHEN** the merged content pack defines buildable buildings from base and mods
- **THEN** the client build toolbar shows exactly those buildings with labels from YAML

#### Scenario: Colors match registry
- **WHEN** the renderer draws terrain and buildings
- **THEN** colors are taken from the merged content definitions, not hardcoded constants in TypeScript

#### Scenario: Rust tests use same YAML files
- **WHEN** `cargo test` runs content registry tests
- **THEN** the test helper can load merged YAML equivalent to the client merge result
