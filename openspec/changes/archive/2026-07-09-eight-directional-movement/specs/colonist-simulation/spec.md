## MODIFIED Requirements

### Requirement: A* pathfinding
Colonists SHALL use A* pathfinding to navigate to their task destination across walkable tiles. Other colonists SHALL NOT be treated as impassable during path computation. Pathfinding SHALL explore eight directions (four cardinal and four diagonal). Orthogonal steps SHALL cost 1; diagonal steps SHALL cost √2. A diagonal step SHALL be allowed only when the destination cell is walkable and both orthogonally adjacent cells that share an edge with the diagonal step are walkable (no corner-cutting through blocked tiles).

#### Scenario: Path to berry bush
- **WHEN** a colonist is assigned an Eat task at a distant BerryBush
- **THEN** the colonist follows a valid A* path to an adjacent stand tile next to the bush, avoiding impassable tiles but not avoiding cells occupied by other colonists

#### Scenario: Diagonal shortcut in open area
- **WHEN** a colonist needs a path across open walkable tiles where a diagonal route is shorter than an orthogonal-only route
- **THEN** the computed path includes diagonal waypoints

#### Scenario: Corner-cutting blocked
- **WHEN** a diagonal step would cut through a corner bounded by two impassable orthogonally adjacent cells
- **THEN** that diagonal neighbor is not used and the path routes around the corner

#### Scenario: No path available
- **WHEN** no walkable path exists to the task stand tile
- **THEN** the task is cancelled and the colonist returns to idle
