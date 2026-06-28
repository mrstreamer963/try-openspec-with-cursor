## MODIFIED Requirements

### Requirement: Speed multiplier
The worker bridge SHALL apply a speed multiplier to the delta time passed to `tick()`, supporting 1×, 5×, and 10× speeds.

#### Scenario: Five times speed
- **WHEN** speed is set to 5×
- **THEN** each tick call passes `dt = 0.25` instead of `0.05`

#### Scenario: Ten times speed
- **WHEN** speed is set to 10×
- **THEN** each tick call passes `dt = 0.50` instead of `0.05`
