## MODIFIED Requirements

### Requirement: Camera pan and zoom
The view layer SHALL support camera panning via right mouse button drag, keyboard (W/A/S/D and arrow keys), and zooming via scroll wheel or pinch.

#### Scenario: Pan camera with right mouse drag
- **WHEN** the user drags with the right mouse button held on the canvas
- **THEN** the camera offset updates and the world view scrolls accordingly

#### Scenario: Pan camera with keyboard
- **WHEN** the user holds **W**, **A**, **S**, or **D** (or the corresponding arrow key) while the game session is active
- **THEN** the camera offset updates continuously in that direction until the key is released

#### Scenario: Diagonal keyboard pan
- **WHEN** the user holds two perpendicular direction keys simultaneously (e.g. **W** and **D**)
- **THEN** the camera pans diagonally at the combined direction

#### Scenario: Zoom camera
- **WHEN** the user scrolls the mouse wheel
- **THEN** the camera zoom level changes while keeping the cursor position anchored

#### Scenario: Context menu suppressed on canvas
- **WHEN** the user presses the right mouse button on the game canvas
- **THEN** the browser context menu does not appear

### Requirement: Build toolbar
The view layer SHALL provide a build toolbar with one button per building marked `buildable: true` in the loaded content pack, using each building's `label` (and optional icon) from YAML.

#### Scenario: Select build mode
- **WHEN** the user clicks the bed button in the toolbar
- **THEN** the cursor enters `bed` placement mode and subsequent tile clicks send build commands with building id `bed`

#### Scenario: Place building via click
- **WHEN** the user clicks a tile while in `bed` placement mode
- **THEN** an `IncomingEvent::Build` with building id `bed` and tile coordinates is sent to the worker

#### Scenario: Toolbar reflects content pack
- **WHEN** the base content pack is loaded
- **THEN** the toolbar shows Wall, Bed, and Berry Bush buttons with labels from YAML definitions

### Requirement: Deconstruct click handling
When deconstruct mode is active, tile clicks SHALL send `IncomingEvent::Deconstruct` with the clicked tile coordinates to the worker.

#### Scenario: Deconstruct via click
- **WHEN** the user clicks a tile while in deconstruct mode
- **THEN** an `IncomingEvent::Deconstruct` with tile coordinates is sent to the worker

## ADDED Requirements

### Requirement: Wall line drag placement
When wall placement mode is active, left mouse button drag SHALL place construction orders along a horizontal or vertical line from the drag start tile to the drag end tile. The line SHALL snap to the dominant axis (compare absolute tile delta X vs delta Y; equal deltas snap horizontal). Diagonal lines SHALL NOT be supported.

#### Scenario: Horizontal wall line
- **WHEN** the user drags left mouse button from tile (2, 5) to (7, 5) while in wall mode
- **THEN** build commands for `wall` are sent for tiles (2,5) through (7,5) inclusive

#### Scenario: Vertical wall line
- **WHEN** the user drags left mouse button from tile (3, 1) to (3, 6) while in wall mode
- **THEN** build commands for `wall` are sent for tiles (3,1) through (3,6) inclusive

#### Scenario: Single wall tap
- **WHEN** the user taps a tile without meaningful drag while in wall mode
- **THEN** one build command for `wall` is sent for that tile

#### Scenario: Skip invalid tiles on wall line
- **WHEN** a wall line drag includes tiles that are not valid build targets (e.g. water or occupied)
- **THEN** those tiles are skipped and valid tiles on the line still receive build commands

#### Scenario: Wall line preview
- **WHEN** the user is dragging a wall line
- **THEN** a semi-transparent preview is shown on affected tiles until the drag ends

### Requirement: Deconstruct rectangle drag
When deconstruct mode is active, left mouse button drag SHALL send deconstruct commands for all deconstructible tiles in the axis-aligned rectangle from drag start to drag end tile (inclusive on both corners).

#### Scenario: Rectangle deconstruct
- **WHEN** the user drags left mouse button from tile (1, 2) to (4, 5) while in deconstruct mode
- **THEN** deconstruct commands are sent for each deconstructible tile in the inclusive rectangle from (1,2) to (4,5)

#### Scenario: Single tile deconstruct tap
- **WHEN** the user taps a tile without meaningful drag while in deconstruct mode
- **THEN** one deconstruct command is sent for that tile if deconstructible

#### Scenario: Skip invalid tiles in rectangle
- **WHEN** a deconstruct rectangle includes tiles with nothing to deconstruct
- **THEN** those tiles are skipped and deconstructible tiles still receive commands

#### Scenario: Deconstruct rectangle preview
- **WHEN** the user is dragging a deconstruct rectangle
- **THEN** a semi-transparent red preview is shown on affected tiles until the drag ends

### Requirement: Build mode suppresses colonist selection
When any build placement mode or deconstruct mode is active, left mouse button press on the canvas SHALL NOT open the colonist info panel. Tile actions for the active tool take priority.

#### Scenario: Wall mode ignores colonist hit
- **WHEN** wall mode is active and the user clicks on a colonist sprite
- **THEN** a wall build command is sent for the tile under the click and the colonist info panel does not open
