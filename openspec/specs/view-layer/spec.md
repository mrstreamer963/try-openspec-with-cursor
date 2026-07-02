# View Layer

## Purpose

PixiJS 8 rendering, camera controls, HUD, build toolbar, and colonist info panel.

## Requirements

### Requirement: Async content boot gate
The view layer SHALL show a main menu on application start. Content loading and the loading screen SHALL run only when the user starts a new game, continues from autosave, or loads a save. The build toolbar, renderer, and colonist info components SHALL not initialize until the async content load for the selected session completes.

#### Scenario: Main menu before content fetch
- **WHEN** the application starts
- **THEN** the main menu is visible and no content YAML fetch or WASM worker has started

#### Scenario: Loading screen during content fetch
- **WHEN** the user starts a game session and mod YAML files are being loaded
- **THEN** the loading screen remains visible and the build toolbar is not interactive

#### Scenario: UI ready after content load
- **WHEN** content fetch, parse, and merge succeed for a session
- **THEN** the view layer initializes toolbar labels, terrain/building colors, and colonist info labels from the merged `ContentPack`

#### Scenario: Content load error on loading screen
- **WHEN** content fetch, parse, or merge fails during session start
- **THEN** the loading screen displays an error message and the user can return to the main menu

### Requirement: PixiJS tile rendering
The view layer SHALL render the 50×50 world as colored tiles using PixiJS 8, with terrain colors loaded from the active content pack terrain definitions.

#### Scenario: Tile colors displayed
- **WHEN** a state snapshot is received from the worker
- **THEN** each tile is rendered with the color from its terrain content definition

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
The view layer SHALL display Save and Load controls in the HUD. Save SHALL write to the last-used manual slot (default slot 1). Load SHALL open a picker listing autosave, slots 1–3, and an option to load from an arbitrary file.

#### Scenario: HUD quick save to slot
- **WHEN** the user clicks Save in the HUD on desktop
- **THEN** the current snapshot is written to the configured quick-save slot in app data

#### Scenario: HUD load opens slot picker
- **WHEN** the user clicks Load in the HUD
- **THEN** a load UI lists autosave and manual slots with timestamps before offering file import

#### Scenario: Load error feedback
- **WHEN** the selected save fails validation or the worker reports a load error
- **THEN** the view layer displays an error message to the user without crashing the game view

#### Scenario: Renderer updates after load
- **WHEN** a load succeeds and a new snapshot arrives
- **THEN** the PixiJS scene updates to show the restored world on the next animation frame

### Requirement: Main menu screen
The view layer SHALL display a main menu on startup with at minimum: **Continue** (when a valid autosave exists), **New Game**, **Load Game**, **Mods**, and **Quit** (desktop) or equivalent web exit behavior.

#### Scenario: Continue from autosave
- **WHEN** a valid `autosave.json` exists and the user clicks Continue
- **THEN** content loads with current `enabled_mods`, the game session starts, and the autosave state is applied via `load_state`

#### Scenario: Continue hidden without autosave
- **WHEN** no valid autosave exists
- **THEN** the Continue button is disabled or hidden

#### Scenario: New game
- **WHEN** the user clicks New Game
- **THEN** a fresh simulation starts with the current `enabled_mods` and no `load_state`

#### Scenario: Return to menu from game
- **WHEN** the user quits to main menu from an active session
- **THEN** the game session is torn down and the main menu is shown

### Requirement: Mod picker screen
The view layer SHALL provide a mod management screen reachable from the main menu and application menu. It SHALL list bundled and user mods with enable/disable toggles (except `base`), show mod source (bundled vs user), and provide **Open Mods Folder** on desktop.

#### Scenario: Toggle mod in picker
- **WHEN** the user enables `hardmode` and applies changes
- **THEN** `settings.enabled_mods` is updated to include `hardmode`

#### Scenario: Mod change requires new session
- **WHEN** the user changes enabled mods from the picker
- **THEN** changes apply to the next game session; an active session is not hot-reloaded

#### Scenario: Open mods folder on desktop
- **WHEN** the user clicks Open Mods Folder on desktop
- **THEN** the OS file manager opens `app_data/mods/`

### Requirement: Native confirm dialogs on desktop
On desktop, confirmation prompts for mod mismatch, quit guard, and destructive actions SHALL use the Tauri dialog plugin instead of `window.confirm`.

#### Scenario: Mod mismatch uses native dialog
- **WHEN** a mod mismatch occurs on desktop
- **THEN** the three-way choice is presented via a native dialog

#### Scenario: Web retains window confirm
- **WHEN** a mod mismatch occurs on web
- **THEN** the browser `window.confirm` or equivalent in-app modal is used

### Requirement: Load game screen
The view layer SHALL provide a load screen listing autosave and manual slots with `saved_at` timestamp and `content_mods` summary, plus an option to import a JSON file from disk.

#### Scenario: Load from slot
- **WHEN** the user selects slot 2 from the load screen
- **THEN** `saves/slot-2.json` is validated and applied

#### Scenario: Import external save file
- **WHEN** the user chooses import and selects a valid external JSON save on desktop
- **THEN** the native open dialog returns the file and load proceeds after validation

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
The view layer SHALL display an info panel when the user clicks on a colonist, showing the colonist's display name, numeric id, needs (with labels from content definitions), current task, and position. For each need, the panel SHALL show the numeric value, a progress bar scaled to that need's `max`, and a critical status label from the active status definition when applicable.

#### Scenario: Open info panel
- **WHEN** the user clicks on a colonist sprite
- **THEN** a panel appears showing the colonist's name and id (e.g. `Mira (#2)`), Food value with bar, Sleep value with bar, current task name, and grid coordinates

#### Scenario: Critical need status visible
- **WHEN** the user opens the info panel for a colonist whose snapshot has `hungry: true`
- **THEN** the food row displays the `hungry` status label from YAML in addition to the numeric value and bar

#### Scenario: Sleep need status visible
- **WHEN** the user opens the info panel for a colonist whose snapshot has `wants_sleep: true`
- **THEN** the sleep row displays the `wants_sleep` status label from YAML in addition to the numeric value and bar

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
