## MODIFIED Requirements

### Requirement: PixiJS tile rendering
The view layer SHALL render the 50×50 world as colored tiles using PixiJS 8, with terrain colors loaded from the active content pack terrain definitions.

#### Scenario: Tile colors displayed
- **WHEN** a state snapshot is received from the worker
- **THEN** each tile is rendered with the color from its terrain content definition

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

### Requirement: Construction site rendering
The view layer SHALL render pending construction sites distinctly from completed buildings. Ghost sprites SHALL use colors from the target building's content definition.

#### Scenario: Ghost sprite for construction site
- **WHEN** a snapshot contains a construction site at a tile
- **THEN** a semi-transparent ghost sprite using the target building's definition color is drawn at that tile

### Requirement: Colonist info panel
The view layer SHALL display an info panel when the user clicks on a colonist, showing the colonist's display name, numeric id, needs (with labels from content definitions), current task, and position. For each need, the panel SHALL show the numeric value, a progress bar scaled to that need's `max`, and a critical status label from the active status definition when applicable.

#### Scenario: Critical need status visible
- **WHEN** the user opens the info panel for a colonist whose snapshot has `hungry: true`
- **THEN** the food row displays the `hungry` status label from YAML in addition to the numeric value and bar

#### Scenario: Sleep need status visible
- **WHEN** the user opens the info panel for a colonist whose snapshot has `wants_sleep: true`
- **THEN** the sleep row displays the `wants_sleep` status label from YAML in addition to the numeric value and bar
