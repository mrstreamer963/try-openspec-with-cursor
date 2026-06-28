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
- **THEN** a construction order for a Bed is created at that tile and no finished Bed exists until colonist work completes

#### Scenario: Reject build on water
- **WHEN** the host sends a build command at a Water tile
- **THEN** the build is rejected and no construction order or building entity is created

#### Scenario: Berry bush provides food
- **WHEN** a colonist interacts with a completed BerryBush building that has berries remaining
- **THEN** the colonist's Food need increases and the bush loses one berry

#### Scenario: Depleted berry bush removed
- **WHEN** a BerryBush's last berry is consumed
- **THEN** the building is removed from the world grid and no longer appears in snapshots

#### Scenario: Bed satisfies sleep
- **WHEN** a colonist interacts with a completed Bed building
- **THEN** the colonist's Sleep need increases

### Requirement: Resource-free construction
Building placement SHALL NOT require any resource cost in v1.

#### Scenario: Unlimited building
- **WHEN** the player places multiple beds in succession
- **THEN** each construction order is created without checking or deducting resources

### Requirement: Finite berry supply
Each newly completed BerryBush SHALL start with a fixed number of berry portions (3).

#### Scenario: New bush berry count
- **WHEN** a BerryBush construction order completes
- **THEN** the finished bush has exactly 3 berries available

### Requirement: Construction orders from build commands
When the host sends a build command for a valid tile, the simulation SHALL create a construction order at that tile instead of an instant finished building.

#### Scenario: Valid construction order
- **WHEN** the host sends a build command for a Wall at a walkable empty tile with no pending construction
- **THEN** a construction order for a Wall is created at that tile

#### Scenario: Reject duplicate construction order
- **WHEN** the host sends a build command at a tile that already has a pending construction order
- **THEN** the command is rejected and the existing order is unchanged

#### Scenario: Reject build on occupied tile
- **WHEN** the host sends a build command at a tile that already has a finished building
- **THEN** the command is rejected

### Requirement: Construction completion
When a construction order's remaining work reaches zero, the simulation SHALL replace it with a finished building on the world grid.

#### Scenario: Complete wall construction
- **WHEN** a Wall construction order's work reaches zero
- **THEN** a Wall building entity exists at that tile, the construction order is removed, and the tile blocks movement

#### Scenario: Construction site walkable until complete
- **WHEN** a construction order exists at a tile and work remains
- **THEN** the tile is walkable and no finished building occupies the world grid at that cell
