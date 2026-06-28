## Context

Colonists use numeric `ColonistId(u32)` for ECS identity and snapshot serialization. The client renders yellow circles with no labels; `ColonistInfo.vue` shows `Colonist #{{ id }}`. We explored giving each colonist a display name visible above the sprite and in the info panel (`Name (#id)`), with player renaming out of scope.

## Goals / Non-Goals

**Goals:**
- Assign a unique display name to each colonist at spawn
- Expose `name` in `ColonistSnapshot` alongside existing `id`
- Render name labels above colonist sprites (always visible)
- Show `Name (#id)` in the colonist info panel when selected

**Non-Goals:**
- Player-assigned renaming
- Hover-only or selection-only name display
- Localized name pools
- Replacing numeric `id` as the internal/client Map key

## Decisions

### ColonistName ECS component

Store display name as `ColonistName(String)` on colonist entities. Keep `ColonistId` unchanged for stable identity and client `Map<number, …>` keys.

**Alternatives considered:**
- *Name only, drop numeric id from snapshot* — breaks existing click/motion maps; id useful in debug panel
- *Generate names on client* — names would not be simulation-authoritative; harder if logic later references colonists by name

### Name pool at spawn

Static pool of ~12 short names in Rust. On `spawn_colonists`, shuffle indices with `getrandom` and assign without replacement for the 3 starting colonists.

**Alternatives considered:**
- *Sequential assignment (Alex, Mira, Finn)* — predictable but less variety across sessions
- *Procedural syllable generator* — overkill for 3 colonists

### Snapshot field

Add `name: String` to `ColonistSnapshot`. Additive protocol change; `id` unchanged.

### PixiJS name labels

Add a `Text` object per colonist in `entitiesLayer`, positioned above the circle center each frame in `drawColonistsFrame`. White text with dark stroke or semi-transparent background for readability on grass. Scale with world container (same as sprites).

**Alternatives considered:**
- *DOM overlay via Vue* — harder to sync with camera pan/zoom and smooth motion
- *BitmapText* — unnecessary for 3 short ASCII labels

### Info panel format

Header: `{{ colonist.name }} (#{{ colonist.id }})` — name primary, id secondary for debugging.

## Risks / Trade-offs

- **[Label overlap when colonists cluster]** → Acceptable with 3 colonists; revisit if stacking rules change
- **[Snapshot size]** → Three short strings negligible at 20 Hz
- **[Future colonist spawn]** → Pool logic must handle N > pool size (append suffix or allow reuse); not needed for v1 (fixed 3 colonists)

## Migration Plan

Additive change — no data migration. Rebuild WASM and client. Existing saves N/A (no persistence).

## Open Questions

_(none — explored and resolved in /opsx-explore)_
