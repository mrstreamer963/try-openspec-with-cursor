## MODIFIED Requirements

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

## ADDED Requirements

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
