## ADDED Requirements

### Requirement: Unique settled cell occupancy
At most one colonist SHALL occupy a grid cell when movement completes a step to that cell. A colonist's settled cell is its current grid cell derived from position.

#### Scenario: Second colonist waits for occupied cell
- **WHEN** a colonist completes movement toward a waypoint cell that is already occupied by another colonist's settled position
- **THEN** the colonist does not snap to that cell, does not advance its path index, and remains at its previous position until the cell is free

#### Scenario: Idle colonist occupies its cell
- **WHEN** a colonist is idle (including after completing an Eat task on a stand tile)
- **THEN** its current grid cell is occupied and no other colonist may settle on that cell

### Requirement: Pass-through during movement
Colonists MAY pass through each other while moving between grid cells. Pathfinding SHALL NOT treat other colonists as impassable obstacles.

#### Scenario: Colonists cross paths mid-movement
- **WHEN** two colonists move toward each other and their interpolated positions overlap before either completes a waypoint snap
- **THEN** both continue moving without blocking each other

#### Scenario: Pathfinding ignores colonist positions
- **WHEN** A* pathfinding computes a route
- **THEN** only terrain and building walkability affect the path, not colonist positions

### Requirement: Eat assignment prefers unoccupied stand tiles
When assigning an Eat task, the simulation SHALL select an adjacent stand tile that is not occupied by another colonist's settled position and not already assigned as a stand to another colonist in the same assignment pass.

#### Scenario: Occupied stand skipped for eat assignment
- **WHEN** a colonist needs Food and the nearest BerryBush has adjacent stand tiles, but the best stand tile is occupied by another colonist
- **THEN** the simulation assigns an Eat task using another free adjacent stand tile for that bush, or a stand tile for another bush, or no Eat task if none are available

#### Scenario: Single stand queue at bush
- **WHEN** two colonists need Food and a BerryBush has only one adjacent stand tile
- **THEN** at most one colonist is assigned that stand tile per assignment pass; the other colonist is assigned elsewhere or remains idle until the stand is free

## MODIFIED Requirements

### Requirement: A* pathfinding
Colonists SHALL use A* pathfinding to navigate to their task destination across walkable tiles. Other colonists SHALL NOT be treated as impassable during path computation.

#### Scenario: Path to berry bush
- **WHEN** a colonist is assigned an Eat task at a distant BerryBush
- **THEN** the colonist follows a valid A* path to an adjacent stand tile next to the bush, avoiding impassable tiles but not avoiding cells occupied by other colonists

#### Scenario: No path available
- **WHEN** no walkable path exists to the task stand tile
- **THEN** the task is cancelled and the colonist returns to idle
