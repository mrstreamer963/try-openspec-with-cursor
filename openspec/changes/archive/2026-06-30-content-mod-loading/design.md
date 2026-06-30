## Context

Content loads from `content/base/*.yaml` via `loadBaseContent()`. `mod.yaml` is fetched but unused. The yaml-content-definitions design deferred multi-mod merge. Modders need override/add workflows without editing base or recompiling WASM.

## Goals / Non-Goals

**Goals:**

- `content/mods.yaml` manifest with ordered mod list
- `content/mods/<id>/` optional partial YAML overlays
- Merge by `id`: later mod replaces entire entry; new ids append
- `loadContent()` returns `{ pack, modIds }`; WASM API unchanged
- Optional `content_mods` in save files; warn on mismatch at load
- Demo `hardmode` mod in repo (disabled by default)
- Vitest tests for merge logic

**Non-Goals:**

- In-game mod picker, zip install, `depends_on` resolution
- Hot-reload during simulation
- Removing base entries (override only)
- Deep-merge of individual fields within one definition

## Decisions

### 1. Manifest at `content/mods.yaml` → URL `/mods.yaml`

Vite `publicDir` is repo `content/`, so manifest is served at root. Missing manifest falls back to `["base"]`.

### 2. Path resolution

- `base` → `/base/`
- other mods → `/mods/<id>/`

### 3. Full entry replacement on override

Modders copy a base entry and edit fields. Simpler than deep-merge; documented in demo mod README comment in YAML.

### 4. Validation split

Client: manifest, paths, mod id match, duplicate ids within one file. WASM: cross-reference validation after merge (unchanged).

### 5. Save `content_mods` without version bump

Optional field on version-1 saves. Absent field treated as `["base"]`.

## Risks / Trade-offs

- **[Override requires full entry]** → Document; demo mod shows pattern
- **[Load save with different mods]** → `window.confirm` warning; WASM may still error on unknown building ids
- **[Merge order for new ids]** → Append after base order; acceptable for v1

## Migration Plan

1. Ship `mods.yaml` with `mods: [base]` only — behavior identical to today
2. Modders add folders under `content/mods/` and list in manifest
3. No WASM or save migration required

## Open Questions

_None — design approved in brainstorming._
