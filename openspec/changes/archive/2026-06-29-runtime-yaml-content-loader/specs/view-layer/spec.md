## ADDED Requirements

### Requirement: Async content boot gate
The view layer SHALL keep the loading screen visible until the base content pack has been fetched and parsed successfully. Toolbar, renderer, and colonist info components SHALL not initialize with content definitions until the async load completes.

#### Scenario: Loading screen during content fetch
- **WHEN** the application starts and YAML files are being fetched
- **THEN** the loading screen remains visible and the build toolbar is not interactive

#### Scenario: UI ready after content load
- **WHEN** content fetch and parse succeed
- **THEN** the view layer initializes toolbar labels, terrain/building colors, and colonist info labels from the loaded `ContentPack`

#### Scenario: Content load error on loading screen
- **WHEN** content fetch or parse fails during boot
- **THEN** the loading screen displays an error message and the game canvas does not become interactive
