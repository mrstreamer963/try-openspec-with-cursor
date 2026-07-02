## MODIFIED Requirements

### Requirement: Async content boot gate
The view layer SHALL show a main menu on application start. Content loading and the loading screen SHALL run only when the user starts a new game, continues from autosave, or loads a save. The build toolbar, renderer, and colonist info components SHALL not initialize until the async content load and atlas load for the selected session completes.

#### Scenario: Main menu before content fetch
- **WHEN** the application starts
- **THEN** the main menu is visible and no content YAML fetch or WASM worker has started

#### Scenario: Loading screen during content fetch
- **WHEN** the user starts a game session and mod YAML files are being loaded
- **THEN** the loading screen remains visible and the build toolbar is not interactive

#### Scenario: Loading screen during atlas fetch
- **WHEN** content load succeeds and atlas PNGs are being loaded
- **THEN** the loading screen remains visible until atlas loading completes or fails with a parse error

#### Scenario: UI ready after content load
- **WHEN** content fetch, parse, merge, and atlas load succeed for a session
- **THEN** the view layer initializes toolbar labels, terrain/building colors, sprite resolver, and colonist info labels from the merged `ContentPack`

#### Scenario: Content load error on loading screen
- **WHEN** content fetch, parse, or merge fails during session start
- **THEN** the loading screen displays an error message and the user can return to the main menu

### Requirement: PixiJS tile rendering
The view layer SHALL render the 50×50 world using PixiJS 8. Terrain tiles SHALL be drawn as sprites when a `sprite` ref resolves, otherwise as colored rectangles using the terrain definition's `color` field.

#### Scenario: Terrain sprites displayed
- **WHEN** a state snapshot is received and terrain definitions include valid `sprite` refs with loaded atlases
- **THEN** each tile is rendered as a sprite from its terrain content definition

#### Scenario: Terrain color fallback displayed
- **WHEN** a terrain definition has no `sprite` ref or the sprite cannot be resolved
- **THEN** the tile is rendered with the color from its terrain content definition

### Requirement: Construction site rendering
The view layer SHALL render pending construction sites distinctly from completed buildings. Ghost sprites SHALL use the target building's sprite when resolvable, otherwise the target building's definition `color`.

#### Scenario: Ghost sprite for construction site
- **WHEN** a snapshot contains a construction site at a tile and the target building sprite resolves
- **THEN** a semi-transparent ghost sprite is drawn at that tile

#### Scenario: Ghost color fallback for construction site
- **WHEN** a snapshot contains a construction site and the target building sprite cannot be resolved
- **THEN** a semi-transparent ghost rectangle using the target building's definition color is drawn at that tile

#### Scenario: Progress indication
- **WHEN** a construction site has partial progress
- **THEN** the ghost sprite visually reflects progress (e.g. opacity or fill proportional to completion)

#### Scenario: Completed building replaces ghost
- **WHEN** a construction site completes and a finished building appears in the snapshot
- **THEN** the ghost is removed and the normal building sprite is shown

### Requirement: Float colonist rendering
The view layer SHALL render colonist sprites at their float world position from the snapshot, converting tile units to pixels via `position * TILE_SIZE`. Colonists SHALL use a static sprite from the `colonist` entity definition when resolvable, otherwise the existing colored circle fallback.

#### Scenario: Sub-tile sprite position
- **WHEN** a colonist snapshot has position `(5.4, 7.2)`
- **THEN** the colonist sprite center is drawn at pixel coordinates `(5.4 * TILE_SIZE + TILE_SIZE/2, 7.2 * TILE_SIZE + TILE_SIZE/2)`

#### Scenario: Colonist tile sprite
- **WHEN** the colonist entity definition has a valid `sprite` ref and atlas frame
- **THEN** each colonist is drawn as a static sprite centered on its position

#### Scenario: Colonist color fallback
- **WHEN** the colonist entity sprite cannot be resolved
- **THEN** the renderer draws the existing colored circle at the colonist position

#### Scenario: Smooth motion between snapshots
- **WHEN** a colonist is moving and `at_task_stand` is `false`
- **THEN** the renderer extrapolates position between 20 Hz snapshots for smooth animation

#### Scenario: Frozen sprite at task stand
- **WHEN** a colonist snapshot has `at_task_stand: true`
- **THEN** the renderer draws the colonist at the snapshot position without extrapolation

### Requirement: Deconstruction site rendering
The view layer SHALL render pending deconstruction sites distinctly from construction sites. Overlays SHALL use red coloring to distinguish from green construction ghosts. Building sprites underneath SHALL use sprite rendering when resolvable, with color fallback otherwise.

#### Scenario: Red overlay for deconstruction site
- **WHEN** a snapshot contains a deconstruction site at a tile
- **THEN** a semi-transparent red overlay is drawn at that tile with alpha scaling with progress

#### Scenario: Deconstruction progress bar
- **WHEN** a deconstruction site has partial progress
- **THEN** a red progress bar is drawn above the tile

#### Scenario: Building visible until complete
- **WHEN** a deconstruction site targets a finished building
- **THEN** the building sprite (or color fallback) remains visible underneath the red overlay until deconstruction completes

#### Scenario: Completed deconstruction removes overlay
- **WHEN** a deconstruction site completes and the building is removed from the snapshot
- **THEN** the red overlay and progress bar are removed and the tile shows only terrain (or underlying content)
