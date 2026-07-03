## MODIFIED Requirements

### Requirement: Camera pan and zoom
The view layer SHALL support camera panning via mouse drag, keyboard (W/A/S/D and arrow keys), and zooming via scroll wheel or pinch.

#### Scenario: Pan camera with mouse drag
- **WHEN** the user drags the mouse across the canvas
- **THEN** the camera offset updates and the world view scrolls accordingly

#### Scenario: Pan camera with keyboard
- **WHEN** the user holds **W**, **A**, **S**, or **D** (or the corresponding arrow key) while the game session is active
- **THEN** the camera offset updates continuously in that direction until the key is released

#### Scenario: Diagonal keyboard pan
- **WHEN** the user holds two perpendicular direction keys simultaneously (e.g. **W** and **D**)
- **THEN** the camera pans diagonally at the combined direction

#### Scenario: Zoom camera
- **WHEN** the user scrolls the mouse wheel
- **THEN** the camera zoom level changes while keeping the cursor position anchored
