## MODIFIED Requirements

### Requirement: Three rendering layers
The view layer SHALL render content in three PixiJS layers: terrain tiles (bottom), building sprites (middle), and entity sprites (top).

#### Scenario: Layer ordering
- **WHEN** a colonist stands on a tile with a building
- **THEN** the colonist sprite renders above the building sprite, which renders above the terrain tile

#### Scenario: Construction site layer
- **WHEN** a state snapshot includes pending construction sites
- **THEN** construction site sprites render on the buildings layer below entity sprites and above terrain tiles

### Requirement: Build toolbar
The view layer SHALL provide a toolbar with buttons for Wall, Bed, and BerryBush placement modes.

#### Scenario: Select build mode
- **WHEN** the user clicks the Bed button in the toolbar
- **THEN** the cursor enters Bed placement mode and subsequent tile clicks send build commands

#### Scenario: Place building via click
- **WHEN** the user clicks a tile while in Bed placement mode
- **THEN** an `IncomingEvent::Build(Bed, x, y)` is sent to the worker

## ADDED Requirements

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
