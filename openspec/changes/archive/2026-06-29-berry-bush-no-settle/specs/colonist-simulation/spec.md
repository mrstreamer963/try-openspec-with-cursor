## ADDED Requirements

### Requirement: Berry bush pass-through without settling
Colonists SHALL be able to traverse BerryBush tiles during movement, but SHALL NOT treat a BerryBush tile as a valid settled cell or movement waypoint terminus.

#### Scenario: Bush waypoint skipped during movement
- **WHEN** a colonist's next path waypoint is a BerryBush tile
- **THEN** the colonist does not snap to that tile and advances to the following waypoint while continuing movement

#### Scenario: Wander avoids bush destination
- **WHEN** the simulation picks an idle wander destination
- **THEN** it does not select a BerryBush tile as the wander target

#### Scenario: Colonist ejected from bush cell
- **WHEN** a colonist's settled grid cell is a BerryBush tile after movement
- **THEN** the simulation moves the colonist to a nearby settleable cell if one exists

#### Scenario: Eat stand not on bush
- **WHEN** assigning an Eat or Build stand tile
- **THEN** the simulation does not select a BerryBush tile as the stand
