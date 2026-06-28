## MODIFIED Requirements

### Requirement: Placeable buildings
The simulation SHALL support placing three building types: Bed, BerryBush, and Wall (via build commands from the host).

#### Scenario: Berry bush provides food
- **WHEN** a colonist interacts with a BerryBush building that has berries remaining
- **THEN** the colonist's Food need increases and the bush loses one berry

#### Scenario: Depleted berry bush removed
- **WHEN** a BerryBush's last berry is consumed
- **THEN** the building is removed from the world grid and no longer appears in snapshots

## ADDED Requirements

### Requirement: Finite berry supply
Each newly placed BerryBush SHALL start with a fixed number of berry portions (3).

#### Scenario: New bush berry count
- **WHEN** a BerryBush is placed
- **THEN** it has exactly 3 berries available
