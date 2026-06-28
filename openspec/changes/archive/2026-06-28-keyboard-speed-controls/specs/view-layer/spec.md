## MODIFIED Requirements

### Requirement: HUD with pause and speed controls
The view layer SHALL display a HUD with a pause button and speed controls (1×, 5×, 10×).

#### Scenario: Pause simulation
- **WHEN** the user clicks the pause button
- **THEN** an `IncomingEvent::SetPaused(true)` is sent to the worker and the HUD reflects paused state

#### Scenario: Change simulation speed
- **WHEN** the user selects 5× speed
- **THEN** an `IncomingEvent::SetSpeed(5.0)` is sent to the worker

## ADDED Requirements

### Requirement: Keyboard pause and speed shortcuts
The view layer SHALL respond to keyboard shortcuts for pause and speed while the game is loaded: **Space** toggles pause/resume, **1** sets 1×, **2** sets 5×, **3** sets 10×.

#### Scenario: Toggle pause with Space
- **WHEN** the user presses Space and the game is not loading
- **THEN** pause state toggles and the same events are sent as clicking the HUD pause button

#### Scenario: Set speed with digit keys
- **WHEN** the user presses **2**
- **THEN** simulation speed is set to 5× and the HUD active speed indicator updates

#### Scenario: Speed key while paused
- **WHEN** the user presses **3** while paused
- **THEN** speed is set to 10× (simulation remains paused until Space or resume is triggered)
