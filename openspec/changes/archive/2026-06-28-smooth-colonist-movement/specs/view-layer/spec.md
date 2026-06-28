## ADDED Requirements

### Requirement: Float colonist rendering
The view layer SHALL render colonist sprites at their float world position from the snapshot, converting tile units to pixels via `position * TILE_SIZE`.

#### Scenario: Sub-tile sprite position
- **WHEN** a colonist snapshot has position `(5.4, 7.2)`
- **THEN** the colonist sprite center is drawn at pixel coordinates `(5.4 * TILE_SIZE + TILE_SIZE/2, 7.2 * TILE_SIZE + TILE_SIZE/2)`

## MODIFIED Requirements

### Requirement: Colonist info panel
The view layer SHALL display an info panel when the user clicks on a colonist, showing needs, current task, and position. Position SHALL display the colonist's grid cell (floored coordinates).

#### Scenario: Open info panel
- **WHEN** the user clicks on a colonist sprite
- **THEN** a panel appears showing Food value, Sleep value, current task name, and grid cell coordinates

#### Scenario: Close info panel
- **WHEN** the user clicks elsewhere on the canvas
- **THEN** the colonist info panel is dismissed

### Requirement: Colonist click detection
The view layer SHALL detect colonist clicks by distance from the click point to the colonist sprite center, not by tile coordinate equality.

#### Scenario: Click on moving colonist
- **WHEN** the user clicks within the colonist sprite radius while the colonist is between grid cells
- **THEN** the colonist info panel opens for that colonist
