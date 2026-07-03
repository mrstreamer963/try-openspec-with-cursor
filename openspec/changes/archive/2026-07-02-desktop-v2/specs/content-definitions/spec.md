## MODIFIED Requirements

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

## ADDED Requirements

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
