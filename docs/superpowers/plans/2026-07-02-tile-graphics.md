# Tile Graphics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace PixiJS color-rectangle rendering with sprites from CC0 multi-atlas tile sheets, configured via YAML, with color fallback on missing assets.

**Architecture:** Load `atlases.yaml` + PNG sheets into `AtlasManager` (one GPU texture per atlas, lazy frame `Texture` views). Extend `ContentPack` with `entities` and optional `sprite` refs. `SpriteResolver` maps content ids to textures. `PixiRenderer` draws `Sprite` nodes with `Graphics` color fallback. Atlases load in `App.vue` session boot before `GameSession` mounts.

**Tech Stack:** PixiJS 8 (`Assets`, `Texture`, `Sprite`, `Rectangle`), Vue 3, Vitest, js-yaml, Vite `publicDir` → `content/`

**Spec:** `docs/superpowers/specs/2026-07-02-tile-graphics-design.md`

---

## File map

| File | Action | Responsibility |
|------|--------|----------------|
| `content/assets/*/…png` | Create | CC0 atlas PNGs (one per pack) |
| `content/base/atlases.yaml` | Create | Atlas metadata (tile_size, spacing, columns) |
| `content/base/entities.yaml` | Create | Colonist entity def + sprite ref |
| `content/base/terrain.yaml` | Modify | Add `sprite` refs |
| `content/base/buildings.yaml` | Modify | Add `sprite` refs |
| `packages/client/src/content/types.ts` | Modify | `SpriteRef`, `EntityDef`, `AtlasDef`, extend defs |
| `packages/client/src/content/mergeContent.ts` | Modify | Merge `entities` |
| `packages/client/src/content/loadContent.ts` | Modify | Load optional `entities.yaml` |
| `packages/client/src/content/loadBaseContent.ts` | Modify | Strip view-only fields from WASM JSON |
| `packages/client/src/game/atlasFrameMath.ts` | Create | Pure frame/column math (testable) |
| `packages/client/src/game/atlasManager.ts` | Create | Load PNGs, cache frame textures |
| `packages/client/src/game/loadAtlases.ts` | Create | Parse `atlases.yaml`, orchestrate load |
| `packages/client/src/game/spriteResolver.ts` | Create | content id → `Texture` |
| `packages/client/src/game/PixiRenderer.ts` | Modify | Sprite rendering + fallback |
| `packages/client/src/App.vue` | Modify | Load atlases in `beginSession` |
| `packages/client/src/components/GameSession.vue` | Modify | Accept `SpriteResolver`, pass to renderer |
| `packages/client/src/game/atlasFrameMath.test.ts` | Create | Unit tests for frame math |
| `packages/client/src/game/spriteResolver.test.ts` | Create | Unit tests for resolver |
| `packages/client/src/content/loadContent.test.ts` | Modify | Cover `entities.yaml` loading |

---

### Task 1: Content types — `SpriteRef`, `EntityDef`, `AtlasDef`

**Files:**
- Modify: `packages/client/src/content/types.ts`

- [ ] **Step 1: Add types after `TerrainDef`**

```ts
export interface SpriteRef {
  atlas: string;
  frame: number;
}

export interface AtlasDef {
  id: string;
  path: string;
  tile_size: number;
  spacing: number;
  columns: number | 'auto';
}

export interface EntityDef {
  id: string;
  color: number;
  sprite?: SpriteRef;
}
```

- [ ] **Step 2: Extend existing defs**

Add to `TerrainDef` and `BuildingDef`:

```ts
  sprite?: SpriteRef;
```

Extend `ContentPack`:

```ts
export interface ContentPack {
  needs: NeedDef[];
  statuses: StatusDef[];
  terrain: TerrainDef[];
  buildings: BuildingDef[];
  entities: EntityDef[];
}
```

- [ ] **Step 3: Run typecheck**

Run: `cd packages/client && npx vue-tsc -b --noEmit`
Expected: FAIL — `emptyContentPack` and merge code missing `entities`

- [ ] **Step 4: Commit**

```bash
git add packages/client/src/content/types.ts
git commit -m "feat(client): add sprite and entity types for tile graphics"
```

---

### Task 2: Content loading — `entities.yaml` merge

**Files:**
- Modify: `packages/client/src/content/mergeContent.ts`
- Modify: `packages/client/src/content/loadContent.ts`
- Modify: `packages/client/src/content/loadContent.test.ts`

- [ ] **Step 1: Write failing test for entities load**

Add to `basePackFiles` in `loadContent.test.ts`:

```ts
  'base/entities.yaml': 'entities:\n  - id: colonist\n    color: 0xf6e05e',
```

Add test:

```ts
  it('loads entities.yaml from base mod', async () => {
    const resources = mockResourceManager({
      bundled: {
        ...basePackFiles,
        'base/entities.yaml': 'entities:\n  - id: colonist\n    color: 0xf6e05e',
      },
    });
    const { pack } = await loadContent({ enabledModIds: ['base'], resources });
    expect(pack.entities).toHaveLength(1);
    expect(pack.entities[0]?.id).toBe('colonist');
  });
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd packages/client && npm test -- src/content/loadContent.test.ts -t "loads entities"`
Expected: FAIL — `entities` undefined or empty

- [ ] **Step 3: Update `mergeContent.ts`**

```ts
export function mergeContentPacks(base: ContentPack, overlay: Partial<ContentPack>): ContentPack {
  return {
    needs: overlay.needs ? mergeById(base.needs, overlay.needs, 'needs') : base.needs,
    statuses: overlay.statuses
      ? mergeById(base.statuses, overlay.statuses, 'statuses')
      : base.statuses,
    buildings: overlay.buildings
      ? mergeById(base.buildings, overlay.buildings, 'buildings')
      : base.buildings,
    terrain: overlay.terrain ? mergeById(base.terrain, overlay.terrain, 'terrain') : base.terrain,
    entities: overlay.entities
      ? mergeById(base.entities, overlay.entities, 'entities')
      : base.entities,
  };
}

export function emptyContentPack(): ContentPack {
  return { needs: [], statuses: [], buildings: [], terrain: [], entities: [] };
}
```

- [ ] **Step 4: Update `loadContent.ts`**

Change `CATEGORY_FILES`:

```ts
const CATEGORY_FILES = ['needs', 'statuses', 'buildings', 'terrain', 'entities'] as const;
```

In `loadModPartial`, add branch:

```ts
    else if (category === 'entities') partial.entities = items as ContentPack['entities'];
```

Make `entities` optional for non-base mods (same as statuses — skip if missing). Base mod requires all categories in `CATEGORY_FILES` except make entities required for base by including `base/entities.yaml` in repo in Task 4.

For base mod: add `entities` to required files OR treat as optional with default `[]`. Use **optional** (skip if missing) like statuses in hardmode — base will have the file.

- [ ] **Step 5: Run tests**

Run: `cd packages/client && npm test -- src/content/loadContent.test.ts`
Expected: PASS (update `basePackFiles` in all tests to include empty entities or the colonist entry)

Add to every `basePackFiles` fixture:

```ts
  'base/entities.yaml': 'entities:\n  - id: colonist\n    color: 0xf6e05e',
```

- [ ] **Step 6: Strip view-only fields from WASM JSON**

Modify `contentPackToJson` in `loadBaseContent.ts`:

```ts
export function contentPackToJson(pack: ContentPack): string {
  const simPack = {
    needs: pack.needs,
    statuses: pack.statuses,
    terrain: pack.terrain.map(({ id, walkable, color }) => ({ id, walkable, color })),
    buildings: pack.buildings.map(
      ({ id, label, work_required, work_to_deconstruct, blocks_movement, blocks_settle, buildable, color, on_complete, interactions }) => ({
        id, label, work_required, work_to_deconstruct, blocks_movement, blocks_settle, buildable, color, on_complete, interactions,
      }),
    ),
  };
  return JSON.stringify(simPack);
}
```

- [ ] **Step 7: Commit**

```bash
git add packages/client/src/content/mergeContent.ts packages/client/src/content/loadContent.ts packages/client/src/content/loadBaseContent.ts packages/client/src/content/loadContent.test.ts
git commit -m "feat(client): load entities.yaml and strip sprite fields from WASM JSON"
```

---

### Task 3: Atlas frame math (TDD)

**Files:**
- Create: `packages/client/src/game/atlasFrameMath.ts`
- Create: `packages/client/src/game/atlasFrameMath.test.ts`

- [ ] **Step 1: Write failing tests**

```ts
import { describe, expect, it } from 'vitest';
import {
  resolveColumns,
  frameToRect,
  maxFrameCount,
} from './atlasFrameMath';

describe('resolveColumns', () => {
  it('returns explicit column count', () => {
    expect(resolveColumns(800, 16, 1, 12)).toBe(12);
  });

  it('computes auto columns from image width', () => {
    // 12 * (16+1) - 1 = 203, use width 204 → 12 cols with spacing 1
    expect(resolveColumns(204, 16, 1, 'auto')).toBe(12);
  });

  it('computes auto columns with zero spacing', () => {
    expect(resolveColumns(256, 16, 0, 'auto')).toBe(16);
  });
});

describe('frameToRect', () => {
  const columns = 12;
  const tileSize = 16;
  const spacing = 1;

  it('maps frame 0 to origin', () => {
    expect(frameToRect(0, columns, tileSize, spacing)).toEqual({ x: 0, y: 0, width: 16, height: 16 });
  });

  it('maps frame 1 to second column', () => {
    expect(frameToRect(1, columns, tileSize, spacing)).toEqual({ x: 17, y: 0, width: 16, height: 16 });
  });

  it('maps frame 12 to second row', () => {
    expect(frameToRect(12, columns, tileSize, spacing)).toEqual({ x: 0, y: 17, width: 16, height: 16 });
  });

  it('returns null when frame is out of bounds', () => {
    expect(frameToRect(-1, columns, tileSize, spacing)).toBeNull();
    expect(frameToRect(9999, columns, tileSize, spacing, 11)).toBeNull();
  });
});

describe('maxFrameCount', () => {
  it('counts frames for a gridded sheet', () => {
    expect(maxFrameCount(204, 187, 16, 1, 12)).toBe(12 * 11);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd packages/client && npm test -- src/game/atlasFrameMath.test.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Implement `atlasFrameMath.ts`**

```ts
export interface FrameRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function resolveColumns(
  imageWidth: number,
  tileSize: number,
  spacing: number,
  columns: number | 'auto',
): number {
  if (columns !== 'auto') return columns;
  const stride = tileSize + spacing;
  if (stride <= 0) return 1;
  return Math.max(1, Math.floor((imageWidth + spacing) / stride));
}

export function rowCount(
  imageHeight: number,
  tileSize: number,
  spacing: number,
): number {
  const stride = tileSize + spacing;
  if (stride <= 0) return 1;
  return Math.max(1, Math.floor((imageHeight + spacing) / stride));
}

export function maxFrameCount(
  imageWidth: number,
  imageHeight: number,
  tileSize: number,
  spacing: number,
  columns: number | 'auto',
): number {
  const cols = resolveColumns(imageWidth, tileSize, spacing, columns);
  const rows = rowCount(imageHeight, tileSize, spacing);
  return cols * rows;
}

export function frameToRect(
  frame: number,
  columns: number,
  tileSize: number,
  spacing: number,
  rows?: number,
): FrameRect | null {
  if (frame < 0 || columns <= 0) return null;
  const col = frame % columns;
  const row = Math.floor(frame / columns);
  if (rows !== undefined && row >= rows) return null;
  const stride = tileSize + spacing;
  return {
    x: col * stride,
    y: row * stride,
    width: tileSize,
    height: tileSize,
  };
}
```

- [ ] **Step 4: Run tests**

Run: `cd packages/client && npm test -- src/game/atlasFrameMath.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add packages/client/src/game/atlasFrameMath.ts packages/client/src/game/atlasFrameMath.test.ts
git commit -m "feat(client): add atlas frame math utilities with tests"
```

---

### Task 4: Download asset packs and add YAML config

**Files:**
- Create: `content/base/atlases.yaml`
- Create: `content/base/entities.yaml`
- Modify: `content/base/terrain.yaml`
- Modify: `content/base/buildings.yaml`
- Create: `content/assets/kenney-roguelike/spritesheet.png`
- Create: `content/assets/kenney-tiny-town/tilemap.png`
- Create: `content/assets/ever-rogue/everroguetileset_full_packed.png`

- [ ] **Step 1: Download CC0 packs**

Run (requires network):

```bash
mkdir -p /tmp/tile-assets && cd /tmp/tile-assets

# Kenney Roguelike/RPG — https://kenney.nl/assets/roguelike-rpg-pack
curl -L -o roguelike.zip "https://kenney.nl/media/pages/assets/roguelike-rpg-pack/017c9414d8-1677589452/roguelike-pack.zip"

# Kenney Tiny Town — https://kenney.nl/assets/tiny-town
curl -L -o tiny-town.zip "https://kenney.nl/media/pages/assets/tiny-town/097e0cd35e-1674436632/kenney_tiny-town.zip"

# Ever Rogue — https://opengameart.org/content/ever-rogue-tileset
curl -L -o ever-rogue.zip "https://opengameart.org/sites/default/files/everroguetileset_2.0_.zip"

unzip -l roguelike.zip | head -30
unzip -l tiny-town.zip | head -30
unzip -l ever-rogue.zip | head -30
```

If Kenney URLs redirect or change, download manually from the pages above and adjust paths.

- [ ] **Step 2: Copy PNG spritesheets into `content/assets/`**

```bash
REPO=/Users/mr.streamer/restore/try-openspec-with-cursor
mkdir -p "$REPO/content/assets/kenney-roguelike" \
         "$REPO/content/assets/kenney-tiny-town" \
         "$REPO/content/assets/ever-rogue"

# Roguelike pack: find the main transparent spritesheet
ROGUELIKE_PNG=$(unzip -Z1 roguelike.zip | grep -iE 'spritesheet.*\.png$|roguelike.*sheet.*\.png$' | head -1)
unzip -j roguelike.zip "$ROGUELIKE_PNG" -d "$REPO/content/assets/kenney-roguelike/"
mv "$REPO/content/assets/kenney-roguelike/"*.png "$REPO/content/assets/kenney-roguelike/spritesheet.png"

# Tiny Town: tilemap PNG
TOWN_PNG=$(unzip -Z1 tiny-town.zip | grep -i 'tilemap.*\.png$' | head -1)
unzip -j tiny-town.zip "$TOWN_PNG" -d "$REPO/content/assets/kenney-tiny-town/"
mv "$REPO/content/assets/kenney-tiny-town/"*.png "$REPO/content/assets/kenney-tiny-town/tilemap.png"

# Ever Rogue: packed sheet
unzip -j ever-rogue.zip '*packed*.png' -d "$REPO/content/assets/ever-rogue/" 2>/dev/null || \
  curl -L -o "$REPO/content/assets/ever-rogue/everroguetileset_full_packed.png" \
    "https://opengameart.org/sites/default/files/everroguetileset_full_packed.png"

# Copy tilemap.txt for column verification (roguelike)
unzip -j roguelike.zip '*tilemap.txt' -d "$REPO/content/assets/kenney-roguelike/" 2>/dev/null || true
```

- [ ] **Step 3: Measure PNG dimensions and set columns**

Run:

```bash
python3 -c "
from pathlib import Path
try:
  from PIL import Image
except ImportError:
  import struct, zlib
  def png_size(p):
    with open(p,'rb') as f:
      f.read(16); w,h=struct.unpack('>II', f.read(8)); return w,h
  Image = type('I',(),{'open': staticmethod(lambda p: type('S',(),{'size': png_size(p)})())})
for label, path in [
  ('roguelike', 'content/assets/kenney-roguelike/spritesheet.png'),
  ('tiny-town', 'content/assets/kenney-tiny-town/tilemap.png'),
  ('ever-rogue', 'content/assets/ever-rogue/everroguetileset_full_packed.png'),
]:
  p = Path('$REPO') / path
  w,h = Image.open(p).size
  print(label, w, h, 'cols@16+1=', (w+1)//17, 'cols@16+0=', w//16)
"
```

Record widths in `atlases.yaml`. Kenney Roguelike typically `columns: 49` with `spacing: 1`; verify against `tilemap.txt` if extracted.

- [ ] **Step 4: Create `content/base/atlases.yaml`**

```yaml
# CC0 sources: kenney.nl (Roguelike/RPG, Tiny Town), opengameart.org (Ever Rogue)
atlases:
  - id: kenney-roguelike
    path: assets/kenney-roguelike/spritesheet.png
    tile_size: 16
    spacing: 1
    columns: 49
  - id: kenney-tiny-town
    path: assets/kenney-tiny-town/tilemap.png
    tile_size: 16
    spacing: 1
    columns: 12
  - id: ever-rogue
    path: assets/ever-rogue/everroguetileset_full_packed.png
    tile_size: 16
    spacing: 0
    columns: auto
```

Adjust `columns: 49` if Step 3 shows a different value.

- [ ] **Step 5: Pick frame indices visually**

Open `content/assets/kenney-roguelike/spritesheet.png` in an image viewer with a grid overlay (16×16 cells, 1px gap). Record row-major index: `frame = row * columns + col`.

Target tiles (roguelike sheet unless a better match exists in another atlas):

| Content id | Suggested starting frame | Notes |
|------------|-------------------------|-------|
| `water` | 266 | blue water tile — verify visually |
| `sand` | 267 | tan/sand tile adjacent to water |
| `grass` | 268 | green grass tile |
| `wall` | 845 | grey stone wall |
| `bed` | 524 | bed furniture row |
| `berry_bush` | 313 | bush/shrub tile |
| `colonist` | 736 | humanoid character tile |

Adjust indices after visual inspection. Mixed atlases are allowed, e.g. `grass` from `kenney-tiny-town` if it looks better:

```yaml
sprite: { atlas: kenney-tiny-town, frame: 42 }
```

- [ ] **Step 6: Update content YAML files**

`content/base/entities.yaml`:

```yaml
entities:
  - id: colonist
    color: 0xf6e05e
    sprite: { atlas: kenney-roguelike, frame: 736 }
```

`content/base/terrain.yaml` — add `sprite` to each entry (keep existing `color`):

```yaml
terrain:
  - id: water
    walkable: false
    color: 0x2b6cb0
    sprite: { atlas: kenney-roguelike, frame: 266 }
  - id: sand
    walkable: true
    color: 0xd69e2e
    sprite: { atlas: kenney-roguelike, frame: 267 }
  - id: grass
    walkable: true
    color: 0x38a169
    sprite: { atlas: kenney-roguelike, frame: 268 }
```

`content/base/buildings.yaml` — add `sprite` to `wall`, `bed`, `berry_bush` (keep all other fields):

```yaml
    sprite: { atlas: kenney-roguelike, frame: 845 }   # wall
    sprite: { atlas: kenney-roguelike, frame: 524 }   # bed
    sprite: { atlas: kenney-roguelike, frame: 313 }   # berry_bush
```

- [ ] **Step 7: Commit assets and YAML**

```bash
git add content/assets content/base/atlases.yaml content/base/entities.yaml content/base/terrain.yaml content/base/buildings.yaml
git commit -m "feat(content): add CC0 tile atlases and sprite YAML mappings"
```

---

### Task 5: `AtlasManager` — load PNGs and resolve frames

**Files:**
- Create: `packages/client/src/game/atlasManager.ts`

- [ ] **Step 1: Implement AtlasManager**

```ts
import { Assets, Rectangle, Texture } from 'pixi.js';
import type { AtlasDef } from '../content/types';
import { frameToRect, maxFrameCount, resolveColumns, rowCount } from './atlasFrameMath';

interface AtlasState {
  def: AtlasDef;
  texture: Texture;
  columns: number;
  rows: number;
  frameCache: Map<number, Texture>;
}

export class AtlasManager {
  private atlases = new Map<string, AtlasState>();
  private readonly warnedAtlases = new Set<string>();

  async loadAll(defs: AtlasDef[], baseUrl = import.meta.env.BASE_URL): Promise<void> {
    await Promise.all(defs.map((def) => this.loadOne(def, baseUrl)));
  }

  private async loadOne(def: AtlasDef, baseUrl: string): Promise<void> {
    const url = `${baseUrl}${def.path}`.replace(/\/{2,}/g, '/').replace(':/', '://');
    try {
      const texture = await Assets.load<Texture>({ src: url });
      const columns = resolveColumns(texture.width, def.tile_size, def.spacing, def.columns);
      const rows = rowCount(texture.height, def.tile_size, def.spacing);
      this.atlases.set(def.id, {
        def,
        texture,
        columns,
        rows,
        frameCache: new Map(),
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.warn(`[AtlasManager] Failed to load atlas "${def.id}" from ${url}: ${message}`);
    }
  }

  getFrameTexture(atlasId: string, frame: number): Texture | null {
    const state = this.atlases.get(atlasId);
    if (!state) {
      if (!this.warnedAtlases.has(atlasId)) {
        console.warn(`[AtlasManager] Atlas not loaded: "${atlasId}"`);
        this.warnedAtlases.add(atlasId);
      }
      return null;
    }

    const max = maxFrameCount(
      state.texture.width,
      state.texture.height,
      state.def.tile_size,
      state.def.spacing,
      state.columns,
    );
    if (frame < 0 || frame >= max) {
      console.warn(`[AtlasManager] Frame ${frame} out of bounds for atlas "${atlasId}" (max ${max - 1})`);
      return null;
    }

    const cached = state.frameCache.get(frame);
    if (cached) return cached;

    const rect = frameToRect(frame, state.columns, state.def.tile_size, state.def.spacing, state.rows);
    if (!rect) return null;

    const frameTexture = new Texture({
      source: state.texture.source,
      frame: new Rectangle(rect.x, rect.y, rect.width, rect.height),
    });
    state.frameCache.set(frame, frameTexture);
    return frameTexture;
  }

  hasAtlas(atlasId: string): boolean {
    return this.atlases.has(atlasId);
  }
}
```

- [ ] **Step 2: Create `loadAtlases.ts`**

```ts
import { load as parseYaml } from 'js-yaml';
import type { ResourceManager } from '../resources/types';
import type { AtlasDef } from '../content/types';
import { AtlasManager } from './atlasManager';

interface AtlasesDoc {
  atlases: AtlasDef[];
}

export async function loadAtlases(
  resources: ResourceManager,
  baseUrl = import.meta.env.BASE_URL,
): Promise<AtlasManager> {
  const raw = await resources.readText('bundled', 'base/atlases.yaml');
  let doc: AtlasesDoc;
  try {
    doc = parseYaml(raw) as AtlasesDoc;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to parse atlases.yaml: ${message}`);
  }
  if (!Array.isArray(doc.atlases)) {
    throw new Error('atlases.yaml is missing an atlases array');
  }

  const manager = new AtlasManager();
  await manager.loadAll(doc.atlases, baseUrl);
  return manager;
}
```

- [ ] **Step 3: Commit**

```bash
git add packages/client/src/game/atlasManager.ts packages/client/src/game/loadAtlases.ts
git commit -m "feat(client): add AtlasManager and loadAtlases"
```

---

### Task 6: `SpriteResolver` (TDD)

**Files:**
- Create: `packages/client/src/game/spriteResolver.ts`
- Create: `packages/client/src/game/spriteResolver.test.ts`

- [ ] **Step 1: Write failing tests**

```ts
import { describe, expect, it, vi } from 'vitest';
import { SpriteResolver } from './spriteResolver';
import type { ContentPack } from '../content/types';
import type { AtlasManager } from './atlasManager';
import type { Texture } from 'pixi.js';

const fakeTexture = { label: 'frame' } as unknown as Texture;

function mockAtlasManager(frames: Record<string, Texture | null>): AtlasManager {
  return {
    getFrameTexture: vi.fn((atlas: string, frame: number) => frames[`${atlas}:${frame}`] ?? null),
    hasAtlas: vi.fn((atlas: string) => atlas in { 'kenney-roguelike': true }),
  } as unknown as AtlasManager;
}

const pack: ContentPack = {
  needs: [],
  statuses: [],
  entities: [{ id: 'colonist', color: 0xf6e05e, sprite: { atlas: 'kenney-roguelike', frame: 1 } }],
  terrain: [{ id: 'grass', walkable: true, color: 1, sprite: { atlas: 'kenney-roguelike', frame: 2 } }],
  buildings: [{ id: 'wall', label: 'w', work_required: 1, blocks_movement: true, blocks_settle: false, color: 2, sprite: { atlas: 'kenney-roguelike', frame: 3 }, on_complete: [], interactions: [] }],
};

describe('SpriteResolver', () => {
  it('resolves terrain texture', () => {
    const atlas = mockAtlasManager({ 'kenney-roguelike:2': fakeTexture });
    const resolver = new SpriteResolver(pack, atlas);
    expect(resolver.resolveTerrain('grass')).toBe(fakeTexture);
  });

  it('returns null when sprite field is missing', () => {
    const noSprite: ContentPack = { ...pack, terrain: [{ id: 'grass', walkable: true, color: 1 }] };
    const atlas = mockAtlasManager({});
    const resolver = new SpriteResolver(noSprite, atlas);
    expect(resolver.resolveTerrain('grass')).toBeNull();
  });

  it('returns null when atlas frame is missing', () => {
    const atlas = mockAtlasManager({});
    const resolver = new SpriteResolver(pack, atlas);
    expect(resolver.resolveBuilding('wall')).toBeNull();
  });

  it('resolves entity texture', () => {
    const atlas = mockAtlasManager({ 'kenney-roguelike:1': fakeTexture });
    const resolver = new SpriteResolver(pack, atlas);
    expect(resolver.resolveEntity('colonist')).toBe(fakeTexture);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd packages/client && npm test -- src/game/spriteResolver.test.ts`
Expected: FAIL

- [ ] **Step 3: Implement `spriteResolver.ts`**

```ts
import type { Texture } from 'pixi.js';
import type { ContentPack, SpriteRef } from '../content/types';
import type { AtlasManager } from './atlasManager';

export class SpriteResolver {
  private readonly warnedIds = new Set<string>();

  constructor(
    private readonly content: ContentPack,
    private readonly atlases: AtlasManager,
  ) {}

  resolveTerrain(id: string): Texture | null {
    const def = this.content.terrain.find((t) => t.id === id);
    return this.resolveSpriteRef(def?.sprite, `terrain:${id}`);
  }

  resolveBuilding(id: string): Texture | null {
    const def = this.content.buildings.find((b) => b.id === id);
    return this.resolveSpriteRef(def?.sprite, `building:${id}`);
  }

  resolveEntity(id: string): Texture | null {
    const def = this.content.entities.find((e) => e.id === id);
    return this.resolveSpriteRef(def?.sprite, `entity:${id}`);
  }

  private resolveSpriteRef(ref: SpriteRef | undefined, label: string): Texture | null {
    if (!ref) {
      this.warnOnce(label, `no sprite field in content`);
      return null;
    }
    const texture = this.atlases.getFrameTexture(ref.atlas, ref.frame);
    if (!texture) {
      this.warnOnce(label, `atlas "${ref.atlas}" frame ${ref.frame} unavailable`);
    }
    return texture;
  }

  private warnOnce(label: string, detail: string): void {
    if (this.warnedIds.has(label)) return;
    console.warn(`[SpriteResolver] ${label}: ${detail}`);
    this.warnedIds.add(label);
  }
}
```

- [ ] **Step 4: Run tests**

Run: `cd packages/client && npm test -- src/game/spriteResolver.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add packages/client/src/game/spriteResolver.ts packages/client/src/game/spriteResolver.test.ts
git commit -m "feat(client): add SpriteResolver with unit tests"
```

---

### Task 7: `PixiRenderer` — sprite rendering with color fallback

**Files:**
- Modify: `packages/client/src/game/PixiRenderer.ts`

- [ ] **Step 1: Update imports and constructor**

Add imports:

```ts
import { Application, Container, Graphics, Sprite, Text, TextStyle } from 'pixi.js';
import type { SpriteResolver } from './spriteResolver';
```

Change constructor:

```ts
  constructor(
    private mount: HTMLElement,
    content: ContentPack,
    private readonly spriteResolver: SpriteResolver,
  ) {
```

- [ ] **Step 2: Add helper to place sprite or color rect**

```ts
  private addTileGraphic(
    layer: Container,
    x: number,
    y: number,
    texture: import('pixi.js').Texture | null,
    color: number,
    alpha = 1,
    inset = 0,
  ): void {
    const px = x * TILE_SIZE + inset;
    const py = y * TILE_SIZE + inset;
    const size = TILE_SIZE - inset * 2;
    if (texture) {
      const sprite = new Sprite(texture);
      sprite.x = px;
      sprite.y = py;
      if (inset > 0) {
        sprite.width = size;
        sprite.height = size;
      }
      sprite.alpha = alpha;
      layer.addChild(sprite);
      return;
    }
    const g = new Graphics();
    g.rect(px, py, size, size);
    g.fill({ color, alpha });
    layer.addChild(g);
  }
```

- [ ] **Step 3: Update `drawTerrain`**

Replace body loop:

```ts
  private drawTerrain(snapshot: StateSnapshot): void {
    this.terrainLayer.removeChildren();
    for (const tile of snapshot.tiles) {
      const texture = this.spriteResolver.resolveTerrain(tile.terrain);
      const color = this.terrainColors[tile.terrain] ?? 0x4a5568;
      this.addTileGraphic(this.terrainLayer, tile.x, tile.y, texture, color);
    }
  }
```

- [ ] **Step 4: Update `drawBuildings`**

For construction sites, buildings, and deconstruction sites — replace `Graphics` rect fills with `addTileGraphic` using `resolveBuilding(site.building)` / `resolveBuilding(b.building)` and the same alpha logic as today. Keep progress bar methods unchanged.

- [ ] **Step 5: Update colonist rendering in `drawColonistsFrame`**

At start of colonist loop, resolve once outside loop:

```ts
    const colonistTexture = this.spriteResolver.resolveEntity('colonist');
```

Replace circle drawing block:

```ts
      if (colonistTexture) {
        let sprite = this.colonistGraphics.get(c.id) as Sprite | undefined;
        if (!(sprite instanceof Sprite)) {
          if (g) {
            this.entitiesLayer.removeChild(g);
            this.colonistGraphics.delete(c.id);
          }
          sprite = new Sprite(colonistTexture);
          sprite.anchor.set(0.5, 0.5);
          this.colonistGraphics.set(c.id, sprite as unknown as Graphics);
          this.entitiesLayer.addChild(sprite);
        }
        sprite.x = cx;
        sprite.y = cy;
        sprite.width = TILE_SIZE * 0.7;
        sprite.height = TILE_SIZE * 0.7;
      } else {
        // existing Graphics circle fallback
```

Refactor `colonistGraphics` map type to `Map<number, Graphics | Sprite>` or split into separate maps for clarity.

- [ ] **Step 6: Run typecheck and tests**

Run: `cd packages/client && npx vue-tsc -b --noEmit && npm test`
Expected: PASS (fix any type errors from colonist map refactor)

- [ ] **Step 7: Commit**

```bash
git add packages/client/src/game/PixiRenderer.ts
git commit -m "feat(client): render terrain, buildings, and colonists from tile sprites"
```

---

### Task 8: Wire atlas loading into session boot

**Files:**
- Modify: `packages/client/src/App.vue`
- Modify: `packages/client/src/components/GameSession.vue`

- [ ] **Step 1: Load atlases in `App.vue` `beginSession`**

Add imports:

```ts
import { loadAtlases } from './game/loadAtlases';
import { SpriteResolver } from './game/spriteResolver';
import type { SpriteResolver as SpriteResolverType } from './game/spriteResolver';
```

Add ref:

```ts
const spriteResolver = shallowRef<SpriteResolverType | null>(null);
```

In `beginSession`, after `loadContent`:

```ts
    const atlasManager = await loadAtlases(getResources());
    spriteResolver.value = new SpriteResolver(loaded.pack, atlasManager);
```

Pass to `GameSession` in template:

```vue
    <GameSession
      v-if="contentPack && spriteResolver"
      :key="sessionKey"
      :content-pack="contentPack"
      :sprite-resolver="spriteResolver"
      ...
    />
```

- [ ] **Step 2: Update `GameSession.vue`**

Add prop:

```ts
import type { SpriteResolver } from '../game/spriteResolver';

const props = defineProps<{
  contentPack: ContentPack;
  spriteResolver: SpriteResolver;
  ...
}>();
```

Change renderer construction:

```ts
    renderer = new PixiRenderer(canvasMount.value, props.contentPack, props.spriteResolver);
```

- [ ] **Step 3: Run dev build**

Run: `cd packages/client && npm run build`
Expected: SUCCESS

- [ ] **Step 4: Commit**

```bash
git add packages/client/src/App.vue packages/client/src/components/GameSession.vue
git commit -m "feat(client): load tile atlases during session boot"
```

---

### Task 9: Manual verification

- [ ] **Step 1: Start dev server**

Run: `cd packages/client && npm run dev`

- [ ] **Step 2: Visual check**

- Start new game
- Confirm terrain shows tile art (not flat colors)
- Build wall, bed, berry bush — confirm building sprites
- Confirm colonists show character sprites with name labels
- Pan, zoom, click colonist — interactions still work

- [ ] **Step 3: Fallback check**

Rename `content/assets/kenney-roguelike/spritesheet.png` temporarily, reload game.

Expected: colored rectangles + `console.warn` messages; game still playable.

Restore the PNG after test.

- [ ] **Step 4: Run full test suite**

Run: `cd packages/client && npm test`
Expected: all PASS

- [ ] **Step 5: Final commit if frame indices were tuned**

```bash
git add content/base/terrain.yaml content/base/buildings.yaml content/base/entities.yaml
git commit -m "fix(content): tune sprite frame indices after visual review"
```

---

## Spec coverage checklist

| Spec requirement | Task |
|------------------|------|
| Multi-atlas PNG load | Task 4, 5, 8 |
| `atlases.yaml` metadata | Task 4 |
| `sprite` in content defs | Task 4 |
| `entities.yaml` colonist | Task 2, 4 |
| Color fallback + warn | Task 6, 7 |
| Terrain/building/colonist sprites | Task 7 |
| Static colonist (no animation) | Task 7 |
| Unit tests frame math + resolver | Task 3, 6 |
| WASM JSON excludes view fields | Task 2 |
| Mod atlases out of scope | N/A |

## Out of scope (do not implement)

- Walk animation
- Mod-provided atlas PNGs
- Sprite HUD / progress bars
- Build-time mega-atlas merge
