## MODIFIED Requirements

### Requirement: Placeable buildings
The simulation SHALL support placing three building types: Bed, BerryBush, and Wall (via build commands from the host).

#### Scenario: Place bed on grass
- **WHEN** the host sends a build command for a Bed at a walkable Grass tile
- **THEN** a Bed building entity is created at that tile position with no occupant

#### Scenario: Reject build on water
- **WHEN** the host sends a build command at a Water tile
- **THEN** the build is rejected and no building entity is created

#### Scenario: Berry bush provides food
- **WHEN** a colonist interacts with a BerryBush building from an orthogonally adjacent tile and the bush has berries remaining
- **THEN** the colonist's Food need increases and the bush loses one berry

#### Scenario: Depleted berry bush removed
- **WHEN** a BerryBush's last berry is consumed
- **THEN** the building is removed from the world grid and no longer appears in snapshots

#### Scenario: Bed satisfies sleep
- **WHEN** a colonist interacts with a Bed building while standing on the bed tile and holding the bed reservation
- **THEN** the colonist's Sleep need increases
