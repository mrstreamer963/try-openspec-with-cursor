## ADDED Requirements

### Requirement: Content mods in save file
Save files MAY include an optional `content_mods` string array recording the mod ids active when the game was saved. When present, entries SHALL be ordered mod ids from the manifest (e.g. `["base", "hardmode"]`).

#### Scenario: Save records active mods
- **WHEN** the user saves with mods `base` and `hardmode` active
- **THEN** the downloaded JSON includes `content_mods: ["base", "hardmode"]`

#### Scenario: Legacy save without content_mods
- **WHEN** the user loads a save file with no `content_mods` field
- **THEN** load proceeds treating the save as created with `["base"]` only

### Requirement: Mod mismatch warning on load
The view layer SHALL compare save `content_mods` to the currently loaded mod list when the user loads a save file. When they differ, the user SHALL be prompted to confirm before applying the save.

#### Scenario: Matching mod lists load silently
- **WHEN** save `content_mods` equals the current loaded mod list
- **THEN** load proceeds without a mod mismatch prompt

#### Scenario: Mismatched mod lists prompt user
- **WHEN** save `content_mods` differs from the current loaded mod list
- **THEN** the view layer shows a confirmation dialog describing both lists before calling `load_state`
