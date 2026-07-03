# Tile Graphics

## Purpose

Multi-atlas PNG sprite loading, frame rectangle math, sprite resolution from content definitions, and PixiJS sprite rendering with YAML color fallback.

## Requirements

### Requirement: Atlas metadata configuration
The client SHALL ship `content/base/atlases.yaml` defining one or more atlas entries. Each entry SHALL specify at minimum: `id`, `path` (relative to content root), `tile_size`, `spacing`, and `columns` (integer or `auto`).

#### Scenario: Base atlases parse successfully
- **WHEN** the client loads `base/atlases.yaml` at session start
- **THEN** all atlas entries parse without error and include valid `id` and `path` fields

#### Scenario: Auto column computation
- **WHEN** an atlas entry has `columns: auto`
- **THEN** the client computes column count as `floor((imageWidth + spacing) / (tile_size + spacing))` after the PNG loads

### Requirement: Parallel atlas PNG loading
The client SHALL load each atlas PNG in parallel via PixiJS `Assets.load`, storing one base `Texture` per atlas. Failed atlas loads SHALL log `console.warn` and skip that atlas without aborting the session.

#### Scenario: All atlases load
- **WHEN** all atlas PNG paths resolve successfully
- **THEN** `AtlasManager` reports each atlas id as available

#### Scenario: Single atlas load failure
- **WHEN** one atlas PNG fails to load
- **THEN** the client logs a warning, continues loading other atlases, and starts the game session

#### Scenario: Atlases YAML parse failure
- **WHEN** `atlases.yaml` fails to parse
- **THEN** session start fails with an error on the loading screen (same as content load failure)

### Requirement: Frame index to texture rectangle
The client SHALL map a row-major frame index to a sub-rectangle within an atlas using `col = frame % columns`, `row = floor(frame / columns)`, `x = col * (tile_size + spacing)`, `y = row * (tile_size + spacing)`. Frame textures SHALL be lightweight `Texture` views sharing the atlas source, cached lazily per frame.

#### Scenario: Frame zero at origin
- **WHEN** frame index `0` is requested for a loaded atlas
- **THEN** the returned texture rectangle starts at `(0, 0)` with size `(tile_size, tile_size)`

#### Scenario: Frame with spacing offset
- **WHEN** frame index `1` is requested for an atlas with `tile_size: 16`, `spacing: 1`, and `columns: 12`
- **THEN** the returned texture rectangle starts at `(17, 0)`

#### Scenario: Out-of-bounds frame rejected
- **WHEN** a frame index is negative or exceeds the atlas grid capacity
- **THEN** `getFrameTexture` returns `null` and logs a warning

### Requirement: SpriteResolver content mapping
The client SHALL provide a `SpriteResolver` that maps content definition ids to `Texture | null` using optional `sprite: { atlas, frame }` fields from the merged `ContentPack`.

#### Scenario: Resolve terrain sprite
- **WHEN** terrain `grass` has a valid `sprite` ref and atlas frame
- **THEN** `resolveTerrain('grass')` returns the corresponding frame texture

#### Scenario: Resolve building sprite
- **WHEN** building `wall` has a valid `sprite` ref and atlas frame
- **THEN** `resolveBuilding('wall')` returns the corresponding frame texture

#### Scenario: Resolve entity sprite
- **WHEN** entity `colonist` has a valid `sprite` ref and atlas frame
- **THEN** `resolveEntity('colonist')` returns the corresponding frame texture

#### Scenario: Missing sprite field
- **WHEN** a content definition has no `sprite` field
- **THEN** the resolver returns `null` and warns once per content id

#### Scenario: Missing atlas or frame
- **WHEN** a `sprite` ref points to an unloaded atlas or out-of-bounds frame
- **THEN** the resolver returns `null` and warns once per content id

### Requirement: Sprite rendering with color fallback
The renderer SHALL draw a PixiJS `Sprite` when `SpriteResolver` returns a texture, and SHALL fall back to the existing `Graphics` color fill using the definition's `color` field when the resolver returns `null`.

#### Scenario: Terrain sprite drawn
- **WHEN** a terrain tile's sprite resolves successfully
- **THEN** a `Sprite` is drawn at the tile position instead of a colored rectangle

#### Scenario: Terrain color fallback
- **WHEN** a terrain tile's sprite cannot be resolved
- **THEN** a colored rectangle using the terrain definition's `color` is drawn at the tile position

#### Scenario: Building sprite with construction alpha
- **WHEN** a construction site ghost resolves a building sprite
- **THEN** the sprite is drawn with the same semi-transparent alpha rules as the current ghost rectangles

#### Scenario: Colonist sprite fallback
- **WHEN** the colonist entity sprite cannot be resolved
- **THEN** the renderer draws the existing colored circle fallback

### Requirement: Atlas frame math unit tests
The client SHALL include unit tests for frame column resolution, rectangle computation (spacing 0 and 1), and out-of-bounds frame handling.

#### Scenario: Frame math tests pass
- **WHEN** `npm test` runs in `packages/client`
- **THEN** atlas frame math tests pass

### Requirement: SpriteResolver unit tests
The client SHALL include unit tests for valid sprite resolution, missing sprite fields, and missing atlas frames.

#### Scenario: Resolver tests pass
- **WHEN** `npm test` runs in `packages/client`
- **THEN** sprite resolver tests pass
