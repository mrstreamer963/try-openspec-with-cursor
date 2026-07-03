## MODIFIED Requirements

### Requirement: Three starting colonists
The simulation SHALL spawn exactly 3 colonists at game start on walkable tiles.

#### Scenario: Initial colonist count
- **WHEN** the game initializes
- **THEN** exactly 3 colonist entities exist with unique IDs, unique display names, and valid positions

## ADDED Requirements

### Requirement: Colonist display names
Each colonist SHALL have a unique display name assigned at spawn from a fixed name pool.

#### Scenario: Unique names at spawn
- **WHEN** the game initializes with 3 colonists
- **THEN** each colonist has a distinct non-empty name

#### Scenario: Name in snapshot
- **WHEN** a state snapshot is built
- **THEN** each colonist entry includes its display name alongside its numeric id
