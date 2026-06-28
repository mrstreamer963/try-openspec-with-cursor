# World Simulation

## Purpose

50×50 tile world generation, terrain types, and placeable buildings.

## Requirements

### Requirement: 50×50 tile world
The simulation SHALL generate a fixed 50×50 grid world at initialization.

#### Scenario: World dimensions
- **WHEN** the game starts
- **THEN** the world contains exactly 2500 tiles arranged in a 50-column by 50-row grid

#### Scenario: Deterministic generation
- **WHEN** the game is initialized with the same seed twice
- **THEN** both worlds produce identical tile layouts

### Requirement: Terrain types
Each tile SHALL have one of three terrain types: Water, Sand, or Grass.

#### Scenario: Terrain assignment
- **WHEN** the world is generated
- **THEN** every tile is assigned exactly one terrain type from {Water, Sand, Grass}

#### Scenario: Walkability by terrain
- **WHEN** a colonist attempts to pathfind through a Water tile
- **THEN** the Water tile is treated as impassable

### Requirement: Placeable buildings
The simulation SHALL support placing three building types: Bed, BerryBush, and Wall (via build commands from the host).

#### Scenario: Place bed on grass
- **WHEN** the host sends a build command for a Bed at a walkable Grass tile
- **THEN** a Bed building entity is created at that tile position

#### Scenario: Reject build on water
- **WHEN** the host sends a build command at a Water tile
- **THEN** the build is rejected and no building entity is created

#### Scenario: Berry bush provides food
- **WHEN** a colonist interacts with a BerryBush building that has berries remaining
- **THEN** the colonist's Food need increases and the bush loses one berry

#### Scenario: Depleted berry bush removed
- **WHEN** a BerryBush's last berry is consumed
- **THEN** the building is removed from the world grid and no longer appears in snapshots

#### Scenario: Bed satisfies sleep
- **WHEN** a colonist interacts with a Bed building
- **THEN** the colonist's Sleep need increases

### Requirement: Resource-free construction
Building placement SHALL NOT require any resource cost in v1.

#### Scenario: Unlimited building
- **WHEN** the player places multiple beds in succession
- **THEN** each placement succeeds without checking or deducting resources

### Requirement: Finite berry supply
Each newly placed BerryBush SHALL start with a fixed number of berry portions (3).

#### Scenario: New bush berry count
- **WHEN** a BerryBush is placed
- **THEN** it has exactly 3 berries available
