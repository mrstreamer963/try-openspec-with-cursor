# View Layer

## Purpose

PixiJS 8 rendering, camera controls, HUD, build toolbar, and colonist info panel.

## Requirements

### Requirement: PixiJS tile rendering
The view layer SHALL render the 50×50 world as colored tiles using PixiJS 8, with distinct colors for Water, Sand, and Grass.

#### Scenario: Tile colors displayed
- **WHEN** a state snapshot is received from the worker
- **THEN** each tile is rendered with its corresponding terrain color on the canvas

### Requirement: Three rendering layers
The view layer SHALL render content in three PixiJS layers: terrain tiles (bottom), building sprites (middle), and entity sprites (top).

#### Scenario: Layer ordering
- **WHEN** a colonist stands on a tile with a building
- **THEN** the colonist sprite renders above the building sprite, which renders above the terrain tile

#### Scenario: Construction site layer
- **WHEN** a state snapshot includes pending construction sites
- **THEN** construction site sprites render on the buildings layer below entity sprites and above terrain tiles

### Requirement: Camera pan and zoom
The view layer SHALL support camera panning via drag and zooming via scroll wheel or pinch.

#### Scenario: Pan camera
- **WHEN** the user drags the mouse across the canvas
- **THEN** the camera offset updates and the world view scrolls accordingly

#### Scenario: Zoom camera
- **WHEN** the user scrolls the mouse wheel
- **THEN** the camera zoom level changes while keeping the cursor position anchored

### Requirement: HUD with pause and speed controls
The view layer SHALL display a HUD with a pause button and speed controls (1×, 5×, 10×).

#### Scenario: Pause simulation
- **WHEN** the user clicks the pause button
- **THEN** an `IncomingEvent::SetPaused(true)` is sent to the worker and the HUD reflects paused state

#### Scenario: Change simulation speed
- **WHEN** the user selects 5× speed
- **THEN** an `IncomingEvent::SetSpeed(5.0)` is sent to the worker

### Requirement: HUD save and load controls
The view layer SHALL display Save and Load buttons in the HUD alongside existing pause and speed controls.

#### Scenario: Save button triggers download
- **WHEN** the user clicks Save in the HUD
- **THEN** the application downloads a version-1 JSON save file for the current colony

#### Scenario: Load button opens file picker
- **WHEN** the user clicks Load in the HUD
- **THEN** a native file picker opens filtered to `.json` files

#### Scenario: Load error feedback
- **WHEN** the selected save file fails validation or the worker reports a load error
- **THEN** the view layer displays an error message to the user without crashing the game view

#### Scenario: Renderer updates after load
- **WHEN** a load succeeds and a new snapshot arrives
- **THEN** the PixiJS scene updates to show the restored world on the next animation frame

### Requirement: Keyboard pause and speed shortcuts
The view layer SHALL respond to keyboard shortcuts for pause and speed while the game is loaded: **Space** toggles pause/resume, **1** sets 1×, **2** sets 5×, **3** sets 10×.

#### Scenario: Toggle pause with Space
- **WHEN** the user presses Space and the game is not loading
- **THEN** pause state toggles and the same events are sent as clicking the HUD pause button

#### Scenario: Set speed with digit keys
- **WHEN** the user presses **2**
- **THEN** simulation speed is set to 5× and the HUD active speed indicator updates

#### Scenario: Speed key while paused
- **WHEN** the user presses **3** while paused
- **THEN** speed is set to 10× (simulation remains paused until Space or resume is triggered)

### Requirement: Build toolbar
The view layer SHALL provide a toolbar with buttons for Wall, Bed, and BerryBush placement modes.

#### Scenario: Select build mode
- **WHEN** the user clicks the Bed button in the toolbar
- **THEN** the cursor enters Bed placement mode and subsequent tile clicks send build commands

#### Scenario: Place building via click
- **WHEN** the user clicks a tile while in Bed placement mode
- **THEN** an `IncomingEvent::Build(Bed, x, y)` is sent to the worker

### Requirement: Construction site rendering
The view layer SHALL render pending construction sites distinctly from completed buildings so the player can see work in progress.

#### Scenario: Ghost sprite for construction site
- **WHEN** a snapshot contains a construction site at a tile
- **THEN** a semi-transparent ghost sprite for the target building type is drawn at that tile

#### Scenario: Progress indication
- **WHEN** a construction site has partial progress
- **THEN** the ghost sprite visually reflects progress (e.g. opacity or fill proportional to completion)

#### Scenario: Completed building replaces ghost
- **WHEN** a construction site completes and a finished building appears in the snapshot
- **THEN** the ghost is removed and the normal building sprite is shown

### Requirement: Float colonist rendering
The view layer SHALL render colonist sprites at their float world position from the snapshot, converting tile units to pixels via `position * TILE_SIZE`.

#### Scenario: Sub-tile sprite position
- **WHEN** a colonist snapshot has position `(5.4, 7.2)`
- **THEN** the colonist sprite center is drawn at pixel coordinates `(5.4 * TILE_SIZE + TILE_SIZE/2, 7.2 * TILE_SIZE + TILE_SIZE/2)`

#### Scenario: Smooth motion between snapshots
- **WHEN** a colonist is moving and `at_task_stand` is `false`
- **THEN** the renderer extrapolates position between 20 Hz snapshots for smooth animation

#### Scenario: Frozen sprite at task stand
- **WHEN** a colonist snapshot has `at_task_stand: true`
- **THEN** the renderer draws the colonist at the snapshot position without extrapolation

### Requirement: Colonist click detection
The view layer SHALL detect colonist clicks by distance from the click point to the colonist sprite center, not by tile coordinate equality.

#### Scenario: Click on moving colonist
- **WHEN** the user clicks within the colonist sprite radius while the colonist is between grid cells
- **THEN** the colonist info panel opens for that colonist

### Requirement: Colonist info panel
The view layer SHALL display an info panel when the user clicks on a colonist, showing the colonist's display name, numeric id, needs, current task, and position. Position SHALL display the colonist's grid cell (floored coordinates). For each need (Food and Sleep), the panel SHALL show the numeric value, a progress bar, and a critical status label when that need is below the simulation threshold.

#### Scenario: Open info panel
- **WHEN** the user clicks on a colonist sprite
- **THEN** a panel appears showing the colonist's name and id (e.g. `Mira (#2)`), Food value with bar, Sleep value with bar, current task name, and grid coordinates

#### Scenario: Critical need status visible
- **WHEN** the user opens the info panel for a colonist whose snapshot has `hungry: true`
- **THEN** the Food row displays a visible "Hungry" status label in addition to the numeric value and bar

#### Scenario: Sleep need status visible
- **WHEN** the user opens the info panel for a colonist whose snapshot has `wants_sleep: true`
- **THEN** the Sleep row displays a visible "Wants sleep" status label in addition to the numeric value and bar

#### Scenario: Satisfied needs show no critical label
- **WHEN** the user opens the info panel for a colonist whose snapshot has `hungry: false` and `wants_sleep: false`
- **THEN** neither need row shows a critical status label

#### Scenario: Close info panel
- **WHEN** the user clicks elsewhere on the canvas
- **THEN** the colonist info panel is dismissed

### Requirement: Colonist name labels
The view layer SHALL render each colonist's display name as a text label positioned above the colonist sprite at all times.

#### Scenario: Name above sprite
- **WHEN** a colonist is visible on the canvas
- **THEN** its display name is drawn centered above the colonist sprite and moves with the sprite

#### Scenario: Name from snapshot
- **WHEN** a state snapshot updates a colonist's position
- **THEN** the name label follows the colonist's rendered position
