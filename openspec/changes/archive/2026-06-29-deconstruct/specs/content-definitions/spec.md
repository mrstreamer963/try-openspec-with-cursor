## MODIFIED Requirements

### Requirement: YAML schema for buildings
Each building definition in YAML SHALL specify at minimum: `id`, `label`, `work_required`, `work_to_deconstruct`, `blocks_movement`, `blocks_settle`, display `color`, and `on_complete` / `interactions` primitives sufficient to express v1 wall, bed, and berry bush behavior.

#### Scenario: Wall blocks movement from definition
- **WHEN** a finished `wall` building occupies a tile
- **THEN** that tile is impassable per `blocks_movement: true` in the building definition

#### Scenario: Berry bush supply from definition
- **WHEN** a `berry_bush` construction order completes
- **THEN** the finished building spawns with berry supply equal to the amount specified in the building definition's `on_complete` supply primitive

## ADDED Requirements

### Requirement: work_to_deconstruct field
Each buildable building definition in YAML SHALL include a `work_to_deconstruct` field specifying the labor units required to remove that building. The game core SHALL expose this via `ContentRegistry::work_to_deconstruct(id: BuildingId) -> f32`.

#### Scenario: Base pack deconstruct values
- **WHEN** the base content pack is loaded
- **THEN** `wall` has `work_to_deconstruct: 15`, `bed` has `work_to_deconstruct: 25`, and `berry_bush` has `work_to_deconstruct: 20`

#### Scenario: Deconstruct work from registry
- **WHEN** a deconstruction site is created for a finished `bed`
- **THEN** its initial `work_remaining` equals the `work_to_deconstruct` value from the bed definition in YAML
