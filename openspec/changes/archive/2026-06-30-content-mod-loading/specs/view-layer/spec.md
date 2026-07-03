## MODIFIED Requirements

### Requirement: Async content boot gate
The view layer SHALL keep the loading screen visible until the merged content pack from the mod manifest has been fetched, parsed, and merged successfully. Toolbar, renderer, and colonist info components SHALL not initialize with content definitions until the async load completes.

#### Scenario: Loading screen during content fetch
- **WHEN** the application starts and mod manifest or YAML files are being fetched
- **THEN** the loading screen remains visible and the build toolbar is not interactive

#### Scenario: UI ready after content load
- **WHEN** content fetch, parse, and merge succeed
- **THEN** the view layer initializes toolbar labels, terrain/building colors, and colonist info labels from the merged `ContentPack`

#### Scenario: Content load error on loading screen
- **WHEN** content fetch, parse, or merge fails during boot
- **THEN** the loading screen displays an error message and the game canvas does not become interactive
