## ADDED Requirements

### Requirement: Deconstruct toolbar
The view layer SHALL provide a **Deconstruct** toolbar button that enters deconstruct placement mode. Deconstruct mode SHALL be mutually exclusive with Select mode and all build-type placement modes.

#### Scenario: Enter deconstruct mode
- **WHEN** the user clicks the Deconstruct button in the toolbar
- **THEN** the cursor enters deconstruct mode and subsequent tile clicks send deconstruct commands instead of build commands

#### Scenario: Deconstruct mode exclusive with build
- **WHEN** the user clicks a build-type button while deconstruct mode is active
- **THEN** deconstruct mode is deactivated and the selected build mode becomes active

#### Scenario: Deconstruct mode exclusive with select
- **WHEN** the user clicks Select while deconstruct mode is active
- **THEN** deconstruct mode is deactivated and select mode becomes active

### Requirement: Deconstruct click handling
When deconstruct mode is active, tile clicks SHALL send `IncomingEvent::Deconstruct` with the clicked tile coordinates to the worker.

#### Scenario: Deconstruct via click
- **WHEN** the user clicks a tile while in deconstruct mode
- **THEN** an `IncomingEvent::Deconstruct` with tile coordinates is sent to the worker

### Requirement: Deconstruction site rendering
The view layer SHALL render pending deconstruction sites distinctly from construction sites. Overlays SHALL use red coloring to distinguish from green construction ghosts.

#### Scenario: Red overlay for deconstruction site
- **WHEN** a snapshot contains a deconstruction site at a tile
- **THEN** a semi-transparent red overlay is drawn at that tile with alpha scaling with progress

#### Scenario: Deconstruction progress bar
- **WHEN** a deconstruction site has partial progress
- **THEN** a red progress bar is drawn above the tile

#### Scenario: Building visible until complete
- **WHEN** a deconstruction site targets a finished building
- **THEN** the building sprite remains visible underneath the red overlay until deconstruction completes

#### Scenario: Completed deconstruction removes overlay
- **WHEN** a deconstruction site completes and the building is removed from the snapshot
- **THEN** the red overlay and progress bar are removed and the tile shows only terrain (or underlying content)
