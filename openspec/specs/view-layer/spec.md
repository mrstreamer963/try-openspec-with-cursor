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

### Requirement: Camera pan and zoom
The view layer SHALL support camera panning via drag and zooming via scroll wheel or pinch.

#### Scenario: Pan camera
- **WHEN** the user drags the mouse across the canvas
- **THEN** the camera offset updates and the world view scrolls accordingly

#### Scenario: Zoom camera
- **WHEN** the user scrolls the mouse wheel
- **THEN** the camera zoom level changes while keeping the cursor position anchored

### Requirement: HUD with pause and speed controls
The view layer SHALL display a HUD with a pause button and speed controls (1×, 2×, 3×).

#### Scenario: Pause simulation
- **WHEN** the user clicks the pause button
- **THEN** an `IncomingEvent::SetPaused(true)` is sent to the worker and the HUD reflects paused state

#### Scenario: Change simulation speed
- **WHEN** the user selects 2× speed
- **THEN** an `IncomingEvent::SetSpeed(2.0)` is sent to the worker

### Requirement: Build toolbar
The view layer SHALL provide a toolbar with buttons for Wall, Bed, and BerryBush placement modes.

#### Scenario: Select build mode
- **WHEN** the user clicks the Bed button in the toolbar
- **THEN** the cursor enters Bed placement mode and subsequent tile clicks send build commands

#### Scenario: Place building via click
- **WHEN** the user clicks a tile while in Bed placement mode
- **THEN** an `IncomingEvent::Build(Bed, x, y)` is sent to the worker

### Requirement: Colonist info panel
The view layer SHALL display an info panel when the user clicks on a colonist, showing needs, current task, and position.

#### Scenario: Open info panel
- **WHEN** the user clicks on a colonist sprite
- **THEN** a panel appears showing Food value, Sleep value, current task name, and grid coordinates

#### Scenario: Close info panel
- **WHEN** the user clicks elsewhere on the canvas
- **THEN** the colonist info panel is dismissed
