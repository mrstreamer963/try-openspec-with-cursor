# Content Mod Loading — Design

**Date:** 2026-06-30  
**Status:** Approved  
**OpenSpec change:** `content-mod-loading`

## Summary

Multi-mod YAML content loading: manifest at `content/mods.yaml`, overlay merge by `id`, optional `content_mods` in saves.

## File layout

```
content/
  mods.yaml              # mods: [base, hardmode, ...]
  base/                  # required foundation
  mods/
    hardmode/            # demo mod (off by default)
```

## Merge rules

- Load mods in manifest order
- Per category (`needs`, `statuses`, `buildings`, `terrain`): same `id` → full replace; new `id` → append
- Mod YAML files optional except `base` requires all four categories
- Override = copy full entry from base, edit fields

## Client API

- `loadContent()` → `{ pack, modIds }`
- `loadBaseContent()` → `pack` only (backward compatible)
- WASM unchanged: `Game::new(content_json)`

## Save/load

- Optional `content_mods: string[]` on save v1
- Mismatch → `window.confirm` before load

## Out of scope

Mod UI, zip install, `depends_on`, hot-reload, deep field merge, removing base entries.
