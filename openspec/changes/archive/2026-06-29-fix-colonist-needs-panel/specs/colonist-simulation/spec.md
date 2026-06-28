## MODIFIED Requirements

### Requirement: Colonist needs
Each colonist SHALL have two needs: Food and Sleep, each represented as a value from 0 (critical) to 100 (satisfied).

#### Scenario: Needs decay over time
- **WHEN** the simulation ticks while a colonist is idle
- **THEN** both Food and Sleep values decrease at a configurable rate

#### Scenario: Critical need threshold
- **WHEN** a colonist's Food or Sleep value drops below a defined threshold (e.g., 30)
- **THEN** the colonist is flagged as needing that resource

#### Scenario: Need status in snapshot
- **WHEN** a state snapshot is built
- **THEN** each colonist entry includes `hungry: true` when Food is below the critical threshold and `wants_sleep: true` when Sleep is below the critical threshold; otherwise the corresponding flag is `false`
